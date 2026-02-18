use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tera::{Context as TeraContext, Tera};

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
/// | `{{ project.version }}`      | Current project full version (e.g. 0.5.12)  |
/// | `{{ project.major_version }}`| e.g. `v0`                                   |
/// | `{{ project.minor_version }}`| commit count                                |
/// | `{{ project.patch_version }}`| line changes since last release tag         |
/// | `{{ project.name }}`         | from manifest or directory name             |
/// | `{{ projects.ALIAS.* }}`     | same set of fields for each configured entry|
/// | `{{ datetime.iso }}`         | RFC 3339 timestamp                          |
/// | `{{ datetime.date }}`        | YYYY-MM-DD                                  |
/// | `{{ datetime.time }}`        | HH:MM:SS                                    |
/// | `{{ datetime.year/month/day}}`| individual components                      |
///
/// ## Unknown alias detection
///
/// Before rendering, the engine scans the template text for all `projects.<alias>`
/// references.  Any alias that has not been registered via [`WstemplateEngine::add_entry`]
/// causes a hard error with a descriptive message that lists the configured aliases and
/// suggests the `ws wstemplate add` command.
pub struct WstemplateEngine {
    current_version: VersionInfo,
    current_name: Option<String>,
    entries: Vec<EngineEntry>,
}

struct EngineEntry {
    root: PathBuf,
    alias: String,
    version_info: VersionInfo,
    project_name: Option<String>,
}

impl WstemplateEngine {
    /// Create an engine for the current project.
    ///
    /// `current_version` and `current_name` populate `{{ project.* }}`.
    /// Call [`add_entry`] for every cross-project reference you need.
    pub fn new(current_version: VersionInfo, current_name: Option<String>) -> Self {
        Self {
            current_version,
            current_name,
            entries: Vec::new(),
        }
    }

    /// Register a project root to scan for templates and to expose as
    /// `{{ projects.ALIAS.* }}` in every template's context.
    pub fn add_entry(
        &mut self,
        root: PathBuf,
        alias: String,
        version_info: VersionInfo,
        project_name: Option<String>,
    ) {
        self.entries.push(EngineEntry {
            root,
            alias,
            version_info,
            project_name,
        });
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

    /// Discover all `.wstemplate` files across all configured entry roots using `rg`.
    ///
    /// `rg` is required — the function fails hard if it is not available.
    /// `rg` respects `.gitignore` automatically when run inside a git repository.
    pub fn discover_all(&self) -> Result<Vec<PathBuf>> {
        let mut all_paths = Vec::new();
        for entry in &self.entries {
            let mut paths = discover_in(&entry.root)?;
            all_paths.append(&mut paths);
        }
        // Deduplicate in case the same file is reachable via multiple entries.
        all_paths.sort();
        all_paths.dedup();
        Ok(all_paths)
    }

    /// Build the Tera context for rendering.  Populates `project.*`, `projects.*` (one
    /// key per registered entry), and `datetime.*`.
    pub fn build_context(&self) -> TeraContext {
        let mut ctx = TeraContext::new();

        // Current project → {{ project.* }}
        ctx.insert("project", &project_map(&self.current_version, self.current_name.as_deref()));

        // Other projects → {{ projects.ALIAS.* }}
        let mut projects: HashMap<String, HashMap<String, String>> = HashMap::new();
        for entry in &self.entries {
            projects.insert(
                entry.alias.clone(),
                project_map(&entry.version_info, entry.project_name.as_deref()),
            );
        }
        ctx.insert("projects", &projects);

        // Datetime → {{ datetime.* }}
        let now = chrono::Local::now();
        let mut datetime: HashMap<String, String> = HashMap::new();
        datetime.insert("iso".to_string(), now.to_rfc3339());
        datetime.insert("date".to_string(), now.format("%Y-%m-%d").to_string());
        datetime.insert("time".to_string(), now.format("%H:%M:%S").to_string());
        datetime.insert("year".to_string(), now.format("%Y").to_string());
        datetime.insert("month".to_string(), now.format("%m").to_string());
        datetime.insert("day".to_string(), now.format("%d").to_string());
        ctx.insert("datetime", &datetime);

        ctx
    }

    /// Render a single `.wstemplate` file using the supplied Tera context.
    ///
    /// Validates that every `projects.<alias>` reference in the template corresponds
    /// to a registered entry — any unknown alias is a hard error.
    pub fn render_one(&self, template_path: &Path, ctx: &TeraContext) -> Result<RenderedTemplate> {
        let path_str = template_path.to_string_lossy();
        anyhow::ensure!(
            path_str.ends_with(".wstemplate"),
            "Template path does not end with .wstemplate: {}",
            template_path.display()
        );

        let output_path = PathBuf::from(&path_str[..path_str.len() - ".wstemplate".len()]);

        let content = fs::read_to_string(template_path)
            .with_context(|| format!("Cannot read {}", template_path.display()))?;

        // Validate all project alias references before rendering.
        validate_aliases(&content, &self.entries, template_path)?;

        let mut tera = Tera::default();
        let tpl_name = template_path.display().to_string();
        tera.add_raw_template(&tpl_name, &content)
            .with_context(|| format!("Tera parse error in {}", template_path.display()))?;

        let rendered = tera
            .render(&tpl_name, ctx)
            .with_context(|| format!("Tera render error in {}", template_path.display()))?;

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Cannot create parent directory for {}", output_path.display())
            })?;
        }

        fs::write(&output_path, &rendered)
            .with_context(|| format!("Cannot write {}", output_path.display()))?;

        Ok(RenderedTemplate {
            source_path: template_path.to_path_buf(),
            output_path,
        })
    }

    /// Render all `.wstemplate` files discovered across all entry roots.
    ///
    /// Individual render failures are logged to stderr and do not abort remaining
    /// templates.  Returns the list of successfully rendered templates.
    pub fn render_all(&self) -> Result<Vec<RenderedTemplate>> {
        let paths = self.discover_all()?;
        let ctx = self.build_context();
        let mut rendered = Vec::new();

        for path in &paths {
            match self.render_one(path, &ctx) {
                Ok(r) => {
                    log::info!(
                        "wstemplate: {} -> {}",
                        r.source_path.display(),
                        r.output_path.display()
                    );
                    rendered.push(r);
                }
                Err(e) => {
                    log::error!("wstemplate render failed for {}: {:#}", path.display(), e);
                    eprintln!(
                        "Error: wstemplate render failed for {}: {:#}",
                        path.display(),
                        e
                    );
                }
            }
        }

        Ok(rendered)
    }

    /// Return the number of registered entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Return all registered entry aliases.
    pub fn aliases(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.alias.as_str()).collect()
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

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

