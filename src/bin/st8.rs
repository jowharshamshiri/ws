use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use nomion::st8::{St8Config, VersionInfo, detect_project_files, update_version_file, TemplateManager};
use nomion::nomion_state::NomionState;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "st8")]
#[command(about = "Automatic version bumping based on git commits and changes - part of the nomion tool suite")]
#[command(version = nomion::get_version())]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Install st8 as a pre-commit hook in the current git repository
    Install {
        /// Force reinstallation even if already installed
        #[arg(short, long)]
        force: bool,
    },
    
    /// Remove st8 from pre-commit hooks
    #[command(alias = "unhook")]
    Uninstall,
    
    /// Show current version information without updating
    Show,
    
    /// Update project state (version, templates) manually (normally done automatically via git hook)
    Update {
        /// Run update even when not in a git repository
        #[arg(long)]
        no_git: bool,
        /// Add updated files to git staging area after state update
        #[arg(long)]
        git_add: bool,
    },
    
    /// Show st8 status and configuration
    Status,
    
    /// Template management commands
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
}

#[derive(Subcommand, Debug)]
enum TemplateAction {
    /// Add a new template
    Add {
        /// Template name
        name: String,
        /// Path to template file or template content
        #[arg(short, long)]
        template: String,
        /// Output path for rendered template
        #[arg(short, long)]
        output: String,
        /// Template description
        #[arg(short, long)]
        description: Option<String>,
    },
    
    /// Remove a template
    Remove {
        /// Template name to remove
        name: String,
    },
    
    /// List all templates
    List,
    
    /// Enable or disable a template
    Enable {
        /// Template name
        name: String,
        /// Disable the template instead of enabling it
        #[arg(short, long)]
        disable: bool,
    },
    
    /// Render all enabled templates
    Render,
    
    /// Show template details
    Show {
        /// Template name
        name: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {:#}", "Error".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Some(Commands::Install { force }) => install_hook(force)?,
        Some(Commands::Uninstall) => uninstall_hook()?,
        Some(Commands::Show) => show_version()?,
        Some(Commands::Update { no_git, git_add }) => update_state(no_git, git_add)?,
        Some(Commands::Status) => show_status()?,
        Some(Commands::Template { action }) => handle_template_command(action)?,
        None => {
            // Default behavior: install hook if not installed, otherwise update state
            if is_hook_installed()? {
                update_state(false, false)?;
            } else {
                install_hook(false)?;
            }
        }
    }
    
    Ok(())
}

fn install_hook(force: bool) -> Result<()> {
    if !is_git_repository() {
        anyhow::bail!("Not in a git repository. Please run st8 from within a git repository.");
    }
    
    let git_root = get_git_root()?;
    let hooks_dir = git_root.join(".git").join("hooks");
    let hook_file = hooks_dir.join("pre-commit");
    
    // Create hooks directory if it doesn't exist
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)
            .context("Failed to create git hooks directory")?;
        log_action(&format!("Created git hooks directory: {}", hooks_dir.display()));
    }
    
    // Check if already installed
    if !force && is_hook_installed()? {
        println!("{} st8 is already installed as a pre-commit hook", "Info".blue());
        println!("{} Use 'st8 install --force' to reinstall", "Tip".yellow());
        return Ok(());
    }
    
    // Get current binary path
    let current_exe = env::current_exe()
        .context("Failed to get current executable path")?;
    
    let st8_block = format!(
        "#!/bin/bash\n# === ST8 BLOCK START ===\n# DO NOT EDIT THIS BLOCK MANUALLY\n# Use 'st8 uninstall' to remove this hook\n{} update --git-add\n# === ST8 BLOCK END ===\n",
        current_exe.display()
    );
    
    if hook_file.exists() {
        // Read existing hook content
        let existing_content = fs::read_to_string(&hook_file)
            .context("Failed to read existing pre-commit hook")?;
        
        // Remove any existing st8 block
        let cleaned_content = remove_st8_block(&existing_content);
        
        // Append new st8 block
        let new_content = if cleaned_content.trim().is_empty() {
            st8_block
        } else {
            format!("{}\n{}", cleaned_content.trim_end(), st8_block)
        };
        
        fs::write(&hook_file, new_content)
            .context("Failed to update pre-commit hook")?;
        
        log_action(&format!("Updated existing pre-commit hook: {}", hook_file.display()));
    } else {
        // Create new hook file
        fs::write(&hook_file, &st8_block)
            .context("Failed to create pre-commit hook")?;
        
        log_action(&format!("Created new pre-commit hook: {}", hook_file.display()));
    }
    
    // Make hook executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_file, perms)?;
    }
    
    println!("{} st8 installed successfully as a pre-commit hook", "Success".green());
    println!("{} Version will be automatically updated on each commit", "Info".blue());
    
    Ok(())
}

