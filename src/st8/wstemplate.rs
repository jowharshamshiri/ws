use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tera::{Context as TeraContext, Tera};
use walkdir::WalkDir;

use crate::st8::VersionInfo;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct RenderedTemplate {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
}

/// The template engine for `.wstemplate` files.
///
/// ## Context variables available in every template
///
/// | Variable                     | Description                                 |
/// |------------------------------|---------------------------------------------|
/// | `{{ project.version }}`      | Owning project's full version (e.g. 0.5.12) |
/// | `{{ project.major_version }}`| e.g. `v0`                                   |
/// | `{{ project.minor_version }}`| commit count                                |
/// | `{{ project.patch_version }}`| line changes since last release tag         |
/// | `{{ project.name }}`         | from manifest or directory name             |
/// | `{{ projects.ALIAS.* }}`     | same set of fields for any discoverable proj|
/// | `{{ datetime.iso }}`         | RFC 3339 timestamp                          |
/// | `{{ datetime.date }}`        | YYYY-MM-DD                                  |
/// | `{{ datetime.time }}`        | HH:MM:SS                                    |
/// | `{{ datetime.year/month/day}}`| individual components                      |
///
/// ## Template selection
///
/// When [`render_relevant`] is called, only templates that satisfy at least one
/// of the following conditions are rendered:
/// 1. The template lives under `project_root` (own templates).
/// 2. The template's text contains a reference to `projects.SELF_ALIAS.*`.
///
/// ## Version resolution for referenced aliases
///
/// For `{{ project.* }}` the version is read from `version.txt` of the project
/// that owns the template file (identified by the closest `.ws/state.json`).
/// For the current project (`self_alias`) the freshly-computed `current_version`
/// is used so that the update is always current.
///
/// For every other `{{ projects.ALIAS.* }}` reference the engine:
/// 1. Walks `scan_root` to find all `.ws/state.json` files.
/// 2. Finds the one that registers alias.
/// 3. Reads `{project_root}/version.txt` for that project.
///
/// A missing `version.txt` is a hard error — run `ws update` in the dependency
/// project first.  An unresolvable alias is also a hard error.
pub struct WstemplateEngine {
    current_version: VersionInfo,
    current_name: Option<String>,
    /// Alias for the current project (from its `wstemplate_entry().alias`)
    self_alias: String,
    /// The current project's own directory
    project_root: PathBuf,
    /// Root to scan for `.wstemplate` files and peer `.ws/state.json` files
    scan_root: PathBuf,
}

impl WstemplateEngine {
    /// Create an engine for the current project.
    ///
    /// - `current_version` — freshly-computed version of this project
    /// - `current_name`    — human-readable name of this project
    /// - `self_alias`      — Tera alias for this project (from the wstemplate entry)
    /// - `project_root`    — this project's own root directory
    /// - `scan_root`       — the workspace root to scan for templates and peers
    pub fn new(
        current_version: VersionInfo,
        current_name: Option<String>,
        self_alias: String,
        project_root: PathBuf,
        scan_root: PathBuf,
    ) -> Self {
        Self {
            current_version,
            current_name,
            self_alias,
            project_root,
            scan_root,
        }
    }

    /// Derive a deterministic, Tera-compatible alias from a project root path.
    ///
    /// Algorithm:
    /// 1. Take the last path component (directory basename).
    /// 2. Lowercase every character.
    /// 3. Replace any character that is not ASCII alphanumeric with `_`.
    /// 4. Collapse runs of `_` into a single `_` and strip leading/trailing `_`.
    /// 5. If the result starts with a digit, prepend `p_`.
    /// 6. If the result is empty, return `"project"`.
    ///
    /// Examples: `my-app` → `my_app`, `API Service` → `api_service`, `123abc` → `p_123abc`.
    pub fn derive_alias(path: &Path) -> String {
        let base = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");

        // Step 2-3: lowercase and replace non-alphanumeric.
        let raw: String = base
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect();

        // Step 4: collapse consecutive underscores.
        let mut slug = String::with_capacity(raw.len());
        let mut prev_underscore = true; // treat start as underscore to strip leading
        for ch in raw.chars() {
            if ch == '_' {
                if !prev_underscore {
                    slug.push('_');
                }
                prev_underscore = true;
            } else {
                slug.push(ch);
                prev_underscore = false;
            }
        }
        // Strip trailing underscore.
        let slug = slug.trim_end_matches('_').to_string();

        if slug.is_empty() {
            return "project".to_string();
        }

        // Step 5: must not start with a digit.
        if slug.starts_with(|c: char| c.is_ascii_digit()) {
            format!("p_{}", slug)
        } else {
            slug
        }
    }

