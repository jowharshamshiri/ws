use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use workspace::st8::{St8Config, VersionInfo, detect_project_files, update_version_file, TemplateManager};
use workspace::workspace_state::WorkspaceState;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command};

#[derive(Parser, Debug)]
#[command(name = "ws")]
#[command(about = "Workspace workspace tools - comprehensive file operations, version management, and development workflow automation")]
#[command(version = workspace::get_version())]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Recursive string replacement in file/folder names and contents with collision detection
    Refactor {
        #[command(flatten)]
        args: workspace::cli::Args,
    },
    
    /// Automatic version bumping based on git commits and changes
    St8 {
        #[command(subcommand)]
        command: Option<St8Commands>,
    },
    
    /// Local trash can using a .scrap folder for files you want to delete
    Scrap {
        /// Paths to files or directories to move to .scrap folder
        paths: Vec<std::path::PathBuf>,
        #[command(subcommand)]
        command: Option<ScrapCommands>,
    },
    
    /// Restore files from .scrap folder to their original locations
    Unscrap {
        /// Name of file/directory in .scrap to restore
        name: Option<String>,
        /// Force restore even if destination exists
        #[arg(short, long)]
        force: bool,
        /// Restore to a different location
        #[arg(short = 't', long)]
        to: Option<std::path::PathBuf>,
    },
    
    /// Process input lines, replacing repeated tokens with a substitute character
    Ldiff {
        /// Character to use for substitution (default: ░)
        #[arg(default_value = "░")]
        substitute_char: String,
    },
}

#[derive(Subcommand, Debug)]
enum St8Commands {
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
        action: St8TemplateAction,
    },
}

#[derive(Subcommand, Debug)]
enum St8TemplateAction {
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

#[derive(Subcommand, Debug)]
enum ScrapCommands {
    /// List contents of .scrap folder
    #[command(alias = "ls")]
    List {
        /// Sort by: name, date, size
        #[arg(short, long, default_value = "date")]
        sort: String,
    },