/// Run `rg --files --glob '*.wstemplate' <root>` and return the discovered paths.
///
/// Exit code 0 = matches found; exit code 1 = no matches — both are valid.
/// Any other exit code indicates an `rg` error and propagates.
fn discover_in(root: &Path) -> Result<Vec<PathBuf>> {
    let root_str = root
        .to_str()
        .with_context(|| format!("wstemplate root path is not valid UTF-8: {}", root.display()))?;

    let output = Command::new("rg")
        .args(["--files", "--glob", "*.wstemplate", root_str])
        .output()
        .context("Failed to execute rg. Ensure ripgrep is installed and available in PATH.")?;

    match output.status.code() {
        Some(0) | Some(1) => {
            let stdout =
                String::from_utf8(output.stdout).context("rg produced non-UTF-8 output")?;
            Ok(stdout
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| PathBuf::from(l.trim()))
                .collect())
        }
        Some(code) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("rg exited with error code {}: {}", code, stderr.trim());
        }
        None => anyhow::bail!("rg process was terminated by signal"),
    }
}

/// Scan `template_text` for every `{{ projects.ALIAS` reference and verify that
/// each ALIAS is present in `entries`.
///
/// Any alias not found in the entries is a hard error with a descriptive message that
/// lists the available aliases and suggests the `ws wstemplate add` command.
fn validate_aliases(
    template_text: &str,
    entries: &[EngineEntry],
    template_path: &Path,
) -> Result<()> {
    // Match `projects.ALIAS` where ALIAS is a sequence of word characters.
    let re = Regex::new(r"\bprojects\.(\w+)").expect("static regex must compile");

    let configured: Vec<&str> = entries.iter().map(|e| e.alias.as_str()).collect();

    let mut unknown: Vec<String> = re
        .captures_iter(template_text)
        .map(|cap| cap[1].to_string())
        .filter(|alias| !configured.contains(&alias.as_str()))
        .collect();

    unknown.sort();
    unknown.dedup();

    if !unknown.is_empty() {
        let configured_list = if configured.is_empty() {
            "none configured".to_string()
        } else {
            configured.join(", ")
        };
        anyhow::bail!(
            "Template {} references unknown project alias(es): {}\n\
             Configured aliases: {}\n\
             Add a project with: ws wstemplate add <path> [--alias <name>]",
            template_path.display(),
            unknown.join(", "),
            configured_list,
        );
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_vi(version: &str) -> VersionInfo {
        VersionInfo {
            major_version: "v0".to_string(),
            minor_version: 1,
            patch_version: 0,
            full_version: version.to_string(),
        }
    }

    fn basic_engine() -> WstemplateEngine {
        WstemplateEngine::new(make_vi("1.0.0"), Some("myapp".to_string()))
    }

    fn engine_with_entry(root: &Path, alias: &str, version: &str) -> WstemplateEngine {
        let mut engine = basic_engine();
        engine.add_entry(
            root.to_path_buf(),
            alias.to_string(),
            make_vi(version),
            Some(alias.to_string()),
        );
        engine
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

    // ── discover_all ──────────────────────────────────────────────────────────

    #[test]
    fn test_discover_all_finds_files_in_single_entry() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("a.txt.wstemplate"), "").unwrap();
        fs::write(dir.path().join("sub/b.md.wstemplate"), "").unwrap();
        fs::write(dir.path().join("c.rs"), "").unwrap();

        let engine = engine_with_entry(dir.path(), "proj", "1.0.0");
        let mut paths = engine.discover_all().unwrap();
        paths.sort();

        assert_eq!(paths.len(), 2);
        assert!(paths.iter().any(|p| p.ends_with("a.txt.wstemplate")));
        assert!(paths.iter().any(|p| p.ends_with("b.md.wstemplate")));
    }

    #[test]
    fn test_discover_all_deduplicates_overlapping_roots() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("x.txt.wstemplate"), "").unwrap();

        let mut engine = basic_engine();
        // Add the same root twice with different aliases.
        engine.add_entry(dir.path().to_path_buf(), "a1".to_string(), make_vi("1.0.0"), None);
        engine.add_entry(dir.path().to_path_buf(), "a2".to_string(), make_vi("2.0.0"), None);

        let paths = engine.discover_all().unwrap();
        assert_eq!(paths.len(), 1, "duplicate paths must be deduplicated");
    }

    #[test]
    fn test_discover_all_finds_from_multiple_entry_roots() {
        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();
        fs::write(dir_a.path().join("alpha.wstemplate"), "").unwrap();
        fs::write(dir_b.path().join("beta.wstemplate"), "").unwrap();

        let mut engine = basic_engine();
        engine.add_entry(dir_a.path().to_path_buf(), "a".to_string(), make_vi("1.0.0"), None);
        engine.add_entry(dir_b.path().to_path_buf(), "b".to_string(), make_vi("2.0.0"), None);

        let paths = engine.discover_all().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().any(|p| p.ends_with("alpha.wstemplate")));
        assert!(paths.iter().any(|p| p.ends_with("beta.wstemplate")));
    }

    #[test]
    fn test_discover_respects_gitignore() {
        let dir = TempDir::new().unwrap();
        let init_out = Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("git init must succeed");
        assert!(init_out.status.success());

        fs::write(dir.path().join(".gitignore"), "ignored/\n").unwrap();
        fs::create_dir_all(dir.path().join("ignored")).unwrap();
        fs::write(dir.path().join("ignored/skip.wstemplate"), "").unwrap();
        fs::write(dir.path().join("visible.wstemplate"), "").unwrap();

        let engine = engine_with_entry(dir.path(), "proj", "1.0.0");
        let paths = engine.discover_all().unwrap();

        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("visible.wstemplate"));
    }

    // ── build_context ─────────────────────────────────────────────────────────

    #[test]
    fn test_build_context_current_project_fields() {
        let engine = WstemplateEngine::new(
            VersionInfo {
                major_version: "v2".to_string(),
                minor_version: 10,
                patch_version: 500,
                full_version: "2.10.500".to_string(),
            },
            Some("testproject".to_string()),
        );
        let ctx = engine.build_context();

        let mut tera = Tera::default();
        tera.add_raw_template(
            "t",
            "{{ project.version }}|{{ project.major_version }}|{{ project.minor_version }}|\
             {{ project.patch_version }}|{{ project.name }}|\
             {{ datetime.date }}|{{ datetime.year }}",
        )
        .unwrap();

        let rendered = tera.render("t", &ctx).unwrap();
        let parts: Vec<&str> = rendered.split('|').collect();

        assert_eq!(parts[0], "2.10.500");
        assert_eq!(parts[1], "v2");
        assert_eq!(parts[2], "10");
        assert_eq!(parts[3], "500");
        assert_eq!(parts[4], "testproject");
        assert!(!parts[5].is_empty(), "datetime.date must be non-empty");
        assert!(parts[6].starts_with("20"), "datetime.year must look like 20xx");
    }

    #[test]
    fn test_build_context_includes_entry_as_projects_key() {
        let dir = TempDir::new().unwrap();
        let mut engine = WstemplateEngine::new(make_vi("1.0.0"), Some("main".to_string()));
        engine.add_entry(
            dir.path().to_path_buf(),
            "other_svc".to_string(),
            VersionInfo {
                major_version: "v3".to_string(),
                minor_version: 7,
                patch_version: 42,
                full_version: "3.7.42".to_string(),
            },
            Some("other-service".to_string()),
        );

        let ctx = engine.build_context();
        let mut tera = Tera::default();
        tera.add_raw_template(
            "t",
            "{{ projects.other_svc.version }}|{{ projects.other_svc.name }}",
        )
        .unwrap();

        let rendered = tera.render("t", &ctx).unwrap();
        assert_eq!(rendered, "3.7.42|other-service");
    }

    // ── render_one ────────────────────────────────────────────────────────────

    #[test]
    fn test_render_one_strips_suffix_and_substitutes_version() {
        let dir = TempDir::new().unwrap();
        let tpl = dir.path().join("Cargo.toml.wstemplate");
        fs::write(&tpl, r#"version = "{{ project.version }}""#).unwrap();

        let engine = WstemplateEngine::new(make_vi("0.99.12345"), None);
        let ctx = engine.build_context();
        let result = engine.render_one(&tpl, &ctx).unwrap();

        assert_eq!(result.output_path, dir.path().join("Cargo.toml"));
        assert!(result.output_path.exists());
        let content = fs::read_to_string(&result.output_path).unwrap();
        assert_eq!(content, r#"version = "0.99.12345""#);
    }

    #[test]
    fn test_render_one_nested_path_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let tpl = dir.path().join("sub/dir/README.md.wstemplate");
        fs::create_dir_all(tpl.parent().unwrap()).unwrap();
        fs::write(&tpl, "version: {{ project.version }}").unwrap();

        let engine = WstemplateEngine::new(make_vi("1.2.3"), None);
        let ctx = engine.build_context();
        let result = engine.render_one(&tpl, &ctx).unwrap();

        assert_eq!(result.output_path, dir.path().join("sub/dir/README.md"));
        let content = fs::read_to_string(&result.output_path).unwrap();
        assert_eq!(content, "version: 1.2.3");
    }

    #[test]
    fn test_render_one_rejects_non_wstemplate_path() {
        let dir = TempDir::new().unwrap();
        let bad = dir.path().join("Cargo.toml");
        fs::write(&bad, "irrelevant").unwrap();

        let engine = WstemplateEngine::new(make_vi("0.1.0"), None);
        let ctx = engine.build_context();
        assert!(
            engine.render_one(&bad, &ctx).is_err(),
            "must reject path without .wstemplate suffix"
        );
    }

    #[test]
    fn test_render_one_invalid_tera_syntax_returns_error() {
        let dir = TempDir::new().unwrap();
        let tpl = dir.path().join("bad.txt.wstemplate");
        fs::write(&tpl, "{{ unclosed").unwrap();

        let engine = WstemplateEngine::new(make_vi("0.1.0"), None);
        let ctx = engine.build_context();
        assert!(
            engine.render_one(&tpl, &ctx).is_err(),
            "must propagate Tera parse error"
        );
    }

    // ── alias validation ─────────────────────────────────────────────────────

    #[test]
    fn test_render_one_rejects_unknown_alias() {
        let dir = TempDir::new().unwrap();
        let tpl = dir.path().join("out.txt.wstemplate");
        // References "missing_svc" which is not registered.
        fs::write(&tpl, "ver={{ projects.missing_svc.version }}").unwrap();

        let engine = WstemplateEngine::new(make_vi("1.0.0"), None);
        let ctx = engine.build_context();
        let err = engine.render_one(&tpl, &ctx).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("missing_svc"),
            "error must name the unknown alias, got: {}",
            msg
        );
        assert!(
            msg.contains("ws wstemplate add"),
            "error must suggest the fix command, got: {}",
            msg
        );
    }

    #[test]
    fn test_render_one_unknown_alias_error_lists_configured_aliases() {
        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();
        let tpl = dir_a.path().join("out.txt.wstemplate");
        fs::write(
            &tpl,
            "{{ projects.known.version }} {{ projects.ghost.version }}",
        )
        .unwrap();

        let mut engine = WstemplateEngine::new(make_vi("1.0.0"), None);
        engine.add_entry(dir_b.path().to_path_buf(), "known".to_string(), make_vi("2.0.0"), None);

        let ctx = engine.build_context();
        let err = engine.render_one(&tpl, &ctx).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("ghost"), "unknown alias must appear in error");
        assert!(msg.contains("known"), "configured alias must appear in error");
    }

    #[test]
    fn test_render_one_known_alias_succeeds() {
        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();
        let tpl = dir_a.path().join("out.txt.wstemplate");
        fs::write(&tpl, "{{ projects.sibling.version }}").unwrap();

        let mut engine = WstemplateEngine::new(make_vi("1.0.0"), None);
        engine.add_entry(
            dir_b.path().to_path_buf(),
            "sibling".to_string(),
            make_vi("9.8.7"),
            None,
        );

        let ctx = engine.build_context();
        let result = engine.render_one(&tpl, &ctx).unwrap();
        let content = fs::read_to_string(&result.output_path).unwrap();
        assert_eq!(content, "9.8.7");
    }

    // ── render_all ────────────────────────────────────────────────────────────

    #[test]
    fn test_render_all_produces_all_outputs() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.txt.wstemplate"), "{{ project.version }}").unwrap();
        fs::write(
            dir.path().join("b.txt.wstemplate"),
            "{{ project.name | default(value='unknown') }}",
        )
        .unwrap();
        fs::write(dir.path().join("c.txt.wstemplate"), "{{ datetime.date }}").unwrap();

        let mut engine = WstemplateEngine::new(make_vi("0.5.0"), Some("myproject".to_string()));
        engine.add_entry(dir.path().to_path_buf(), "self".to_string(), make_vi("0.5.0"), Some("myproject".to_string()));

        let rendered = engine.render_all().unwrap();

        assert_eq!(rendered.len(), 3);
        assert!(dir.path().join("a.txt").exists());
        assert!(dir.path().join("b.txt").exists());
        assert!(dir.path().join("c.txt").exists());

        assert_eq!(
            fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "0.5.0"
        );
        assert_eq!(
            fs::read_to_string(dir.path().join("b.txt")).unwrap(),
            "myproject"
        );
    }

    #[test]
    fn test_render_all_continues_past_single_bad_template() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("good1.txt.wstemplate"), "{{ project.version }}").unwrap();
        fs::write(dir.path().join("bad.txt.wstemplate"), "{{ unclosed").unwrap();
        fs::write(dir.path().join("good2.txt.wstemplate"), "ok").unwrap();

        let mut engine = WstemplateEngine::new(make_vi("0.1.0"), None);
        engine.add_entry(dir.path().to_path_buf(), "proj".to_string(), make_vi("0.1.0"), None);

        let rendered = engine.render_all().unwrap();

        assert_eq!(rendered.len(), 2, "must return exactly the two good renders");
        assert!(dir.path().join("good1.txt").exists());
        assert!(dir.path().join("good2.txt").exists());
        assert!(!dir.path().join("bad.txt").exists(), "bad template must not produce output");
    }

    #[test]
    fn test_render_all_cross_project_reference() {
        let dir_main = TempDir::new().unwrap();
        let dir_other = TempDir::new().unwrap();

        // Template lives in dir_main and references a different project's version.
        fs::write(
            dir_main.path().join("VERSIONS.txt.wstemplate"),
            "main={{ project.version }}\nother={{ projects.other_proj.version }}",
        )
        .unwrap();

        let mut engine =
            WstemplateEngine::new(make_vi("1.0.0"), Some("main".to_string()));
        engine.add_entry(
            dir_main.path().to_path_buf(),
            "self_proj".to_string(),
            make_vi("1.0.0"),
            Some("main".to_string()),
        );
        engine.add_entry(
            dir_other.path().to_path_buf(),
            "other_proj".to_string(),
            make_vi("2.5.3"),
            Some("other".to_string()),
        );

        let rendered = engine.render_all().unwrap();
        assert_eq!(rendered.len(), 1);

        let content =
            fs::read_to_string(dir_main.path().join("VERSIONS.txt")).unwrap();
        assert_eq!(content, "main=1.0.0\nother=2.5.3");
    }
}