    /// Discover all `.wstemplate` files in `scan_root` using `rg`.
    ///
    /// `rg` is required — the function fails hard if it is not available.
    /// `rg` respects `.gitignore` automatically when run inside a git repository.
    pub fn discover_all(&self) -> Result<Vec<PathBuf>> {
        discover_in(&self.scan_root)
    }

    /// Build a map of `alias → project_root` by scanning all `.ws/state.json`
    /// files in the scan root.
    pub fn discover_project_roots(&self) -> Result<HashMap<String, PathBuf>> {
        find_all_project_roots(&self.scan_root)
    }

    /// Discover templates that are relevant to the current project:
    /// - templates *owned* by this project (closest project root is `project_root`), OR
    /// - templates anywhere in `scan_root` that reference `{{ projects.SELF_ALIAS.* }}`
    ///
    /// Ownership is determined by the most-specific project root: a template at
    /// `/workspace/sub/foo.wstemplate` is owned by `/workspace/sub` if that is a
    /// registered project, even if `/workspace` is also a registered project.
    ///
    /// Irrelevant templates (owned by other projects and not referencing self_alias)
    /// are silently skipped.
    pub fn discover_relevant(
        &self,
        project_roots: &HashMap<String, PathBuf>,
    ) -> Result<Vec<PathBuf>> {
        let all = self.discover_all()?;
        let mut relevant = Vec::new();
        for path in all {
            if self.is_own_template(&path, project_roots) {
                relevant.push(path);
                continue;
            }
            // Read content to check for self alias reference.
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Cannot read template {}", path.display()))?;
            if references_alias(&content, &self.self_alias) {
                relevant.push(path);
            }
        }
        Ok(relevant)
    }

    /// Render all templates relevant to the current project.
    ///
    /// See [`discover_relevant`] for the selection criteria.
    ///
    /// For each selected template the context is built with:
    /// - `{{ project.* }}` = version of the project that *owns* the template
    ///   (for own templates this is `current_version`; for foreign templates it
    ///   comes from that project's `version.txt`)
    /// - `{{ projects.SELF_ALIAS.* }}` = `current_version`
    /// - `{{ projects.OTHER.* }}` = read from `{other_project_root}/version.txt`
    ///
    /// Any unresolvable alias or missing `version.txt` is a hard error.
    pub fn render_relevant(&self) -> Result<Vec<RenderedTemplate>> {
        // Build a complete alias→project_root map by scanning the workspace.
        let project_roots = find_all_project_roots(&self.scan_root)?;

        let relevant = self.discover_relevant(&project_roots)?;
        let mut rendered = Vec::new();

        for path in &relevant {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Cannot read template {}", path.display()))?;

            let is_own = self.is_own_template(path, &project_roots);

            // Determine which project owns this template file.
            let owning_version =
                self.find_owning_project_version(path, &project_roots, is_own)?;

            let owning_name = if is_own {
                self.current_name.as_deref()
            } else {
                None // Foreign templates don't carry the current project's name
            };

            let ctx = build_context_for_template(
                &content,
                path,
                &owning_version,
                owning_name,
                &self.self_alias,
                &self.current_version,
                self.current_name.as_deref(),
                &project_roots,
            )
            .with_context(|| format!("Cannot build render context for {}", path.display()))?;

            let r = render_one_with_content(path, &content, &ctx)
                .with_context(|| format!("Cannot render template {}", path.display()))?;

            log::info!(
                "wstemplate: {} -> {}",
                r.source_path.display(),
                r.output_path.display()
            );
            rendered.push(r);
        }

        Ok(rendered)
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

impl WstemplateEngine {
    /// Check if a template is owned by this project (not by a more-specific subproject).
    ///
    /// A template is "own" when `self.project_root` is the most-specific project
    /// root containing it.  For example, if project_root = `/workspace` and a
    /// subproject at `/workspace/sub` also exists, templates under `/workspace/sub`
    /// are NOT own templates of `/workspace`.
    fn is_own_template(
        &self,
        template_path: &Path,
        project_roots: &HashMap<String, PathBuf>,
    ) -> bool {
        if !template_path.starts_with(&self.project_root) {
            return false;
        }
        // Check that no more-specific project root also contains this template.
        let self_depth = self.project_root.components().count();
        for root in project_roots.values() {
            if root == &self.project_root {
                continue;
            }
            if template_path.starts_with(root)
                && root.components().count() > self_depth
            {
                return false; // a deeper project owns this template
            }
        }
        true
    }

    /// Determine the version to use for `{{ project.* }}` in a template.
    ///
    /// - If `is_own` is true → `self.current_version` (freshly computed)
    /// - Otherwise find the nearest owning project in `project_roots` → its `version.txt`
    fn find_owning_project_version(
        &self,
        template_path: &Path,
        project_roots: &HashMap<String, PathBuf>,
        is_own: bool,
    ) -> Result<VersionInfo> {
        if is_own {
            return Ok(self.current_version.clone());
        }

        // Find the most-specific project root that contains this template.
        if let Some(root) = find_most_specific_root(template_path, project_roots) {
            return read_version_from_path(&root);
        }

        anyhow::bail!(
            "Cannot determine the owning project for template at {}. \
             The template is not under any project registered in {}",
            template_path.display(),
            self.scan_root.display()
        );
    }
}

/// Minimal view of `.ws/state.json` needed for alias discovery.
#[derive(Deserialize)]
struct StateSnapshot {
    project_root: PathBuf,
    #[serde(default)]
    wstemplate_entries: Vec<StateEntry>,
}

#[derive(Deserialize)]
struct StateEntry {
    alias: String,
    #[allow(dead_code)]
    root: PathBuf,
}

/// Walk `scan_root` and return a map of `alias → project_root` by reading
/// every `.ws/state.json` found.
///
/// Duplicate aliases are a hard error — every alias must be unique across the
/// workspace.
fn find_all_project_roots(scan_root: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut map: HashMap<String, PathBuf> = HashMap::new();

    for entry in WalkDir::new(scan_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Always descend into the scan root itself.
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_str().unwrap_or("");
            // Descend into .ws dirs; skip build/vendor/VCS dirs.
            if name.starts_with('.') && name != ".ws" {
                return false;
            }
            !matches!(name, "target" | "node_modules" | "__pycache__")
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // skip permission errors
        };

        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "state.json" {
            continue;
        }

        // Verify parent directory is named ".ws"
        let is_in_ws = entry
            .path()
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n == ".ws")
            .unwrap_or(false);
        if !is_in_ws {
            continue;
        }

        let content = match fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue, // unreadable — skip
        };

