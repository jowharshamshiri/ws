use anyhow::{Context, Result};
use log;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct St8Config {
    pub version: u32,
    pub enabled: bool,
    pub version_file: String,
    #[serde(default = "default_auto_detect")]
    pub auto_detect_project_files: bool,
    #[serde(default)]
    pub project_files: Vec<String>,
}

fn default_auto_detect() -> bool {
    true
}

impl Default for St8Config {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: true,
            version_file: "version.txt".to_string(),
            auto_detect_project_files: true,
            project_files: Vec::new(),
        }
    }
}

impl St8Config {
    pub fn load(repo_root: &Path) -> Result<Self> {
        let db_path = repo_root.join(".ws/project.db");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            load_st8_config_from_db(&db_path).await
        })
    }

    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let db_path = repo_root.join(".ws/project.db");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            save_st8_config_to_db(&db_path, self).await
        })
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub major_version: String,
    pub minor_version: u32,
    pub patch_version: u32,
    pub full_version: String,
}

#[derive(Debug, Clone)]
pub struct VersionCalculationInfo {
    pub major_version: u32,
    pub total_commits: u32,
    pub changes_since_release: u32,
    pub last_release_tag: Option<String>,
    pub git_root: Option<PathBuf>,
    pub calculation_method: String,
}

impl VersionInfo {
    pub fn calculate() -> Result<Self> {
        let major_version = get_tag_version()?;
        let minor_version = get_commit_count_since_tag(&major_version)?;
        let patch_version = get_total_changes()?;
        
        let full_version = format!("{}.{}.{}", 
            major_version.strip_prefix('v').unwrap_or(&major_version),
            minor_version,
            patch_version
        );

        Ok(Self {
            major_version,
            minor_version,
            patch_version,
            full_version,
        })
    }

    /// Calculate version with database-stored major version
    pub fn calculate_with_major(major: u32) -> Result<Self> {
        let minor_version = get_total_commit_count()?;
        let patch_version = get_changes_since_last_release_tag(major)?;
        
        let full_version = format!("{}.{}.{}", major, minor_version, patch_version);
        let major_version = format!("v{}", major);

        Ok(Self {
            major_version,
            minor_version,
            patch_version,
            full_version,
        })
    }

    /// Get calculation breakdown for debugging
    pub fn get_calculation_info(major: u32) -> Result<VersionCalculationInfo> {
        let total_commits = get_total_commit_count()?;
        let changes_since_release = get_changes_since_last_release_tag(major)?;
        let last_release_tag = find_last_release_tag(major)?;
        let git_root = get_git_root().ok();

        Ok(VersionCalculationInfo {
            major_version: major,
            total_commits,
            changes_since_release,
            last_release_tag,
            git_root,
            calculation_method: "Database major + git commits + changes since release".to_string(),
        })
    }
}

fn get_tag_version() -> Result<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8(output.stdout)
                .context("Invalid UTF-8 in git tag output")?
                .trim()
                .to_string();
            Ok(version)
        }
        _ => Ok("v0".to_string()),
    }
}

fn get_commit_count_since_tag(tag_version: &str) -> Result<u32> {
    let output = if tag_version == "v0" {
        Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .output()
            .context("Failed to run git rev-list command")?
    } else {
        let range = format!("{}..HEAD", tag_version);
        Command::new("git")
            .args(["rev-list", "--count", &range])
            .output()
            .context("Failed to run git rev-list command")?
    };

    if !output.status.success() {
        return Ok(0);
    }

    let count_str = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git rev-list output")?
        .trim()
        .to_string();

    count_str.parse::<u32>()
        .context("Failed to parse commit count")
}

fn get_total_changes() -> Result<u32> {
    let output = Command::new("git")
        .args(["log", "--pretty=tformat:", "--numstat"])
        .output()
        .context("Failed to run git log command")?;

    if !output.status.success() {
        return Ok(0);
    }

    let log_stat = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git log output")?;

    let mut total = 0u32;
    for line in log_stat.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(additions), Ok(deletions)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                total = total.saturating_add(additions).saturating_add(deletions);
            }
        }
    }

    Ok(total)
}