fn uninstall_hook() -> Result<()> {
    if !is_git_repository() {
        anyhow::bail!("Not in a git repository. Please run st8 from within a git repository.");
    }
    
    let git_root = get_git_root()?;
    let hook_file = git_root.join(".git").join("hooks").join("pre-commit");
    
    if !hook_file.exists() {
        println!("{} No pre-commit hook found", "Info".yellow());
        return Ok(());
    }
    
    let existing_content = fs::read_to_string(&hook_file)
        .context("Failed to read pre-commit hook")?;
    
    if !existing_content.contains("=== ST8 BLOCK START ===") {
        println!("{} st8 is not installed in the pre-commit hook", "Info".yellow());
        return Ok(());
    }
    
    let cleaned_content = remove_st8_block(&existing_content);
    
    if cleaned_content.trim().is_empty() || cleaned_content.trim() == "#!/bin/bash" {
        // Remove the entire hook file if only st8 was in it or only has shebang
        fs::remove_file(&hook_file)
            .context("Failed to remove pre-commit hook file")?;
        log_action(&format!("Removed empty pre-commit hook file: {}", hook_file.display()));
    } else {
        // Update hook file with st8 block removed
        fs::write(&hook_file, cleaned_content)
            .context("Failed to update pre-commit hook")?;
        log_action(&format!("Removed st8 block from pre-commit hook: {}", hook_file.display()));
    }
    
    println!("{} st8 uninstalled from pre-commit hook", "Success".green());
    
    Ok(())
}

fn show_version() -> Result<()> {
    if !is_git_repository() {
        anyhow::bail!("Not in a git repository. Please run st8 from within a git repository.");
    }
    
    let version_info = VersionInfo::calculate()?;
    
    println!("{}", "Current Version Information:".bold());
    println!("  {}: {}", "Major (tag)".cyan(), version_info.major_version);
    println!("  {}: {}", "Minor (commits since tag)".cyan(), version_info.minor_version);
    println!("  {}: {}", "Patch (total changes)".cyan(), version_info.patch_version);
    println!("  {}: {}", "Full Version".green().bold(), version_info.full_version);
    
    Ok(())
}