    /// Clean old items from .scrap folder
    Clean {
        /// Remove items older than N days
        #[arg(short, long, default_value = "30")]
        days: u64,
        
        /// Show what would be removed without actually removing
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Remove all items from .scrap folder
    Purge {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Search for files in .scrap folder
    #[command(alias = "search")]
    Find {
        /// Pattern to search for (supports regex)
        pattern: String,
        
        /// Search in file contents as well
        #[arg(short, long)]
        content: bool,
    },

    /// Archive .scrap folder contents  
    Archive {
        /// Output archive file name (defaults to .scrap-YYYY-MM-DD.tar.gz)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
        
        /// Remove files after archiving
        #[arg(short, long)]
        remove: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {:#}", "Error".red(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Refactor { args } => {
            match workspace::run_refac(args) {
                Ok(()) => {}
                Err(error) => {
                    eprintln!("{} {}", "ERROR:".red().bold(), error);
                    
                    // Print error chain if available
                    let mut source = error.source();
                    while let Some(err) = source {
                        eprintln!("  Caused by: {}", err);
                        source = err.source();
                    }
                    
                    process::exit(1);
                }
            }
        }
        
        Commands::St8 { command } => {
            run_st8_command(command)?;
        }
        
        Commands::Scrap { paths, command } => {
            run_scrap_command(paths, command)?;
        }
        
        Commands::Unscrap { name, force, to } => {
            run_unscrap_command(name, force, to)?;
        }
        
        Commands::Ldiff { substitute_char } => {
            run_ldiff_command(substitute_char)?;
        }
    }
    
    Ok(())
}

fn run_st8_command(command: Option<St8Commands>) -> Result<()> {
    match command {
        Some(St8Commands::Install { force }) => install_hook(force)?,
        Some(St8Commands::Uninstall) => uninstall_hook()?,
        Some(St8Commands::Show) => show_version()?,
        Some(St8Commands::Update { no_git, git_add }) => update_state(no_git, git_add)?,
        Some(St8Commands::Status) => show_status()?,
        Some(St8Commands::Template { action }) => handle_template_command(action)?,
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

fn run_scrap_command(paths: Vec<std::path::PathBuf>, command: Option<ScrapCommands>) -> Result<()> {
    let mut args = Vec::new();
    
    // Convert clap ScrapCommands to original scrap binary arguments
    match command {
        Some(ScrapCommands::List { sort }) => {
            args.push("list".to_string());
            args.push("--sort".to_string());
            args.push(sort);
        }
        Some(ScrapCommands::Clean { days, dry_run }) => {
            args.push("clean".to_string());
            args.push("--days".to_string());
            args.push(days.to_string());
            if dry_run {
                args.push("--dry-run".to_string());
            }
        }
        Some(ScrapCommands::Purge { force }) => {
            args.push("purge".to_string());
            if force {
                args.push("--force".to_string());
            }
        }
        Some(ScrapCommands::Find { pattern, content }) => {
            args.push("find".to_string());
            args.push(pattern);
            if content {
                args.push("--content".to_string());
            }
        }
        Some(ScrapCommands::Archive { output, remove }) => {
            args.push("archive".to_string());
            if let Some(output_path) = output {
                args.push("--output".to_string());
                args.push(output_path.to_string_lossy().to_string());
            }
            if remove {
                args.push("--remove".to_string());
            }
        }
        None => {
            // Add all paths as arguments
            for path in paths {
                args.push(path.to_string_lossy().to_string());
            }
        }
    }
    
    workspace::run_scrap(args)
}

fn run_unscrap_command(name: Option<String>, force: bool, to: Option<std::path::PathBuf>) -> Result<()> {
    let mut args = Vec::new();
    
    // Add name if provided
    if let Some(name) = name {
        args.push(name);
    }
    
    // Add force flag if set
    if force {
        args.push("--force".to_string());
    }
    
    // Add to destination if provided
    if let Some(to_path) = to {
        args.push("--to".to_string());
        args.push(to_path.to_string_lossy().to_string());
    }
    
    workspace::run_unscrap(args)
}

fn run_ldiff_command(substitute_char: String) -> Result<()> {
    // Check if input is available from stdin
    if atty::is(atty::Stream::Stdin) {
        eprintln!("{}: No input provided. ldiff reads from stdin.", "Error".red());
        eprintln!("Usage examples:");
        eprintln!("  cat /var/log/system.log | tail -n 100 | ws ldiff");
        eprintln!("  find / | ws ldiff");
        eprintln!("  ws ldiff < input.txt");
        std::process::exit(1);
    }

    // Validate substitute character
    let substitute_char = substitute_char.chars().next()
        .context("Substitute character cannot be empty")?;

    // Process stdin directly using the ldiff library
    workspace::ldiff::process_stdin(substitute_char)
        .context("Failed to process input from stdin")?;

    Ok(())
}

// St8 helper functions
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
        println!("{} Use 'ws st8 install --force' to reinstall", "Tip".yellow());
        return Ok(());
    }
    
    // Get current binary path
    let current_exe = env::current_exe()
        .context("Failed to get current executable path")?;
    
    let st8_block = format!(
        "#!/bin/bash\n# === ST8 BLOCK START ===\n# DO NOT EDIT THIS BLOCK MANUALLY\n# Use 'ws st8 uninstall' to remove this hook\n{} st8 update --git-add\n# === ST8 BLOCK END ===\n",
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
            if let Ok(state) = WorkspaceState::load(&project_root) {
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
            match WorkspaceState::load(&project_root) {
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

fn handle_template_command(action: St8TemplateAction) -> Result<()> {
    let project_root = get_project_root()?;
    let state = WorkspaceState::load(&project_root)?;
    let mut template_manager = TemplateManager::new(&state)?;
    
    match action {
        St8TemplateAction::Add { name, template, output, description } => {
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
        
        St8TemplateAction::Remove { name } => {
            let removed = template_manager.remove_template(&name)?;
            if removed {
                println!("{} Removed template '{}'", "Success".green(), name);
            } else {
                println!("{} Template '{}' not found", "Warning".yellow(), name);
            }
        }
        
        St8TemplateAction::List => {
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
        
        St8TemplateAction::Enable { name, disable } => {
            let enabled = !disable;
            let updated = template_manager.set_template_enabled(&name, enabled)?;
            if updated {
                let status = if enabled { "enabled" } else { "disabled" };
                println!("{} Template '{}' {}", "Success".green(), name, status);
            } else {
                println!("{} Template '{}' not found", "Warning".yellow(), name);
            }
        }
        
        St8TemplateAction::Render => {
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
        
        St8TemplateAction::Show { name } => {
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
    use chrono::prelude::*;
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {}\n", timestamp, message);
    
    // Try to append to log file in .ws/st8/logs/, but don't fail if we can't
    if let Ok(git_root) = get_git_root() {
        if let Ok(state) = WorkspaceState::load(&git_root) {
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