        let snapshot: StateSnapshot = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => continue, // malformed — skip
        };

        for ws_entry in &snapshot.wstemplate_entries {
            if let Some(existing) = map.get(&ws_entry.alias) {
                anyhow::bail!(
                    "Duplicate wstemplate alias '{}': registered by both {} and {}. \
                     Each alias must be unique across the workspace.",
                    ws_entry.alias,
                    existing.display(),
                    snapshot.project_root.display()
                );
            }
            map.insert(ws_entry.alias.clone(), snapshot.project_root.clone());
        }
    }

    Ok(map)
}

/// Return the project root that most specifically contains `path` (longest prefix).
fn find_most_specific_root(
    path: &Path,
    project_roots: &HashMap<String, PathBuf>,
) -> Option<PathBuf> {
    project_roots
        .values()
        .filter(|root| path.starts_with(root.as_path()))
        .max_by_key(|root| root.components().count())
        .cloned()
}

/// Read `version.txt` from `project_root` and parse it into a [`VersionInfo`].
///
/// Fails hard if the file does not exist (meaning `ws update` has never been
/// run for that project) or cannot be parsed.
pub fn read_version_from_path(project_root: &Path) -> Result<VersionInfo> {
    let version_file = project_root.join("version.txt");
    let content = fs::read_to_string(&version_file).with_context(|| {
        format!(
            "Cannot read version.txt for project at {}. \
             Run 'ws update' in that project first.",
            project_root.display()
        )
    })?;
    parse_version_string(content.trim())
}

/// Parse a `major.minor.patch` string into a [`VersionInfo`].
pub fn parse_version_string(s: &str) -> Result<VersionInfo> {
    let parts: Vec<&str> = s.splitn(3, '.').collect();
    anyhow::ensure!(
        parts.len() == 3,
        "Invalid version string '{}': expected major.minor.patch",
        s
    );
    let major: u32 = parts[0]
        .parse()
        .with_context(|| format!("Invalid major in version string '{}'", s))?;
    let minor: u32 = parts[1]
        .parse()
        .with_context(|| format!("Invalid minor in version string '{}'", s))?;
    let patch: u32 = parts[2]
        .parse()
        .with_context(|| format!("Invalid patch in version string '{}'", s))?;

    Ok(VersionInfo {
        major_version: format!("v{}", major),
        minor_version: minor,
        patch_version: patch,
        full_version: s.to_string(),
    })
}

/// Return true if `template_text` contains a `{{ projects.ALIAS.*}}` reference.
fn references_alias(template_text: &str, alias: &str) -> bool {
    template_text.contains(&format!("projects.{}", alias))
}

/// Extract every alias referenced as `{{ projects.ALIAS.* }}` in `template_text`.
fn extract_referenced_aliases(template_text: &str) -> Vec<String> {
    let re = Regex::new(r"\bprojects\.(\w+)").expect("static regex must compile");
    let mut aliases: Vec<String> = re
        .captures_iter(template_text)
        .map(|cap| cap[1].to_string())
        .collect();
    aliases.sort();
    aliases.dedup();
    aliases
}