fn update_state(no_git: bool, git_add: bool) -> Result<()> {
    if !no_git && !is_git_repository() {
        anyhow::bail!("Not in a git repository. Use --no-git to update anyway.");
    }
    
    let git_root = if is_git_repository() {
        get_git_root()?
    } else {
        env::current_dir().context("Failed to get current directory")?
    };
    
    let config = St8Config::load(&git_root)?;
    
    if !config.enabled {
        println!("{} st8 is disabled in configuration", "Info".yellow());
        return Ok(());
    }
    
    let version_info = VersionInfo::calculate()?;
    let updated = update_version_file(&version_info, &config)?;
    
    if updated {
        println!("{} Updated version to: {}", "Success".green(), version_info.full_version);
        log_action(&format!("Updated version to: {} (file: {})", version_info.full_version, config.version_file));
        
        let mut files_to_add = vec![config.version_file.clone()];
        
        // Render templates after version update
        if let Ok(project_root) = get_project_root() {
            if let Ok(state) = NomionState::load(&project_root) {
                if let Ok(template_manager) = TemplateManager::new(&state) {
                    match template_manager.render_all_templates(&version_info, state.project_name.as_deref()) {
                        Ok(rendered_files) => {
                            if !rendered_files.is_empty() {
                                println!("{} Rendered {} template(s):", "Info".blue(), rendered_files.len());
                                for file in &rendered_files {
                                    println!("  • {}", file);
                                    log_action(&format!("Rendered template: {}", file));
                                    files_to_add.push(file.clone());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{} Failed to render templates: {}", "Warning".yellow(), e);
                        }
                    }
                }
            }
        }
        
        // Add files to git if requested and in git repository
        if git_add && is_git_repository() {
            let git_add_result = add_files_to_git(&files_to_add);
            match git_add_result {
                Ok(added_files) => {
                    if !added_files.is_empty() {
                        println!("{} Added {} file(s) to git:", "Info".blue(), added_files.len());
                        for file in &added_files {
                            println!("  • {}", file);
                            log_action(&format!("Added to git: {}", file));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} Failed to add files to git: {}", "Warning".yellow(), e);
                }
            }
        }
    }
    
    Ok(())
}

fn show_status() -> Result<()> {
    if !is_git_repository() {
        println!("{}: Not in a git repository", "Status".red());
        return Ok(());
    }
    
    let git_root = get_git_root()?;
    let config = St8Config::load(&git_root)?;
    
    println!("{}", "St8 Status:".bold());
    println!("  {}: {}", "Git Repository".cyan(), "✓".green());
    println!("  {}: {}", "Hook Installed".cyan(), 
        if is_hook_installed()? { "✓".green() } else { "✗".red() });
    println!("  {}: {}", "Enabled".cyan(), 
        if config.enabled { "✓".green() } else { "✗".red() });
    println!("  {}: {}", "Version File".cyan(), config.version_file);
    
    if let Ok(version_info) = VersionInfo::calculate() {
        println!("  {}: {}", "Current Version".cyan(), version_info.full_version);
    }
    
    // Check if version file exists
    let version_file_path = PathBuf::from(&config.version_file);
    println!("  {}: {}", "Version File Exists".cyan(),
        if version_file_path.exists() { "✓".green() } else { "✗".red() });
    
    // Show auto-detection status
    println!("  {}: {}", "Auto-detect Project Files".cyan(),
        if config.auto_detect_project_files { "✓".green() } else { "✗".red() });
    
    // Show detected project files
    if config.auto_detect_project_files {
        match detect_project_files(&git_root) {
            Ok(project_files) => {
                if !project_files.is_empty() {
                    println!("  {}: ", "Detected Project Files".cyan());
                    for project_file in &project_files {
                        println!("    • {} ({})", 
                            project_file.path.display(),
                            project_file.file_type.file_name());
                    }
                } else {
                    println!("  {}: {}", "Detected Project Files".cyan(), "None".yellow());
                }
            }
            Err(e) => {
                println!("  {}: {} ({})", "Detected Project Files".cyan(), "Error".red(), e);
            }
        }
    }
    
    // Show manually configured project files
    if !config.project_files.is_empty() {
        println!("  {}: ", "Configured Project Files".cyan());
        for file_path in &config.project_files {
            let full_path = git_root.join(file_path);
            println!("    • {} ({})", 
                file_path,
                if full_path.exists() { "✓".green() } else { "✗".red() });
        }
    }
    
    // Show template status
    match get_project_root() {
        Ok(project_root) => {
            match NomionState::load(&project_root) {
                Ok(state) => {
                    match TemplateManager::new(&state) {
                        Ok(template_manager) => {
                            let templates = template_manager.list_templates();
                            println!("  {}: {}", "Templates".cyan(), templates.len());
                            
                            if !templates.is_empty() {
                                for template in templates {
                                    let status = if template.enabled { "✓".green() } else { "✗".red() };
                                    println!("    • {} → {} ({})", 
                                        template.name,
                                        template.output_path,
                                        status);
                                }
                            }
                        }
                        Err(_) => {
                            println!("  {}: {}", "Templates".cyan(), "Error loading".red());
                        }
                    }
                }
                Err(_) => {
                    println!("  {}: {}", "Templates".cyan(), "Not initialized".yellow());
                }
            }
        }
        Err(_) => {}
    }
    
    Ok(())
}

fn handle_template_command(action: TemplateAction) -> Result<()> {
    let project_root = get_project_root()?;
    let state = NomionState::load(&project_root)?;
    let mut template_manager = TemplateManager::new(&state)?;
    
    match action {
        TemplateAction::Add { name, template, output, description } => {
            let template_path = std::path::Path::new(&template);
            let template_content = if template_path.exists() {
                fs::read_to_string(&template)
                    .with_context(|| format!("Failed to read template file: {}", template))?
            } else if template.contains('\n') || template.contains('{') || !template.contains('.') {
                // Treat as template content if it contains template syntax or newlines, or doesn't look like a filename
                template
            } else {
                // If it looks like a filename but doesn't exist, return an error
                anyhow::bail!("Template file not found: {}", template);
            };
            
            template_manager.add_template(&name, &template_content, &output, description)?;
            println!("{} Added template '{}' → {}", "Success".green(), name, output);
        }
        
        TemplateAction::Remove { name } => {
            let removed = template_manager.remove_template(&name)?;
            if removed {
                println!("{} Removed template '{}'", "Success".green(), name);
            } else {
                println!("{} Template '{}' not found", "Warning".yellow(), name);
            }
        }
        
        TemplateAction::List => {
            let templates = template_manager.list_templates();
            if templates.is_empty() {
                println!("{} No templates configured", "Info".blue());
            } else {
                println!("{}", "Templates:".bold());
                for template in templates {
                    let status = if template.enabled { "enabled" } else { "disabled" };
                    let description = template.description.as_deref().unwrap_or("No description");
                    
                    println!("  {} → {} ({})", 
                        template.name.cyan(), 
                        template.output_path, 
                        status.yellow());
                    println!("    {}", description);
                }
            }
        }
        
        TemplateAction::Enable { name, disable } => {
            let enabled = !disable;
            let updated = template_manager.set_template_enabled(&name, enabled)?;
            if updated {
                let status = if enabled { "enabled" } else { "disabled" };
                println!("{} Template '{}' {}", "Success".green(), name, status);
            } else {
                println!("{} Template '{}' not found", "Warning".yellow(), name);
            }
        }
        
        TemplateAction::Render => {
            let version_info = VersionInfo::calculate()?;
            let project_name = state.project_name.as_deref();
            
            let rendered_files = template_manager.render_all_templates(&version_info, project_name)?;
            
            if rendered_files.is_empty() {
                println!("{} No enabled templates to render", "Info".blue());
            } else {
                println!("{} Rendered {} template(s):", "Success".green(), rendered_files.len());
                for file in &rendered_files {
                    println!("  • {}", file);
                }
            }
        }
        
        TemplateAction::Show { name } => {
            if let Some(template) = template_manager.get_template(&name) {
                println!("{}", format!("Template: {}", template.name).bold());
                println!("  {}: {}", "Output".cyan(), template.output_path);
                println!("  {}: {}", "Enabled".cyan(), 
                    if template.enabled { "Yes".green() } else { "No".red() });
                
                if let Some(description) = &template.description {
                    println!("  {}: {}", "Description".cyan(), description);
                }
                
                // Show template content
                let template_path = state.tool_dir("st8").join("templates").join(&template.source_path);
                if template_path.exists() {
                    match fs::read_to_string(&template_path) {
                        Ok(content) => {
                            println!("  {}:", "Content".cyan());
                            for line in content.lines() {
                                println!("    {}", line);
                            }
                        }
                        Err(e) => {
                            println!("  {}: {}", "Content".cyan(), format!("Error reading file: {}", e).red());
                        }
                    }
                }
            } else {
                eprintln!("{} Template '{}' not found", "Error".red(), name);
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

fn get_project_root() -> Result<PathBuf> {
    if is_git_repository() {
        get_git_root()
    } else {
        env::current_dir().context("Failed to get current directory")
    }
}

fn is_hook_installed() -> Result<bool> {
    if !is_git_repository() {
        return Ok(false);
    }
    
    let git_root = get_git_root()?;
    let hook_file = git_root.join(".git").join("hooks").join("pre-commit");
    
    if !hook_file.exists() {
        return Ok(false);
    }
    
    let content = fs::read_to_string(&hook_file)
        .context("Failed to read pre-commit hook")?;
    
    Ok(content.contains("=== ST8 BLOCK START ==="))
}

fn remove_st8_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_st8_block = false;
    let ends_with_newline = content.ends_with('\n');
    
    for line in lines {
        if line.contains("=== ST8 BLOCK START ===") {
            in_st8_block = true;
            continue;
        }
        
        if line.contains("=== ST8 BLOCK END ===") {
            in_st8_block = false;
            continue;
        }
        
        if !in_st8_block {
            result.push(line);
        }
    }
    
    let mut output = result.join("\n");
    if ends_with_newline && !output.is_empty() {
        output.push('\n');
    }
    output
}

fn log_action(message: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {}\n", timestamp, message);
    
    // Try to append to log file in .nomion/st8/logs/, but don't fail if we can't
    if let Ok(git_root) = get_git_root() {
        if let Ok(state) = NomionState::load(&git_root) {
            let logs_dir = state.tool_dir("st8").join("logs");
            let log_file = logs_dir.join("st8.log");
            
            // Ensure logs directory exists
            let _ = std::fs::create_dir_all(&logs_dir);
            
            let _ = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)
                .and_then(|mut file| file.write_all(log_entry.as_bytes()));
        }
    }
}

fn is_git_repository() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_git_root() -> Result<PathBuf> {
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

fn add_files_to_git(files: &[String]) -> Result<Vec<String>> {
    let mut added_files = Vec::new();
    
    for file in files {
        // Check if file exists before trying to add it
        if std::path::Path::new(file).exists() {
            let output = Command::new("git")
                .args(["add", file])
                .output()
                .with_context(|| format!("Failed to run git add for file: {}", file))?;

            if output.status.success() {
                added_files.push(file.clone());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("{} Failed to add file '{}' to git: {}", "Warning".yellow(), file, stderr.trim());
            }
        } else {
            eprintln!("{} File '{}' does not exist, skipping git add", "Warning".yellow(), file);
        }
    }
    
    Ok(added_files)
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_remove_st8_block() {
        let content = r#"#!/bin/bash
# Some existing content
echo "Before st8"

# === ST8 BLOCK START ===
# DO NOT EDIT THIS BLOCK MANUALLY
/path/to/st8 update --git-add
# === ST8 BLOCK END ===

echo "After st8"
"#;
        
        let result = remove_st8_block(content);
        assert!(!result.contains("ST8 BLOCK"));
        assert!(result.contains("Before st8"));
        assert!(result.contains("After st8"));
    }
    
    #[test]
    fn test_remove_st8_block_only() {
        let content = r#"#!/bin/bash
# === ST8 BLOCK START ===
/path/to/st8 update --git-add
# === ST8 BLOCK END ===
"#;
        
        let result = remove_st8_block(content);
        assert_eq!(result.trim(), "#!/bin/bash");
    }
    
    #[test]
    fn test_remove_st8_block_none() {
        let content = r#"#!/bin/bash
echo "No st8 block here"
"#;
        
        let result = remove_st8_block(content);
        assert_eq!(result, content);
    }
}