pub fn update_version_file(version_info: &VersionInfo, config: &St8Config) -> Result<bool> {
    // Skip if no version file is configured
    if config.version_file.is_empty() {
        log::info!("No version file configured, skipping version file update");
        return Ok(false);
    }
    
    // Check if version has actually changed
    let version_file_path = PathBuf::from(&config.version_file);
    let current_version_content = if version_file_path.exists() {
        fs::read_to_string(&version_file_path).unwrap_or_default().trim().to_string()
    } else {
        String::new()
    };
    
    if current_version_content == version_info.full_version {
        log::info!("Version {} is already up to date", version_info.full_version);
        println!("Version {} is already up to date", version_info.full_version);
        return Ok(false);
    }
    
    // Update the main version file
    fs::write(&version_file_path, format!("{}\n", version_info.full_version))
        .with_context(|| format!("Failed to write version to {}", version_file_path.display()))?;

    // Try to stage the version file if we're in a git repository
    if is_git_repository() {
        if let Some(file_str) = version_file_path.to_str() {
            let output = Command::new("git")
                .args(["add", file_str])
                .output();
                
            match output {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        log::warn!("Failed to stage version file {}: {}", file_str, stderr);
                        // Don't fail the entire operation - just warn
                    } else {
                        log::info!("Staged version file: {}", file_str);
                    }
                }
                Err(e) => {
                    log::warn!("Could not run git add for version file: {}", e);
                    // Don't fail - we're not in a git repo or git is not available
                }
            }
        }
    } else {
        log::info!("Not in a git repository, skipping version file staging");
    }

    // Auto-detect and update project files if enabled
    if config.auto_detect_project_files {
        // Try to use git root, but fallback to current directory if not in a git repo
        let project_root = get_git_root().unwrap_or_else(|_| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
        
        match detect_project_files(&project_root) {
            Ok(project_files) => {
                if !project_files.is_empty() {
                    match update_project_files(version_info, &project_files) {
                        Ok(updated_files) => {
                            if !updated_files.is_empty() {
                                log::info!("Updated project files: {}", updated_files.join(", "));
                                println!("Updated project files: {}", updated_files.join(", "));
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to update some project files: {}", e);
                            eprintln!("Warning: Failed to update some project files: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to detect project files: {}", e);
                eprintln!("Warning: Failed to detect project files: {}", e);
            }
        }
    }

    // Update manually specified project files
    if !config.project_files.is_empty() {
        let project_root = get_git_root().unwrap_or_else(|_| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
        
        let manual_files: Vec<ProjectFile> = config.project_files
            .iter()
            .filter_map(|file_path| {
                let full_path = project_root.join(file_path);
                if full_path.exists() {
                    // Try to detect file type from extension/name
                    detect_file_type(&full_path).map(|file_type| ProjectFile {
                        path: full_path,
                        file_type,
                    })
                } else {
                    eprintln!("Warning: Configured project file not found: {}", file_path);
                    None
                }
            })
            .collect();
        
        if !manual_files.is_empty() {
            match update_project_files(version_info, &manual_files) {
                Ok(updated_files) => {
                    if !updated_files.is_empty() {
                        println!("Updated configured project files: {}", updated_files.join(", "));
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to update configured project files: {}", e);
                }
            }
        }
    }

    Ok(true)
}

fn detect_file_type(path: &Path) -> Option<ProjectFileType> {
    let filename = path.file_name()?.to_str()?;
    match filename {
        "Cargo.toml" => Some(ProjectFileType::CargoToml),
        "package.json" => Some(ProjectFileType::PackageJson),
        "pyproject.toml" => Some(ProjectFileType::PyprojectToml),
        "setup.py" => Some(ProjectFileType::SetupPy),
        "composer.json" => Some(ProjectFileType::ComposerJson),
        "pubspec.yaml" => Some(ProjectFileType::PubspecYaml),
        "pom.xml" => Some(ProjectFileType::PomXml),
        "build.gradle" => Some(ProjectFileType::BuildGradle),
        "build.gradle.kts" => Some(ProjectFileType::BuildGradleKts),
        "CMakeLists.txt" => Some(ProjectFileType::CMakeLists),
        "Package.swift" => Some(ProjectFileType::PackageSwift),
        "go.mod" => Some(ProjectFileType::GoMod),
        "mix.exs" => Some(ProjectFileType::MixExs),
        "build.sbt" => Some(ProjectFileType::BuildSbt),
        "shard.yml" => Some(ProjectFileType::ShardYml),
        "Project.toml" => Some(ProjectFileType::JuliaProject),
        _ => {
            // Handle wildcard file types by extension
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                match extension {
                    "json" => Some(ProjectFileType::PackageJson),
                    "gemspec" => Some(ProjectFileType::Gemspec),
                    "csproj" => Some(ProjectFileType::Csproj),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

pub fn is_git_repository() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_git_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to get git root directory")?;

    if !output.status.success() {
        anyhow::bail!("Not in a git repository");
    }

    let root = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git root output")?
        .trim()
        .to_string();

    Ok(PathBuf::from(root))
}

#[derive(Debug, Clone)]
pub struct ProjectFile {
    pub path: PathBuf,
    pub file_type: ProjectFileType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectFileType {
    CargoToml,
    PackageJson,
    PyprojectToml,
    SetupPy,
    ComposerJson,
    PubspecYaml,
    PomXml,
    BuildGradle,
    BuildGradleKts,
    CMakeLists,
    PackageSwift,
    Gemspec,
    Csproj,
    GoMod,
    MixExs,
    BuildSbt,
    ShardYml,
    JuliaProject,
}

impl ProjectFileType {
    pub fn file_name(&self) -> &'static str {
        match self {
            ProjectFileType::CargoToml => "Cargo.toml",
            ProjectFileType::PackageJson => "package.json",
            ProjectFileType::PyprojectToml => "pyproject.toml",
            ProjectFileType::SetupPy => "setup.py",
            ProjectFileType::ComposerJson => "composer.json",
            ProjectFileType::PubspecYaml => "pubspec.yaml",
            ProjectFileType::PomXml => "pom.xml",
            ProjectFileType::BuildGradle => "build.gradle",
            ProjectFileType::BuildGradleKts => "build.gradle.kts",
            ProjectFileType::CMakeLists => "CMakeLists.txt",
            ProjectFileType::PackageSwift => "Package.swift",
            ProjectFileType::Gemspec => "*.gemspec",
            ProjectFileType::Csproj => "*.csproj",
            ProjectFileType::GoMod => "go.mod",
            ProjectFileType::MixExs => "mix.exs",
            ProjectFileType::BuildSbt => "build.sbt",
            ProjectFileType::ShardYml => "shard.yml",
            ProjectFileType::JuliaProject => "Project.toml",
        }
    }
}

pub fn detect_project_files(repo_root: &Path) -> Result<Vec<ProjectFile>> {
    let mut project_files = Vec::new();

    // Define project file types with exact names to detect
    let exact_file_types = [
        ProjectFileType::CargoToml,
        ProjectFileType::PackageJson,
        ProjectFileType::PyprojectToml,
        ProjectFileType::SetupPy,
        ProjectFileType::ComposerJson,
        ProjectFileType::PubspecYaml,
        ProjectFileType::PomXml,
        ProjectFileType::BuildGradle,
        ProjectFileType::BuildGradleKts,
        ProjectFileType::CMakeLists,
        ProjectFileType::PackageSwift,
        ProjectFileType::GoMod,
        ProjectFileType::MixExs,
        ProjectFileType::BuildSbt,
        ProjectFileType::ShardYml,
        ProjectFileType::JuliaProject,
    ];

    for file_type in &exact_file_types {
        let file_path = repo_root.join(file_type.file_name());
        if file_path.exists() {
            project_files.push(ProjectFile {
                path: file_path,
                file_type: file_type.clone(),
            });
        }
    }

    // Detect wildcard file types (*.gemspec, *.csproj)
    if let Ok(entries) = std::fs::read_dir(repo_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "gemspec" => {
                            project_files.push(ProjectFile {
                                path,
                                file_type: ProjectFileType::Gemspec,
                            });
                        }
                        "csproj" => {
                            project_files.push(ProjectFile {
                                path,
                                file_type: ProjectFileType::Csproj,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(project_files)
}

pub fn update_project_files(version_info: &VersionInfo, project_files: &[ProjectFile]) -> Result<Vec<String>> {
    let mut updated_files = Vec::new();
    
    for project_file in project_files {
        match update_project_file(version_info, project_file) {
            Ok(()) => {
                updated_files.push(project_file.path.display().to_string());
                
                // Try to stage the updated file if we're in a git repository
                if is_git_repository() {
                    if let Some(file_str) = project_file.path.to_str() {
                        let output = Command::new("git")
                            .args(["add", file_str])
                            .output();
                            
                        match output {
                            Ok(output) => {
                                if !output.status.success() {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    log::warn!("Failed to stage project file {}: {}", file_str, stderr);
                                } else {
                                    log::info!("Staged project file: {}", file_str);
                                }
                            }
                            Err(e) => {
                                log::warn!("Could not run git add for project file: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to update {}: {}", project_file.path.display(), e);
            }
        }
    }
    
    Ok(updated_files)
}

pub fn update_project_file(version_info: &VersionInfo, project_file: &ProjectFile) -> Result<()> {
    let content = fs::read_to_string(&project_file.path)
        .with_context(|| format!("Failed to read {}", project_file.path.display()))?;

    let updated_content = match project_file.file_type {
        ProjectFileType::CargoToml => update_cargo_toml(&content, &version_info.full_version)?,
        ProjectFileType::PackageJson => update_package_json(&content, &version_info.full_version)?,
        ProjectFileType::PyprojectToml => update_pyproject_toml(&content, &version_info.full_version)?,
        ProjectFileType::SetupPy => update_setup_py(&content, &version_info.full_version)?,
        ProjectFileType::ComposerJson => update_composer_json(&content, &version_info.full_version)?,
        ProjectFileType::PubspecYaml => update_pubspec_yaml(&content, &version_info.full_version)?,
        ProjectFileType::PomXml => update_pom_xml(&content, &version_info.full_version)?,
        ProjectFileType::BuildGradle => update_build_gradle(&content, &version_info.full_version)?,
        ProjectFileType::BuildGradleKts => update_build_gradle_kts(&content, &version_info.full_version)?,
        ProjectFileType::CMakeLists => update_cmake_lists(&content, &version_info.full_version)?,
        ProjectFileType::PackageSwift => update_package_swift(&content, &version_info.full_version)?,
        ProjectFileType::Gemspec => update_gemspec(&content, &version_info.full_version)?,
        ProjectFileType::Csproj => update_csproj(&content, &version_info.full_version)?,
        ProjectFileType::GoMod => update_go_mod(&content, &version_info.full_version)?,
        ProjectFileType::MixExs => update_mix_exs(&content, &version_info.full_version)?,
        ProjectFileType::BuildSbt => update_build_sbt(&content, &version_info.full_version)?,
        ProjectFileType::ShardYml => update_shard_yml(&content, &version_info.full_version)?,
        ProjectFileType::JuliaProject => update_julia_project(&content, &version_info.full_version)?,
    };

    fs::write(&project_file.path, updated_content)
        .with_context(|| format!("Failed to write updated {}", project_file.path.display()))?;

    Ok(())
}

fn update_cargo_toml(content: &str, version: &str) -> Result<String> {
    let mut parsed: toml::Value = content.parse()
        .context("Failed to parse Cargo.toml")?;
    
    if let Some(package) = parsed.get_mut("package").and_then(|p| p.as_table_mut()) {
        package.insert("version".to_string(), toml::Value::String(version.to_string()));
    }
    
    Ok(toml::to_string(&parsed)?)
}

fn update_package_json(content: &str, version: &str) -> Result<String> {
    let mut parsed: serde_json::Value = serde_json::from_str(content)
        .context("Failed to parse package.json")?;
    
    if let Some(obj) = parsed.as_object_mut() {
        obj.insert("version".to_string(), serde_json::Value::String(version.to_string()));
    }
    
    Ok(serde_json::to_string_pretty(&parsed)?)
}

fn update_pyproject_toml(content: &str, version: &str) -> Result<String> {
    let mut parsed: toml::Value = content.parse()
        .context("Failed to parse pyproject.toml")?;
    
    // Try tool.poetry.version first, then project.version
    if let Some(tool) = parsed.get_mut("tool") {
        if let Some(poetry) = tool.get_mut("poetry").and_then(|p| p.as_table_mut()) {
            poetry.insert("version".to_string(), toml::Value::String(version.to_string()));
        }
    }
    
    if let Some(project) = parsed.get_mut("project").and_then(|p| p.as_table_mut()) {
        project.insert("version".to_string(), toml::Value::String(version.to_string()));
    }
    
    Ok(toml::to_string(&parsed)?)
}

fn update_setup_py(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r#"version\s*=\s*["'][^"']*["']"#)
        .context("Failed to create regex for setup.py")?;
    
    let updated = version_regex.replace_all(content, &format!(r#"version="{}""#, version));
    Ok(updated.to_string())
}

fn update_composer_json(content: &str, version: &str) -> Result<String> {
    let mut parsed: serde_json::Value = serde_json::from_str(content)
        .context("Failed to parse composer.json")?;
    
    if let Some(obj) = parsed.as_object_mut() {
        obj.insert("version".to_string(), serde_json::Value::String(version.to_string()));
    }
    
    Ok(serde_json::to_string_pretty(&parsed)?)
}

fn update_pubspec_yaml(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r"(?m)^version:\s*.*$")
        .context("Failed to create regex for pubspec.yaml")?;
    
    let updated = version_regex.replace_all(content, &format!("version: {}", version));
    Ok(updated.to_string())
}

fn update_pom_xml(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r"<version>[^<]*</version>")
        .context("Failed to create regex for pom.xml")?;
    
    let updated = version_regex.replace(content, &format!("<version>{}</version>", version));
    Ok(updated.to_string())
}

fn update_build_gradle(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r#"version\s*=\s*['"][^'"]*['"]"#)
        .context("Failed to create regex for build.gradle")?;
    
    let updated = version_regex.replace_all(content, &format!(r#"version = '{}'"#, version));
    Ok(updated.to_string())
}

fn update_cmake_lists(content: &str, version: &str) -> Result<String> {
    let version_regex = Regex::new(r"(?i)project\s*\([^)]*VERSION\s+[^\s)]+")
        .context("Failed to create regex for CMakeLists.txt")?;

    let updated = version_regex.replace_all(content, |caps: &regex::Captures| {
        let matched = caps.get(0).unwrap().as_str();
        let version_part_regex = Regex::new(r"VERSION\s+[^\s)]+").unwrap();
        version_part_regex.replace(matched, &format!("VERSION {}", version)).to_string()
    });

    Ok(updated.to_string())
}

fn update_build_gradle_kts(content: &str, version: &str) -> Result<String> {
    // Kotlin DSL uses version = "x.y.z" syntax
    let version_regex = Regex::new(r#"version\s*=\s*"[^"]*""#)
        .context("Failed to create regex for build.gradle.kts")?;

    let updated = version_regex.replace_all(content, &format!(r#"version = "{}""#, version));
    Ok(updated.to_string())
}

fn update_package_swift(content: &str, version: &str) -> Result<String> {
    // Swift Package Manager uses version in Package.swift
    // Example: let package = Package(name: "MyPackage", version: "1.0.0", ...)
    // Or in products/targets with version comments
    let version_regex = Regex::new(r#"//\s*version:\s*[^\n]*"#)
        .context("Failed to create regex for Package.swift version comment")?;

    // First try to update version comment
    let updated = if version_regex.is_match(content) {
        version_regex.replace_all(content, &format!("// version: {}", version)).to_string()
    } else {
        // Try to update version string in Package initializer
        let pkg_version_regex = Regex::new(r#"version\s*:\s*"[^"]*""#)
            .context("Failed to create regex for Package.swift")?;
        if pkg_version_regex.is_match(content) {
            pkg_version_regex.replace_all(content, &format!(r#"version: "{}""#, version)).to_string()
        } else {
            // Add version comment at the top of the file
            format!("// version: {}\n{}", version, content)
        }
    };

    Ok(updated)
}

fn update_gemspec(content: &str, version: &str) -> Result<String> {
    // Ruby gemspec: spec.version = "1.0.0" or s.version = "1.0.0"
    let version_regex = Regex::new(r#"(\w+)\.version\s*=\s*["'][^"']*["']"#)
        .context("Failed to create regex for gemspec")?;

    let updated = version_regex.replace_all(content, |caps: &regex::Captures| {
        let var_name = caps.get(1).unwrap().as_str();
        format!(r#"{}.version = "{}""#, var_name, version)
    });

    Ok(updated.to_string())
}

fn update_csproj(content: &str, version: &str) -> Result<String> {
    // .NET csproj: <Version>1.0.0</Version>
    let version_regex = Regex::new(r"<Version>[^<]*</Version>")
        .context("Failed to create regex for csproj Version")?;

    let updated = if version_regex.is_match(content) {
        version_regex.replace_all(content, &format!("<Version>{}</Version>", version)).to_string()
    } else {
        // Try PackageVersion tag
        let pkg_version_regex = Regex::new(r"<PackageVersion>[^<]*</PackageVersion>")
            .context("Failed to create regex for csproj PackageVersion")?;
        if pkg_version_regex.is_match(content) {
            pkg_version_regex.replace_all(content, &format!("<PackageVersion>{}</PackageVersion>", version)).to_string()
        } else {
            // Try AssemblyVersion tag
            let asm_version_regex = Regex::new(r"<AssemblyVersion>[^<]*</AssemblyVersion>")
                .context("Failed to create regex for csproj AssemblyVersion")?;
            asm_version_regex.replace_all(content, &format!("<AssemblyVersion>{}</AssemblyVersion>", version)).to_string()
        }
    };

    Ok(updated)
}

fn update_go_mod(content: &str, version: &str) -> Result<String> {
    // Go modules don't have a version field in go.mod for the main module
    // Version is typically managed via git tags
    // However, we can add/update a version comment
    let version_regex = Regex::new(r"//\s*version:\s*[^\n]*")
        .context("Failed to create regex for go.mod version comment")?;

    let updated = if version_regex.is_match(content) {
        version_regex.replace_all(content, &format!("// version: {}", version)).to_string()
    } else {
        // Add version comment after the module line
        let module_regex = Regex::new(r"(module\s+[^\n]+)")
            .context("Failed to create regex for go.mod module")?;
        module_regex.replace(content, &format!("$1\n// version: {}", version)).to_string()
    };

    Ok(updated)
}

fn update_mix_exs(content: &str, version: &str) -> Result<String> {
    // Elixir mix.exs: version: "1.0.0"
    let version_regex = Regex::new(r#"version:\s*"[^"]*""#)
        .context("Failed to create regex for mix.exs")?;

    let updated = version_regex.replace_all(content, &format!(r#"version: "{}""#, version));
    Ok(updated.to_string())
}

fn update_build_sbt(content: &str, version: &str) -> Result<String> {
    // Scala build.sbt: version := "1.0.0"
    let version_regex = Regex::new(r#"version\s*:=\s*"[^"]*""#)
        .context("Failed to create regex for build.sbt")?;

    let updated = version_regex.replace_all(content, &format!(r#"version := "{}""#, version));
    Ok(updated.to_string())
}

fn update_shard_yml(content: &str, version: &str) -> Result<String> {
    // Crystal shard.yml: version: 1.0.0
    let version_regex = Regex::new(r"(?m)^version:\s*.*$")
        .context("Failed to create regex for shard.yml")?;

    let updated = version_regex.replace_all(content, &format!("version: {}", version));
    Ok(updated.to_string())
}

fn update_julia_project(content: &str, version: &str) -> Result<String> {
    // Julia Project.toml: version = "1.0.0"
    let mut parsed: toml::Value = content.parse()
        .context("Failed to parse Julia Project.toml")?;

    if let Some(table) = parsed.as_table_mut() {
        table.insert("version".to_string(), toml::Value::String(version.to_string()));
    }

    Ok(toml::to_string(&parsed)?)
}

// Database integration functions for St8Config
async fn load_st8_config_from_db(db_path: &std::path::Path) -> Result<St8Config> {
    use sqlx::{SqlitePool, migrate::MigrateDatabase, Row};
    
    // Ensure database exists and is initialized
    let database_url = format!("sqlite:{}", db_path.display());
    if !sqlx::Sqlite::database_exists(&database_url).await.unwrap_or(false) {
        sqlx::Sqlite::create_database(&database_url).await?;
    }
    
    let pool = SqlitePool::connect(&database_url).await?;
    
    // Initialize database tables if needed
    super::super::entities::database::initialize_database(db_path).await?;
    
    // Try to get config from existing project
    let result = sqlx::query(r#"
        SELECT version_file, auto_detect_project_files, project_files 
        FROM projects 
        LIMIT 1
    "#)
    .fetch_optional(&pool)
    .await?;
    
    if let Some(row) = result {
        let project_files: Vec<String> = if let Some(json_str) = row.get::<Option<String>, _>("project_files") {
            serde_json::from_str(&json_str).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        Ok(St8Config {
            version: 1,
            enabled: true, // Default to enabled since column removed
            version_file: row.get::<String, _>("version_file"),
            auto_detect_project_files: row.get::<bool, _>("auto_detect_project_files"),
            project_files,
        })
    } else {
        // No project exists, create default project with config
        let default_config = St8Config::default();
        create_default_project_with_config(&pool, &default_config).await?;
        Ok(default_config)
    }
}

async fn save_st8_config_to_db(db_path: &std::path::Path, config: &St8Config) -> Result<()> {
    use sqlx::SqlitePool;
    
    let database_url = format!("sqlite:{}", db_path.display());
    let pool = SqlitePool::connect(&database_url).await?;
    
    let project_files_json = serde_json::to_string(&config.project_files)?;
    
    sqlx::query(r#"
        UPDATE projects 
        SET version_file = ?, 
            auto_detect_project_files = ?, 
            project_files = ?,
            updated_at = datetime('now')
        WHERE id = (SELECT id FROM projects LIMIT 1)
    "#)
    .bind(&config.version_file)
    .bind(config.auto_detect_project_files)
    .bind(project_files_json)
    .execute(&pool)
    .await?;
    
    Ok(())
}

async fn create_default_project_with_config(pool: &sqlx::SqlitePool, config: &St8Config) -> Result<()> {
    let project_files_json = serde_json::to_string(&config.project_files)?;
    
    sqlx::query(r#"
        INSERT INTO projects (
            id, name, description, status, version, major_version,
            version_file, auto_detect_project_files, project_files
        ) VALUES (
            'P001', 'Default Project', 'Auto-created project', 'active', '0.1.0', 0,
            ?, ?, ?
        )
    "#)
    .bind(&config.version_file)
    .bind(config.auto_detect_project_files)
    .bind(project_files_json)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Get total commit count (each commit advances minor version)
fn get_total_commit_count() -> Result<u32> {
    let output = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .context("Failed to run git rev-list command")?;

    if !output.status.success() {
        return Ok(0);
    }

    let count_str = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git rev-list output")?
        .trim()
        .to_string();

    count_str.parse::<u32>()
        .context("Failed to parse commit count")
}

/// Get changes since last release tag for this major version
fn get_changes_since_last_release_tag(major: u32) -> Result<u32> {
    let last_tag = find_last_release_tag(major)?;
    
    let output = if let Some(tag) = last_tag {
        // Count changes since the last release tag
        let range = format!("{}..HEAD", tag);
        Command::new("git")
            .args(["log", "--pretty=tformat:", "--numstat", &range])
            .output()
            .context("Failed to run git log command")?
    } else {
        // No release tags for this major version, count all changes
        Command::new("git")
            .args(["log", "--pretty=tformat:", "--numstat"])
            .output()
            .context("Failed to run git log command")?
    };

    if !output.status.success() {
        return Ok(0);
    }

    let log_stat = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git log output")?;

    let mut total = 0u32;
    for line in log_stat.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(additions), Ok(deletions)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                total = total.saturating_add(additions).saturating_add(deletions);
            }
        }
    }

    Ok(total)
}

/// Find the most recent release tag for this major version (v{major}.*)
fn find_last_release_tag(major: u32) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["tag", "--list", &format!("v{}.*", major), "--sort=-version:refname"])
        .output()
        .context("Failed to run git tag command")?;

    if !output.status.success() {
        return Ok(None);
    }

    let tags_output = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git tag output")?;

    // Return the first (most recent) tag for this major version
    for line in tags_output.lines() {
        let tag = line.trim();
        if !tag.is_empty() {
            return Ok(Some(tag.to_string()));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_st8_config_default() {
        let config = St8Config::default();
        assert_eq!(config.version, 1);
        assert!(config.enabled);
        assert_eq!(config.version_file, "version.txt");
    }

    fn create_temp_st8_config_file(dir: &Path, config: &St8Config) -> Result<()> {
        let config_path = dir.join(".st8.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(config)?)?;
        Ok(())
    }

    fn load_st8_config_file_only(dir: &Path) -> Result<St8Config> {
        let config_path = dir.join(".st8.json");
        let content = std::fs::read_to_string(&config_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    #[test]
    fn test_st8_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config = St8Config::default();
        
        // Use file-only operations for unit test
        create_temp_st8_config_file(temp_dir.path(), &config).unwrap();
        let loaded_config = load_st8_config_file_only(temp_dir.path()).unwrap();
        
        assert_eq!(config.version, loaded_config.version);
        assert_eq!(config.enabled, loaded_config.enabled);
        assert_eq!(config.version_file, loaded_config.version_file);
    }

    #[test]
    fn test_version_info_format() {
        let version_info = VersionInfo {
            major_version: "v1.0".to_string(),
            minor_version: 5,
            patch_version: 100,
            full_version: "1.0.5.100".to_string(),
        };
        
        assert_eq!(version_info.full_version, "1.0.5.100");
    }

    #[test]
    fn test_is_git_repository() {
        // This test will pass if run in a git repository
        // In CI/testing environments, this might be false
        let _ = is_git_repository();
    }

    #[test]
    fn test_project_file_type_file_name() {
        assert_eq!(ProjectFileType::CargoToml.file_name(), "Cargo.toml");
        assert_eq!(ProjectFileType::PackageJson.file_name(), "package.json");
        assert_eq!(ProjectFileType::PyprojectToml.file_name(), "pyproject.toml");
        assert_eq!(ProjectFileType::SetupPy.file_name(), "setup.py");
        assert_eq!(ProjectFileType::ComposerJson.file_name(), "composer.json");
        assert_eq!(ProjectFileType::PubspecYaml.file_name(), "pubspec.yaml");
        assert_eq!(ProjectFileType::PomXml.file_name(), "pom.xml");
        assert_eq!(ProjectFileType::BuildGradle.file_name(), "build.gradle");
        assert_eq!(ProjectFileType::BuildGradleKts.file_name(), "build.gradle.kts");
        assert_eq!(ProjectFileType::CMakeLists.file_name(), "CMakeLists.txt");
        assert_eq!(ProjectFileType::PackageSwift.file_name(), "Package.swift");
        assert_eq!(ProjectFileType::Gemspec.file_name(), "*.gemspec");
        assert_eq!(ProjectFileType::Csproj.file_name(), "*.csproj");
        assert_eq!(ProjectFileType::GoMod.file_name(), "go.mod");
        assert_eq!(ProjectFileType::MixExs.file_name(), "mix.exs");
        assert_eq!(ProjectFileType::BuildSbt.file_name(), "build.sbt");
        assert_eq!(ProjectFileType::ShardYml.file_name(), "shard.yml");
        assert_eq!(ProjectFileType::JuliaProject.file_name(), "Project.toml");
    }

    #[test]
    fn test_detect_file_type() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test Cargo.toml detection
        let cargo_path = temp_dir.path().join("Cargo.toml");
        assert_eq!(detect_file_type(&cargo_path), Some(ProjectFileType::CargoToml));
        
        // Test package.json detection
        let package_path = temp_dir.path().join("package.json");
        assert_eq!(detect_file_type(&package_path), Some(ProjectFileType::PackageJson));
        
        // Test generic JSON file detection
        let custom_json_path = temp_dir.path().join("custom.json");
        assert_eq!(detect_file_type(&custom_json_path), Some(ProjectFileType::PackageJson));
        
        // Test unknown file
        let unknown_path = temp_dir.path().join("unknown.txt");
        assert_eq!(detect_file_type(&unknown_path), None);
    }

    #[test]
    fn test_detect_project_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some project files
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"").unwrap();
        fs::write(temp_dir.path().join("package.json"), "{\"name\": \"test\", \"version\": \"1.0.0\"}").unwrap();
        
        let project_files = detect_project_files(temp_dir.path()).unwrap();
        assert_eq!(project_files.len(), 2);
        
        let file_types: Vec<_> = project_files.iter().map(|f| &f.file_type).collect();
        assert!(file_types.contains(&&ProjectFileType::CargoToml));
        assert!(file_types.contains(&&ProjectFileType::PackageJson));
    }

    #[test]
    fn test_update_cargo_toml() {
        let content = r#"[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
        
        let updated = update_cargo_toml(content, "1.2.3").unwrap();
        assert!(updated.contains("version = \"1.2.3\""));
        assert!(updated.contains("name = \"test-package\""));
        assert!(updated.contains("serde = \"1.0\""));
    }

    #[test]
    fn test_update_package_json() {
        let content = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "description": "A test package",
  "main": "index.js"
}"#;
        
        let updated = update_package_json(content, "2.1.0").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(parsed["name"], "test-package");
    }

    #[test]
    fn test_update_pyproject_toml() {
        let content = r#"[tool.poetry]
name = "test-package"
version = "0.1.0"
description = "A test package"

[project]
name = "test-package"
version = "0.1.0"
"#;
        
        let updated = update_pyproject_toml(content, "1.5.2").unwrap();
        assert!(updated.contains("version = \"1.5.2\""));
        // Should update both poetry and project sections
        let version_count = updated.matches("version = \"1.5.2\"").count();
        assert_eq!(version_count, 2);
    }

    #[test]
    fn test_update_setup_py() {
        let content = r#"from setuptools import setup

setup(
    name="test-package",
    version="0.1.0",
    description="A test package",
    author="Test Author",
    packages=["test_package"],
)
"#;
        
        let updated = update_setup_py(content, "2.0.1").unwrap();
        assert!(updated.contains("version=\"2.0.1\""));
        assert!(updated.contains("name=\"test-package\""));
    }

    #[test]
    fn test_update_composer_json() {
        let content = r#"{
    "name": "vendor/package",
    "version": "1.0.0",
    "description": "A test package",
    "type": "library"
}"#;
        
        let updated = update_composer_json(content, "1.5.0").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(parsed["version"], "1.5.0");
        assert_eq!(parsed["name"], "vendor/package");
    }

    #[test]
    fn test_update_pubspec_yaml() {
        let content = r#"name: test_package
description: A test Flutter package
version: 1.0.0+1
environment:
  sdk: ">=2.12.0 <4.0.0"
"#;
        
        let updated = update_pubspec_yaml(content, "2.1.0+5").unwrap();
        assert!(updated.contains("version: 2.1.0+5"));
        assert!(updated.contains("name: test_package"));
    }

    #[test]
    fn test_update_pom_xml() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test-project</artifactId>
    <version>1.0.0</version>
    <packaging>jar</packaging>
</project>
"#;
        
        let updated = update_pom_xml(content, "2.3.1").unwrap();
        assert!(updated.contains("<version>2.3.1</version>"));
        assert!(updated.contains("<artifactId>test-project</artifactId>"));
    }

    #[test]
    fn test_update_build_gradle() {
        let content = r#"plugins {
    id 'java'
}

group = 'com.example'
version = '1.0.0'
sourceCompatibility = '11'

repositories {
    mavenCentral()
}
"#;
        
        let updated = update_build_gradle(content, "3.2.1").unwrap();
        assert!(updated.contains("version = '3.2.1'"));
        assert!(updated.contains("group = 'com.example'"));
    }

    #[test]
    fn test_update_cmake_lists() {
        let content = r#"cmake_minimum_required(VERSION 3.16)

project(TestProject
    VERSION 1.0.0
    DESCRIPTION "A test project"
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 17)
"#;
        
        let updated = update_cmake_lists(content, "2.1.5").unwrap();
        assert!(updated.contains("VERSION 2.1.5"));
        assert!(updated.contains("project(TestProject"));
    }

    #[test]
    fn test_st8_config_with_auto_detect() {
        let temp_dir = TempDir::new().unwrap();
        
        let config = St8Config {
            version: 1,
            enabled: true,
            version_file: "version.txt".to_string(),
            auto_detect_project_files: true,
            project_files: vec!["custom.toml".to_string()],
        };
        
        // Use file-only operations for unit test
        create_temp_st8_config_file(temp_dir.path(), &config).unwrap();
        let loaded_config = load_st8_config_file_only(temp_dir.path()).unwrap();
        
        assert_eq!(loaded_config.auto_detect_project_files, true);
        assert_eq!(loaded_config.project_files, vec!["custom.toml"]);
    }

    #[test]
    fn test_st8_config_default_auto_detect() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test that auto_detect defaults to true when not specified
        let config_content = r#"{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt"
}"#;
        std::fs::write(temp_dir.path().join(".st8.json"), config_content).unwrap();
        
        let loaded_config = load_st8_config_file_only(temp_dir.path()).unwrap();
        assert_eq!(loaded_config.auto_detect_project_files, true);
        assert!(loaded_config.project_files.is_empty());
    }

    #[test]
    fn test_update_version_file_no_change() {
        let temp_dir = TempDir::new().unwrap();
        let config = St8Config::default();

        let version_info = VersionInfo {
            major_version: "v1.0".to_string(),
            minor_version: 5,
            patch_version: 100,
            full_version: "1.0.5.100".to_string(),
        };

        // Create version file with same version
        let version_file_path = temp_dir.path().join("version.txt");
        fs::write(&version_file_path, "1.0.5.100\n").unwrap();

        // Change working directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Update should detect no change
        let result = update_version_file(&version_info, &config);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Should return false indicating no update

        // File should still exist with same content
        let content = fs::read_to_string(&version_file_path).unwrap();
        assert_eq!(content.trim(), "1.0.5.100");
    }

    #[test]
    fn test_update_build_gradle_kts() {
        let content = r#"plugins {
    kotlin("jvm") version "1.9.0"
}

group = "com.example"
version = "1.0.0"

repositories {
    mavenCentral()
}
"#;

        let updated = update_build_gradle_kts(content, "2.5.0").unwrap();
        assert!(updated.contains(r#"version = "2.5.0""#));
        assert!(updated.contains(r#"group = "com.example""#));
    }

    #[test]
    fn test_update_package_swift() {
        let content = r#"// swift-tools-version: 5.9
// version: 1.0.0
import PackageDescription

let package = Package(
    name: "MyPackage",
    products: [
        .library(name: "MyPackage", targets: ["MyPackage"]),
    ],
    targets: [
        .target(name: "MyPackage"),
    ]
)
"#;

        let updated = update_package_swift(content, "2.0.0").unwrap();
        assert!(updated.contains("// version: 2.0.0"));
    }

    #[test]
    fn test_update_gemspec() {
        let content = r#"Gem::Specification.new do |spec|
  spec.name          = "my_gem"
  spec.version       = "1.0.0"
  spec.authors       = ["Author"]
  spec.summary       = "A test gem"
end
"#;

        let updated = update_gemspec(content, "1.5.0").unwrap();
        assert!(updated.contains(r#"spec.version = "1.5.0""#));
        assert!(updated.contains(r#"spec.name          = "my_gem""#));
    }

    #[test]
    fn test_update_csproj() {
        let content = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <Version>1.0.0</Version>
  </PropertyGroup>
</Project>
"#;

        let updated = update_csproj(content, "2.1.0").unwrap();
        assert!(updated.contains("<Version>2.1.0</Version>"));
    }

    #[test]
    fn test_update_go_mod() {
        let content = r#"module github.com/example/myproject

go 1.21
"#;

        let updated = update_go_mod(content, "1.2.0").unwrap();
        assert!(updated.contains("// version: 1.2.0"));
        assert!(updated.contains("module github.com/example/myproject"));
    }

    #[test]
    fn test_update_mix_exs() {
        let content = r#"defmodule MyApp.MixProject do
  use Mix.Project

  def project do
    [
      app: :my_app,
      version: "0.1.0",
      elixir: "~> 1.14",
      deps: deps()
    ]
  end
end
"#;

        let updated = update_mix_exs(content, "1.0.0").unwrap();
        assert!(updated.contains(r#"version: "1.0.0""#));
    }

    #[test]
    fn test_update_build_sbt() {
        let content = r#"name := "my-project"
version := "0.1.0"
scalaVersion := "3.3.0"
"#;

        let updated = update_build_sbt(content, "1.0.0").unwrap();
        assert!(updated.contains(r#"version := "1.0.0""#));
        assert!(updated.contains(r#"name := "my-project""#));
    }

    #[test]
    fn test_update_shard_yml() {
        let content = r#"name: my_shard
version: 0.1.0

authors:
  - Author

crystal: ">= 1.0.0"
"#;

        let updated = update_shard_yml(content, "1.0.0").unwrap();
        assert!(updated.contains("version: 1.0.0"));
        assert!(updated.contains("name: my_shard"));
    }

    #[test]
    fn test_update_julia_project() {
        let content = r#"name = "MyPackage"
uuid = "12345678-1234-5678-1234-567812345678"
version = "0.1.0"

[deps]
"#;

        let updated = update_julia_project(content, "1.0.0").unwrap();
        assert!(updated.contains("version = \"1.0.0\""));
        assert!(updated.contains("name = \"MyPackage\""));
    }

    #[test]
    fn test_detect_new_file_types() {
        let temp_dir = TempDir::new().unwrap();

        // Test new file type detection
        let swift_path = temp_dir.path().join("Package.swift");
        assert_eq!(detect_file_type(&swift_path), Some(ProjectFileType::PackageSwift));

        let gradle_kts_path = temp_dir.path().join("build.gradle.kts");
        assert_eq!(detect_file_type(&gradle_kts_path), Some(ProjectFileType::BuildGradleKts));

        let go_mod_path = temp_dir.path().join("go.mod");
        assert_eq!(detect_file_type(&go_mod_path), Some(ProjectFileType::GoMod));

        let mix_path = temp_dir.path().join("mix.exs");
        assert_eq!(detect_file_type(&mix_path), Some(ProjectFileType::MixExs));

        let sbt_path = temp_dir.path().join("build.sbt");
        assert_eq!(detect_file_type(&sbt_path), Some(ProjectFileType::BuildSbt));

        let shard_path = temp_dir.path().join("shard.yml");
        assert_eq!(detect_file_type(&shard_path), Some(ProjectFileType::ShardYml));

        let julia_path = temp_dir.path().join("Project.toml");
        assert_eq!(detect_file_type(&julia_path), Some(ProjectFileType::JuliaProject));

        // Test wildcard extensions
        let gemspec_path = temp_dir.path().join("my_gem.gemspec");
        assert_eq!(detect_file_type(&gemspec_path), Some(ProjectFileType::Gemspec));

        let csproj_path = temp_dir.path().join("MyProject.csproj");
        assert_eq!(detect_file_type(&csproj_path), Some(ProjectFileType::Csproj));
    }

    #[test]
    fn test_detect_wildcard_project_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create wildcard-pattern files
        fs::write(temp_dir.path().join("my_gem.gemspec"), "Gem::Specification.new").unwrap();
        fs::write(temp_dir.path().join("MyApp.csproj"), "<Project></Project>").unwrap();

        let project_files = detect_project_files(temp_dir.path()).unwrap();

        let file_types: Vec<_> = project_files.iter().map(|f| &f.file_type).collect();
        assert!(file_types.contains(&&ProjectFileType::Gemspec));
        assert!(file_types.contains(&&ProjectFileType::Csproj));
    }
}