fn project_map(vi: &VersionInfo, name: Option<&str>) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("version".to_string(), vi.full_version.clone());
    m.insert("major_version".to_string(), vi.major_version.clone());
    m.insert("minor_version".to_string(), vi.minor_version.to_string());
    m.insert("patch_version".to_string(), vi.patch_version.to_string());
    if let Some(n) = name {
        m.insert("name".to_string(), n.to_string());
    }
    m
}

/// Build the Tera context for a specific template.
///
/// - `{{ project.* }}`        = `owning_version` / `owning_name`
/// - `{{ projects.self_alias.*}}` = `self_version` / `self_name`  (current run)
/// - `{{ projects.OTHER.* }}`  = read from `project_roots[OTHER]/version.txt`
#[allow(clippy::too_many_arguments)]
fn build_context_for_template(
    template_text: &str,
    template_path: &Path,
    owning_version: &VersionInfo,
    owning_name: Option<&str>,
    self_alias: &str,
    self_version: &VersionInfo,
    self_name: Option<&str>,
    project_roots: &HashMap<String, PathBuf>,
) -> Result<TeraContext> {
    let mut ctx = TeraContext::new();

    // {{ project.* }} — version of the project that owns the template file
    ctx.insert("project", &project_map(owning_version, owning_name));

    // {{ projects.ALIAS.* }} — one entry per alias referenced in the template
    let mut projects: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Always include the current (self) project.
    projects.insert(self_alias.to_string(), project_map(self_version, self_name));

    for alias in extract_referenced_aliases(template_text) {
        if alias == self_alias {
            continue; // already added
        }
        let root = project_roots.get(&alias).ok_or_else(|| {
            let mut known: Vec<&str> = project_roots.keys().map(|s| s.as_str()).collect();
            known.sort();
            anyhow::anyhow!(
                "Template {} references alias '{}' which cannot be resolved.\n\
                 Known aliases: [{}]\n\
                 Either the project is missing a .ws/state.json or its alias doesn't match.",
                template_path.display(),
                alias,
                known.join(", "),
            )
        })?;
        let version = read_version_from_path(root)
            .with_context(|| format!("Cannot resolve version for alias '{}'", alias))?;
        projects.insert(alias, project_map(&version, None));
    }

    ctx.insert("projects", &projects);

    // {{ datetime.* }}
    let now = chrono::Local::now();
    let mut datetime: HashMap<String, String> = HashMap::new();
    datetime.insert("iso".to_string(), now.to_rfc3339());
    datetime.insert("date".to_string(), now.format("%Y-%m-%d").to_string());
    datetime.insert("time".to_string(), now.format("%H:%M:%S").to_string());
    datetime.insert("year".to_string(), now.format("%Y").to_string());
    datetime.insert("month".to_string(), now.format("%m").to_string());
    datetime.insert("day".to_string(), now.format("%d").to_string());
    ctx.insert("datetime", &datetime);

    Ok(ctx)
}

/// Render a single `.wstemplate` file given pre-read content and a pre-built context.
///
/// The output path is derived by stripping the `.wstemplate` suffix.
fn render_one_with_content(
    template_path: &Path,
    content: &str,
    ctx: &TeraContext,
) -> Result<RenderedTemplate> {
    let path_str = template_path.to_string_lossy();
    anyhow::ensure!(
        path_str.ends_with(".wstemplate"),
        "Template path does not end with .wstemplate: {}",
        template_path.display()
    );

    let output_path = PathBuf::from(&path_str[..path_str.len() - ".wstemplate".len()]);

    let mut tera = Tera::default();
    let tpl_name = template_path.display().to_string();
    tera.add_raw_template(&tpl_name, content)
        .with_context(|| format!("Tera parse error in {}", template_path.display()))?;

    let rendered = tera
        .render(&tpl_name, ctx)
        .with_context(|| format!("Tera render error in {}", template_path.display()))?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Cannot create parent directory for {}",
                output_path.display()
            )
        })?;
    }

    fs::write(&output_path, &rendered)
        .with_context(|| format!("Cannot write {}", output_path.display()))?;

    Ok(RenderedTemplate {
        source_path: template_path.to_path_buf(),
        output_path,
    })
}

/// Run `rg --files --glob '*.wstemplate' <root>` and return the discovered paths.
///
/// Exit code 0 = matches found; exit code 1 = no matches — both are valid.
/// Any other exit code indicates an `rg` error and propagates as a hard error.
fn discover_in(root: &Path) -> Result<Vec<PathBuf>> {
    let root_str = root
        .to_str()
        .with_context(|| {
            format!(
                "wstemplate root path is not valid UTF-8: {}",
                root.display()
            )
        })?;

    let output = Command::new("rg")
        .args(["--files", "--glob", "*.wstemplate", root_str])
        .output()
        .context("Failed to execute rg. Ensure ripgrep is installed and available in PATH.")?;

    match output.status.code() {
        Some(0) | Some(1) => {
            let stdout =
                String::from_utf8(output.stdout).context("rg produced non-UTF-8 output")?;
            let mut paths: Vec<PathBuf> = stdout
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| PathBuf::from(l.trim()))
                .collect();
            paths.sort();
            paths.dedup();
            Ok(paths)
        }
        Some(code) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("rg exited with error code {}: {}", code, stderr.trim());
        }
        None => anyhow::bail!("rg process was terminated by signal"),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_vi(version: &str) -> VersionInfo {
        let parts: Vec<&str> = version.splitn(3, '.').collect();
        let major: u32 = parts[0].parse().unwrap_or(0);
        let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        VersionInfo {
            major_version: format!("v{}", major),
            minor_version: minor,
            patch_version: patch,
            full_version: version.to_string(),
        }
    }

    /// Create a minimal project directory with `.ws/state.json` and `version.txt`.
    fn make_project(
        parent: &Path,
        name: &str,
        alias: &str,
        version: &str,
        scan_root: &Path,
    ) -> PathBuf {
        let project_dir = parent.join(name);
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(project_dir.join(".ws")).unwrap();
        fs::write(project_dir.join("version.txt"), version).unwrap();

        let state = serde_json::json!({
            "version": 1,
            "project_root": project_dir.to_str().unwrap(),
            "project_name": name,
            "tools": {},
            "wstemplate_entries": [
                { "alias": alias, "root": scan_root.to_str().unwrap() }
            ]
        });
        fs::write(
            project_dir.join(".ws").join("state.json"),
            serde_json::to_string_pretty(&state).unwrap(),
        )
        .unwrap();
        project_dir
    }

    // ── derive_alias ──────────────────────────────────────────────────────────

    #[test]
    fn test_derive_alias_simple_name() {
        assert_eq!(WstemplateEngine::derive_alias(Path::new("/path/to/myapp")), "myapp");
    }

    #[test]
    fn test_derive_alias_hyphenated() {
        assert_eq!(
            WstemplateEngine::derive_alias(Path::new("/path/to/my-service")),
            "my_service"
        );
    }

    #[test]
    fn test_derive_alias_spaces_and_mixed_case() {
        assert_eq!(
            WstemplateEngine::derive_alias(Path::new("/path/API Service")),
            "api_service"
        );
    }

    #[test]
    fn test_derive_alias_leading_digit() {
        assert_eq!(
            WstemplateEngine::derive_alias(Path::new("/path/123project")),
            "p_123project"
        );
    }

    #[test]
    fn test_derive_alias_collapses_multiple_separators() {
        assert_eq!(
            WstemplateEngine::derive_alias(Path::new("/path/my--app__v2")),
            "my_app_v2"
        );
    }

    #[test]
    fn test_derive_alias_empty_basename_returns_project() {
        // A path that ends in separator produces an empty basename.
        assert_eq!(WstemplateEngine::derive_alias(Path::new("/")), "project");
    }

    #[test]
    fn test_derive_alias_is_deterministic() {
        let path = Path::new("/some/dir/my-cool-project");
        assert_eq!(
            WstemplateEngine::derive_alias(path),
            WstemplateEngine::derive_alias(path)
        );
    }

    // ── parse_version_string ──────────────────────────────────────────────────

    #[test]
    fn test_parse_version_string_valid() {
        let vi = parse_version_string("1.23.456").unwrap();
        assert_eq!(vi.full_version, "1.23.456");
        assert_eq!(vi.major_version, "v1");
        assert_eq!(vi.minor_version, 23);
        assert_eq!(vi.patch_version, 456);
    }

    #[test]
    fn test_parse_version_string_zeros() {
        let vi = parse_version_string("0.0.0").unwrap();
        assert_eq!(vi.full_version, "0.0.0");
        assert_eq!(vi.major_version, "v0");
        assert_eq!(vi.minor_version, 0);
        assert_eq!(vi.patch_version, 0);
    }

    #[test]
    fn test_parse_version_string_missing_patch_fails() {
        assert!(parse_version_string("1.2").is_err());
    }

    #[test]
    fn test_parse_version_string_non_numeric_fails() {
        assert!(parse_version_string("a.b.c").is_err());
    }

    #[test]
    fn test_parse_version_string_empty_fails() {
        assert!(parse_version_string("").is_err());
    }

    // ── read_version_from_path ────────────────────────────────────────────────

    #[test]
    fn test_read_version_from_path_success() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("version.txt"), "2.100.5000\n").unwrap();
        let vi = read_version_from_path(dir.path()).unwrap();
        assert_eq!(vi.full_version, "2.100.5000");
        assert_eq!(vi.minor_version, 100);
    }

    #[test]
    fn test_read_version_from_path_missing_fails_hard() {
        let dir = TempDir::new().unwrap();
        let err = read_version_from_path(dir.path()).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("version.txt"),
            "error must mention version.txt, got: {}",
            msg
        );
        assert!(
            msg.contains("ws update"),
            "error must suggest ws update, got: {}",
            msg
        );
    }

    // ── find_all_project_roots ────────────────────────────────────────────────

    #[test]
    fn test_find_all_project_roots_discovers_projects() {
        let workspace = TempDir::new().unwrap();
        make_project(workspace.path(), "alpha", "alpha", "1.0.0", workspace.path());
        make_project(workspace.path(), "beta", "beta", "2.0.0", workspace.path());

        let roots = find_all_project_roots(workspace.path()).unwrap();
        assert!(roots.contains_key("alpha"), "must find alpha");
        assert!(roots.contains_key("beta"), "must find beta");
        assert_eq!(roots["alpha"], workspace.path().join("alpha"));
        assert_eq!(roots["beta"], workspace.path().join("beta"));
    }

    #[test]
    fn test_find_all_project_roots_skips_target_directories() {
        let workspace = TempDir::new().unwrap();
        make_project(workspace.path(), "real_proj", "real_proj", "1.0.0", workspace.path());

        // Simulate a Cargo target directory with a stale state.json inside.
        let target_ws = workspace.path().join("target").join(".ws");
        fs::create_dir_all(&target_ws).unwrap();
        let fake_state = serde_json::json!({
            "version": 1,
            "project_root": workspace.path().join("target").to_str().unwrap(),
            "project_name": "fake",
            "tools": {},
            "wstemplate_entries": [{ "alias": "should_be_skipped", "root": workspace.path().to_str().unwrap() }]
        });
        fs::write(
            target_ws.join("state.json"),
            serde_json::to_string_pretty(&fake_state).unwrap(),
        )
        .unwrap();

        let roots = find_all_project_roots(workspace.path()).unwrap();
        assert!(roots.contains_key("real_proj"), "real project must be found");
        assert!(
            !roots.contains_key("should_be_skipped"),
            "projects inside target/ must be skipped"
        );
    }

    #[test]
    fn test_find_all_project_roots_duplicate_alias_fails_hard() {
        let workspace = TempDir::new().unwrap();
        make_project(workspace.path(), "proj_a", "shared_alias", "1.0.0", workspace.path());
        make_project(workspace.path(), "proj_b", "shared_alias", "2.0.0", workspace.path());

        let err = find_all_project_roots(workspace.path()).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("shared_alias"),
            "error must name the duplicate alias, got: {}",
            msg
        );
    }

    // ── discover_all ─────────────────────────────────────────────────────────

    #[test]
    fn test_discover_all_finds_wstemplate_files() {
        let workspace = TempDir::new().unwrap();
        let proj = make_project(workspace.path(), "myapp", "myapp", "1.0.0", workspace.path());
        fs::write(proj.join("Cargo.toml.wstemplate"), "version = \"{{ project.version }}\"").unwrap();
        fs::write(proj.join("README.md"), "not a template").unwrap();

        let engine = WstemplateEngine::new(
            make_vi("1.0.0"),
            Some("myapp".to_string()),
            "myapp".to_string(),
            proj.clone(),
            workspace.path().to_path_buf(),
        );

        let found = engine.discover_all().unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("Cargo.toml.wstemplate"));
    }

    // ── discover_relevant ─────────────────────────────────────────────────────

    #[test]
    fn test_discover_relevant_includes_own_templates() {
        let workspace = TempDir::new().unwrap();
        let proj = make_project(workspace.path(), "self_proj", "self_proj", "1.0.0", workspace.path());
        let other = make_project(workspace.path(), "other", "other", "2.0.0", workspace.path());

        // Own template (under project_root) — always relevant.
        fs::write(proj.join("a.txt.wstemplate"), "{{ project.version }}").unwrap();
        // Template in other project, does NOT reference self_proj.
        fs::write(other.join("b.txt.wstemplate"), "{{ project.version }}").unwrap();

        let engine = WstemplateEngine::new(
            make_vi("1.0.0"),
            Some("self_proj".to_string()),
            "self_proj".to_string(),
            proj.clone(),
            workspace.path().to_path_buf(),
        );

        let project_roots = find_all_project_roots(workspace.path()).unwrap();
        let relevant = engine.discover_relevant(&project_roots).unwrap();
        assert_eq!(relevant.len(), 1, "only own template must be included");
        assert!(relevant[0].starts_with(&proj));
    }

    #[test]
    fn test_discover_relevant_includes_foreign_template_referencing_self() {
        let workspace = TempDir::new().unwrap();
        let proj = make_project(workspace.path(), "lib", "lib", "3.0.0", workspace.path());
        let consumer = make_project(workspace.path(), "app", "app", "1.0.0", workspace.path());

        // Consumer's template references lib (self_alias = "lib").
        fs::write(
            consumer.join("Cargo.toml.wstemplate"),
            "dep_version = \"{{ projects.lib.version }}\"",
        )
        .unwrap();

        let engine = WstemplateEngine::new(
            make_vi("3.0.0"),
            Some("lib".to_string()),
            "lib".to_string(),
            proj.clone(),
            workspace.path().to_path_buf(),
        );

        let project_roots = find_all_project_roots(workspace.path()).unwrap();
        let relevant = engine.discover_relevant(&project_roots).unwrap();
        assert_eq!(relevant.len(), 1, "consumer's template must be included");
        assert!(relevant[0].starts_with(&consumer));
    }

    #[test]
    fn test_discover_relevant_skips_unrelated_templates() {
        let workspace = TempDir::new().unwrap();
        let proj = make_project(workspace.path(), "lib", "lib", "1.0.0", workspace.path());
        let unrelated = make_project(workspace.path(), "other", "other", "2.0.0", workspace.path());

        // unrelated template only uses project.version — no reference to "lib".
        fs::write(unrelated.join("x.txt.wstemplate"), "{{ project.version }}").unwrap();

        let engine = WstemplateEngine::new(
            make_vi("1.0.0"),
            Some("lib".to_string()),
            "lib".to_string(),
            proj.clone(),
            workspace.path().to_path_buf(),
        );

        let project_roots = find_all_project_roots(workspace.path()).unwrap();
        let relevant = engine.discover_relevant(&project_roots).unwrap();
        assert!(relevant.is_empty(), "unrelated template must be skipped");
    }

    #[test]
    fn test_discover_relevant_root_project_does_not_claim_subproject_templates() {
        // When project_root == scan_root (root project), templates under
        // subprojects must NOT be claimed as "own".
        let workspace = TempDir::new().unwrap();

        // Root project at workspace level.
        let root = make_project(workspace.path(), "root", "root", "9.0.0", workspace.path());
        // Subproject inside root.
        let sub = make_project(&root, "sub", "sub", "1.0.0", workspace.path());

        // Root's own template.
        fs::write(root.join("root.txt.wstemplate"), "{{ project.version }}").unwrap();
        // Sub's own template — must NOT be claimed by root.
        fs::write(sub.join("sub.txt.wstemplate"), "{{ project.version }}").unwrap();

        let engine = WstemplateEngine::new(
            make_vi("9.0.0"),
            Some("root".to_string()),
            "root".to_string(),
            root.clone(),
            workspace.path().to_path_buf(),
        );

        let project_roots = find_all_project_roots(workspace.path()).unwrap();
        let relevant = engine.discover_relevant(&project_roots).unwrap();

        // Only root's own template should be relevant.
        assert_eq!(relevant.len(), 1, "sub's template must not be claimed by root");
        assert!(
            relevant[0].ends_with("root.txt.wstemplate"),
            "only root's own template, got: {}",
            relevant[0].display()
        );
    }

    #[test]
    fn test_render_root_project_does_not_clobber_subproject_versions() {
        // When running ws update from a root project that encompasses subprojects,
        // subproject templates must get their OWN version, not the root's version.
        let workspace = TempDir::new().unwrap();

        let root = make_project(workspace.path(), "root", "root", "9.0.0", workspace.path());
        let sub = make_project(&root, "sub", "sub", "1.5.100", workspace.path());

        // Sub's template references root's version.
        fs::write(
            sub.join("file.txt.wstemplate"),
            "root={{ projects.root.version }} own={{ project.version }}",
        )
        .unwrap();

        let engine = WstemplateEngine::new(
            make_vi("9.0.0"),
            Some("root".to_string()),
            "root".to_string(),
            root.clone(),
            workspace.path().to_path_buf(),
        );

        let rendered = engine.render_relevant().unwrap();
        assert_eq!(rendered.len(), 1, "sub's template references root");

        let content = fs::read_to_string(sub.join("file.txt")).unwrap();
        assert!(
            content.contains("root=9.0.0"),
            "root version must be the freshly computed one, got: {}",
            content
        );
        assert!(
            content.contains("own=1.5.100"),
            "own version must come from sub's version.txt (not root's), got: {}",
            content
        );
    }

    // ── render_relevant ───────────────────────────────────────────────────────

    #[test]
    fn test_render_relevant_renders_own_template_with_current_version() {
        let workspace = TempDir::new().unwrap();
        let proj = make_project(workspace.path(), "mylib", "mylib", "5.10.2000", workspace.path());

        fs::write(
            proj.join("Cargo.toml.wstemplate"),
            "version = \"{{ project.version }}\"",
        )
        .unwrap();

        let engine = WstemplateEngine::new(
            make_vi("5.10.2000"),
            Some("mylib".to_string()),
            "mylib".to_string(),
            proj.clone(),
            workspace.path().to_path_buf(),
        );

        let rendered = engine.render_relevant().unwrap();
        assert_eq!(rendered.len(), 1);

        let content = fs::read_to_string(proj.join("Cargo.toml")).unwrap();
        assert_eq!(content, "version = \"5.10.2000\"");
    }

    #[test]
    fn test_render_relevant_resolves_peer_alias_dynamically() {
        let workspace = TempDir::new().unwrap();
        // lib is the project being updated.
        let lib = make_project(workspace.path(), "lib", "lib", "3.0.0", workspace.path());
        // app depends on lib.
        let app = make_project(workspace.path(), "app", "app", "1.0.0", workspace.path());

        fs::write(
            app.join("Cargo.toml.wstemplate"),
            "lib_dep = \"{{ projects.lib.version }}\"\nown = \"{{ project.version }}\"",
        )
        .unwrap();

        let engine = WstemplateEngine::new(
            make_vi("3.0.0"),
            Some("lib".to_string()),
            "lib".to_string(),
            lib.clone(),
            workspace.path().to_path_buf(),
        );

        let rendered = engine.render_relevant().unwrap();
        assert_eq!(rendered.len(), 1, "app's template must be rendered");

        let content = fs::read_to_string(app.join("Cargo.toml")).unwrap();
        // app's own version comes from app's version.txt = "1.0.0"
        assert!(
            content.contains("lib_dep = \"3.0.0\""),
            "lib version must be the freshly-computed one"
        );
        assert!(
            content.contains("own = \"1.0.0\""),
            "own version must come from app's version.txt"
        );
    }

    #[test]
    fn test_render_relevant_fails_hard_on_unresolvable_alias() {
        let workspace = TempDir::new().unwrap();
        let proj = make_project(workspace.path(), "myapp", "myapp", "1.0.0", workspace.path());

        // Template references an alias that has no matching state.json.
        fs::write(
            proj.join("file.txt.wstemplate"),
            "v = \"{{ projects.ghost_dep.version }}\"",
        )
        .unwrap();

        let engine = WstemplateEngine::new(
            make_vi("1.0.0"),
            Some("myapp".to_string()),
            "myapp".to_string(),
            proj.clone(),
            workspace.path().to_path_buf(),
        );

        let err = engine.render_relevant().unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("ghost_dep"),
            "error must name the unresolvable alias, got: {}",
            msg
        );
        assert!(
            msg.contains("Known aliases:"),
            "error must list known aliases, got: {}",
            msg
        );
        assert!(
            msg.contains("myapp"),
            "known aliases must include myapp, got: {}",
            msg
        );
    }

    #[test]
    fn test_render_relevant_fails_hard_when_version_txt_missing() {
        let workspace = TempDir::new().unwrap();
        let lib = make_project(workspace.path(), "lib", "lib", "1.0.0", workspace.path());
        let app = make_project(workspace.path(), "app", "app", "1.0.0", workspace.path());

        // Remove app's version.txt to simulate never-updated project.
        fs::remove_file(app.join("version.txt")).unwrap();

        // app's template references lib — so lib's update will try to render it.
        fs::write(
            app.join("file.txt.wstemplate"),
            "v = \"{{ projects.lib.version }}\"\nown = \"{{ project.version }}\"",
        )
        .unwrap();

        let engine = WstemplateEngine::new(
            make_vi("1.0.0"),
            Some("lib".to_string()),
            "lib".to_string(),
            lib.clone(),
            workspace.path().to_path_buf(),
        );

        // The render must fail because app has no version.txt.
        let err = engine.render_relevant().unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("version.txt"),
            "error must mention version.txt, got: {}",
            msg
        );
    }

    #[test]
    fn test_render_relevant_own_template_does_not_need_version_txt() {
        // Own templates (under project_root) must render without version.txt in
        // other projects — no dynamic resolution is needed.
        let workspace = TempDir::new().unwrap();
        let proj = make_project(workspace.path(), "proj", "proj", "7.0.0", workspace.path());

        // Template only uses project.version — no peer references.
        fs::write(
            proj.join("version.h.wstemplate"),
            "#define VERSION \"{{ project.version }}\"",
        )
        .unwrap();

        let engine = WstemplateEngine::new(
            make_vi("7.0.0"),
            None,
            "proj".to_string(),
            proj.clone(),
            workspace.path().to_path_buf(),
        );

        let rendered = engine.render_relevant().unwrap();
        assert_eq!(rendered.len(), 1);
        let content = fs::read_to_string(proj.join("version.h")).unwrap();
        assert_eq!(content, "#define VERSION \"7.0.0\"");
    }
}
