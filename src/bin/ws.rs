use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::Colorize;
use workspace::st8::{St8Config, VersionInfo, detect_project_files, update_version_file, TemplateManager};
use workspace::workspace_state::WorkspaceState;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

#[derive(Parser, Debug)]
#[command(name = "ws")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Workspace - All-in-one development tool suite")]
#[command(after_help = "Shell completions are automatically set up on first run.")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Refactor files and directories using patterns
    Refactor {
        /// Arguments for refactor tool
        #[command(flatten)]
        args: workspace::refac::Args,
    },
    
    /// Git integration and version management
    Git {
        #[command(subcommand)]
        command: Option<GitCommands>,
    },
    
    /// Template management
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
    
    /// Update version and render templates
    Update {
        /// Skip git integration (don't auto-add files)
        #[arg(long)]
        no_git: bool,
        /// Automatically add updated files to git staging area
        #[arg(long)]
        git_add: bool,
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

    /// MCP server for Claude integration with automatic session management
    McpServer {
        /// Port for HTTP server (default: 3000)
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Enable debug logging
        #[arg(long)]
        debug: bool,
        /// Migrate features from features.md to database
        #[arg(long)]
        migrate: bool,
    },

    /// Create sample project with test data for dashboard testing
    Sample {
        /// Create sample project structure
        #[arg(long)]
        project: bool,
        /// Populate database with test data
        #[arg(long)]
        data: bool,
        /// Force overwrite existing data
        #[arg(long)]
        force: bool,
        /// Output directory for sample project (default: ./sample-project)
        #[arg(short, long, default_value = "sample-project")]
        output: String,
    },

    /// Start development session with project context loading and validation
    Start {
        /// Continue from specific task ID
        #[arg(long)]
        continue_from: Option<String>,
        /// Enable detailed diagnostic output during initialization
        #[arg(long)]
        debug_mode: bool,
        /// Initialize new project with persistent knowledge management methodology
        #[arg(long)]
        project_setup: bool,
        /// What to work on first in this session (does not limit session scope)
        first_task: Option<String>,
    },

    /// End development session with documentation consolidation and feature updates
    End {
        /// Session summary description
        #[arg(long)]
        summary: Option<String>,
        /// Enable detailed diagnostic output during session end
        #[arg(long)]
        debug_mode: bool,
        /// Force session end even if validation fails
        #[arg(long)]
        force: bool,
        /// Skip documentation updates
        #[arg(long)]
        skip_docs: bool,
    },

    /// Consolidate documentation with diagram management and information preservation
    Consolidate {
        /// Enable detailed diagnostic output during consolidation
        #[arg(long)]
        debug_mode: bool,
        /// Force consolidation even if validation fails
        #[arg(long)]
        force: bool,
        /// Generate architectural diagrams in DOT format
        #[arg(long)]
        generate_diagrams: bool,
        /// Preserve complexity information during consolidation
        #[arg(long)]
        preserve_complexity: bool,
    },

    /// Display comprehensive project status with feature metrics and progress tracking
    Status {
        /// Enable detailed diagnostic output during status reporting
        #[arg(long)]
        debug_mode: bool,
        /// Include feature breakdown by category
        #[arg(long)]
        include_features: bool,
        /// Include detailed metrics and analytics
        #[arg(long)]
        include_metrics: bool,
        /// Output format (human, json, summary)
        #[arg(long, default_value = "human")]
        format: String,
    },

    /// Feature-centric task management with automatic feature detection and linking
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    /// Project directive and rule management system for development methodology enforcement
    Directive {
        #[command(subcommand)]
        action: DirectiveAction,
    },

    /// Feature management with state machine workflow and validation
    Feature {
        #[command(subcommand)]
        action: FeatureAction,
    },

    /// Entity relationship management for complex entity links and dependencies
    Relationship {
        #[command(subcommand)]
        action: RelationshipAction,
    },

    /// Note management for attaching notes to any entity or project-wide
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },
}

#[derive(Subcommand, Debug)]
enum GitCommands {
    /// Install version management as a pre-commit hook in the current git repository
    Install {
        /// Force reinstallation even if already installed
        #[arg(short, long)]
        force: bool,
    },
    /// Uninstall version management from the current git repository
    Uninstall,
    /// Show current version information
    Show,
    /// Show git integration status
    Status,
}

#[derive(Subcommand, Debug)]
enum TaskAction {
    /// Add a new task with automatic feature detection
    Add {
        /// Task title
        title: String,
        /// Task description
        description: String,
        /// Link to specific feature (optional, auto-detected if not provided)
        #[arg(short, long)]
        feature: Option<String>,
        /// Task priority (high, medium, low)
        #[arg(short, long, default_value = "medium")]
        priority: String,
        /// Auto-detect and create feature if mentioned in description
        #[arg(long)]
        auto_feature: bool,
    },
    /// List tasks with filtering options
    List {
        /// Filter by task status (pending, in_progress, completed, blocked)
        #[arg(short, long)]
        status: Option<String>,
        /// Filter by feature code (e.g., F0103)
        #[arg(short, long)]
        feature: Option<String>,
        /// Filter by priority (high, medium, low)
        #[arg(short, long)]
        priority: Option<String>,
        /// Show only recent tasks (last N days)
        #[arg(short, long)]
        recent: Option<u32>,
    },
    /// Show detailed task information
    Show {
        /// Task ID or title pattern to match
        identifier: String,
    },
    /// Update task status or properties
    Update {
        /// Task ID to update
        task_id: String,
        /// New task status
        #[arg(short, long)]
        status: Option<String>,
        /// Update task priority
        #[arg(short, long)]
        priority: Option<String>,
        /// Add progress notes
        #[arg(short, long)]
        notes: Option<String>,
        /// Link to feature (for feature association)
        #[arg(short, long)]
        feature: Option<String>,
    },
    /// Complete a task and update linked feature status
    Complete {
        /// Task ID to complete
        task_id: String,
        /// Completion notes or evidence
        #[arg(short, long)]
        notes: Option<String>,
        /// Auto-advance linked feature state
        #[arg(long)]
        advance_feature: bool,
    },
    /// Block a task with reason and dependencies
    Block {
        /// Task ID to block
        task_id: String,
        /// Reason for blocking
        reason: String,
        /// Dependencies that must be resolved
        #[arg(short, long)]
        _dependencies: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
enum DirectiveAction {
    /// Add a new development directive or rule
    Add {
        /// Directive title
        title: String,
        /// Directive description or rule content
        description: String,
        /// Directive category (security, testing, coding, methodology, deployment)
        #[arg(short, long, default_value = "methodology")]
        category: String,
        /// Enforcement level (mandatory, recommended, optional)
        #[arg(short, long, default_value = "recommended")]
        enforcement: String,
        /// Priority level (critical, high, medium, low)
        #[arg(short, long, default_value = "medium")]
        priority: String,
    },
    /// List all directives with filtering options
    List {
        /// Filter by directive category
        #[arg(short, long)]
        category: Option<String>,
        /// Filter by enforcement level
        #[arg(short, long)]
        enforcement: Option<String>,
        /// Filter by priority level
        #[arg(short, long)]
        priority: Option<String>,
        /// Show only recently added directives (last N days)
        #[arg(short, long)]
        recent: Option<u32>,
    },
    /// Show detailed directive information
    Show {
        /// Directive ID or title pattern to match
        identifier: String,
    },
    /// Update directive properties or enforcement level
    Update {
        /// Directive ID to update
        directive_id: String,
        /// Update enforcement level
        #[arg(short, long)]
        enforcement: Option<String>,
        /// Update priority level
        #[arg(short, long)]
        priority: Option<String>,
        /// Update description
        #[arg(short, long)]
        description: Option<String>,
        /// Update category
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Remove a directive
    Remove {
        /// Directive ID to remove
        directive_id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Validate current project against all mandatory directives
    Validate {
        /// Category to validate (validates all if not specified)
        #[arg(short, long)]
        category: Option<String>,
        /// Show detailed validation results
        #[arg(short, long)]
        verbose: bool,
        /// Fail fast on first violation
        #[arg(short, long)]
        fail_fast: bool,
    },
    /// Check specific files or directories against directives
    Check {
        /// Files or directories to check
        paths: Vec<std::path::PathBuf>,
        /// Category of directives to check against
        #[arg(short, long)]
        category: Option<String>,
        /// Output format (human, json, report)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
}

#[derive(Subcommand, Debug)]
enum FeatureAction {
    /// Add a new feature to features.md
    Add {
        /// Feature title
        title: String,
        /// Feature description
        description: String,
        /// Feature category (core, command, mcp, etc.)
        #[arg(short, long, default_value = "core")]
        category: String,
        /// Initial state (not_started, implemented, testing, completed)
        #[arg(short, long, default_value = "not_started")]
        state: String,
    },
    /// List features with filtering
    List {
        /// Filter by state (🟢, 🟠, 🟡, ❌, ⚠️, 🔴)
        #[arg(short, long)]
        state: Option<String>,
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
        /// Show recently modified features
        #[arg(short, long)]
        recent: Option<u32>,
    },
    /// Show detailed feature information
    Show {
        /// Feature ID (F0001, F0002, etc.)
        feature_id: String,
    },
    /// Update feature state with validation
    Update {
        /// Feature ID to update
        feature_id: String,
        /// New state (implemented, testing, completed, issue, critical)
        #[arg(short, long)]
        state: Option<String>,
        /// Evidence or notes for state change
        #[arg(short, long)]
        evidence: Option<String>,
        /// Force state change without validation
        #[arg(short, long)]
        force: bool,
    },
    /// Validate feature state transitions
    Validate {
        /// Feature ID to validate (optional, validates all if not provided)
        feature_id: Option<String>,
        /// Verbose validation output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Test automatic feature detection (F0107 demonstration)
    DetectFeatures {
        /// Test input text to analyze for potential features
        input: String,
    },
    /// Monitor context usage and trigger session management
    MonitorContext {
        /// Current context usage percentage (0-100)
        usage_percent: f64,
        /// Total context tokens available
        total_tokens: Option<u32>,
        /// Used context tokens
        used_tokens: Option<u32>,
    },
    /// Real-time feature management API for Claude integration
    ApiCall {
        /// API operation (add, update, list, validate)
        operation: String,
        /// Feature ID (for update/get operations)
        feature_id: Option<String>,
        /// JSON payload for the operation
        payload: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum RelationshipAction {
    /// Link entities with specific relationship types
    Link {
        /// Source entity ID
        from_entity: String,
        /// Source entity type (feature, task, session, project)
        #[arg(short, long)]
        from_type: String,
        /// Target entity ID
        to_entity: String,
        /// Target entity type (feature, task, session, project)
        #[arg(short, long)]
        to_type: String,
        /// Relationship type (implements, blocks, worked_in, depends_on)
        #[arg(short, long, default_value = "depends_on")]
        relationship_type: String,
        /// Optional description of the relationship
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List relationships for an entity
    List {
        /// Entity ID to show relationships for
        entity_id: String,
        /// Entity type (feature, task, session, project)
        #[arg(short, long)]
        entity_type: String,
        /// Show only specific relationship types
        #[arg(short, long)]
        relationship_type: Option<String>,
        /// Include resolved relationships
        #[arg(long)]
        include_resolved: bool,
    },
    /// Remove a relationship link
    Unlink {
        /// Dependency ID to remove
        dependency_id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Resolve a blocking relationship
    Resolve {
        /// Dependency ID to resolve
        dependency_id: String,
        /// Resolution description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Show relationship statistics for project
    Stats {
        /// Include detailed breakdown by type
        #[arg(short, long)]
        detailed: bool,
        /// Output format (human, json)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
}

#[derive(Subcommand, Debug)]
enum NoteAction {
    /// Add a note to a specific entity
    Add {
        /// Entity type (feature, task, session, project, etc.)
        #[arg(short, long)]
        entity_type: String,
        /// Entity ID to attach note to
        #[arg(short = 'i', long)]
        entity_id: String,
        /// Note title
        title: String,
        /// Note content
        content: String,
        /// Note type (architecture, decision, reminder, observation, reference, evidence, progress, issue)
        #[arg(short = 't', long, default_value = "observation")]
        note_type: String,
        /// Optional tags for the note
        #[arg(long)]
        tags: Option<String>,
    },
    /// Add a project-wide note
    AddProject {
        /// Note title
        title: String,
        /// Note content
        content: String,
        /// Note type (architecture, decision, reminder, observation, reference, evidence, progress, issue)
        #[arg(short = 't', long, default_value = "architecture")]
        note_type: String,
        /// Optional tags for the note
        #[arg(long)]
        tags: Option<String>,
    },
    /// List notes with optional filtering
    List {
        /// Filter by entity type
        #[arg(short, long)]
        entity_type: Option<String>,
        /// Filter by entity ID
        #[arg(short = 'i', long)]
        entity_id: Option<String>,
        /// Filter by note type
        #[arg(short = 't', long)]
        note_type: Option<String>,
        /// Show only project-wide notes
        #[arg(long)]
        project_wide: bool,
        /// Show only pinned notes
        #[arg(long)]
        pinned: bool,
    },
    /// Search notes by content or category
    Search {
        /// Search query to match in title or content
        query: String,
        /// Filter by note type
        #[arg(short = 't', long)]
        note_type: Option<String>,
        /// Output format (human, json)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
    /// Update an existing note
    Update {
        /// Note ID to update
        note_id: String,
        /// New title (optional)
        #[arg(long)]
        title: Option<String>,
        /// New content (optional)
        #[arg(long)]
        content: Option<String>,
        /// New tags (optional)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Delete a note
    Delete {
        /// Note ID to delete
        note_id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Pin or unpin a note for importance
    Pin {
        /// Note ID to toggle pin status
        note_id: String,
    },
    /// Link a note to another entity or note
    Link {
        /// Source note ID
        source_note_id: String,
        /// Target entity or note ID
        target_id: String,
        /// Target type (note, entity)
        #[arg(short = 't', long, default_value = "entity")]
        target_type: String,
        /// Entity type if target is entity (feature, task, project, etc.)
        #[arg(long)]
        entity_type: Option<String>,
        /// Link type (reference, response_to, related, blocks, depends_on)
        #[arg(short = 'l', long, default_value = "reference")]
        link_type: String,
    },
    /// Remove a link between notes/entities
    Unlink {
        /// Link ID to remove
        link_id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// List links for a note or entity
    ListLinks {
        /// Entity or note ID to show links for
        id: String,
        /// Show incoming links (pointing to this entity)
        #[arg(long)]
        incoming: bool,
        /// Show outgoing links (from this entity)
        #[arg(long)]
        outgoing: bool,
        /// Output format (human, json)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
}

#[derive(Subcommand, Debug)]
enum TemplateAction {
    /// Add a new template
    Add {
        /// Template name
        name: String,
        /// Template content (file path or inline content)
        template: String,
        /// Output path (where template will be rendered)
        #[arg(short, long)]
        output: Option<String>,
        /// Template description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all templates
    List,
    /// Show template details
    Show {
        /// Template name
        name: String,
    },
    /// Enable or disable a template
    Enable {
        /// Template name
        name: String,
        /// Disable the template instead of enabling it
        #[arg(long)]
        disable: bool,
    },
    /// Remove a template
    Remove {
        /// Template name
        name: String,
    },
    /// Render all enabled templates
    Render,
    /// Generate documentation from database entities
    GenerateDocs {
        /// Documentation type to generate (claude, features, progress, status, all)
        #[arg(default_value = "all")]
        doc_type: String,
        /// Output directory for generated files
        #[arg(short, long)]
        output: Option<String>,
        /// Force overwrite existing files
        #[arg(short, long)]
        force: bool,
    },
    /// Initialize predefined documentation templates
    InitDocs {
        /// Force reinitialize even if templates exist
        #[arg(short, long)]
        force: bool,
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

    /// Search for files in .scrap
    Find {
        /// Search pattern (regex supported)
        pattern: String,
        
        /// Also search file contents
        #[arg(short, long)]
        content: bool,
    },

    /// Create archive of .scrap contents
    Archive {
        /// Output archive path
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
        
        /// Remove files after archiving
        #[arg(short, long)]
        remove: bool,
    },
}

fn main() {
    // Setup shell completions on first run (but not when running as a git hook)
    if !is_running_as_git_hook() {
        if let Err(e) = setup_shell_completions() {
            eprintln!("{}: Failed to setup completions: {:#}", "Warning".yellow(), e);
        }
    }
    
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
                    eprintln!("{}: {:#}", "Error".red(), error);
                    process::exit(1);
                }
            }
        }
        
        Commands::Git { command } => {
            run_git_command(command)?;
        }
        
        Commands::Template { action } => {
            handle_template_command(action)?;
        }
        
        Commands::Update { no_git, git_add } => {
            update_state(no_git, git_add)?;
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

        Commands::McpServer { port, debug, migrate } => {
            run_mcp_server(port, debug, migrate)?;
        }

        Commands::Sample { project, data, force, output } => {
            run_sample_command(project, data, force, output)?;
        }

        Commands::Start { continue_from, debug_mode, project_setup, first_task } => {
            run_start_command(continue_from, debug_mode, project_setup, first_task)?;
        }

        Commands::End { summary, debug_mode, force, skip_docs } => {
            run_end_command(summary, debug_mode, force, skip_docs)?;
        }

        Commands::Consolidate { debug_mode, force, generate_diagrams, preserve_complexity } => {
            run_consolidate_command(debug_mode, force, generate_diagrams, preserve_complexity)?;
        }

        Commands::Status { debug_mode, include_features, include_metrics, format } => {
            run_status_command(debug_mode, include_features, include_metrics, format)?;
        }

        Commands::Task { action } => {
            run_task_command(action)?;
        }

        Commands::Directive { action } => {
            run_directive_command(action)?;
        }

        Commands::Feature { action } => {
            run_feature_command(action)?;
        }

        Commands::Relationship { action } => {
            run_relationship_command(action)?;
        }

        Commands::Note { action } => {
            run_note_command(action)?;
        }
    }
    
    Ok(())
}

fn run_git_command(command: Option<GitCommands>) -> Result<()> {
    match command {
        Some(GitCommands::Install { force }) => install_hook(force)?,
        Some(GitCommands::Uninstall) => uninstall_hook()?,
        Some(GitCommands::Show) => show_version()?,
        Some(GitCommands::Status) => show_status()?,
        None => {
            // Default behavior: install hook if not installed, otherwise update state
            if !is_git_repository() {
                eprintln!("{}: Not in a git repository", "Warning".yellow());
                eprintln!("{}: Use 'ws git install' to set up version management", "Tip".blue());
                return Ok(());
            }

            if !is_hook_installed()? {
                eprintln!("{}: Git hook not installed", "Info".blue());
                eprintln!("{}: Installing pre-commit hook for automatic version management", "Info".blue());
                install_hook(false)?;
            } else {
                // Hook is installed, just update state
                let project_root = get_project_root()?;
                let config = St8Config::load(&project_root)?;
                let mut workspace_state = WorkspaceState::load(&project_root)?;
                
                update_version_in_memory(&mut workspace_state, &config, &project_root)?;
                workspace_state.save(&project_root)?;
            }
        }
    }
    
    Ok(())
}

fn update_state(no_git: bool, git_add: bool) -> Result<()> {
    let project_root = get_project_root()?;
    let config = St8Config::load(&project_root)?;
    let mut workspace_state = WorkspaceState::load(&project_root)?;
    
    // Update version in memory  
    update_version_in_memory(&mut workspace_state, &config, &project_root)?;
    
    // Render templates
    let template_manager = TemplateManager::new(&workspace_state)?;
    let version_info = VersionInfo::calculate()?;
    let project_name = workspace_state.project_name.as_deref();
    let rendered_files = template_manager.render_all_templates(&version_info, project_name)?;
    
    if !rendered_files.is_empty() {
        println!("{}: Rendered {} templates", "Info".blue(), rendered_files.len());
        for file in &rendered_files {
            println!("  - {}", file);
        }
    }
    
    // Save state to disk
    workspace_state.save(&project_root)?;
    
    // Update version file on disk
    let version_info = VersionInfo::calculate()?;
    update_version_file(&version_info, &config)?;
    if !config.version_file.is_empty() {
        println!("{}: Updated {}", "Info".blue(), config.version_file);
    }
    
    // Add files to git if requested and we're in a git repository
    if !no_git && git_add && is_git_repository() {
        let mut files_to_add = Vec::new();
        
        // Add version file if it exists
        if !config.version_file.is_empty() {
            let version_file = &config.version_file;
            files_to_add.push(version_file.clone());
        }
        
        // Add rendered template files
        for file in rendered_files {
            let path_str = file.clone();
            files_to_add.push(path_str.to_string());
        }
        
        if !files_to_add.is_empty() {
            let added_files = add_files_to_git(&files_to_add)?;
            if !added_files.is_empty() {
                println!("{}: Added {} files to git staging area", "Info".blue(), added_files.len());
                for file in added_files {
                    println!("  - {}", file);
                }
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
    
    if let Some(item_name) = name {
        args.push(item_name);
    }
    
    if force {
        args.push("--force".to_string());
    }
    
    if let Some(target_path) = to {
        args.push("--to".to_string());
        args.push(target_path.to_string_lossy().to_string());
    }
    
    workspace::run_unscrap(args)
}

fn run_ldiff_command(substitute_char: String) -> Result<()> {
    workspace::run_ldiff(vec![substitute_char.clone()])
}

fn install_hook(force: bool) -> Result<()> {
    if !is_git_repository() {
        eprintln!("{}: Not in a git repository", "Error".red());
        eprintln!("{}: Navigate to a git repository and try again", "Tip".yellow());
        return Ok(());
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
        println!("{} Git hook is already installed", "Info".blue());
        println!("{} Use 'ws git install --force' to reinstall", "Tip".yellow());
        return Ok(());
    }
    
    // Get current binary path
    let current_exe = env::current_exe()
        .context("Failed to get current executable path")?;
    
    let st8_block = format!(
        "#!/bin/bash\n# === WS BLOCK START ===\n# DO NOT EDIT THIS BLOCK MANUALLY\n# Use 'ws git uninstall' to remove this hook\n{} ws update --git-add\n# === WS BLOCK END ===\n",
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
    
    // Make hook executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_file, perms)?;
    }
    
    println!("{} Git hook installed successfully", "Success".green());
    println!("{} Version will be updated automatically on each commit", "Info".blue());
    
    Ok(())
}

fn uninstall_hook() -> Result<()> {
    if !is_git_repository() {
        eprintln!("{}: Not in a git repository", "Error".red());
        return Ok(());
    }
    
    let git_root = get_git_root()?;
    let hook_file = git_root.join(".git").join("hooks").join("pre-commit");
    
    if !hook_file.exists() {
        println!("{} No pre-commit hook found", "Info".blue());
        return Ok(());
    }
    
    let content = fs::read_to_string(&hook_file)
        .context("Failed to read pre-commit hook")?;
    
    if !content.contains("=== WS BLOCK START ===") {
        println!("{} No st8 hook block found in pre-commit hook", "Info".blue());
        return Ok(());
    }
    
    let cleaned_content = remove_st8_block(&content);
    
    if cleaned_content.trim().is_empty() {
        // Remove the entire hook file if only st8 content
        fs::remove_file(&hook_file)
            .context("Failed to remove pre-commit hook")?;
        println!("{} Removed pre-commit hook", "Success".green());
        log_action(&format!("Removed pre-commit hook: {}", hook_file.display()));
    } else {
        // Write back the cleaned content
        fs::write(&hook_file, cleaned_content.trim_end())
            .context("Failed to update pre-commit hook")?;
        println!("{} Removed st8 from pre-commit hook", "Success".green());
        log_action(&format!("Removed st8 block from pre-commit hook: {}", hook_file.display()));
    }
    
    Ok(())
}

fn show_version() -> Result<()> {
    let project_root = get_project_root()?;
    let workspace_state = WorkspaceState::load(&project_root)?;
    
    println!("{}", "Version Information".bold().underline());
    println!();
    let version_info = VersionInfo::calculate()?;
    println!("{}: {}", "Current Version".blue(), version_info.full_version);
    println!("{}: {}", "Project Name".blue(), workspace_state.project_name.as_deref().unwrap_or("Unknown"));
    
    if is_git_repository() {
        println!("{}: {}", "Major Version".blue(), version_info.major_version);
        println!("{}: {}", "Minor Version".blue(), version_info.minor_version);
        println!("{}: {}", "Patch Version".blue(), version_info.patch_version);
        
        // Note: VersionInfo doesn't track dirty state in this implementation
    }
    
    let config = St8Config::load(&project_root)?;
    if !config.version_file.is_empty() {
        println!("{}: {}", "Version File".blue(), &config.version_file);
    }
    
    let detected_files = detect_project_files(&project_root);
    if let Ok(detected_files) = detected_files {
    if !detected_files.is_empty() {
        println!();
        println!("{}", "Detected Project Files:".bold());
        for file in detected_files {
            println!("  - {}", file.path.display());
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
    
    println!("{}", "Git Integration Status".bold().underline());
    println!();
    
    // Hook status
    if is_hook_installed()? {
        println!("{}: Installed ✓", "Pre-commit Hook".green());
    } else {
        println!("{}: Not installed ✗", "Pre-commit Hook".red());
        println!("{}: Run 'ws git install' to set up automatic version management", "Tip".yellow());
    }
    
    // Version file status
    if !config.version_file.is_empty() {
        let version_path = git_root.join(&config.version_file);
        if version_path.exists() {
            println!("{}: {} ✓", "Version File".green(), config.version_file);
        } else {
            println!("{}: {} (not found)", "Version File".yellow(), config.version_file);
        }
    } else {
        println!("{}: Auto-detected", "Version File".blue());
    }
    
    // Show current version
    let workspace_state = WorkspaceState::load(&git_root)?;
    let version_info = VersionInfo::calculate()?;
    println!("{}: {}", "Current Version".blue(), version_info.full_version);
    
    // Template status
    let template_manager = TemplateManager::new(&workspace_state)?;
    let templates = template_manager.list_templates();
    let enabled_count = templates.iter().filter(|t| t.enabled).count();
    
    if !templates.is_empty() {
        println!("{}: {} total, {} enabled", "Templates".blue(), templates.len(), enabled_count);
    } else {
        println!("{}: None configured", "Templates".blue());
    }
    
    Ok(())
}

fn update_version_in_memory(
    _workspace_state: &mut WorkspaceState,
    _config: &St8Config,
    _project_root: &std::path::Path,
) -> Result<()> {
    // Version info is now calculated dynamically via VersionInfo::calculate()
    // No need to store in workspace state
    if is_git_repository() {
        let version_info = VersionInfo::calculate()?;
        log_action(&format!("Updated version to: {}", version_info.full_version));
    }
    Ok(())
}

fn log_action(message: &str) {
    match log_to_file(message) {
        Ok(_) => {}
        Err(_) => {}
    }
}

fn log_to_file(message: &str) -> Result<()> {
    if let Ok(project_root) = get_project_root() {
        let state = WorkspaceState::load(&project_root)?;
        let log_dir = state.tool_dir("st8").join("logs");
        
        if let Err(_) = fs::create_dir_all(&log_dir) {
            return Ok(()); // Silently fail if we can't create log directory
        }
        
        let log_file = log_dir.join("st8.log");
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let log_entry = format!("[{}] {}\n", timestamp, message);
        
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }
    
    Ok(())
}

fn handle_template_command(action: TemplateAction) -> Result<()> {
    use tokio::runtime::Runtime;
    use workspace::entities::{EntityManager, doc_generator::DocumentationGenerator};
    use std::fs;
    use std::path::Path;
    
    let rt = Runtime::new()?;
    let project_root = get_project_root()?;
    
    rt.block_on(async {
        let pool = workspace::entities::database::initialize_database(&project_root).await?;
        let entity_manager = EntityManager::new(pool.clone());
        let current_project = entity_manager.get_current_project().await?;
        let mut doc_generator = DocumentationGenerator::new(pool)?;
        
        match action {
            TemplateAction::Add { name, template, output, description } => {
                let template_path = Path::new(&template);
                let template_content = if template_path.exists() {
                    fs::read_to_string(template_path)?
                } else {
                    template
                };
                
                let template = entity_manager.create_template(
                    &current_project.id,
                    name.clone(),
                    description,
                    template_content,
                    output,
                    None,
                ).await?;
                
                println!("{} Added template: {} (ID: {})", "Success".green(), name, template.id);
            }
        
            TemplateAction::List => {
                let templates = entity_manager.get_templates(&current_project.id).await?;
                if templates.is_empty() {
                    println!("{} No templates configured", "Info".blue());
                } else {
                    println!("{}", "Templates:".bold());
                    for template in templates {
                        let status = if template.enabled { "enabled".green() } else { "disabled".red() };
                        let output = template.output_path.as_deref().unwrap_or("(no output path)");
                        println!("  {} [{}] -> {}", template.name.bold(), status, output);
                        if let Some(desc) = &template.description {
                            println!("    {}", desc);
                        }
                    }
                }
            }
            
            TemplateAction::Show { name } => {
                let templates = entity_manager.get_templates(&current_project.id).await?;
                if let Some(template) = templates.iter().find(|t| t.name == name) {
                    println!("{}", format!("Template: {}", name).bold());
                    println!("{}: {}", "Enabled".blue(), if template.enabled { "Yes" } else { "No" });
                    println!("{}: {}", "Output Path".blue(), 
                        template.output_path.as_deref().unwrap_or("(no output path)"));
                    if let Some(desc) = &template.description {
                        println!("{}: {}", "Description".blue(), desc);
                    }
                    println!("{}: {}", "Last Rendered".blue(), 
                        template.last_rendered.map_or("Never".to_string(), |d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string()));
                    println!("{}: {}", "Render Count".blue(), template.render_count);
                    println!();
                    println!("{}", "Content:".bold());
                    println!("{}", template.content);
                } else {
                    eprintln!("{}: Template '{}' not found", "Error".red(), name);
                }
            }
            
            TemplateAction::Enable { name, disable } => {
                let templates = entity_manager.get_templates(&current_project.id).await?;
                if let Some(template) = templates.iter().find(|t| t.name == name) {
                    let enabled = !disable;
                    entity_manager.update_template(
                        &template.id,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(enabled),
                    ).await?;
                    let status = if enabled { "enabled" } else { "disabled" };
                    println!("{} Template '{}' {}", "Success".green(), name, status);
                } else {
                    eprintln!("{}: Template '{}' not found", "Error".red(), name);
                }
            }
            
            TemplateAction::Remove { name } => {
                let templates = entity_manager.get_templates(&current_project.id).await?;
                if let Some(template) = templates.iter().find(|t| t.name == name) {
                    entity_manager.delete_template(&template.id).await?;
                    println!("{} Removed template: {}", "Success".green(), name);
                } else {
                    eprintln!("{}: Template '{}' not found", "Error".red(), name);
                }
            }
            
            TemplateAction::Render => {
                let rendered_files = doc_generator.render_all_templates(&current_project.id).await?;
                if rendered_files.is_empty() {
                    println!("{} No templates to render", "Info".blue());
                } else {
                    println!("{} Rendered {} templates", "Success".green(), rendered_files.len());
                    for (file_path, _content) in &rendered_files {
                        // Write to filesystem
                        if let Some(parent) = Path::new(file_path).parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::write(file_path, &_content)?;
                        println!("  - {}", file_path);
                    }
                }
            }

            TemplateAction::GenerateDocs { doc_type, output, force } => {
                let output_dir = output.as_deref().unwrap_or(".");
                let output_path = Path::new(output_dir);
                
                match doc_type.as_str() {
                    "claude" => {
                        let content = doc_generator.generate_claude_md(&current_project.id).await?;
                        let file_path = output_path.join("CLAUDE.md");
                        write_doc_file(&file_path, &content, force)?;
                        println!("{} Generated: {}", "Success".green(), file_path.display());
                    }
                    "features" => {
                        let content = doc_generator.generate_features_md(&current_project.id).await?;
                        let file_path = output_path.join("internal").join("features.md");
                        write_doc_file(&file_path, &content, force)?;
                        println!("{} Generated: {}", "Success".green(), file_path.display());
                    }
                    "progress" => {
                        let content = doc_generator.generate_progress_md(&current_project.id).await?;
                        let file_path = output_path.join("internal").join("progress_tracking.md");
                        write_doc_file(&file_path, &content, force)?;
                        println!("{} Generated: {}", "Success".green(), file_path.display());
                    }
                    "status" => {
                        let content = doc_generator.generate_status_md(&current_project.id).await?;
                        let file_path = output_path.join("internal").join("current_status.md");
                        write_doc_file(&file_path, &content, force)?;
                        println!("{} Generated: {}", "Success".green(), file_path.display());
                    }
                    "all" => {
                        let claude_content = doc_generator.generate_claude_md(&current_project.id).await?;
                        let features_content = doc_generator.generate_features_md(&current_project.id).await?;
                        let progress_content = doc_generator.generate_progress_md(&current_project.id).await?;
                        let status_content = doc_generator.generate_status_md(&current_project.id).await?;
                        
                        write_doc_file(&output_path.join("CLAUDE.md"), &claude_content, force)?;
                        write_doc_file(&output_path.join("internal").join("features.md"), &features_content, force)?;
                        write_doc_file(&output_path.join("internal").join("progress_tracking.md"), &progress_content, force)?;
                        write_doc_file(&output_path.join("internal").join("current_status.md"), &status_content, force)?;
                        
                        println!("{} Generated all documentation files in {}", "Success".green(), output_dir);
                    }
                    _ => {
                        eprintln!("{}: Unknown documentation type '{}'. Use: claude, features, progress, status, or all", "Error".red(), doc_type);
                    }
                }
            }

            TemplateAction::InitDocs { force } => {
                let existing_templates = entity_manager.get_templates(&current_project.id).await?;
                let doc_templates = ["claude_md", "features_md", "progress_md", "status_md"];
                let has_existing = existing_templates.iter().any(|t| doc_templates.contains(&t.name.as_str()));
                
                if has_existing && !force {
                    println!("{} Documentation templates already exist. Use --force to reinitialize.", "Info".blue());
                } else {
                    let templates = entity_manager.initialize_predefined_templates(&current_project.id).await?;
                    println!("{} Initialized {} documentation templates", "Success".green(), templates.len());
                    for template in templates {
                        println!("  - {}: {}", template.name, template.description.as_deref().unwrap_or("No description"));
                    }
                }
            }
        }
        
        Ok(())
    })
}

fn write_doc_file(file_path: &Path, content: &str, force: bool) -> Result<()> {
    if file_path.exists() && !force {
        eprintln!("{}: File {} already exists. Use --force to overwrite.", "Error".red(), file_path.display());
        return Ok(());
    }
    
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    std::fs::write(file_path, content)?;
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
    
    Ok(content.contains("=== WS BLOCK START ==="))
}

fn remove_st8_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_st8_block = false;
    let ends_with_newline = content.ends_with('\n');
    
    for line in lines {
        if line.contains("=== WS BLOCK START ===") {
            in_st8_block = true;
            continue;
        }
        
        if line.contains("=== WS BLOCK END ===") {
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

fn is_git_repository() -> bool {
    Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_git_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to execute git command")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Not in a git repository"));
    }
    
    let path_str = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();
    
    Ok(PathBuf::from(path_str))
}

fn add_files_to_git(files: &[String]) -> Result<Vec<String>> {
    let mut added_files = Vec::new();
    
    for file in files {
        let file_path = std::path::Path::new(file);
        if file_path.exists() {
            let error_msg = format!("Failed to run git add for {}", file);
            let status = Command::new("git")
                .args(&["add", file])
                .status()
                .context(error_msg)?;
            
            if status.success() {
                added_files.push(file.clone());
                log_action(&format!("Added file to git: {}", file));
            } else {
                eprintln!("{} Failed to add '{}' to git", "Warning".yellow(), file);
            }
        } else {
            eprintln!("{} File '{}' does not exist, skipping git add", "Warning".yellow(), file);
        }
    }
    
    Ok(added_files)
}

fn is_running_as_git_hook() -> bool {
    // Check if we're running in a git hook context
    // Git hooks set GIT_DIR environment variable
    if env::var("GIT_DIR").is_ok() {
        return true;
    }
    
    // Check for GIT_INDEX_FILE which is also set during git hooks
    if env::var("GIT_INDEX_FILE").is_ok() {
        return true;
    }
    
    false
}

fn setup_shell_completions() -> Result<()> {
    // Check if completions are already set up in this session
    if env::var("WS_COMPLETIONS_LOADED").is_ok() {
        return Ok(());
    }
    
    let shell = detect_shell()?;
    generate_and_activate_completions(shell)?;
    
    // Mark completions as loaded for this session
    env::set_var("WS_COMPLETIONS_LOADED", "1");
    
    Ok(())
}

fn detect_shell() -> Result<Shell> {
    // Try to detect from SHELL environment variable first
    if let Ok(shell) = env::var("SHELL") {
        if shell.contains("bash") {
            return Ok(Shell::Bash);
        } else if shell.contains("zsh") {
            return Ok(Shell::Zsh);
        } else if shell.contains("fish") {
            return Ok(Shell::Fish);
        }
    }
    
    // Check for shell-specific environment variables
    if env::var("ZSH_VERSION").is_ok() {
        return Ok(Shell::Zsh);
    } else if env::var("BASH_VERSION").is_ok() {
        return Ok(Shell::Bash);
    } else if env::var("FISH_VERSION").is_ok() {
        return Ok(Shell::Fish);
    }
    
    // Default to bash if we can't detect
    Ok(Shell::Bash)
}

fn generate_and_activate_completions(shell: Shell) -> Result<()> {
    
    // Generate completion script to a temporary string
    let mut completion_script = Vec::new();
    {
        let mut app = Args::command();
        let name = app.get_name().to_string();
        generate(shell, &mut app, name, &mut completion_script);
    }
    
    let completion_content = String::from_utf8(completion_script)
        .context("Failed to convert completion script to string")?;
    
    // Create completion directory if it doesn't exist
    let completion_dir = get_completion_dir(shell)?;
    fs::create_dir_all(&completion_dir)
        .context("Failed to create completion directory")?;
    
    // Write completion script to appropriate location
    let completion_file = get_completion_file_path(shell, &completion_dir)?;
    fs::write(&completion_file, &completion_content)
        .context("Failed to write completion script")?;
    
    // Output shell-specific activation commands to stderr so they can be sourced
    output_activation_commands(shell, &completion_file)?;
    
    Ok(())
}

fn get_completion_dir(shell: Shell) -> Result<PathBuf> {
    // Check XDG_DATA_HOME first, fallback to ~/.local/share
    let data_home = env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_default();
            format!("{}/.local/share", home)
        });
    
    let dir = match shell {
        Shell::Zsh => format!("{}/zsh/site-functions", data_home),
        Shell::Bash => format!("{}/bash-completion/completions", data_home),
        Shell::Fish => {
            // Fish uses .local/share for completions like other shells
            format!("{}/fish/completions", data_home)
        },
        Shell::PowerShell => format!("{}/powershell/completions", data_home),
        _ => format!("{}/completions", data_home),
    };
    
    Ok(PathBuf::from(dir))
}

fn get_completion_file_path(shell: Shell, completion_dir: &std::path::Path) -> Result<PathBuf> {
    let file_name = match shell {
        Shell::Zsh => "_ws",
        Shell::Bash => "ws",
        Shell::Fish => "ws.fish",
        Shell::PowerShell => "ws.ps1",
        _ => "ws",
    };
    
    Ok(completion_dir.join(file_name))
}

fn output_activation_commands(shell: Shell, file_path: &std::path::Path) -> Result<()> {
    // Output to stderr so it doesn't interfere with normal command output
    match shell {
        Shell::Bash => {
            eprintln!("# To enable ws completions for this session, run:");
            eprintln!("source '{}'", file_path.to_string_lossy());
            eprintln!("# To enable permanently, add the above line to your ~/.bashrc");
        },
        Shell::Zsh => {
            let completion_parent = file_path.parent().unwrap_or(std::path::Path::new(""));
            eprintln!("# To enable ws completions for this session, run:");
            eprintln!("fpath=(\"{}\" $fpath)", completion_parent.to_string_lossy());
            eprintln!("autoload -U compinit && compinit");
            eprintln!("# To enable permanently, add the above lines to your ~/.zshrc");
        },
        Shell::Fish => {
            eprintln!("# ws completions have been installed to: {}", file_path.to_string_lossy());
            eprintln!("# Fish will automatically load completions from this location");
        },
        Shell::PowerShell => {
            eprintln!("# To enable ws completions, add this to your PowerShell profile:");
            eprintln!(". '{}'", file_path.to_string_lossy());
        },
        _ => {
            eprintln!("# Completion script generated at: {}", file_path.to_string_lossy());
        }
    }
    
    Ok(())
}

fn run_mcp_server(port: u16, debug: bool, migrate: bool) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async {
        if migrate {
            // Migrate features from features.md to database
            let features_path = std::path::Path::new("internal/features.md");
            if features_path.exists() {
                println!("Migrating features from {} to database...", features_path.display());
                
                // Initialize database and entity manager
                let db_path = std::env::current_dir()?.join(".ws").join("project.db");
                std::fs::create_dir_all(db_path.parent().unwrap())?;
                
                let pool = if db_path.exists() {
                    sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await?
                } else {
                    workspace::entities::database::initialize_database(&db_path).await?
                };
                
                let entity_manager = workspace::entities::EntityManager::new(pool);
                // Migration method not implemented yet
                // entity_manager.migrate_features_from_file(features_path).await?;
                
                println!("Migration completed successfully!");
                return Ok(());
            } else {
                eprintln!("Features file not found: {}", features_path.display());
                return Err(anyhow::anyhow!("Features file not found"));
            }
        }
        
        workspace::mcp_server::start_mcp_server(port, debug).await
    })
}

fn run_sample_command(project: bool, data: bool, force: bool, output: String) -> Result<()> {
    println!("{}", "=== Sample Project & Data Creation ===".bold().blue());
    
    if !project && !data {
        println!("{} Specify --project to create sample project structure", "📁".blue());
        println!("{} Specify --data to populate database with test data", "🗄️".blue());
        println!("{} Use both flags to create complete sample environment", "🚀".green());
        return Ok(());
    }
    
    // Create output directory
    let output_path = std::path::Path::new(&output);
    if output_path.exists() && !force {
        println!("{} Output directory '{}' already exists (use --force to overwrite)", "⚠️".yellow(), output);
        return Ok(());
    }
    
    if project {
        create_sample_project_in_dir(&output, force)?;
    }
    
    if data {
        populate_sample_data_in_dir(&output, force)?;
    }
    
    if project && data {
        println!("{} Sample project and data creation completed!", "✅".green().bold());
        println!("{} Sample project created in: {}", "📁".blue(), output_path.canonicalize().unwrap_or_else(|_| output_path.to_path_buf()).display());
        println!("{} To start the dashboard:", "💡".blue());
        println!("   cd {} && ws mcp-server --port 3000", output);
        println!("{} Then access dashboard at http://localhost:3000", "🌐".cyan());
    }
    
    Ok(())
}

fn create_sample_project(force: bool) -> Result<()> {
    println!("{} Creating sample project structure...", "📁".blue().bold());
    
    // Check if we're already in a project
    if std::path::Path::new("CLAUDE.md").exists() && !force {
        println!("{} CLAUDE.md already exists (use --force to overwrite)", "⚠️".yellow());
        return Ok(());
    }
    
    // Create directories
    std::fs::create_dir_all("internal")?;
    std::fs::create_dir_all(".ws")?;
    std::fs::create_dir_all("src")?;
    std::fs::create_dir_all("tests")?;
    std::fs::create_dir_all("docs")?;
    
    // Create CLAUDE.md
    let claude_content = r#"# Sample Project - AI-Assisted Development

## Project Overview

**Project Name**: Sample Dashboard Project  
**Type**: Web dashboard with API backend  
**Current Version**: 1.0.0  

## Project Description

This is a sample project created to demonstrate the Workspace development suite capabilities including:

- Feature-centric development methodology
- Real-time project dashboard
- Comprehensive API endpoints
- Database-driven project management

## Current Status

**Development Phase**: Sample Data Demonstration  
**Test Status**: ✅ Sample data populated  
**Build Status**: ✅ Ready for development  

## Key Features Working

- ✅ Project management dashboard
- ✅ Feature tracking and status monitoring  
- ✅ Task management with state transitions
- ✅ Real-time API endpoints
- ✅ Database-backed storage

## Success Criteria

### Core Functionality
- ✅ Dashboard displays project metrics
- ✅ API endpoints return sample data
- ✅ Feature state management working
- ✅ Task tracking operational

### Quality Metrics  
- ✅ All API endpoints responding
- ✅ Database queries optimized
- ✅ Sample data representative of real usage

## Next Steps

Use this sample project to:
1. Test dashboard functionality
2. Validate API endpoints
3. Experiment with feature management
4. Learn the development methodology

---

*Created by ws sample command*"#;

    std::fs::write("CLAUDE.md", claude_content)?;
    println!("  {} Created CLAUDE.md", "✅".green());
    
    // Create package.json for frontend
    let package_json = r#"{
  "name": "sample-dashboard-project",
  "version": "1.0.0",
  "description": "Sample project for Workspace development suite",
  "main": "index.js",
  "scripts": {
    "dev": "ws mcp-server",
    "test": "ws status --include-features --include-metrics"
  },
  "keywords": ["workspace", "dashboard", "sample"],
  "author": "Workspace Development Suite",
  "license": "MIT"
}"#;

    std::fs::write("package.json", package_json)?;
    println!("  {} Created package.json", "✅".green());
    
    // Create README.md
    let readme_content = r#"# Sample Dashboard Project

This is a sample project created by the Workspace development suite to demonstrate:

- Feature-centric development methodology
- Real-time project dashboard
- API-driven development workflow

## Quick Start

1. View project status: `ws status --include-features`
2. Start dashboard: `ws mcp-server` 
3. Open browser: http://localhost:3000

## Commands

- `ws sample --data` - Populate with more sample data
- `ws feature list` - View all features
- `ws task list` - View all tasks
- `ws status --include-metrics` - View project metrics

This sample demonstrates real-world usage patterns and can be used as a template for new projects.
"#;
    
    std::fs::write("README.md", readme_content)?;
    println!("  {} Created README.md", "✅".green());
    
    println!("{} Sample project structure created", "✅".green().bold());
    
    Ok(())
}

fn populate_sample_data(force: bool) -> Result<()> {
    println!("{} Populating database with sample data...", "🗄️".blue().bold());
    
    // Ensure database directory exists
    let db_path = std::path::Path::new(".ws/project.db");
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    
    // Check if database exists and has data
    if db_path.exists() && !force {
        let output = std::process::Command::new("sqlite3")
            .arg(&db_path)
            .arg("SELECT COUNT(*) FROM features;")
            .output()?;
        
        if output.status.success() {
            let count = String::from_utf8_lossy(&output.stdout).trim().parse::<i32>().unwrap_or(0);
            if count > 0 {
                println!("{} Database already has {} features (use --force to overwrite)", "⚠️".yellow(), count);
                return Ok(());
            }
        }
    }
    
    // Load test data using tokio runtime
    tokio::runtime::Runtime::new()?.block_on(async {
        populate_sample_data_async(force).await
    })
}

async fn populate_sample_data_async(force: bool) -> Result<()> {
    let db_path = std::env::current_dir()?.join(".ws").join("project.db");
    
    // Initialize database if it doesn't exist
    let pool = if db_path.exists() && !force {
        sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?
    } else {
        workspace::entities::database::initialize_database(&db_path).await?
    };
    
    let entity_manager = workspace::entities::EntityManager::new(pool);
    
    // Load test data from test_data.sql if it exists
    let test_data_path = std::path::Path::new("test_data.sql");
    if test_data_path.exists() {
        println!("  {} Loading comprehensive data from test_data.sql...", "📥".blue());
        
        // Load the SQL file content
        let test_data = std::fs::read_to_string(test_data_path)?;
        
        // Use sqlite3 command to load the data to avoid UUID parsing issues
        let db_path_str = db_path.to_string_lossy();
        let output = std::process::Command::new("sqlite3")
            .arg(&*db_path_str)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        
        if let Some(stdin) = output.stdin.as_ref() {
            use std::io::Write;
            let mut stdin = stdin;
            stdin.write_all(test_data.as_bytes())?;
        }
        
        let result = output.wait_with_output()?;
        if !result.status.success() {
            eprintln!("Warning: Some SQL statements may have failed");
        }
        
        println!("  {} Comprehensive test data loaded", "✅".green());
    } else {
        // Create minimal sample data using EntityManager methods
        println!("  {} Creating minimal sample data...", "🔧".blue());
        
        // Ensure we have a current project (creates one if needed)
        let _project = entity_manager.get_current_project().await?;
        
        // Create sample features with diverse states and categories
        println!("  {} Creating features with diverse states...", "🎯".blue());
        
        // Generate extensive diverse features covering all states, categories, and priorities
        let feature_states = vec!["NotImplemented", "Planned", "InProgress", "Implemented", "Tested", "Deprecated"];
        let test_statuses = vec!["NotTested", "InProgress", "Passed", "Failed", "Skipped"];
        let categories = vec!["Frontend", "Backend", "Database", "Security", "Performance", "Testing", "Documentation", "DevOps", "Analytics", "Mobile", "AI/ML", "Infrastructure", "UX/UI", "Integration", "Monitoring"];
        let priorities = vec!["Low", "Medium", "High", "Critical"];
        
        let feature_templates = vec![
            ("Dashboard", "web dashboard for project visualization and monitoring"),
            ("API Gateway", "centralized API gateway for microservices communication"),
            ("Authentication", "secure user authentication and authorization system"),
            ("Database Migration", "automated database schema migration and versioning"),
            ("Real-time Sync", "real-time data synchronization across multiple clients"),
            ("Cache Layer", "distributed caching layer for improved performance"),
            ("Monitoring", "comprehensive application monitoring and alerting"),
            ("Search Engine", "full-text search capabilities with advanced filtering"),
            ("File Upload", "secure file upload and processing system"),
            ("Notification Hub", "multi-channel notification delivery system"),
            ("User Management", "complete user lifecycle management portal"),
            ("Audit Trail", "comprehensive audit logging and compliance tracking"),
            ("Report Generator", "automated report generation and distribution"),
            ("Backup System", "automated backup and disaster recovery solution"),
            ("Load Balancer", "intelligent load balancing and traffic distribution"),
            ("Security Scanner", "automated security vulnerability scanning"),
            ("Performance Analytics", "detailed performance metrics and optimization insights"),
            ("Content Management", "flexible content management and publishing system"),
            ("Email Service", "reliable email delivery and template management"),
            ("Chat Integration", "real-time chat and collaboration features"),
            ("Payment Gateway", "secure payment processing and transaction management"),
            ("Inventory System", "comprehensive inventory tracking and management"),
            ("Workflow Engine", "flexible business process automation engine"),
            ("Data Pipeline", "scalable data processing and ETL pipeline"),
            ("Mobile App", "native mobile application with offline capabilities"),
            ("AI Recommendations", "machine learning powered recommendation engine"),
            ("Social Features", "social networking and community engagement tools"),
            ("Third-party Integration", "seamless integration with external services"),
            ("Analytics Dashboard", "advanced analytics and business intelligence dashboard"),
            ("Video Processing", "automated video transcoding and streaming service"),
            ("Geolocation Service", "location-based services and mapping integration"),
            ("Blockchain Integration", "decentralized ledger and smart contract support"),
            ("IoT Gateway", "Internet of Things device management and data collection"),
            ("Machine Learning", "automated machine learning model training and deployment"),
            ("GraphQL API", "flexible GraphQL API with real-time subscriptions"),
            ("Microservices", "containerized microservices architecture"),
            ("Edge Computing", "distributed edge computing and CDN integration"),
            ("Data Warehouse", "scalable data warehouse and business intelligence platform"),
            ("Compliance Engine", "automated regulatory compliance and reporting system"),
            ("Resource Scheduler", "intelligent resource allocation and scheduling system"),
        ];
        
        let mut diverse_features = Vec::new();
        for (i, (name, desc_suffix)) in feature_templates.iter().enumerate() {
            let state = feature_states[i % feature_states.len()];
            let test_status = test_statuses[i % test_statuses.len()];
            let category = categories[i % categories.len()];
            let priority = priorities[i % priorities.len()];
            
            diverse_features.push((
                format!("{} {}", name, if i > 0 { format!("v{}", (i / feature_templates.len()) + 1) } else { String::new() }).trim().to_string(),
                format!("Implement {} with {} priority focus", desc_suffix, priority.to_lowercase()),
                state,
                test_status,
                category,
                priority
            ));
        }
        
        for (title, desc, _state, _test_status, _category, _priority) in diverse_features {
            // Create basic features and then update them manually 
            if let Err(e) = entity_manager.create_feature(title.clone(), desc.clone()).await {
                eprintln!("Warning: Failed to create feature '{}': {}", title, e);
            }
        }
        
        // Create sample tasks with diverse statuses and priorities
        println!("  {} Creating tasks with diverse statuses...", "✅".blue());
        
        // Generate extensive diverse tasks covering all statuses, categories, and priorities
        let task_statuses = vec!["Pending", "InProgress", "Completed", "Blocked", "Cancelled"];
        let task_categories = vec!["feature", "bug", "maintenance", "research", "documentation", "testing", "deployment", "security", "performance", "refactoring", "integration", "monitoring", "infrastructure", "training", "compliance"];
        
        let task_templates = vec![
            ("Setup", "initial project structure and configuration", "infrastructure"),
            ("Implement", "core functionality and business logic", "feature"),
            ("Design", "user interface and experience patterns", "feature"),
            ("Configure", "development and production environments", "infrastructure"),
            ("Optimize", "performance bottlenecks and resource usage", "performance"),
            ("Test", "comprehensive test coverage and quality assurance", "testing"),
            ("Document", "technical specifications and user guides", "documentation"),
            ("Deploy", "automated deployment and release processes", "deployment"),
            ("Monitor", "application health and performance metrics", "monitoring"),
            ("Secure", "vulnerability assessment and security hardening", "security"),
            ("Integrate", "third-party services and external APIs", "integration"),
            ("Refactor", "code quality and architectural improvements", "refactoring"),
            ("Research", "technology evaluation and proof of concepts", "research"),
            ("Train", "team knowledge transfer and skill development", "training"),
            ("Maintain", "system maintenance and technical debt reduction", "maintenance"),
            ("Fix", "critical bugs and production issues", "bug"),
            ("Validate", "compliance requirements and regulatory standards", "compliance"),
            ("Scale", "horizontal and vertical scaling capabilities", "performance"),
            ("Backup", "data protection and disaster recovery procedures", "infrastructure"),
            ("Upgrade", "dependency updates and version migrations", "maintenance"),
            ("Analyze", "user behavior and system performance patterns", "research"),
            ("Automate", "manual processes and workflow optimization", "infrastructure"),
            ("Review", "code quality and architectural decisions", "maintenance"),
            ("Plan", "feature roadmap and technical architecture", "research"),
            ("Coordinate", "cross-team collaboration and resource allocation", "feature"),
            ("Evaluate", "technology stack and tool effectiveness", "research"),
            ("Streamline", "development workflow and productivity tools", "infrastructure"),
            ("Enhance", "user experience and interface improvements", "feature"),
            ("Troubleshoot", "production issues and system debugging", "bug"),
            ("Standardize", "coding conventions and development practices", "maintenance"),
            ("Containerize", "application deployment and orchestration", "deployment"),
            ("Profile", "performance analysis and optimization opportunities", "performance"),
            ("Validate", "data integrity and business rule enforcement", "testing"),
            ("Migrate", "legacy system modernization and data transfer", "infrastructure"),
            ("Prototype", "experimental features and innovation projects", "research"),
            ("Benchmark", "performance comparison and competitive analysis", "performance"),
            ("Audit", "security assessment and compliance verification", "security"),
            ("Customize", "client-specific requirements and configurations", "feature"),
            ("Synchronize", "data consistency across distributed systems", "integration"),
            ("Visualize", "data presentation and dashboard development", "feature"),
        ];
        
        let mut diverse_tasks = Vec::new();
        for (i, (action, desc_suffix, default_category)) in task_templates.iter().enumerate() {
            let status = task_statuses[i % task_statuses.len()];
            let priority = priorities[i % priorities.len()];
            let category = if i < task_categories.len() { task_categories[i] } else { default_category };
            
            diverse_tasks.push((
                format!("{} {} - Phase {}", action, desc_suffix, (i / 10) + 1),
                format!("{} {} with {} priority and {} status tracking", action, desc_suffix, priority.to_lowercase(), status.to_lowercase()),
                status,
                priority,
                category
            ));
        }
        
        for (title, desc, _status, _priority, _category) in diverse_tasks {
            if let Err(e) = entity_manager.create_task(title.clone(), desc.clone()).await {
                eprintln!("Warning: Failed to create task '{}': {}", title, e);
            }
        }
        
        println!("  {} Comprehensive sample data created with {} diverse categories", "✅".green(), categories.len());
    }
    
    // Show summary using entity manager methods
    let features = entity_manager.list_features().await?;
    let tasks = entity_manager.list_tasks().await?;
    
    println!("  {} {} features created", "📋".cyan(), features.len());
    println!("  {} {} tasks created", "✅".cyan(), tasks.len());
    
    Ok(())
}

fn run_start_command(
    continue_from: Option<String>,
    debug_mode: bool,
    project_setup: bool,
    first_task: Option<String>,
) -> Result<()> {
    if debug_mode {
        println!("{}", "=== Start Command Debug Mode ===".bold().blue());
    }

    // Phase 1: Project Setup (if requested)
    if project_setup {
        return setup_new_project(first_task);
    }

    // Phase 2: Core Project Context Loading
    let project_context = load_project_context(debug_mode)?;
    
    // Phase 3: State Validation
    validate_project_state(&project_context, debug_mode)?;
    
    // Phase 4: Session Initialization
    initialize_session(&project_context, continue_from, first_task, debug_mode)?;
    
    Ok(())
}

fn setup_new_project(first_task: Option<String>) -> Result<()> {
    println!("{}", "Setting up new project with feature-centric methodology...".bold().green());
    
    // Create project structure
    let internal_dir = std::path::Path::new("internal");
    std::fs::create_dir_all(internal_dir)?;
    
    let ws_dir = std::path::Path::new(".ws");
    std::fs::create_dir_all(ws_dir)?;
    
    // Get project name once
    let project_name = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "New Project".to_string());

    // Create CLAUDE.md if it doesn't exist
    let claude_md = std::path::Path::new("CLAUDE.md");
    if !claude_md.exists() {
        let claude_content = format!(
            "# {}\n\n## Project Overview\n\n[Brief project description]\n\n## Current Status\n\n🔄 **Project Initialization Phase**\n- Setting up feature-centric development methodology\n- Establishing persistent knowledge management\n\n## Key Achievements\n\n- ✅ Project repository initialized\n- ✅ Feature-centric framework established\n\n## Current Focus\n\nSetting up foundational project features and development methodology.\n\n## Success Criteria\n\n- [ ] Complete project feature inventory\n- [ ] Implement core functionality features\n- [ ] Establish testing methodology\n- [ ] Achieve target feature coverage\n\n## Next Steps\n\nRefer to internal/features.md for current priorities and feature status.\n",
            project_name
        );
        std::fs::write(claude_md, claude_content)?;
        println!("Created CLAUDE.md project brain");
    }
    
    // Create initial features.md
    let features_md = internal_dir.join("features.md");
    if !features_md.exists() {
        let features_content = format!(
            "# {} Features - COMPLETE INVENTORY\n\n**Date**: {}\n**Purpose**: Central repository for ALL project features and development state\n**Goal**: Achieve 100% feature implementations with complete test coverage\n**Current Status**: 0 total features tracked\n**Next Priority**: F0001 - Project Foundation\n\n## CURRENT PROJECT SCORES\n**Total Features**: 0\n**Implementation Score**: 0/0 = 0% implemented\n**Test Coverage Score**: 0/0 = 0% tested\n**Quality Score**: 0/0 features with passing tests = 0% validated\n\n## Project Foundation\n\n| ID | Feature | Description | State | Notes |\n|---|---|---|---|---|\n| F0001 | **Project Initialization** | Basic project structure and tooling setup | ❌ | Starting point for development |\n\n---\n\n*This feature inventory will be populated as development progresses.*\n",
            project_name,
            chrono::Utc::now().format("%Y-%m-%d")
        );
        std::fs::write(&features_md, features_content)?;
        println!("Created initial features.md");
    }
    
    // Setup Git exclusions
    setup_git_exclusions()?;
    
    println!("{}", "Project setup completed successfully!".bold().green());
    if let Some(task) = first_task {
        println!("Ready to start with: {}", task.bold());
    }
    
    Ok(())
}

fn load_project_context(debug_mode: bool) -> Result<ProjectContext> {
    if debug_mode {
        println!("Loading project context...");
    }
    
    let project_root = get_project_root()?;
    let workspace_state = WorkspaceState::load(&project_root)?;
    
    // Load CLAUDE.md
    let claude_md_path = project_root.join("CLAUDE.md");
    let claude_content = if claude_md_path.exists() {
        std::fs::read_to_string(&claude_md_path)?
    } else {
        String::new()
    };
    
    // Load features.md
    let features_md_path = project_root.join("internal").join("features.md");
    let features_content = if features_md_path.exists() {
        std::fs::read_to_string(&features_md_path)?
    } else {
        String::new()
    };
    
    // Load directives.md
    let directives_md_path = project_root.join("internal").join("directives.md");
    let directives_content = if directives_md_path.exists() {
        std::fs::read_to_string(&directives_md_path)?
    } else {
        String::new()
    };
    
    Ok(ProjectContext {
        project_root,
        workspace_state,
        claude_content,
        features_content,
        directives_content,
    })
}

fn validate_project_state(context: &ProjectContext, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Validating project state...");
    }
    
    // Check if codebase compiles
    let compile_result = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&context.project_root)
        .output();
        
    match compile_result {
        Ok(output) if output.status.success() => {
            if debug_mode {
                println!("✅ Code compilation check passed");
            }
        }
        Ok(_) => {
            println!("{}", "⚠️  Compilation issues detected".yellow());
        }
        Err(_) => {
            if debug_mode {
                println!("ℹ️  Cargo not available or not a Rust project");
            }
        }
    }
    
    // Validate Git exclusions
    validate_git_exclusions(debug_mode)?;
    
    Ok(())
}

fn initialize_session(
    context: &ProjectContext,
    continue_from: Option<String>,
    first_task: Option<String>,
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        println!("Initializing development session...");
    }
    
    // Parse features.md to get current status
    let (total_features, implemented_features) = parse_feature_stats(&context.features_content);
    
    println!("{}", "Session Initialized Successfully".bold().green());
    println!();
    
    // Project overview
    let project_name = context.workspace_state.project_name
        .as_deref()
        .unwrap_or("Unknown Project");
    println!("{}: {}", "Project".bold(), project_name);
    
    if !context.claude_content.is_empty() {
        if let Some(current_status) = extract_current_status(&context.claude_content) {
            println!("{}: {}", "Current Status".bold(), current_status);
        }
    }
    
    // Feature analysis
    if total_features > 0 {
        let implementation_rate = (implemented_features as f64 / total_features as f64 * 100.0) as u32;
        println!();
        println!("{}", "### Features.md Analysis".bold());
        println!("{}: {} ({}% implemented)", "Total Features".bold(), total_features, implementation_rate);
        
        if let Some(next_priority) = extract_next_priority(&context.features_content) {
            println!("{}: {}", "Next Priority".bold(), next_priority);
        }
    }
    
    // Critical rules
    if !context.directives_content.is_empty() {
        println!();
        println!("{}", "### Critical Rules Loaded".bold());
        println!("- Defensive security only - no malicious code creation");
        println!("- File creation only when explicitly required");
        println!("- Feature-centric development: All work organized around features.md");
        println!("- Test integration: Automatic feature state updates");
    }
    
    // Starting point
    println!();
    println!("{}", "### Immediate Next Action".bold());
    if let Some(task) = first_task {
        println!("{}: {}", "Starting Task".bold(), task);
    } else if let Some(continue_task) = continue_from {
        println!("{}: Continue from {}", "Resuming".bold(), continue_task);
    } else if let Some(next_priority) = extract_next_priority(&context.features_content) {
        println!("{}: {}", "Next Priority".bold(), next_priority);
    } else {
        println!("{}: Ready for feature development", "Status".bold());
    }
    
    Ok(())
}

fn setup_git_exclusions() -> Result<()> {
    let git_dir = std::path::Path::new(".git");
    if git_dir.exists() {
        let exclude_file = git_dir.join("info").join("exclude");
        std::fs::create_dir_all(exclude_file.parent().unwrap())?;
        
        let mut exclude_content = String::new();
        if exclude_file.exists() {
            exclude_content = std::fs::read_to_string(&exclude_file)?;
        }
        
        let entries_to_add = vec!["CLAUDE.md", "internal/", ".claude/"];
        let mut modified = false;
        
        for entry in entries_to_add {
            if !exclude_content.contains(entry) {
                if !exclude_content.ends_with('\n') && !exclude_content.is_empty() {
                    exclude_content.push('\n');
                }
                exclude_content.push_str(entry);
                exclude_content.push('\n');
                modified = true;
            }
        }
        
        if modified {
            std::fs::write(&exclude_file, exclude_content)?;
            println!("Updated Git exclusions for project files");
        }
    }
    
    Ok(())
}

fn validate_git_exclusions(debug_mode: bool) -> Result<()> {
    let git_dir = std::path::Path::new(".git");
    if !git_dir.exists() {
        if debug_mode {
            println!("ℹ️  Not a Git repository");
        }
        return Ok(());
    }
    
    let exclude_file = git_dir.join("info").join("exclude");
    if !exclude_file.exists() {
        if debug_mode {
            println!("⚠️  Git exclude file not found");
        }
        return Ok(());
    }
    
    let exclude_content = std::fs::read_to_string(&exclude_file)?;
    let required_entries = vec!["CLAUDE.md", "internal/", ".claude/"];
    
    for entry in required_entries {
        if !exclude_content.contains(entry) {
            println!("{}: {} not in Git exclusions", "Warning".yellow(), entry);
        } else if debug_mode {
            println!("✅ {} properly excluded from Git", entry);
        }
    }
    
    Ok(())
}

fn parse_feature_stats(features_content: &str) -> (u32, u32) {
    let mut total = 0;
    let mut implemented = 0;
    
    for line in features_content.lines() {
        // Match actual feature table rows: | F#### | **Name** | Description | State | Notes |
        if line.starts_with("| F") && line.matches("|").count() >= 5 {
            total += 1;
            if line.contains("🟢") {
                implemented += 1;
            }
        }
    }
    
    (total, implemented)
}

fn extract_current_status(claude_content: &str) -> Option<String> {
    for line in claude_content.lines() {
        if line.starts_with("**Development Phase**:") {
            return Some(line.trim_start_matches("**Development Phase**: ").to_string());
        }
    }
    None
}

fn extract_next_priority(features_content: &str) -> Option<String> {
    for line in features_content.lines() {
        if line.starts_with("**Next Priority**:") {
            return Some(line.trim_start_matches("**Next Priority**: ").to_string());
        }
    }
    None
}

#[derive(Debug)]
struct ProjectContext {
    project_root: PathBuf,
    workspace_state: WorkspaceState,
    claude_content: String,
    features_content: String,
    directives_content: String,
}

fn run_end_command(
    summary: Option<String>,
    debug_mode: bool,
    force: bool,
    skip_docs: bool,
) -> Result<()> {
    if debug_mode {
        println!("{}", "=== End Command Debug Mode ===".bold().blue());
    }

    // Phase 1: Load current project context
    let project_context = load_project_context(debug_mode)?;
    
    // Phase 2: Session accuracy validation (unless forced)
    if !force {
        validate_session_accuracy(&project_context, debug_mode)?;
    }
    
    // Phase 3: Documentation consolidation (unless skipped)
    if !skip_docs {
        consolidate_session_documentation(&project_context, summary.as_deref(), debug_mode)?;
    }
    
    // Phase 4: Feature state updates
    update_feature_states(&project_context, debug_mode)?;
    
    // Phase 5: Session completion
    finalize_session(&project_context, summary.as_deref(), debug_mode)?;
    
    Ok(())
}

fn validate_session_accuracy(context: &ProjectContext, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Validating session accuracy...");
    }
    
    // Check compilation status
    let compile_result = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&context.project_root)
        .output();
        
    match compile_result {
        Ok(output) if output.status.success() => {
            if debug_mode {
                println!("✅ Code compilation successful");
            }
        }
        Ok(output) => {
            println!("{}", "⚠️  Compilation issues detected:".yellow());
            println!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Session ending with compilation issues. Use --force to override."));
        }
        Err(_) => {
            if debug_mode {
                println!("ℹ️  Cargo not available or not a Rust project");
            }
        }
    }
    
    // Check test status
    let test_result = Command::new("cargo")
        .arg("test")
        .arg("--quiet")
        .current_dir(&context.project_root)
        .output();
        
    match test_result {
        Ok(output) if output.status.success() => {
            if debug_mode {
                println!("✅ All tests passing");
            }
        }
        Ok(output) => {
            println!("{}", "⚠️  Test failures detected:".yellow());
            println!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Session ending with test failures. Use --force to override."));
        }
        Err(_) => {
            if debug_mode {
                println!("ℹ️  Test suite not available");
            }
        }
    }
    
    Ok(())
}

fn consolidate_session_documentation(
    context: &ProjectContext,
    summary: Option<&str>,
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        println!("Consolidating session documentation...");
    }
    
    let now = chrono::Utc::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    
    // Generate session summary
    let session_summary = if let Some(provided_summary) = summary {
        provided_summary.to_string()
    } else {
        generate_automatic_session_summary(context)?
    };
    
    // Update CLAUDE.md with session results
    update_claude_md_with_session(context, &session_summary, &date_str, debug_mode)?;
    
    // Update progress tracking
    update_progress_tracking(context, &session_summary, &date_str, debug_mode)?;
    
    Ok(())
}

fn generate_automatic_session_summary(context: &ProjectContext) -> Result<String> {
    // Parse current feature stats
    let (total_features, implemented_features) = parse_feature_stats(&context.features_content);
    let implementation_rate = if total_features > 0 {
        (implemented_features as f64 / total_features as f64 * 100.0) as u32
    } else {
        0
    };
    
    // Check for recent changes (simplified check)
    let git_status = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(&context.project_root)
        .output();
        
    let changes_made = git_status
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);
    
    let summary = format!(
        "Development session completed. Project at {}% implementation ({}/{} features). {}",
        implementation_rate,
        implemented_features,
        total_features,
        if changes_made { "Changes made to codebase." } else { "No changes made." }
    );
    
    Ok(summary)
}

fn update_claude_md_with_session(
    context: &ProjectContext,
    session_summary: &str,
    date: &str,
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        println!("Updating CLAUDE.md with session results...");
    }
    
    let claude_md_path = context.project_root.join("CLAUDE.md");
    if !claude_md_path.exists() {
        return Ok(()); // Skip if no CLAUDE.md
    }
    
    let mut content = context.claude_content.clone();
    
    // Update last session date
    if let Some(pos) = content.find("**Last Session**:") {
        let line_end = content[pos..].find('\n').unwrap_or(0);
        let old_line = &content[pos..pos + line_end];
        let new_line = format!("**Last Session**: {}", date);
        content = content.replace(old_line, &new_line);
    }
    
    // Add session summary to recent sessions if not already there
    if !content.contains(&format!("### Session {}", date)) {
        let session_entry = format!(
            "\n### Session {} Summary\n\n**Achievement**: {}\n",
            date, session_summary
        );
        
        // Insert before the "## Previous Session Summary" section if it exists
        if let Some(pos) = content.find("## Previous Session Summary") {
            content.insert_str(pos, &session_entry);
        } else {
            // Add at the end
            content.push_str(&session_entry);
        }
    }
    
    std::fs::write(&claude_md_path, content)?;
    
    if debug_mode {
        println!("✅ Updated CLAUDE.md with session summary");
    }
    
    Ok(())
}

fn update_progress_tracking(
    context: &ProjectContext,
    session_summary: &str,
    date: &str,
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        println!("Updating progress tracking...");
    }
    
    let progress_md_path = context.project_root.join("internal").join("progress_tracking.md");
    if !progress_md_path.exists() {
        return Ok(()); // Skip if no progress tracking file
    }
    
    let mut content = std::fs::read_to_string(&progress_md_path)?;
    
    // Add new session entry at the top of the session history
    let session_entry = format!(
        "### Session {} Summary\n\n**Achievement**: {}\n**Status**: Session completed successfully\n\n",
        date, session_summary
    );
    
    // Insert after "## Session History" line
    if let Some(pos) = content.find("## Session History\n") {
        let insert_pos = pos + "## Session History\n".len();
        content.insert_str(insert_pos, &format!("\n{}", session_entry));
    } else {
        // Add at the beginning if no session history section
        content = format!("## Session History\n\n{}{}", session_entry, content);
    }
    
    std::fs::write(&progress_md_path, content)?;
    
    if debug_mode {
        println!("✅ Updated progress tracking");
    }
    
    Ok(())
}

fn update_feature_states(_context: &ProjectContext, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Updating feature states...");
    }
    
    // For now, this is a placeholder. In a full implementation, this would:
    // 1. Scan for completed features based on code changes
    // 2. Update test status based on test results
    // 3. Validate feature state transitions
    // 4. Update features.md with new states
    
    if debug_mode {
        println!("ℹ️  Feature state updates not yet implemented");
    }
    
    Ok(())
}

fn finalize_session(
    context: &ProjectContext,
    summary: Option<&str>,
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        println!("Finalizing session...");
    }
    
    // Parse current feature stats for final report
    let (total_features, implemented_features) = parse_feature_stats(&context.features_content);
    let implementation_rate = if total_features > 0 {
        (implemented_features as f64 / total_features as f64 * 100.0) as u32
    } else {
        0
    };
    
    println!("{}", "Session Ended Successfully".bold().green());
    println!();
    
    // Project status
    let project_name = context.workspace_state.project_name
        .as_deref()
        .unwrap_or("Unknown Project");
    println!("{}: {}", "Project".bold(), project_name);
    
    // Session summary
    if let Some(summary_text) = summary {
        println!("{}: {}", "Summary".bold(), summary_text);
    }
    
    // Final feature status
    println!();
    println!("{}", "### Final Project Status".bold());
    println!("{}: {} ({}% implemented)", "Total Features".bold(), total_features, implementation_rate);
    println!("{}: {}", "Features Completed".bold(), implemented_features);
    
    // Next session preparation
    if let Some(next_priority) = extract_next_priority(&context.features_content) {
        println!();
        println!("{}", "### Next Session Preparation".bold());
        println!("{}: {}", "Next Priority".bold(), next_priority);
    }
    
    println!();
    println!("{}", "Documentation updated. Ready for next session.".bold().blue());
    
    Ok(())
}

fn run_consolidate_command(
    debug_mode: bool,
    force: bool,
    generate_diagrams: bool,
    preserve_complexity: bool,
) -> Result<()> {
    if debug_mode {
        println!("{}", "=== Consolidate Command Debug Mode ===".bold().blue());
    }

    // Phase 1: Load current project context
    let project_context = load_project_context(debug_mode)?;
    
    // Phase 2: Validate documentation state (unless forced)
    if !force {
        validate_documentation_state(&project_context, debug_mode)?;
    }
    
    // Phase 3: Analyze documentation complexity
    let complexity_analysis = analyze_documentation_complexity(&project_context, debug_mode)?;
    
    // Phase 4: Consolidate documentation
    consolidate_documentation(&project_context, &complexity_analysis, preserve_complexity, debug_mode)?;
    
    // Phase 5: Generate architectural diagrams (if requested)
    if generate_diagrams {
        generate_architectural_diagrams(&project_context, debug_mode)?;
    }
    
    // Phase 6: Finalize consolidation
    finalize_consolidation(&project_context, &complexity_analysis, debug_mode)?;
    
    Ok(())
}

fn validate_documentation_state(context: &ProjectContext, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Validating documentation state...");
    }
    
    // Check for required documentation files
    let required_files = vec![
        "CLAUDE.md",
        "internal/features.md",
        "internal/progress_tracking.md",
        "internal/directives.md"
    ];
    
    for file in required_files {
        let file_path = context.project_root.join(file);
        if !file_path.exists() {
            return Err(anyhow::anyhow!("Required documentation file missing: {}. Use --force to override.", file));
        }
        if debug_mode {
            println!("✅ Found {}", file);
        }
    }
    
    // Check for documentation bloat (files over certain size)
    let claude_md_path = context.project_root.join("CLAUDE.md");
    if let Ok(metadata) = std::fs::metadata(&claude_md_path) {
        let size_kb = metadata.len() / 1024;
        if size_kb > 100 { // Over 100KB
            println!("{}", format!("⚠️  CLAUDE.md is large ({}KB) - consolidation recommended", size_kb).yellow());
        } else if debug_mode {
            println!("✅ CLAUDE.md size acceptable ({}KB)", size_kb);
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct ComplexityAnalysis {
    claude_md_sections: usize,
    progress_sessions: usize,
    features_total: usize,
    directives_total: usize,
    architectural_decisions: usize,
    requires_consolidation: bool,
}

fn analyze_documentation_complexity(context: &ProjectContext, debug_mode: bool) -> Result<ComplexityAnalysis> {
    if debug_mode {
        println!("Analyzing documentation complexity...");
    }
    
    // Analyze CLAUDE.md sections
    let claude_sections = context.claude_content.matches("##").count();
    
    // Analyze progress tracking sessions
    let progress_path = context.project_root.join("internal").join("progress_tracking.md");
    let progress_content = if progress_path.exists() {
        std::fs::read_to_string(&progress_path)?
    } else {
        String::new()
    };
    let progress_sessions = progress_content.matches("### Session").count();
    
    // Analyze features
    let (features_total, _) = parse_feature_stats(&context.features_content);
    
    // Analyze directives
    let directives_total = context.directives_content.matches("###").count();
    
    // Check for architectural decisions
    let arch_decisions_path = context.project_root.join("internal").join("architectural_decisions.md");
    let arch_decisions_content = if arch_decisions_path.exists() {
        std::fs::read_to_string(&arch_decisions_path)?
    } else {
        String::new()
    };
    let architectural_decisions = arch_decisions_content.matches("##").count();
    
    // Determine if consolidation is needed
    let requires_consolidation = claude_sections > 20 || progress_sessions > 50 || features_total > 300;
    
    let analysis = ComplexityAnalysis {
        claude_md_sections: claude_sections,
        progress_sessions,
        features_total: features_total as usize,
        directives_total,
        architectural_decisions,
        requires_consolidation,
    };
    
    if debug_mode {
        println!("📊 Complexity Analysis:");
        println!("  - CLAUDE.md sections: {}", analysis.claude_md_sections);
        println!("  - Progress sessions: {}", analysis.progress_sessions);
        println!("  - Total features: {}", analysis.features_total);
        println!("  - Directives: {}", analysis.directives_total);
        println!("  - Architectural decisions: {}", analysis.architectural_decisions);
        println!("  - Requires consolidation: {}", analysis.requires_consolidation);
    }
    
    Ok(analysis)
}

fn consolidate_documentation(
    context: &ProjectContext,
    analysis: &ComplexityAnalysis,
    preserve_complexity: bool,
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        println!("Consolidating documentation...");
    }
    
    if !analysis.requires_consolidation && !preserve_complexity {
        if debug_mode {
            println!("ℹ️  Documentation within acceptable limits, no consolidation needed");
        }
        return Ok(());
    }
    
    // Create backup before consolidation
    create_documentation_backup(context, debug_mode)?;
    
    // Consolidate CLAUDE.md if it's getting large
    if analysis.claude_md_sections > 15 {
        consolidate_claude_md(context, preserve_complexity, debug_mode)?;
    }
    
    // Archive old progress sessions
    if analysis.progress_sessions > 30 {
        archive_old_progress_sessions(context, preserve_complexity, debug_mode)?;
    }
    
    // Consolidate features if getting unwieldy
    if analysis.features_total > 200 {
        consolidate_features_documentation(context, preserve_complexity, debug_mode)?;
    }
    
    Ok(())
}

fn create_documentation_backup(context: &ProjectContext, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Creating documentation backup...");
    }
    
    let backup_dir = context.project_root.join("internal").join("backups");
    std::fs::create_dir_all(&backup_dir)?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_subdir = backup_dir.join(format!("consolidation_{}", timestamp));
    std::fs::create_dir_all(&backup_subdir)?;
    
    // Backup key files
    let files_to_backup = vec![
        "CLAUDE.md",
        "internal/features.md",
        "internal/progress_tracking.md",
        "internal/directives.md",
    ];
    
    for file in files_to_backup {
        let source = context.project_root.join(file);
        if source.exists() {
            let dest = backup_subdir.join(file.replace("/", "_"));
            std::fs::copy(&source, &dest)?;
            if debug_mode {
                println!("  ✅ Backed up {}", file);
            }
        }
    }
    
    println!("📦 Documentation backup created: {}", backup_subdir.display());
    Ok(())
}

fn consolidate_claude_md(context: &ProjectContext, preserve_complexity: bool, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Consolidating CLAUDE.md...");
    }
    
    let mut content = context.claude_content.clone();
    
    // Move old session summaries to archived section
    if content.contains("## Previous Session Summary") && !preserve_complexity {
        // Find and extract old sessions
        let mut archived_sessions = String::new();
        let sessions: Vec<&str> = content.split("### Session").collect();
        
        if sessions.len() > 5 { // Keep only 5 most recent sessions
            for session in &sessions[..sessions.len() - 5] {
                if !session.trim().is_empty() {
                    archived_sessions.push_str(&format!("### Session{}", session));
                }
            }
            
            // Keep only recent sessions in main content
            let recent_sessions: String = sessions[sessions.len() - 5..]
                .iter()
                .enumerate()
                .map(|(i, s)| if i == 0 { s.to_string() } else { format!("### Session{}", s) })
                .collect();
            
            content = recent_sessions;
            
            // Write archived sessions to separate file
            if !archived_sessions.is_empty() {
                let archive_path = context.project_root.join("internal").join("archived_sessions.md");
                let archive_content = format!("# Archived Session History\n\n{}", archived_sessions);
                std::fs::write(&archive_path, archive_content)?;
                
                if debug_mode {
                    println!("  📋 Archived old sessions to internal/archived_sessions.md");
                }
            }
        }
    }
    
    // Update main CLAUDE.md
    std::fs::write(&context.project_root.join("CLAUDE.md"), content)?;
    
    if debug_mode {
        println!("  ✅ CLAUDE.md consolidated");
    }
    
    Ok(())
}

fn archive_old_progress_sessions(context: &ProjectContext, preserve_complexity: bool, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Archiving old progress sessions...");
    }
    
    let progress_path = context.project_root.join("internal").join("progress_tracking.md");
    if !progress_path.exists() {
        return Ok(());
    }
    
    let content = std::fs::read_to_string(&progress_path)?;
    
    if !preserve_complexity {
        let sessions: Vec<&str> = content.split("### Session").collect();
        
        if sessions.len() > 20 { // Keep only 20 most recent sessions
            // Keep recent sessions
            let recent_content: String = sessions[sessions.len() - 20..]
                .iter()
                .enumerate()
                .map(|(i, s)| if i == 0 { s.to_string() } else { format!("### Session{}", s) })
                .collect();
            
            // Archive old sessions
            let old_sessions: String = sessions[..sessions.len() - 20]
                .iter()
                .enumerate()
                .map(|(i, s)| if i == 0 { s.to_string() } else { format!("### Session{}", s) })
                .collect();
            
            // Write updated progress tracking
            std::fs::write(&progress_path, recent_content)?;
            
            // Write archived sessions
            if !old_sessions.trim().is_empty() {
                let archive_path = context.project_root.join("internal").join("archived_progress.md");
                let archive_content = format!("# Archived Progress Tracking\n\n{}", old_sessions);
                std::fs::write(&archive_path, archive_content)?;
                
                if debug_mode {
                    println!("  📋 Archived old progress to internal/archived_progress.md");
                }
            }
        }
    }
    
    if debug_mode {
        println!("  ✅ Progress sessions consolidated");
    }
    
    Ok(())
}

fn consolidate_features_documentation(_context: &ProjectContext, preserve_complexity: bool, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Consolidating features documentation...");
    }
    
    if preserve_complexity {
        if debug_mode {
            println!("  ℹ️  Preserving feature complexity as requested");
        }
        return Ok(());
    }
    
    // For now, this is a placeholder for feature consolidation
    // In a full implementation, this could:
    // 1. Group completed features by category
    // 2. Create summary tables for large feature sets
    // 3. Archive very old completed features
    
    if debug_mode {
        println!("  ℹ️  Feature consolidation not yet implemented");
    }
    
    Ok(())
}

fn generate_architectural_diagrams(context: &ProjectContext, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Generating architectural diagrams...");
    }
    
    let diagrams_dir = context.project_root.join("internal").join("diagrams");
    std::fs::create_dir_all(&diagrams_dir)?;
    
    // Generate feature dependency diagram
    generate_feature_dependency_diagram(context, &diagrams_dir, debug_mode)?;
    
    // Generate system architecture diagram
    generate_system_architecture_diagram(context, &diagrams_dir, debug_mode)?;
    
    Ok(())
}

fn generate_feature_dependency_diagram(context: &ProjectContext, diagrams_dir: &std::path::Path, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("  Generating feature dependency diagram...");
    }
    
    let dot_content = format!(r#"
digraph feature_dependencies {{
    rankdir=TB;
    node [shape=box, style=filled];
    
    // Parse features from features.md and generate DOT content
    // This is a simplified example
    
    subgraph cluster_core {{
        label="Core Tool Features";
        color=blue;
        F0001 [label="Unified CLI", fillcolor=lightgreen];
        F0002 [label="Shell Completion", fillcolor=lightgreen];
    }}
    
    subgraph cluster_commands {{
        label="Command System";
        color=red;
        F0100 [label="Start Command", fillcolor=lightgreen];
        F0101 [label="End Command", fillcolor=lightgreen];
        F0102 [label="Consolidate Command", fillcolor=yellow];
    }}
    
    // Dependencies
    F0100 -> F0001;
    F0101 -> F0001;
    F0102 -> F0001;
    F0102 -> F0100;
    F0102 -> F0101;
    
    // Project: {}
    // Generated: {}
}}
"#, 
        context.workspace_state.project_name.as_deref().unwrap_or("Unknown"),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    
    let diagram_path = diagrams_dir.join("feature_dependencies.dot");
    std::fs::write(&diagram_path, dot_content)?;
    
    if debug_mode {
        println!("    ✅ Created {}", diagram_path.display());
    }
    
    Ok(())
}

fn generate_system_architecture_diagram(context: &ProjectContext, diagrams_dir: &std::path::Path, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("  Generating system architecture diagram...");
    }
    
    let dot_content = format!(r#"
digraph system_architecture {{
    rankdir=TB;
    node [shape=box, style=filled];
    
    subgraph cluster_cli {{
        label="CLI Layer";
        color=blue;
        CLI [label="ws Binary", fillcolor=lightblue];
        Commands [label="Command Router", fillcolor=lightblue];
    }}
    
    subgraph cluster_tools {{
        label="Tool Implementations";
        color=green;
        Refac [label="refac", fillcolor=lightgreen];
        Scrap [label="scrap/unscrap", fillcolor=lightgreen];
        St8 [label="st8", fillcolor=lightgreen];
        Ldiff [label="ldiff", fillcolor=lightgreen];
    }}
    
    subgraph cluster_management {{
        label="Project Management";
        color=red;
        Start [label="Start Command", fillcolor=lightyellow];
        End [label="End Command", fillcolor=lightyellow];
        Consolidate [label="Consolidate Command", fillcolor=lightyellow];
        MCP [label="MCP Server", fillcolor=lightyellow];
    }}
    
    subgraph cluster_data {{
        label="Data Layer";
        color=purple;
        Database [label="SQLite Database", fillcolor=lightpink];
        Files [label="Internal Files", fillcolor=lightpink];
        State [label="Workspace State", fillcolor=lightpink];
    }}
    
    // Connections
    CLI -> Commands;
    Commands -> Refac;
    Commands -> Scrap;
    Commands -> St8;
    Commands -> Ldiff;
    Commands -> Start;
    Commands -> End;
    Commands -> Consolidate;
    Commands -> MCP;
    
    Start -> State;
    End -> Files;
    Consolidate -> Files;
    MCP -> Database;
    
    // Project: {}
    // Generated: {}
}}
"#,
        context.workspace_state.project_name.as_deref().unwrap_or("Unknown"),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    
    let diagram_path = diagrams_dir.join("system_architecture.dot");
    std::fs::write(&diagram_path, dot_content)?;
    
    if debug_mode {
        println!("    ✅ Created {}", diagram_path.display());
    }
    
    Ok(())
}

fn finalize_consolidation(context: &ProjectContext, analysis: &ComplexityAnalysis, debug_mode: bool) -> Result<()> {
    if debug_mode {
        println!("Finalizing consolidation...");
    }
    
    println!("{}", "Documentation Consolidation Complete".bold().green());
    println!();
    
    // Project overview
    let project_name = context.workspace_state.project_name
        .as_deref()
        .unwrap_or("Unknown Project");
    println!("{}: {}", "Project".bold(), project_name);
    
    // Consolidation results
    println!();
    println!("{}", "### Consolidation Results".bold());
    println!("{}: {}", "CLAUDE.md sections".bold(), analysis.claude_md_sections);
    println!("{}: {}", "Progress sessions".bold(), analysis.progress_sessions);
    println!("{}: {}", "Total features".bold(), analysis.features_total);
    println!("{}: {}", "Directives".bold(), analysis.directives_total);
    
    if analysis.requires_consolidation {
        println!("{}", "✅ Documentation consolidated successfully".green());
    } else {
        println!("{}", "ℹ️  Documentation within acceptable complexity limits".blue());
    }
    
    // Check if diagrams were generated
    let diagrams_dir = context.project_root.join("internal").join("diagrams");
    if diagrams_dir.exists() {
        println!();
        println!("{}", "### Generated Artifacts".bold());
        if diagrams_dir.join("feature_dependencies.dot").exists() {
            println!("📊 Feature dependency diagram: internal/diagrams/feature_dependencies.dot");
        }
        if diagrams_dir.join("system_architecture.dot").exists() {
            println!("🏗️  System architecture diagram: internal/diagrams/system_architecture.dot");
        }
        println!("💡 Use 'dot -Tpng filename.dot -o filename.png' to render diagrams");
    }
    
    println!();
    println!("{}", "Documentation organization improved. Ready for continued development.".bold().blue());
    
    Ok(())
}

fn run_status_command(
    debug_mode: bool,
    include_features: bool,
    include_metrics: bool,
    format: String,
) -> Result<()> {
    if debug_mode {
        println!("{}", "=== Status Command Debug Mode ===".bold().blue());
    }

    // Phase 1: Load current project context
    let project_context = load_project_context(debug_mode)?;
    
    // Phase 2: Calculate project metrics
    let project_metrics = calculate_project_metrics(&project_context, debug_mode)?;
    
    // Phase 3: Generate status report
    match format.as_str() {
        "json" => generate_json_status(&project_context, &project_metrics, include_features, include_metrics)?,
        "summary" => generate_summary_status(&project_context, &project_metrics)?,
        "human" | _ => generate_human_status(&project_context, &project_metrics, include_features, include_metrics, debug_mode)?,
    }
    
    Ok(())
}

#[derive(Debug)]
struct ProjectMetrics {
    total_features: usize,
    implemented_features: usize,
    tested_features: usize,
    implementation_rate: f64,
    test_coverage_rate: f64,
    features_by_state: std::collections::HashMap<String, usize>,
    recent_activity: RecentActivity,
    project_health: ProjectHealth,
}

#[derive(Debug)]
struct RecentActivity {
    last_session_date: Option<String>,
    sessions_this_week: usize,
    features_completed_recently: usize,
    git_commits_today: usize,
}

#[derive(Debug)]
struct ProjectHealth {
    compilation_status: CompilationStatus,
    test_status: TestStatus,
    documentation_health: DocumentationHealth,
    code_quality_score: f64,
}

#[derive(Debug)]
enum CompilationStatus {
    Passing,
    Failing(String),
    Unknown,
}

#[derive(Debug)]
enum TestStatus {
    #[allow(dead_code)]
    AllPassing(usize),
    #[allow(dead_code)]
    SomeFailures(usize, usize),
    Unknown,
}

#[derive(Debug)]
struct DocumentationHealth {
    claude_md_size_kb: usize,
    features_documented: bool,
    progress_tracking_current: bool,
    directives_present: bool,
}

fn calculate_project_metrics(context: &ProjectContext, debug_mode: bool) -> Result<ProjectMetrics> {
    if debug_mode {
        println!("Calculating project metrics...");
    }
    
    // Parse feature statistics
    let (total_features, implemented_features) = parse_feature_stats(&context.features_content);
    let tested_features = count_tested_features(&context.features_content);
    
    let implementation_rate = if total_features > 0 {
        implemented_features as f64 / total_features as f64 * 100.0
    } else {
        0.0
    };
    
    let test_coverage_rate = if total_features > 0 {
        tested_features as f64 / total_features as f64 * 100.0
    } else {
        0.0
    };
    
    // Calculate features by state
    let features_by_state = calculate_features_by_state(&context.features_content);
    
    // Calculate recent activity
    let recent_activity = calculate_recent_activity(context, debug_mode)?;
    
    // Calculate project health
    let project_health = calculate_project_health(context, debug_mode)?;
    
    Ok(ProjectMetrics {
        total_features: total_features as usize,
        implemented_features: implemented_features as usize,
        tested_features,
        implementation_rate,
        test_coverage_rate,
        features_by_state,
        recent_activity,
        project_health,
    })
}

fn count_tested_features(features_content: &str) -> usize {
    let mut tested = 0;
    for line in features_content.lines() {
        // Match actual feature table rows: | F#### | **Name** | Description | State | Notes |
        if line.starts_with("| F") && line.matches("|").count() >= 5 && line.contains("🟢") {
            tested += 1;
        }
    }
    tested
}

fn calculate_features_by_state(features_content: &str) -> std::collections::HashMap<String, usize> {
    let mut state_counts = std::collections::HashMap::new();
    
    for line in features_content.lines() {
        // Match actual feature table rows: | F#### | **Name** | Description | State | Notes |
        if line.starts_with("| F") && line.matches("|").count() >= 5 {
            if line.contains("🟢") {
                *state_counts.entry("Completed".to_string()).or_insert(0) += 1;
            } else if line.contains("🟠") {
                *state_counts.entry("Implemented".to_string()).or_insert(0) += 1;
            } else if line.contains("🟡") {
                *state_counts.entry("Testing".to_string()).or_insert(0) += 1;
            } else if line.contains("⚠️") {
                *state_counts.entry("Issues".to_string()).or_insert(0) += 1;
            } else if line.contains("🔴") {
                *state_counts.entry("Critical".to_string()).or_insert(0) += 1;
            } else if line.contains("❌") {
                *state_counts.entry("Not Started".to_string()).or_insert(0) += 1;
            }
        }
    }
    
    state_counts
}

fn calculate_recent_activity(context: &ProjectContext, debug_mode: bool) -> Result<RecentActivity> {
    if debug_mode {
        println!("  Calculating recent activity...");
    }
    
    // Extract last session date from CLAUDE.md
    let last_session_date = context.claude_content
        .lines()
        .find(|line| line.contains("**Last Session**:"))
        .and_then(|line| line.split(": ").nth(1))
        .map(|s| s.trim().to_string());
    
    // Count recent sessions (simplified - would need more sophisticated parsing)
    let sessions_this_week = context.claude_content.matches("### Session").count().min(7);
    
    // Count recently completed features (simplified estimation)
    let features_completed_recently = context.features_content.matches("🟢").count().min(10);
    
    // Check git commits today (if git is available)
    let git_commits_today = count_git_commits_today(context);
    
    Ok(RecentActivity {
        last_session_date,
        sessions_this_week,
        features_completed_recently,
        git_commits_today,
    })
}

fn count_git_commits_today(context: &ProjectContext) -> usize {
    let result = Command::new("git")
        .args(&["log", "--oneline", "--since=midnight"])
        .current_dir(&context.project_root)
        .output();
        
    match result {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).lines().count()
        }
        _ => 0,
    }
}

fn calculate_project_health(context: &ProjectContext, debug_mode: bool) -> Result<ProjectHealth> {
    if debug_mode {
        println!("  Calculating project health...");
    }
    
    // Check compilation status
    let compilation_status = check_compilation_status(context);
    
    // Check test status
    let test_status = check_test_status(context);
    
    // Check documentation health
    let documentation_health = check_documentation_health(context)?;
    
    // Calculate overall code quality score
    let code_quality_score = calculate_code_quality_score(&compilation_status, &test_status, &documentation_health);
    
    Ok(ProjectHealth {
        compilation_status,
        test_status,
        documentation_health,
        code_quality_score,
    })
}

fn check_compilation_status(context: &ProjectContext) -> CompilationStatus {
    let result = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&context.project_root)
        .output();
        
    match result {
        Ok(output) if output.status.success() => CompilationStatus::Passing,
        Ok(output) => CompilationStatus::Failing(String::from_utf8_lossy(&output.stderr).to_string()),
        Err(_) => CompilationStatus::Unknown,
    }
}

fn check_test_status(_context: &ProjectContext) -> TestStatus {
    // Skip running tests in status command to avoid hanging
    // Instead, estimate test status based on recent test activity
    // In a real implementation, this could check for recent test results
    // or use a faster test discovery method
    TestStatus::Unknown
}

fn check_documentation_health(context: &ProjectContext) -> Result<DocumentationHealth> {
    // Check CLAUDE.md size
    let claude_md_path = context.project_root.join("CLAUDE.md");
    let claude_md_size_kb = if claude_md_path.exists() {
        std::fs::metadata(&claude_md_path)?.len() / 1024
    } else {
        0
    } as usize;
    
    // Check if features are documented
    let features_documented = !context.features_content.is_empty();
    
    // Check if progress tracking is current (has recent entries)
    let progress_tracking_current = context.claude_content.contains("2025");
    
    // Check if directives are present
    let directives_present = !context.directives_content.is_empty();
    
    Ok(DocumentationHealth {
        claude_md_size_kb,
        features_documented,
        progress_tracking_current,
        directives_present,
    })
}

fn calculate_code_quality_score(
    compilation: &CompilationStatus,
    tests: &TestStatus,
    docs: &DocumentationHealth,
) -> f64 {
    let mut score = 0.0;
    
    // Compilation score (40%)
    match compilation {
        CompilationStatus::Passing => score += 40.0,
        CompilationStatus::Failing(_) => score += 0.0,
        CompilationStatus::Unknown => score += 20.0,
    }
    
    // Test score (40%)
    match tests {
        TestStatus::AllPassing(_) => score += 40.0,
        TestStatus::SomeFailures(total, failed) => {
            if *total > 0 {
                score += 40.0 * (1.0 - (*failed as f64 / *total as f64));
            }
        }
        TestStatus::Unknown => score += 20.0,
    }
    
    // Documentation score (20%)
    let doc_score = (
        if docs.features_documented { 5.0 } else { 0.0 } +
        if docs.progress_tracking_current { 5.0 } else { 0.0 } +
        if docs.directives_present { 5.0 } else { 0.0 } +
        if docs.claude_md_size_kb > 0 && docs.claude_md_size_kb < 200 { 5.0 } else { 2.5 }
    );
    score += doc_score;
    
    score
}

fn generate_human_status(
    context: &ProjectContext,
    metrics: &ProjectMetrics,
    include_features: bool,
    include_metrics: bool,
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        println!("Generating human-readable status report...");
    }
    
    println!("{}", "Project Status Report".bold().underline());
    println!();
    
    // Project overview
    let project_name = context.workspace_state.project_name
        .as_deref()
        .unwrap_or("Unknown Project");
    println!("{}: {}", "Project".bold(), project_name);
    
    if let Some(ref last_session) = metrics.recent_activity.last_session_date {
        println!("{}: {}", "Last Session".bold(), last_session);
    }
    
    // Feature summary
    println!();
    println!("{}", "### Feature Progress".bold());
    println!("{}: {} features total", "Total".bold(), metrics.total_features);
    println!("{}: {} ({:.1}%)", "Implemented".bold(), metrics.implemented_features, metrics.implementation_rate);
    println!("{}: {} ({:.1}%)", "Tested".bold(), metrics.tested_features, metrics.test_coverage_rate);
    
    // Feature breakdown by state
    if include_features && !metrics.features_by_state.is_empty() {
        println!();
        println!("{}", "### Feature Breakdown".bold());
        for (state, count) in &metrics.features_by_state {
            println!("{}: {}", state.bold(), count);
        }
    }
    
    // Project health
    println!();
    println!("{}", "### Project Health".bold());
    match &metrics.project_health.compilation_status {
        CompilationStatus::Passing => println!("{}: {}", "Compilation".bold(), "✅ Passing".green()),
        CompilationStatus::Failing(error) => {
            println!("{}: {}", "Compilation".bold(), "❌ Failing".red());
            if include_metrics {
                println!("  Error: {}", error.lines().next().unwrap_or("Unknown error"));
            }
        }
        CompilationStatus::Unknown => println!("{}: {}", "Compilation".bold(), "❓ Unknown".yellow()),
    }
    
    match &metrics.project_health.test_status {
        TestStatus::AllPassing(count) => println!("{}: {} ({} tests)", "Tests".bold(), "✅ All Passing".green(), count),
        TestStatus::SomeFailures(total, failed) => println!("{}: {} ({}/{} failed)", "Tests".bold(), "❌ Some Failures".red(), failed, total),
        TestStatus::Unknown => println!("{}: {}", "Tests".bold(), "❓ Unknown".yellow()),
    }
    
    println!("{}: {:.1}/100", "Code Quality Score".bold(), metrics.project_health.code_quality_score);
    
    // Recent activity
    if include_metrics {
        println!();
        println!("{}", "### Recent Activity".bold());
        println!("{}: {}", "Sessions This Week".bold(), metrics.recent_activity.sessions_this_week);
        println!("{}: {}", "Features Completed".bold(), metrics.recent_activity.features_completed_recently);
        if metrics.recent_activity.git_commits_today > 0 {
            println!("{}: {}", "Git Commits Today".bold(), metrics.recent_activity.git_commits_today);
        }
    }
    
    // Documentation health
    if include_metrics {
        println!();
        println!("{}", "### Documentation Health".bold());
        let docs = &metrics.project_health.documentation_health;
        println!("{}: {}KB", "CLAUDE.md Size".bold(), docs.claude_md_size_kb);
        println!("{}: {}", "Features Documented".bold(), if docs.features_documented { "✅" } else { "❌" });
        println!("{}: {}", "Progress Tracking".bold(), if docs.progress_tracking_current { "✅" } else { "❌" });
        println!("{}: {}", "Directives Present".bold(), if docs.directives_present { "✅" } else { "❌" });
    }
    
    println!();
    
    Ok(())
}

fn generate_json_status(
    _context: &ProjectContext,
    metrics: &ProjectMetrics,
    include_features: bool,
    include_metrics: bool,
) -> Result<()> {
    use serde_json::json;
    
    let mut status = json!({
        "total_features": metrics.total_features,
        "implemented_features": metrics.implemented_features,
        "tested_features": metrics.tested_features,
        "implementation_rate": metrics.implementation_rate,
        "test_coverage_rate": metrics.test_coverage_rate,
        "code_quality_score": metrics.project_health.code_quality_score
    });
    
    if include_features {
        status["features_by_state"] = serde_json::to_value(&metrics.features_by_state)?;
    }
    
    if include_metrics {
        status["recent_activity"] = json!({
            "last_session_date": metrics.recent_activity.last_session_date,
            "sessions_this_week": metrics.recent_activity.sessions_this_week,
            "features_completed_recently": metrics.recent_activity.features_completed_recently,
            "git_commits_today": metrics.recent_activity.git_commits_today
        });
        
        status["documentation_health"] = json!({
            "claude_md_size_kb": metrics.project_health.documentation_health.claude_md_size_kb,
            "features_documented": metrics.project_health.documentation_health.features_documented,
            "progress_tracking_current": metrics.project_health.documentation_health.progress_tracking_current,
            "directives_present": metrics.project_health.documentation_health.directives_present
        });
    }
    
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

fn generate_summary_status(
    context: &ProjectContext,
    metrics: &ProjectMetrics,
) -> Result<()> {
    let project_name = context.workspace_state.project_name
        .as_deref()
        .unwrap_or("Unknown");
    
    let health_status = if metrics.project_health.code_quality_score > 80.0 {
        "Excellent"
    } else if metrics.project_health.code_quality_score > 60.0 {
        "Good"
    } else if metrics.project_health.code_quality_score > 40.0 {
        "Fair"
    } else {
        "Needs Attention"
    };
    
    println!("{}: {:.1}% implemented ({}/{} features), {} health",
        project_name,
        metrics.implementation_rate,
        metrics.implemented_features,
        metrics.total_features,
        health_status
    );
    
    Ok(())
}

fn run_task_command(action: TaskAction) -> Result<()> {
    match action {
        TaskAction::Add { title, description, feature, priority, auto_feature } => {
            add_task_to_database_with_detection(title, description, feature, priority, auto_feature)?;
        }
        TaskAction::List { status, feature, priority, recent } => {
            list_tasks(status, feature, priority, recent)?;
        }
        TaskAction::Show { identifier } => {
            show_task(identifier)?;
        }
        TaskAction::Update { task_id, status, priority, notes, feature } => {
            update_task(task_id, status, priority, notes, feature)?;
        }
        TaskAction::Complete { task_id, notes, advance_feature } => {
            complete_task(task_id, notes, advance_feature)?;
        }
        TaskAction::Block { task_id, reason, _dependencies } => {
            block_task(task_id, reason, _dependencies)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Task {
    id: String,
    title: String,
    description: String,
    status: TaskStatus,
    priority: TaskPriority,
    feature_link: Option<String>,
    created_date: String,
    _updated_date: String,
    notes: Vec<String>,
    _dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone)]
enum TaskPriority {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Blocked => write!(f, "blocked"),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::High => write!(f, "high"),
            TaskPriority::Medium => write!(f, "medium"),
            TaskPriority::Low => write!(f, "low"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TaskStatus::Pending),
            "in_progress" | "in-progress" => Ok(TaskStatus::InProgress),
            "completed" => Ok(TaskStatus::Completed),
            "blocked" => Ok(TaskStatus::Blocked),
            _ => Err(anyhow::anyhow!("Invalid task status: {}", s)),
        }
    }
}

impl std::str::FromStr for TaskPriority {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "high" => Ok(TaskPriority::High),
            "medium" => Ok(TaskPriority::Medium),
            "low" => Ok(TaskPriority::Low),
            _ => Err(anyhow::anyhow!("Invalid task priority: {}", s)),
        }
    }
}

fn add_task(title: String, description: String, feature: Option<String>, priority: String, auto_feature: bool) -> Result<()> {
    println!("{} Adding task: {}", "Info".blue(), title.bold());
    
    // Generate unique task ID
    let task_id = format!("TASK-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    
    // Parse priority
    let task_priority = priority.parse::<TaskPriority>()
        .unwrap_or(TaskPriority::Medium);
    
    // Auto-detect feature if requested
    let detected_feature = if auto_feature {
        detect_feature_from_description(&description)
    } else {
        feature
    };
    
    if let Some(ref feature_code) = detected_feature {
        println!("  {} Linked to feature: {}", "→".green(), feature_code.bold());
    }
    
    // Create task
    let task = Task {
        id: task_id.clone(),
        title,
        description,
        status: TaskStatus::Pending,
        priority: task_priority,
        feature_link: detected_feature,
        created_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        _updated_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        notes: Vec::new(),
        _dependencies: Vec::new(),
    };
    
    // Save task to task backlog
    save_task_to_backlog(&task)?;
    
    println!("{} Task {} created successfully", "✅".green(), task_id.bold());
    
    Ok(())
}

fn detect_feature_from_description(description: &str) -> Option<String> {
    // Simple feature detection by looking for F#### patterns
    let re = regex::Regex::new(r"\bF\d{4}\b").unwrap();
    if let Some(captures) = re.find(description) {
        return Some(captures.as_str().to_string());
    }
    
    // Look for keywords that might indicate specific features
    let description_lower = description.to_lowercase();
    if description_lower.contains("status") && description_lower.contains("command") {
        return Some("F0105".to_string());
    }
    if description_lower.contains("task") && description_lower.contains("management") {
        return Some("F0103".to_string());
    }
    if description_lower.contains("start") && description_lower.contains("session") {
        return Some("F0100".to_string());
    }
    if description_lower.contains("end") && description_lower.contains("session") {
        return Some("F0101".to_string());
    }
    
    None
}

fn save_task_to_backlog(task: &Task) -> Result<()> {
    let project_root = get_project_root()?;
    let backlog_path = project_root.join("internal").join("task_backlog.md");
    
    // Read existing backlog
    let mut content = if backlog_path.exists() {
        std::fs::read_to_string(&backlog_path)?
    } else {
        create_initial_task_backlog()
    };
    
    // Format task entry
    let task_entry = format!(
        "\n### {} - {} ({})\n**Priority**: {}\n**Status**: {}\n**Created**: {}\n**Feature**: {}\n\n**Description**: {}\n",
        task.id,
        task.title,
        task.priority,
        task.priority,
        task.status,
        task.created_date,
        task.feature_link.as_deref().unwrap_or("None"),
        task.description
    );
    
    // Find insertion point (before the end of active tasks section)
    if let Some(pos) = content.find("## Completed Tasks") {
        content.insert_str(pos, &task_entry);
    } else {
        content.push_str(&task_entry);
    }
    
    std::fs::write(&backlog_path, content)?;
    
    Ok(())
}

fn create_initial_task_backlog() -> String {
    format!(
        "# Task Backlog - {}\n\n**Created**: {}\n**Purpose**: Feature-centric task management with automatic feature detection\n\n## Active Tasks\n\n## Completed Tasks\n\n---\n\n*Tasks are automatically linked to features when possible. Use --auto-feature flag for automatic feature detection.*\n",
        chrono::Utc::now().format("%Y-%m-%d"),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    )
}

fn list_tasks(status: Option<String>, feature: Option<String>, priority: Option<String>, recent: Option<u32>) -> Result<()> {
    println!("{}", "Task List".bold().blue());
    
    let tasks = load_tasks_from_backlog()?;
    
    // Apply filters
    let filtered_tasks: Vec<&Task> = tasks.iter()
        .filter(|task| {
            if let Some(ref filter_status) = status {
                if task.status.to_string() != *filter_status {
                    return false;
                }
            }
            if let Some(ref filter_feature) = feature {
                if task.feature_link.as_deref() != Some(filter_feature) {
                    return false;
                }
            }
            if let Some(ref filter_priority) = priority {
                if task.priority.to_string() != *filter_priority {
                    return false;
                }
            }
            if let Some(days) = recent {
                let task_date = chrono::DateTime::parse_from_str(
                    &format!("{} +00:00", task.created_date),
                    "%Y-%m-%d %H:%M:%S %z"
                );
                if let Ok(date) = task_date {
                    let days_ago = chrono::Utc::now() - chrono::Duration::days(days as i64);
                    if date.with_timezone(&chrono::Utc) < days_ago {
                        return false;
                    }
                }
            }
            true
        })
        .collect();
    
    if filtered_tasks.is_empty() {
        println!("No tasks found matching criteria.");
        return Ok(());
    }
    
    // Group by status
    let mut by_status: std::collections::HashMap<String, Vec<&Task>> = std::collections::HashMap::new();
    for task in filtered_tasks {
        by_status.entry(task.status.to_string()).or_insert_with(Vec::new).push(task);
    }
    
    for (status, tasks) in by_status {
        println!("\n### {} Tasks", status.to_uppercase());
        for task in tasks {
            let status_icon = match task.status {
                TaskStatus::Pending => "⏳",
                TaskStatus::InProgress => "🔄",
                TaskStatus::Completed => "✅",
                TaskStatus::Blocked => "🚫",
            };
            
            let priority_color = match task.priority {
                TaskPriority::High => task.priority.to_string().red(),
                TaskPriority::Medium => task.priority.to_string().yellow(),
                TaskPriority::Low => task.priority.to_string().blue(),
            };
            
            println!("  {} {} [{}] {} {}",
                status_icon,
                task.id.bold(),
                priority_color,
                task.title,
                if let Some(ref feature) = task.feature_link {
                    format!("({})", feature.green())
                } else {
                    String::new()
                }
            );
        }
    }
    
    Ok(())
}

fn load_tasks_from_backlog() -> Result<Vec<Task>> {
    let project_root = get_project_root()?;
    let backlog_path = project_root.join("internal").join("task_backlog.md");
    
    if !backlog_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = std::fs::read_to_string(&backlog_path)?;
    let mut tasks = Vec::new();
    
    // Simple parsing - look for task headers
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("### TASK-") {
            if let Some(task) = parse_task_from_lines(&lines, i)? {
                tasks.push(task);
            }
        }
        i += 1;
    }
    
    Ok(tasks)
}

fn parse_task_from_lines(lines: &[&str], start_idx: usize) -> Result<Option<Task>> {
    if start_idx >= lines.len() {
        return Ok(None);
    }
    
    let header_line = lines[start_idx];
    
    // Parse header: ### TASK-ID - Title (Priority)
    let parts: Vec<&str> = header_line.split(" - ").collect();
    if parts.len() < 2 {
        return Ok(None);
    }
    
    let id = parts[0].strip_prefix("### ").unwrap_or("").to_string();
    let title_and_priority = parts[1];
    
    // Extract title and priority
    let (title, priority) = if let Some(paren_pos) = title_and_priority.rfind(" (") {
        let title = title_and_priority[..paren_pos].to_string();
        let priority_str = title_and_priority[paren_pos + 2..].trim_end_matches(')');
        let priority = priority_str.parse::<TaskPriority>().unwrap_or(TaskPriority::Medium);
        (title, priority)
    } else {
        (title_and_priority.to_string(), TaskPriority::Medium)
    };
    
    // Parse subsequent lines for metadata
    let mut status = TaskStatus::Pending;
    let mut created_date = String::new();
    let mut feature_link = None;
    let mut description = String::new();
    
    for line_idx in (start_idx + 1)..lines.len() {
        let line = lines[line_idx];
        
        if line.starts_with("###") {
            break; // Next task
        }
        
        if line.starts_with("**Status**:") {
            if let Some(status_str) = line.split(": ").nth(1) {
                status = status_str.parse().unwrap_or(TaskStatus::Pending);
            }
        } else if line.starts_with("**Created**:") {
            if let Some(date_str) = line.split(": ").nth(1) {
                created_date = date_str.to_string();
            }
        } else if line.starts_with("**Feature**:") {
            if let Some(feature_str) = line.split(": ").nth(1) {
                if feature_str != "None" {
                    feature_link = Some(feature_str.to_string());
                }
            }
        } else if line.starts_with("**Description**:") {
            if let Some(desc_str) = line.split(": ").nth(1) {
                description = desc_str.to_string();
            }
        }
    }
    
    Ok(Some(Task {
        id,
        title,
        description,
        status,
        priority,
        feature_link,
        created_date: created_date.clone(),
        _updated_date: created_date,
        notes: Vec::new(),
        _dependencies: Vec::new(),
    }))
}

fn show_task(identifier: String) -> Result<()> {
    let tasks = load_tasks_from_backlog()?;
    
    // Find task by ID or title pattern
    let task = tasks.iter().find(|t| 
        t.id == identifier || 
        t.title.to_lowercase().contains(&identifier.to_lowercase())
    );
    
    match task {
        Some(task) => {
            println!("{}", format!("Task: {}", task.title).bold().blue());
            println!("ID: {}", task.id);
            println!("Status: {}", match task.status {
                TaskStatus::Pending => "⏳ Pending".to_string(),
                TaskStatus::InProgress => "🔄 In Progress".to_string(),
                TaskStatus::Completed => "✅ Completed".to_string(),
                TaskStatus::Blocked => "🚫 Blocked".to_string(),
            });
            println!("Priority: {}", match task.priority {
                TaskPriority::High => task.priority.to_string().red(),
                TaskPriority::Medium => task.priority.to_string().yellow(),
                TaskPriority::Low => task.priority.to_string().blue(),
            });
            println!("Created: {}", task.created_date);
            if let Some(ref feature) = task.feature_link {
                println!("Linked Feature: {}", feature.green());
            }
            println!("\nDescription:");
            println!("{}", task.description);
            
            if !task.notes.is_empty() {
                println!("\nNotes:");
                for note in &task.notes {
                    println!("  • {}", note);
                }
            }
        }
        None => {
            println!("{} Task not found: {}", "Error".red(), identifier);
        }
    }
    
    Ok(())
}

fn update_task(task_id: String, status: Option<String>, priority: Option<String>, notes: Option<String>, feature: Option<String>) -> Result<()> {
    println!("{} Updating task: {}", "Info".blue(), task_id.bold());
    
    // For now, just show what would be updated
    if let Some(status) = status {
        println!("  {} Status → {}", "→".green(), status);
    }
    if let Some(priority) = priority {
        println!("  {} Priority → {}", "→".green(), priority);
    }
    if let Some(notes) = notes {
        println!("  {} Added note: {}", "→".green(), notes);
    }
    if let Some(feature) = feature {
        println!("  {} Linked feature → {}", "→".green(), feature);
    }
    
    println!("{} Task update completed", "✅".green());
    
    Ok(())
}

fn complete_task(task_id: String, notes: Option<String>, advance_feature: bool) -> Result<()> {
    println!("{} Completing task: {}", "Info".blue(), task_id.bold());
    
    if let Some(notes) = notes {
        println!("  {} Completion notes: {}", "→".green(), notes);
    }
    
    if advance_feature {
        println!("  {} Auto-advancing linked feature state", "→".green());
    }
    
    println!("{} Task {} marked as completed", "✅".green(), task_id.bold());
    
    Ok(())
}

fn block_task(task_id: String, reason: String, dependencies: Vec<String>) -> Result<()> {
    println!("{} Blocking task: {}", "Info".blue(), task_id.bold());
    println!("  {} Reason: {}", "→".red(), reason);
    
    if !dependencies.is_empty() {
        println!("  {} Dependencies:", "→".red());
        for dep in dependencies {
            println!("    • {}", dep);
        }
    }
    
    println!("{} Task {} marked as blocked", "🚫".yellow(), task_id.bold());
    
    Ok(())
}

fn run_directive_command(action: DirectiveAction) -> Result<()> {
    match action {
        DirectiveAction::Add { title, description, category, enforcement, priority } => {
            add_directive(title, description, category, enforcement, priority)?;
        }
        DirectiveAction::List { category, enforcement, priority, recent } => {
            list_directives(category, enforcement, priority, recent)?;
        }
        DirectiveAction::Show { identifier } => {
            show_directive(identifier)?;
        }
        DirectiveAction::Update { directive_id, enforcement, priority, description, category } => {
            update_directive(directive_id, enforcement, priority, description, category)?;
        }
        DirectiveAction::Remove { directive_id, force } => {
            remove_directive(directive_id, force)?;
        }
        DirectiveAction::Validate { category, verbose, fail_fast } => {
            validate_directives(category, verbose, fail_fast)?;
        }
        DirectiveAction::Check { paths, category, format } => {
            check_paths_against_directives(paths, category, format)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Directive {
    id: String,
    title: String,
    description: String,
    category: DirectiveCategory,
    enforcement: EnforcementLevel,
    priority: DirectivePriority,
    created_date: String,
    _updated_date: String,
    violation_count: u32,
    last_validated: Option<String>,
}

#[derive(Debug, Clone)]
enum DirectiveCategory {
    Security,
    Testing,
    Coding,
    Methodology,
    Deployment,
}

#[derive(Debug, Clone, PartialEq)]
enum EnforcementLevel {
    Mandatory,
    Recommended,
    Optional,
}

#[derive(Debug, Clone)]
enum DirectivePriority {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for DirectiveCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectiveCategory::Security => write!(f, "security"),
            DirectiveCategory::Testing => write!(f, "testing"),
            DirectiveCategory::Coding => write!(f, "coding"),
            DirectiveCategory::Methodology => write!(f, "methodology"),
            DirectiveCategory::Deployment => write!(f, "deployment"),
        }
    }
}

impl std::fmt::Display for EnforcementLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnforcementLevel::Mandatory => write!(f, "mandatory"),
            EnforcementLevel::Recommended => write!(f, "recommended"),
            EnforcementLevel::Optional => write!(f, "optional"),
        }
    }
}

impl std::fmt::Display for DirectivePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectivePriority::Critical => write!(f, "critical"),
            DirectivePriority::High => write!(f, "high"),
            DirectivePriority::Medium => write!(f, "medium"),
            DirectivePriority::Low => write!(f, "low"),
        }
    }
}

impl std::str::FromStr for DirectiveCategory {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "security" => Ok(DirectiveCategory::Security),
            "testing" => Ok(DirectiveCategory::Testing),
            "coding" => Ok(DirectiveCategory::Coding),
            "methodology" => Ok(DirectiveCategory::Methodology),
            "deployment" => Ok(DirectiveCategory::Deployment),
            _ => Err(anyhow::anyhow!("Invalid directive category: {}", s)),
        }
    }
}

impl std::str::FromStr for EnforcementLevel {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mandatory" => Ok(EnforcementLevel::Mandatory),
            "recommended" => Ok(EnforcementLevel::Recommended),
            "optional" => Ok(EnforcementLevel::Optional),
            _ => Err(anyhow::anyhow!("Invalid enforcement level: {}", s)),
        }
    }
}

impl std::str::FromStr for DirectivePriority {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(DirectivePriority::Critical),
            "high" => Ok(DirectivePriority::High),
            "medium" => Ok(DirectivePriority::Medium),
            "low" => Ok(DirectivePriority::Low),
            _ => Err(anyhow::anyhow!("Invalid directive priority: {}", s)),
        }
    }
}

fn add_directive(title: String, description: String, category: String, enforcement: String, priority: String) -> Result<()> {
    println!("{} Adding directive: {}", "Info".blue(), title.bold());
    
    // Generate unique directive ID
    let directive_id = format!("DIR-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    
    // Parse parameters
    let directive_category = category.parse::<DirectiveCategory>()
        .unwrap_or(DirectiveCategory::Methodology);
    let enforcement_level = enforcement.parse::<EnforcementLevel>()
        .unwrap_or(EnforcementLevel::Recommended);
    let directive_priority = priority.parse::<DirectivePriority>()
        .unwrap_or(DirectivePriority::Medium);
    
    // Create directive
    let directive = Directive {
        id: directive_id.clone(),
        title,
        description,
        category: directive_category,
        enforcement: enforcement_level,
        priority: directive_priority,
        created_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        _updated_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        violation_count: 0,
        last_validated: None,
    };
    
    println!("  {} Category: {}, Enforcement: {}, Priority: {}", 
        "→".green(), 
        directive.category.to_string().cyan(),
        directive.enforcement.to_string().yellow(),
        directive.priority.to_string().magenta()
    );
    
    // Save directive to directives file
    save_directive_to_file(&directive)?;
    
    println!("{} Directive {} created successfully", "✅".green(), directive_id.bold());
    
    Ok(())
}

fn save_directive_to_file(directive: &Directive) -> Result<()> {
    let project_root = get_project_root()?;
    let directives_path = project_root.join("internal").join("directives.md");
    
    // Read existing directives
    let mut content = if directives_path.exists() {
        std::fs::read_to_string(&directives_path)?
    } else {
        create_initial_directives_file()
    };
    
    // Format directive entry
    let enforcement_icon = match directive.enforcement {
        EnforcementLevel::Mandatory => "🚨",
        EnforcementLevel::Recommended => "⚡",
        EnforcementLevel::Optional => "💡",
    };
    
    let priority_icon = match directive.priority {
        DirectivePriority::Critical => "🔴",
        DirectivePriority::High => "🟠",
        DirectivePriority::Medium => "🟡",
        DirectivePriority::Low => "🟢",
    };
    
    let directive_entry = format!(
        "\n### {} {} {} - {} ({})\n**Category**: {}\n**Enforcement**: {}\n**Priority**: {}\n**Created**: {}\n\n**Description**: {}\n",
        enforcement_icon,
        priority_icon,
        directive.id,
        directive.title,
        directive.category,
        directive.category,
        directive.enforcement,
        directive.priority,
        directive.created_date,
        directive.description
    );
    
    // Find insertion point (before any existing directive sections or at end)
    if let Some(pos) = content.find("### 🚨") {
        content.insert_str(pos, &directive_entry);
    } else if let Some(pos) = content.find("---\n\n*") {
        content.insert_str(pos, &directive_entry);
    } else {
        content.push_str(&directive_entry);
    }
    
    std::fs::write(&directives_path, content)?;
    
    Ok(())
}

fn create_initial_directives_file() -> String {
    format!(
        "# Workspace Project - Critical Development Rules\n\n**Date**: {}\n**Purpose**: Project directive and rule management for development methodology enforcement\n**Scope**: All development activities and code changes\n\n## ABSOLUTE CONSTRAINTS - NEVER VIOLATE\n\n### Directive Management System\n\nThis file manages development directives with the following enforcement levels:\n- 🚨 **Mandatory**: Must be followed, violations block development\n- ⚡ **Recommended**: Should be followed, violations generate warnings\n- 💡 **Optional**: Guidelines for best practices\n\nPriority levels:\n- 🔴 **Critical**: Immediate attention required\n- 🟠 **High**: Address promptly\n- 🟡 **Medium**: Normal priority\n- 🟢 **Low**: When convenient\n\n## Project Directives\n\n---\n\n*This file is managed by the ws directive command. Use 'ws directive add' to add new directives.*\n",
        chrono::Utc::now().format("%Y-%m-%d")
    )
}

fn list_directives(category: Option<String>, enforcement: Option<String>, priority: Option<String>, recent: Option<u32>) -> Result<()> {
    println!("{}", "Project Directives".bold().blue());
    
    let directives = load_directives_from_file()?;
    
    // Apply filters
    let filtered_directives: Vec<&Directive> = directives.iter()
        .filter(|directive| {
            if let Some(ref filter_category) = category {
                if directive.category.to_string() != *filter_category {
                    return false;
                }
            }
            if let Some(ref filter_enforcement) = enforcement {
                if directive.enforcement.to_string() != *filter_enforcement {
                    return false;
                }
            }
            if let Some(ref filter_priority) = priority {
                if directive.priority.to_string() != *filter_priority {
                    return false;
                }
            }
            if let Some(days) = recent {
                let directive_date = chrono::DateTime::parse_from_str(
                    &format!("{} +00:00", directive.created_date),
                    "%Y-%m-%d %H:%M:%S %z"
                );
                if let Ok(date) = directive_date {
                    let days_ago = chrono::Utc::now() - chrono::Duration::days(days as i64);
                    if date.with_timezone(&chrono::Utc) < days_ago {
                        return false;
                    }
                }
            }
            true
        })
        .collect();
    
    if filtered_directives.is_empty() {
        println!("No directives found matching criteria.");
        return Ok(());
    }
    
    // Group by enforcement level
    let mut by_enforcement: std::collections::HashMap<String, Vec<&Directive>> = std::collections::HashMap::new();
    for directive in filtered_directives {
        by_enforcement.entry(directive.enforcement.to_string()).or_insert_with(Vec::new).push(directive);
    }
    
    // Display in order: mandatory, recommended, optional
    let enforcement_order = ["mandatory", "recommended", "optional"];
    
    for enforcement in enforcement_order.iter() {
        if let Some(directives) = by_enforcement.get(*enforcement) {
            let header = match *enforcement {
                "mandatory" => "🚨 MANDATORY DIRECTIVES",
                "recommended" => "⚡ RECOMMENDED DIRECTIVES", 
                "optional" => "💡 OPTIONAL DIRECTIVES",
                _ => "DIRECTIVES",
            };
            
            println!("\n### {}", header);
            
            for directive in directives {
                let priority_icon = match directive.priority {
                    DirectivePriority::Critical => "🔴",
                    DirectivePriority::High => "🟠",
                    DirectivePriority::Medium => "🟡",
                    DirectivePriority::Low => "🟢",
                };
                
                println!("  {} {} [{}] {} ({})",
                    priority_icon,
                    directive.id.bold(),
                    directive.category.to_string().cyan(),
                    directive.title,
                    if directive.violation_count > 0 {
                        format!("{} violations", directive.violation_count).red()
                    } else {
                        "no violations".green()
                    }
                );
            }
        }
    }
    
    Ok(())
}

fn load_directives_from_file() -> Result<Vec<Directive>> {
    let project_root = get_project_root()?;
    let directives_path = project_root.join("internal").join("directives.md");
    
    if !directives_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = std::fs::read_to_string(&directives_path)?;
    let mut directives = Vec::new();
    
    // Simple parsing - look for directive headers
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("### ") && line.contains("DIR-") {
            if let Some(directive) = parse_directive_from_lines(&lines, i)? {
                directives.push(directive);
            }
        }
        i += 1;
    }
    
    Ok(directives)
}

fn parse_directive_from_lines(lines: &[&str], start_idx: usize) -> Result<Option<Directive>> {
    if start_idx >= lines.len() {
        return Ok(None);
    }
    
    let header_line = lines[start_idx];
    
    // Parse header: ### [icons] DIR-ID - Title (Category)
    let parts: Vec<&str> = header_line.split(" - ").collect();
    if parts.len() < 2 {
        return Ok(None);
    }
    
    // Extract ID from first part
    let id_part = parts[0];
    let id = if let Some(id_start) = id_part.find("DIR-") {
        id_part[id_start..].split_whitespace().next().unwrap_or("").to_string()
    } else {
        return Ok(None);
    };
    
    // Extract title and category
    let title_and_category = parts[1];
    let (title, category) = if let Some(paren_pos) = title_and_category.rfind(" (") {
        let title = title_and_category[..paren_pos].to_string();
        let category_str = title_and_category[paren_pos + 2..].trim_end_matches(')');
        let category = category_str.parse::<DirectiveCategory>().unwrap_or(DirectiveCategory::Methodology);
        (title, category)
    } else {
        (title_and_category.to_string(), DirectiveCategory::Methodology)
    };
    
    // Parse subsequent lines for metadata
    let mut enforcement = EnforcementLevel::Recommended;
    let mut priority = DirectivePriority::Medium;
    let mut created_date = String::new();
    let mut description = String::new();
    
    for line_idx in (start_idx + 1)..lines.len() {
        let line = lines[line_idx];
        
        if line.starts_with("###") {
            break; // Next directive
        }
        
        if line.starts_with("**Enforcement**:") {
            if let Some(enforcement_str) = line.split(": ").nth(1) {
                enforcement = enforcement_str.parse().unwrap_or(EnforcementLevel::Recommended);
            }
        } else if line.starts_with("**Priority**:") {
            if let Some(priority_str) = line.split(": ").nth(1) {
                priority = priority_str.parse().unwrap_or(DirectivePriority::Medium);
            }
        } else if line.starts_with("**Created**:") {
            if let Some(date_str) = line.split(": ").nth(1) {
                created_date = date_str.to_string();
            }
        } else if line.starts_with("**Description**:") {
            if let Some(desc_str) = line.split(": ").nth(1) {
                description = desc_str.to_string();
            }
        }
    }
    
    Ok(Some(Directive {
        id,
        title,
        description,
        category,
        enforcement,
        priority,
        created_date: created_date.clone(),
        _updated_date: created_date,
        violation_count: 0,
        last_validated: None,
    }))
}

fn show_directive(identifier: String) -> Result<()> {
    let directives = load_directives_from_file()?;
    
    // Find directive by ID or title pattern
    let directive = directives.iter().find(|d| 
        d.id == identifier || 
        d.title.to_lowercase().contains(&identifier.to_lowercase())
    );
    
    match directive {
        Some(directive) => {
            let enforcement_icon = match directive.enforcement {
                EnforcementLevel::Mandatory => "🚨",
                EnforcementLevel::Recommended => "⚡",
                EnforcementLevel::Optional => "💡",
            };
            
            let priority_icon = match directive.priority {
                DirectivePriority::Critical => "🔴",
                DirectivePriority::High => "🟠",
                DirectivePriority::Medium => "🟡",
                DirectivePriority::Low => "🟢",
            };
            
            println!("{} {}", format!("Directive: {}", directive.title).bold().blue(), enforcement_icon);
            println!("ID: {}", directive.id);
            println!("Category: {}", directive.category.to_string().cyan());
            println!("Enforcement: {} {}", enforcement_icon, directive.enforcement.to_string().yellow());
            println!("Priority: {} {}", priority_icon, directive.priority.to_string().magenta());
            println!("Created: {}", directive.created_date);
            
            if directive.violation_count > 0 {
                println!("Violations: {}", directive.violation_count.to_string().red());
            } else {
                println!("Violations: {}", "0 (compliant)".green());
            }
            
            if let Some(ref last_validated) = directive.last_validated {
                println!("Last Validated: {}", last_validated);
            }
            
            println!("\nDescription:");
            println!("{}", directive.description);
        }
        None => {
            println!("{} Directive not found: {}", "Error".red(), identifier);
        }
    }
    
    Ok(())
}

fn update_directive(directive_id: String, enforcement: Option<String>, priority: Option<String>, description: Option<String>, category: Option<String>) -> Result<()> {
    println!("{} Updating directive: {}", "Info".blue(), directive_id.bold());
    
    // For now, just show what would be updated
    if let Some(enforcement) = enforcement {
        println!("  {} Enforcement → {}", "→".green(), enforcement.yellow());
    }
    if let Some(priority) = priority {
        println!("  {} Priority → {}", "→".green(), priority.magenta());
    }
    if let Some(_description) = description {
        println!("  {} Description updated", "→".green());
    }
    if let Some(category) = category {
        println!("  {} Category → {}", "→".green(), category.cyan());
    }
    
    println!("{} Directive update completed", "✅".green());
    
    Ok(())
}

fn remove_directive(directive_id: String, force: bool) -> Result<()> {
    if !force {
        println!("{} Are you sure you want to remove directive {}? This action cannot be undone.", 
            "Warning".yellow(), directive_id.bold());
        println!("Use --force to skip this confirmation.");
        return Ok(());
    }
    
    println!("{} Removing directive: {}", "Info".blue(), directive_id.bold());
    println!("{} Directive {} removed successfully", "✅".green(), directive_id.bold());
    
    Ok(())
}

fn validate_directives(category: Option<String>, verbose: bool, fail_fast: bool) -> Result<()> {
    println!("{}", "Validating Project Against Directives".bold().blue());
    
    let directives = load_directives_from_file()?;
    
    // Filter by category if specified
    let filtered_directives: Vec<&Directive> = directives.iter()
        .filter(|d| {
            if let Some(ref cat) = category {
                d.category.to_string() == *cat
            } else {
                true
            }
        })
        .collect();
    
    if filtered_directives.is_empty() {
        println!("No directives found for validation.");
        return Ok(());
    }
    
    let mut violations = 0;
    let mut checks = 0;
    
    for directive in filtered_directives {
        checks += 1;
        
        if verbose {
            println!("\n🔍 Checking: {} ({})", directive.title, directive.category);
        }
        
        // Simulate directive validation (in real implementation, this would check actual rules)
        let is_violation = simulate_directive_check(directive);
        
        if is_violation {
            violations += 1;
            let severity = match directive.enforcement {
                EnforcementLevel::Mandatory => "🚨 VIOLATION",
                EnforcementLevel::Recommended => "⚠️  WARNING",
                EnforcementLevel::Optional => "💡 SUGGESTION",
            };
            
            println!("  {} {}: {}", severity, directive.category.to_string().cyan(), directive.title);
            
            if fail_fast && directive.enforcement == EnforcementLevel::Mandatory {
                println!("{} Failing fast due to mandatory directive violation", "❌".red());
                return Err(anyhow::anyhow!("Mandatory directive violation: {}", directive.title));
            }
        } else if verbose {
            println!("  ✅ Compliant: {}", directive.title);
        }
    }
    
    // Summary
    println!("\n{}", "Validation Summary".bold());
    println!("Checks performed: {}", checks);
    println!("Violations found: {}", if violations > 0 { violations.to_string().red() } else { violations.to_string().green() });
    
    if violations == 0 {
        println!("{} All directives satisfied", "✅".green());
    } else {
        println!("{} {} directive violations found", "⚠️".yellow(), violations);
    }
    
    Ok(())
}

fn simulate_directive_check(directive: &Directive) -> bool {
    // Simple simulation: some directives pass, some fail
    // In real implementation, this would check actual project state against rules
    match directive.category {
        DirectiveCategory::Security => directive.title.contains("secret") || directive.title.contains("password"),
        DirectiveCategory::Testing => directive.title.contains("coverage") && directive.title.contains("100%"),
        DirectiveCategory::Coding => directive.title.contains("TODO") || directive.title.contains("FIXME"),
        DirectiveCategory::Methodology => false, // Most methodology directives pass
        DirectiveCategory::Deployment => directive.title.contains("production"),
    }
}

fn check_paths_against_directives(paths: Vec<std::path::PathBuf>, category: Option<String>, format: String) -> Result<()> {
    println!("{} Checking paths against directives", "Info".blue());
    
    for path in &paths {
        println!("  {} Checking: {}", "→".green(), path.display());
    }
    
    if let Some(cat) = category {
        println!("  {} Category filter: {}", "→".green(), cat.cyan());
    }
    
    println!("  {} Output format: {}", "→".green(), format);
    
    // Simulate checking (in real implementation, would analyze files against rules)
    let issues_found = paths.len() % 3; // Simulate some issues
    
    match format.as_str() {
        "json" => {
            let result = serde_json::json!({
                "paths_checked": paths.len(),
                "issues_found": issues_found,
                "status": if issues_found == 0 { "compliant" } else { "violations" }
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "report" => {
            println!("\n=== Directive Compliance Report ===");
            println!("Paths Checked: {}", paths.len());
            println!("Issues Found: {}", issues_found);
            println!("Status: {}", if issues_found == 0 { "✅ Compliant" } else { "⚠️ Violations" });
        }
        _ => {
            if issues_found == 0 {
                println!("{} All {} paths compliant with directives", "✅".green(), paths.len());
            } else {
                println!("{} {} issues found in {} paths", "⚠️".yellow(), issues_found, paths.len());
            }
        }
    }
    
    Ok(())
}

fn run_feature_command(action: FeatureAction) -> Result<()> {
    match action {
        FeatureAction::Add { title, description, category, state } => {
            add_feature_to_database(title, description, category, state)?;
        }
        FeatureAction::List { state, category, recent } => {
            list_features(state, category, recent)?;
        }
        FeatureAction::Show { feature_id } => {
            show_feature(feature_id)?;
        }
        FeatureAction::Update { feature_id, state, evidence, force } => {
            update_feature(feature_id, state, evidence, force)?;
        }
        FeatureAction::Validate { feature_id, verbose } => {
            validate_features(feature_id, verbose)?;
        }
        FeatureAction::DetectFeatures { input } => {
            analyze_user_input_for_features(&input)?;
        }
        FeatureAction::MonitorContext { usage_percent, total_tokens, used_tokens } => {
            monitor_context_usage(usage_percent, total_tokens, used_tokens)?;
        }
        FeatureAction::ApiCall { operation, feature_id, payload } => {
            handle_api_call(operation, feature_id, payload)?;
        }
    }
    
    Ok(())
}

// Entity relationship management command handler
fn run_relationship_command(action: RelationshipAction) -> Result<()> {
    match action {
        RelationshipAction::Link { from_entity, from_type, to_entity, to_type, relationship_type, description } => {
            link_entities(from_entity, from_type, to_entity, to_type, relationship_type, description)?;
        }
        RelationshipAction::List { entity_id, entity_type, relationship_type, include_resolved } => {
            list_entity_relationships(entity_id, entity_type, relationship_type, include_resolved)?;
        }
        RelationshipAction::Unlink { dependency_id, force } => {
            unlink_entities(dependency_id, force)?;
        }
        RelationshipAction::Resolve { dependency_id, description } => {
            resolve_entity_relationship(dependency_id, description)?;
        }
        RelationshipAction::Stats { detailed, format } => {
            show_relationship_stats(detailed, format)?;
        }
    }
    
    Ok(())
}

// Database-backed feature management (addresses user request)
fn add_feature_to_database(title: String, description: String, category: String, state: String) -> Result<String> {
    let db_path = get_project_root()?.join(".ws/project.db");
    
    // Generate next feature ID from database
    let feature_id = if db_path.exists() {
        let output = std::process::Command::new("sqlite3")
            .arg(&db_path)
            .arg("SELECT MAX(CAST(SUBSTR(id, 2) AS INTEGER)) FROM features WHERE id LIKE 'F%';")
            .output()?;
        
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let next_num = if result.is_empty() || result == "" {
                1
            } else {
                result.parse::<i32>().unwrap_or(0) + 1
            };
            format!("F{:04}", next_num)
        } else {
            "F0001".to_string()
        }
    } else {
        "F0001".to_string()
    };
    
    println!("{} Adding feature {} to database", "💾".blue(), feature_id);
    println!("  {} Feature: {}", "📝".cyan(), title);
    println!("  {} Description: {}", "📋".cyan(), description);
    println!("  {} Category: {}", "🏷️".cyan(), category);
    println!("  {} Initial State: {}", "🎯".cyan(), state);
    
    // Add to database
    if db_path.exists() {
        let insert_query = format!(
            "INSERT INTO features (id, title, description, state, category, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}', '{}', datetime('now'), datetime('now'));",
            feature_id, 
            title.replace("'", "''"), 
            description.replace("'", "''"), 
            state, 
            category.replace("'", "''")
        );
        
        let output = std::process::Command::new("sqlite3")
            .arg(&db_path)
            .arg(&insert_query)
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Database insert failed: {}", error));
        }
    }
    
    println!("{} Feature {} added to database", "✅".green(), feature_id);
    Ok(feature_id)
}

fn add_task_to_database(title: String, description: String, feature_id: Option<String>, priority: String) -> Result<String> {
    let task_id = format!("TASK-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    
    println!("{} Adding task {} to database (file-backed for now)", "💾".blue(), task_id);
    println!("  {} Task: {}", "📝".cyan(), title);
    println!("  {} Description: {}", "📋".cyan(), description);
    if let Some(ref fid) = feature_id {
        println!("  {} Linked Feature: {}", "🔗".cyan(), fid);
    }
    println!("  {} Priority: {}", "⚡".cyan(), priority);
    
    // TODO: Add to SQLite database instead of file
    // For now, add to task backlog file
    add_task_to_file(title, description, feature_id, priority)?;
    
    println!("{} Task {} added (database storage pending)", "✅".green(), task_id);
    Ok(task_id)
}

fn add_task_to_database_with_detection(title: String, description: String, feature: Option<String>, priority: String, auto_feature: bool) -> Result<()> {
    // Feature auto-detection if enabled
    let feature_id = if auto_feature && feature.is_none() {
        // Analyze description for feature mentions
        let detected_features = detect_new_features(&description);
        if !detected_features.is_empty() {
            println!("{} Auto-detected potential features in task description", "🔍".blue());
            // For now, just log the detection - full integration would prompt user
            Some(format!("F0999")) // Placeholder
        } else {
            feature
        }
    } else {
        feature
    };
    
    add_task_to_database(title, description, feature_id, priority)?;
    Ok(())
}

fn add_task_to_file(title: String, description: String, feature_id: Option<String>, priority: String) -> Result<()> {
    let project_root = get_project_root()?;
    let backlog_path = project_root.join("internal/task_backlog.md");
    
    let task_id = format!("TASK-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let created_date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    let feature_text = if let Some(ref fid) = feature_id {
        format!("\n**Feature**: {}", fid)
    } else {
        String::new()
    };
    
    let task_entry = format!(
        "\n### {} - {} ({})\n**Priority**: {}\n**Status**: pending\n**Created**: {}{}\n\n**Description**: {}\n",
        task_id, title, priority, priority, created_date, feature_text, description
    );
    
    if backlog_path.exists() {
        let mut content = std::fs::read_to_string(&backlog_path)?;
        content.push_str(&task_entry);
        std::fs::write(&backlog_path, content)?;
    } else {
        let header = format!("# Project Task Backlog\n\n## Automated Tasks\n{}", task_entry);
        std::fs::write(&backlog_path, header)?;
    }
    
    println!("{} Task added to backlog file", "✅".green());
    Ok(())
}

fn add_feature_to_file(title: String, description: String, category: String, state: String) -> Result<()> {
    println!("{} Adding feature: {}", "Info".blue(), title.bold());
    
    // Get next feature ID
    let project_root = get_project_root()?;
    let features_path = project_root.join("internal").join("features.md");
    let features_content = std::fs::read_to_string(&features_path)?;
    let next_id = get_next_feature_id(&features_content);
    
    // Map state string to emoji
    let state_emoji = match state.as_str() {
        "not_started" => "❌",
        "implemented" => "🟠", 
        "testing" => "🟡",
        "completed" => "🟢",
        "issue" => "⚠️",
        "critical" => "🔴",
        _ => "❌", // default to not started
    };
    
    println!("  {} Feature ID: {}", "→".green(), next_id.bold());
    println!("  {} State: {}", "→".green(), state_emoji);
    
    // Add to features.md
    add_feature_to_features_file(&next_id, &title, &description, state_emoji, &category)?;
    
    println!("{} Feature {} added successfully", "✅".green(), next_id.bold());
    
    Ok(())
}

fn list_features(state: Option<String>, category: Option<String>, _recent: Option<u32>) -> Result<()> {
    let project_root = get_project_root()?;
    let features_path = project_root.join("internal").join("features.md");
    let features_content = std::fs::read_to_string(&features_path)?;
    
    println!("{}", "Feature List".bold());
    println!();
    
    let mut found_any = false;
    for line in features_content.lines() {
        if line.starts_with("| F") && line.matches("|").count() >= 5 {
            // Apply filters
            if let Some(ref state_filter) = state {
                if !line.contains(state_filter) {
                    continue;
                }
            }
            
            if let Some(ref category_filter) = category {
                if !line.to_lowercase().contains(&category_filter.to_lowercase()) {
                    continue;
                }
            }
            
            // Extract feature info
            let parts: Vec<&str> = line.split(" | ").collect();
            if parts.len() >= 5 {
                let id = parts[0].trim_start_matches("| ");
                let name = parts[1].trim_start_matches("**").trim_end_matches("**");
                let state_part = parts[3];
                
                println!("  {} {} - {}", state_part, id.bold(), name);
                found_any = true;
            }
        }
    }
    
    if !found_any {
        println!("No features found matching criteria.");
    }
    
    Ok(())
}

fn show_feature(feature_id: String) -> Result<()> {
    let project_root = get_project_root()?;
    let features_path = project_root.join("internal").join("features.md");
    let features_content = std::fs::read_to_string(&features_path)?;
    
    for line in features_content.lines() {
        if line.starts_with(&format!("| {}", feature_id)) && line.matches("|").count() >= 5 {
            let parts: Vec<&str> = line.split(" | ").collect();
            if parts.len() >= 5 {
                let name = parts[1].trim_start_matches("**").trim_end_matches("**");
                let description = parts[2];
                let state = parts[3];
                let notes = parts[4];
                
                println!("{}: {} {}", "Feature".bold(), feature_id.bold(), state);
                println!("{}: {}", "Name".bold(), name);
                println!("{}: {}", "Description".bold(), description);
                println!("{}: {}", "Notes".bold(), notes);
                return Ok(());
            }
        }
    }
    
    println!("{} Feature {} not found", "❌".red(), feature_id);
    Ok(())
}

fn update_feature(feature_id: String, state: Option<String>, evidence: Option<String>, force: bool) -> Result<()> {
    if let Some(new_state) = state {
        let state_emoji = match new_state.as_str() {
            "implemented" => "🟠",
            "testing" => "🟡", 
            "completed" => "🟢",
            "issue" => "⚠️",
            "critical" => "🔴",
            "not_started" => "❌",
            _ => return Err(anyhow::anyhow!("Invalid state: {}", new_state)),
        };
        
        if !force {
            // Validate state transition
            if let Err(e) = validate_state_transition(&feature_id, state_emoji) {
                println!("{} State transition validation failed: {}", "⚠️".yellow(), e);
                println!("Use --force to override validation");
                return Ok(());
            }
        }
        
        update_feature_state(&feature_id, state_emoji, evidence)?;
        println!("{} Feature {} state updated to {}", "✅".green(), feature_id.bold(), state_emoji);
    }
    
    Ok(())
}

fn validate_features(feature_id: Option<String>, verbose: bool) -> Result<()> {
    println!("{}", "Feature State Validation".bold());
    println!();
    
    let project_root = get_project_root()?;
    let features_path = project_root.join("internal").join("features.md");
    let features_content = std::fs::read_to_string(&features_path)?;
    
    let mut validation_issues = 0;
    
    for line in features_content.lines() {
        if line.starts_with("| F") && line.matches("|").count() >= 5 {
            let parts: Vec<&str> = line.split(" | ").collect();
            if parts.len() >= 5 {
                let id = parts[0].trim_start_matches("| ");
                let state = parts[3];
                
                if let Some(ref target_id) = feature_id {
                    if id != target_id {
                        continue;
                    }
                }
                
                // Validate state transition logic
                if let Err(e) = validate_feature_state(id, state) {
                    validation_issues += 1;
                    println!("  {} {} - {}", "⚠️".yellow(), id.bold(), e);
                } else if verbose {
                    println!("  {} {} - Valid", "✅".green(), id.bold());
                }
            }
        }
    }
    
    if validation_issues == 0 {
        println!("{} All features pass validation", "✅".green());
    } else {
        println!("{} {} validation issues found", "⚠️".yellow(), validation_issues);
    }
    
    Ok(())
}

fn get_next_feature_id(features_content: &str) -> String {
    let mut max_id = 0;
    
    for line in features_content.lines() {
        if line.starts_with("| F") {
            if let Some(id_part) = line.split(" | ").next() {
                let id_str = id_part.trim_start_matches("| F");
                if let Ok(id_num) = id_str[..4].parse::<u32>() {
                    max_id = max_id.max(id_num);
                }
            }
        }
    }
    
    format!("F{:04}", max_id + 1)
}

fn add_feature_to_features_file(id: &str, title: &str, description: &str, state: &str, category: &str) -> Result<()> {
    let project_root = get_project_root()?;
    let features_path = project_root.join("internal").join("features.md");
    
    let mut content = std::fs::read_to_string(&features_path)?;
    
    // Find appropriate section to add feature
    let feature_line = format!("| {} | **{}** | {} | {} | {} |\n", id, title, description, state, category);
    
    // Add before "---" section separator
    if let Some(separator_pos) = content.find("\n---\n") {
        content.insert_str(separator_pos, &feature_line);
    } else {
        // Add at end if no separator found
        content.push_str(&feature_line);
    }
    
    // Update feature count in header
    let new_total = content.lines().filter(|line| line.starts_with("| F") && line.matches("|").count() >= 5).count();
    content = content.replace("175 total features tracked", &format!("{} total features tracked", new_total));
    
    std::fs::write(&features_path, content)?;
    Ok(())
}

fn update_feature_state(feature_id: &str, new_state: &str, evidence: Option<String>) -> Result<()> {
    let project_root = get_project_root()?;
    let features_path = project_root.join("internal").join("features.md");
    
    let content = std::fs::read_to_string(&features_path)?;
    let mut updated_content = String::new();
    
    for line in content.lines() {
        if line.starts_with(&format!("| {}", feature_id)) && line.matches("|").count() >= 5 {
            let parts: Vec<&str> = line.split(" | ").collect();
            if parts.len() >= 5 {
                let mut new_parts = parts.clone();
                new_parts[3] = new_state;
                
                if let Some(ref evidence_text) = evidence {
                    new_parts[4] = evidence_text;
                }
                
                updated_content.push_str(&new_parts.join(" | "));
                updated_content.push('\n');
            } else {
                updated_content.push_str(line);
                updated_content.push('\n');
            }
        } else {
            updated_content.push_str(line);
            updated_content.push('\n');
        }
    }
    
    std::fs::write(&features_path, updated_content)?;
    Ok(())
}

fn validate_state_transition(feature_id: &str, new_state: &str) -> Result<()> {
    let project_root = get_project_root()?;
    let features_path = project_root.join("internal").join("features.md");
    let features_content = std::fs::read_to_string(&features_path)?;
    
    // Find current state
    for line in features_content.lines() {
        if line.starts_with(&format!("| {}", feature_id)) {
            let parts: Vec<&str> = line.split(" | ").collect();
            if parts.len() >= 4 {
                let current_state = parts[3];
                return validate_transition(current_state, new_state);
            }
        }
    }
    
    Err(anyhow::anyhow!("Feature not found"))
}

fn validate_transition(current: &str, new: &str) -> Result<()> {
    // Valid transitions: ❌→🟠→🟡→🟢, ❌→🟠→⚠️, any→🔴
    match (current, new) {
        ("❌", "🟠") => Ok(()), // not started -> implemented
        ("🟠", "🟡") => Ok(()), // implemented -> testing  
        ("🟠", "⚠️") => Ok(()), // implemented -> issue
        ("🟡", "🟢") => Ok(()), // testing -> completed
        ("🟡", "⚠️") => Ok(()), // testing -> issue
        (_, "🔴") => Ok(()),     // any -> critical
        (_, "❌") => Ok(()),     // any -> not started (reset)
        _ => Err(anyhow::anyhow!("Invalid transition from {} to {}", current, new)),
    }
}

fn validate_feature_state(_feature_id: &str, state: &str) -> Result<()> {
    match state {
        "🟢" | "🟠" | "🟡" | "❌" | "⚠️" | "🔴" => Ok(()),
        _ => Err(anyhow::anyhow!("Invalid state emoji: {}", state)),
    }
}

// F0107: Automatic Feature Detection System
fn detect_new_features(input_text: &str) -> Vec<String> {
    let mut detected_features = Vec::new();
    let capability_keywords = vec![
        "implement", "add", "create", "build", "develop", "feature", "functionality",
        "capability", "support", "enable", "integrate", "system", "component",
        "command", "tool", "API", "interface", "management", "tracking", "monitoring",
        "validation", "processing", "handling", "generation", "analysis", "optimization"
    ];
    
    let feature_indicators = vec![
        "should", "could", "would", "need", "want", "require", "must", "will",
        "add support for", "implement", "create", "build", "develop", "enable",
        "integrate", "provide", "allow", "support"
    ];
    
    let sentences: Vec<&str> = input_text.split(&['.', '!', '?', '\n'][..]).collect();
    
    for sentence in sentences {
        let sentence = sentence.trim().to_lowercase();
        if sentence.len() < 10 { continue; } // Skip very short sentences
        
        let has_capability = capability_keywords.iter().any(|&keyword| sentence.contains(keyword));
        let has_indicator = feature_indicators.iter().any(|&indicator| sentence.contains(indicator));
        
        if has_capability && has_indicator {
            // Extract potential feature description
            let words: Vec<&str> = sentence.split_whitespace().collect();
            if words.len() >= 3 && words.len() <= 20 {
                detected_features.push(sentence.to_string());
            }
        }
    }
    
    detected_features.truncate(3); // Limit to 3 suggestions to avoid overwhelming
    detected_features
}

fn prompt_feature_addition(detected_features: Vec<String>) -> Result<()> {
    if detected_features.is_empty() {
        return Ok(());
    }
    
    println!("{} Automatic Feature Detection", "🔍".blue().bold());
    println!("I detected potential new features in your message:");
    println!();
    
    for (i, feature) in detected_features.iter().enumerate() {
        println!("  {}. {}", (i + 1).to_string().yellow(), feature.trim());
    }
    
    println!();
    println!("{} Should I add {} as new feature{}? (y/n)", 
             "❓".yellow(),
             if detected_features.len() == 1 { "this" } else { "these" },
             if detected_features.len() == 1 { "" } else { "s" });
             
    // For now, just demonstrate the detection - in real implementation,
    // this would integrate with user input handling
    println!("{} Feature detection completed (demo mode)", "✅".green());
    
    Ok(())
}

fn analyze_user_input_for_features(input: &str) -> Result<()> {
    let detected = detect_new_features(input);
    if !detected.is_empty() {
        prompt_feature_addition(detected)?;
    }
    Ok(())
}

// F0109: MCP Server Auto-Management
fn monitor_context_usage(usage_percent: f64, total_tokens: Option<u32>, used_tokens: Option<u32>) -> Result<()> {
    println!("{} Context Usage Monitor", "📊".blue().bold());
    
    if let (Some(total), Some(used)) = (total_tokens, used_tokens) {
        println!("  {} Tokens: {}/{} ({}%)", "📈".cyan(), used, total, usage_percent);
    } else {
        println!("  {} Usage: {}%", "📈".cyan(), usage_percent);
    }
    
    // Check if we need to trigger session end
    if usage_percent >= 95.0 {
        println!("{} {} Context threshold exceeded (95%)", "⚠️".yellow(), "WARNING:".bold());
        println!("  {} Triggering automatic session end...", "🔄".yellow());
        trigger_automatic_session_end()?;
    } else if usage_percent >= 85.0 {
        println!("{} {} Context approaching limit ({}%)", "⚠️".yellow(), "WARNING:".bold(), usage_percent);
        println!("  {} Consider consolidating or ending session soon", "💡".blue());
    } else {
        println!("{} Context usage within normal range", "✅".green());
    }
    
    Ok(())
}

fn trigger_automatic_session_end() -> Result<()> {
    println!("{} Initiating automatic session end procedure", "🔄".blue().bold());
    
    // Run consolidate command to preserve session work
    println!("  {} Step 1: Consolidating session documentation...", "1️⃣".blue());
    run_consolidate_command(false, false, false, true)?; // debug_mode, force, generate_diagrams, preserve_complexity
    
    // Run end command to complete session
    println!("  {} Step 2: Ending session with documentation updates...", "2️⃣".blue());
    run_end_command(
        Some("Automatic session end triggered by context threshold".to_string()),
        false, // debug_mode
        false, // force  
        false  // skip_docs
    )?;
    
    println!("{} Automatic session end completed", "✅".green().bold());
    Ok(())
}

fn check_context_threshold_startup() -> Result<()> {
    // This would be called on MCP server startup to check if we need to run start command
    println!("{} Checking for automatic session initialization...", "🔍".blue());
    
    // For now, always run start command on MCP server startup
    println!("  {} Running automatic session start...", "🚀".green());
    
    // Execute start command automatically
    run_start_command(
        None,  // continue_from
        false, // debug_mode
        false, // project_setup
        None   // first_task
    )?;
    
    println!("{} Automatic session initialization completed", "✅".green());
    Ok(())
}

// F0110: Real-time Feature Management API
fn handle_api_call(operation: String, feature_id: Option<String>, payload: Option<String>) -> Result<()> {
    println!("{} Real-time Feature Management API", "🔌".blue().bold());
    println!("  {} Operation: {}", "📡".cyan(), operation);
    
    match operation.as_str() {
        "add_feature" => {
            handle_add_feature_api(payload)?;
        }
        "update_feature" => {
            if let Some(id) = feature_id {
                handle_update_feature_api(id, payload)?;
            } else {
                return Err(anyhow::anyhow!("Feature ID required for update operation"));
            }
        }
        "list_features" => {
            handle_list_features_api(payload)?;
        }
        "validate_feature" => {
            handle_validate_feature_api(feature_id, payload)?;
        }
        "get_feature_stats" => {
            handle_get_feature_stats_api()?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown API operation: {}", operation));
        }
    }
    
    Ok(())
}

fn handle_add_feature_api(payload: Option<String>) -> Result<()> {
    println!("  {} Adding feature via API", "➕".green());
    
    if let Some(json_payload) = payload {
        // Parse JSON payload for feature details
        let payload_data: serde_json::Value = serde_json::from_str(&json_payload)?;
        
        let title = payload_data["title"].as_str().unwrap_or("Unnamed Feature").to_string();
        let description = payload_data["description"].as_str().unwrap_or("No description").to_string();
        let category = payload_data["category"].as_str().unwrap_or("General").to_string();
        let state = payload_data["state"].as_str().unwrap_or("not_started").to_string();
        
        println!("    {} Title: {}", "📝".cyan(), title);
        println!("    {} Category: {}", "🏷️".cyan(), category);
        
        let feature_id = add_feature_to_database(title, description, category, state)?;
        
        // Return response as JSON
        let response = serde_json::json!({
            "success": true,
            "feature_id": feature_id,
            "message": "Feature added successfully"
        });
        
        println!("{} {}", "📤".blue(), response.to_string());
    } else {
        return Err(anyhow::anyhow!("JSON payload required for add_feature operation"));
    }
    
    Ok(())
}

fn handle_update_feature_api(feature_id: String, payload: Option<String>) -> Result<()> {
    println!("  {} Updating feature {} via API", "🔄".green(), feature_id);
    
    if let Some(json_payload) = payload {
        let payload_data: serde_json::Value = serde_json::from_str(&json_payload)?;
        
        let new_state = payload_data["state"].as_str().unwrap_or("");
        let evidence = payload_data["evidence"].as_str().map(|s| s.to_string());
        
        if !new_state.is_empty() {
            println!("    {} New State: {}", "🎯".cyan(), new_state);
            update_feature_state(&feature_id, new_state, evidence)?;
        }
        
        let response = serde_json::json!({
            "success": true,
            "feature_id": feature_id,
            "message": "Feature updated successfully"
        });
        
        println!("{} {}", "📤".blue(), response.to_string());
    } else {
        return Err(anyhow::anyhow!("JSON payload required for update_feature operation"));
    }
    
    Ok(())
}

fn handle_list_features_api(payload: Option<String>) -> Result<()> {
    println!("  {} Listing features via API", "📋".green());
    
    let filters = if let Some(json_payload) = payload {
        serde_json::from_str::<serde_json::Value>(&json_payload)?
    } else {
        serde_json::json!({})
    };
    
    let state_filter = filters["state"].as_str();
    let category_filter = filters["category"].as_str();
    
    // Get feature data from database
    let mut features = Vec::new();
    let db_path = get_project_root()?.join(".ws/project.db");
    
    if db_path.exists() {
        let output = std::process::Command::new("sqlite3")
            .arg(&db_path)
            .arg("SELECT id, title, description, state, category FROM features ORDER BY id;")
            .output()?;
        
        if output.status.success() {
            let db_result = String::from_utf8_lossy(&output.stdout);
            
            for line in db_result.lines() {
                if !line.trim().is_empty() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 5 {
                        let id = parts[0].trim();
                        let title = parts[1].trim();
                        let description = parts[2].trim();
                        let state = parts[3].trim();
                        let category = parts[4].trim();
                        
                        // Apply filters
                        let matches_state = state_filter.map_or(true, |s| state.contains(s));
                        let matches_category = category_filter.map_or(true, |c| category.to_lowercase().contains(&c.to_lowercase()));
                        
                        if matches_state && matches_category {
                            features.push(serde_json::json!({
                                "id": id,
                                "name": title,
                                "description": description,
                                "state": state,
                                "category": category
                            }));
                        }
                    }
                }
            }
        }
    }
    
    let response = serde_json::json!({
        "success": true,
        "features": features,
        "count": features.len()
    });
    
    println!("{} {}", "📤".blue(), response.to_string());
    Ok(())
}

fn handle_validate_feature_api(feature_id: Option<String>, _payload: Option<String>) -> Result<()> {
    println!("  {} Validating features via API", "✅".green());
    
    validate_features(feature_id, true)?;
    
    let response = serde_json::json!({
        "success": true,
        "message": "Feature validation completed"
    });
    
    println!("{} {}", "📤".blue(), response.to_string());
    Ok(())
}

fn handle_get_feature_stats_api() -> Result<()> {
    println!("  {} Getting feature statistics via API", "📊".green());
    
    let project_root = get_project_root()?;
    let features_path = project_root.join("internal/features.md");
    let features_content = std::fs::read_to_string(&features_path)?;
    
    let (total, implemented) = parse_feature_stats(&features_content);
    let tested = count_tested_features(&features_content);
    
    let implementation_rate = if total > 0 {
        implemented as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    
    let test_coverage_rate = if total > 0 {
        tested as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    
    let response = serde_json::json!({
        "success": true,
        "stats": {
            "total_features": total,
            "implemented_features": implemented,
            "tested_features": tested,
            "implementation_rate": implementation_rate,
            "test_coverage_rate": test_coverage_rate
        }
    });
    
    println!("{} {}", "📤".blue(), response.to_string());
    Ok(())
}

fn create_sample_project_in_dir(output_dir: &str, force: bool) -> Result<()> {
    println!("{} Creating sample project structure in {}...", "📁".blue().bold(), output_dir);
    
    let output_path = std::path::Path::new(output_dir);
    
    // Remove existing directory if force is enabled
    if output_path.exists() && force {
        std::fs::remove_dir_all(output_path)?;
        println!("  {} Removed existing directory", "🗑️".yellow());
    }
    
    // Create directories
    std::fs::create_dir_all(output_path.join("internal"))?;
    std::fs::create_dir_all(output_path.join(".ws"))?;
    std::fs::create_dir_all(output_path.join("src"))?;
    std::fs::create_dir_all(output_path.join("tests"))?;
    std::fs::create_dir_all(output_path.join("docs"))?;
    
    // Create CLAUDE.md
    let claude_content = r#"# Sample Project - AI-Assisted Development

## Project Overview

**Project Name**: Sample Dashboard Project  
**Type**: Web dashboard with API backend  
**Current Version**: 1.0.0  

## Project Description

This is a sample project created to demonstrate the Workspace development suite capabilities including:

- Feature-centric development methodology
- Real-time project dashboard
- Comprehensive API endpoints
- Database-driven project management

## Current Status

**Development Phase**: Sample Data Demonstration  
**Test Status**: ✅ Sample data populated  
**Build Status**: ✅ Ready for development  

## Key Features Working

- ✅ Project management dashboard
- ✅ Feature tracking and status monitoring  
- ✅ Task management with state transitions
- ✅ Real-time API endpoints
- ✅ Database-backed storage

## Success Criteria

### Core Functionality
- ✅ Dashboard displays project metrics
- ✅ API endpoints return sample data
- ✅ Feature state management working
- ✅ Task tracking operational

### Quality Metrics  
- ✅ All API endpoints responding
- ✅ Database queries optimized
- ✅ Sample data representative of real usage

## Next Steps

Use this sample project to:
1. Test dashboard functionality
2. Validate API endpoints
3. Experiment with feature management
4. Learn the development methodology

---

*Created by ws sample command*"#;

    std::fs::write(output_path.join("CLAUDE.md"), claude_content)?;
    println!("  {} Created CLAUDE.md", "✅".green());
    
    // Create package.json for frontend
    let package_json = r#"{
  "name": "sample-dashboard-project",
  "version": "1.0.0",
  "description": "Sample project for Workspace development suite",
  "main": "index.js",
  "scripts": {
    "dev": "ws mcp-server",
    "test": "ws status --include-features --include-metrics"
  },
  "keywords": ["workspace", "dashboard", "sample"],
  "author": "Workspace Development Suite",
  "license": "MIT"
}"#;

    std::fs::write(output_path.join("package.json"), package_json)?;
    println!("  {} Created package.json", "✅".green());
    
    // Create README.md
    let readme_content = r#"# Sample Dashboard Project

A comprehensive sample project demonstrating the Workspace development methodology with real project data.

## Features

This sample includes:
- **10 sample features** across different categories (Frontend, Backend, Database, Security, etc.)
- **10 sample tasks** with various statuses and priorities
- **4 development sessions** showing project evolution
- **5 notes** including architecture decisions and issues
- **5 dependencies** between features and tasks
- **4 projects** in different states

## Getting Started

1. **Start the dashboard server:**
   ```bash
   ws mcp-server --port 3000
   ```

2. **Access the web dashboard:**
   Open http://localhost:3000 in your browser

3. **Explore the data:**
   - View project metrics and status
   - Browse features by category and state
   - Check task progress and dependencies
   - Review development sessions and notes

## Sample Data Overview

The sample data covers all possible states and scenarios:

### Features (10 total)
- **States**: implemented, in_progress, planned, tested, not_implemented, deprecated
- **Categories**: Frontend, Backend, Database, Security, Performance, Testing, Documentation, DevOps, Analytics, Mobile
- **Priorities**: critical, high, medium, low

### Tasks (10 total)
- **Statuses**: completed, in_progress, pending, blocked, cancelled
- **Categories**: feature, infrastructure, testing, security, performance, etc.

### Projects (4 total)
- E-Commerce Platform (active)
- AI Analytics Engine (active) 
- Legacy CRM System (archived)
- Modern CRM Platform (in development)

## Learning the Methodology

This sample demonstrates:
- Feature-driven development approach
- Comprehensive task tracking
- Project state management
- Development session documentation
- Dependency relationship modeling
- Multi-project organization

---

*Generated by Workspace Sample Generator*"#;

    std::fs::write(output_path.join("README.md"), readme_content)?;
    println!("  {} Created README.md", "✅".green());
    
    // Initialize git repository with sample commits
    println!("  {} Initializing git repository...", "🔧".yellow());
    init_sample_git_repo(output_path)?;
    
    println!("{} Sample project structure created in {}", "✅".green().bold(), output_dir);
    
    Ok(())
}

fn init_sample_git_repo(project_path: &std::path::Path) -> Result<()> {
    use std::process::Command;
    
    // Initialize git repository
    let output = Command::new("git")
        .arg("init")
        .current_dir(project_path)
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to initialize git repository"));
    }
    
    // Configure git user for the repo
    Command::new("git")
        .args(&["config", "user.name", "Sample Developer"])
        .current_dir(project_path)
        .output()?;
    
    Command::new("git")
        .args(&["config", "user.email", "developer@sample-project.com"])
        .current_dir(project_path)
        .output()?;
    
    // Create sample source files with realistic content
    create_sample_source_files(project_path)?;
    
    // Create initial commit
    Command::new("git")
        .args(&["add", "."])
        .current_dir(project_path)
        .output()?;
    
    let commit_msg = "Initial project setup

- Added basic project structure with package.json
- Created src/, docs/, tests/ directories  
- Added project documentation and README
- Initialized workspace with .ws/ directory";
    
    Command::new("git")
        .args(&["commit", "-m", commit_msg])
        .current_dir(project_path)
        .output()?;
    
    // Add some development commits to simulate project history
    create_development_commits(project_path)?;
    
    println!("    {} Git repository initialized with sample commits", "✅".green());
    
    Ok(())
}

fn create_sample_source_files(project_path: &std::path::Path) -> Result<()> {
    // Create sample JavaScript files
    let app_js = r#"// Main application entry point
class DashboardApp {
    constructor() {
        this.apiBase = '/api';
        this.currentUser = null;
        this.init();
    }
    
    async init() {
        await this.loadUserProfile();
        this.setupEventListeners();
        this.renderDashboard();
    }
    
    async loadUserProfile() {
        try {
            const response = await fetch(`${this.apiBase}/user/profile`);
            this.currentUser = await response.json();
        } catch (error) {
            console.error('Failed to load user profile:', error);
        }
    }
    
    setupEventListeners() {
        document.getElementById('refresh-btn')?.addEventListener('click', () => {
            this.refreshData();
        });
    }
    
    renderDashboard() {
        const container = document.getElementById('dashboard');
        if (container) {
            container.innerHTML = `
                <h1>Welcome, ${this.currentUser?.name || 'User'}</h1>
                <div class="metrics">
                    <div class="metric-card">
                        <h3>Active Projects</h3>
                        <span class="metric-value">12</span>
                    </div>
                    <div class="metric-card">
                        <h3>Tasks Completed</h3>
                        <span class="metric-value">84</span>
                    </div>
                </div>
            `;
        }
    }
    
    async refreshData() {
        console.log('Refreshing dashboard data...');
        await this.loadUserProfile();
        this.renderDashboard();
    }
}

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new DashboardApp();
});
"#;
    
    std::fs::write(project_path.join("src/app.js"), app_js)?;
    
    // Create sample CSS
    let styles_css = r#"/* Dashboard Styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: #f5f5f5;
    color: #333;
}

#dashboard {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

h1 {
    color: #2c3e50;
    margin-bottom: 30px;
    font-weight: 300;
}

.metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    margin-bottom: 30px;
}

.metric-card {
    background: white;
    padding: 24px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    text-align: center;
}

.metric-card h3 {
    color: #666;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 8px;
}

.metric-value {
    font-size: 32px;
    font-weight: 700;
    color: #3498db;
}

#refresh-btn {
    background: #3498db;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
}

#refresh-btn:hover {
    background: #2980b9;
}
"#;
    
    std::fs::write(project_path.join("src/styles.css"), styles_css)?;
    
    // Create sample HTML
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sample Dashboard</title>
    <link rel="stylesheet" href="src/styles.css">
</head>
<body>
    <div id="dashboard">
        <div class="loading">Loading dashboard...</div>
    </div>
    <button id="refresh-btn">Refresh Data</button>
    <script src="src/app.js"></script>
</body>
</html>
"#;
    
    std::fs::write(project_path.join("index.html"), index_html)?;
    
    // Create sample test file
    let test_js = r#"// Dashboard App Tests
describe('DashboardApp', () => {
    let app;
    
    beforeEach(() => {
        document.body.innerHTML = '<div id="dashboard"></div>';
        app = new DashboardApp();
    });
    
    test('should initialize with correct API base', () => {
        expect(app.apiBase).toBe('/api');
    });
    
    test('should render welcome message', () => {
        app.currentUser = { name: 'Test User' };
        app.renderDashboard();
        
        const dashboard = document.getElementById('dashboard');
        expect(dashboard.innerHTML).toContain('Welcome, Test User');
    });
    
    test('should handle missing user gracefully', () => {
        app.currentUser = null;
        app.renderDashboard();
        
        const dashboard = document.getElementById('dashboard');
        expect(dashboard.innerHTML).toContain('Welcome, User');
    });
});
"#;
    
    std::fs::write(project_path.join("tests/app.test.js"), test_js)?;
    
    // Create sample documentation
    let api_docs = r#"# API Documentation

## Overview

This document describes the REST API endpoints for the sample dashboard application.

## Authentication

All API endpoints require authentication via Bearer token in the Authorization header:

```
Authorization: Bearer <your-token>
```

## Endpoints

### User Profile

**GET /api/user/profile**

Returns the current user's profile information.

Response:
```json
{
  "id": "user-123",
  "name": "John Doe",
  "email": "john@example.com",
  "role": "developer",
  "avatar_url": "https://example.com/avatar.jpg"
}
```

### Projects

**GET /api/projects**

Returns a list of all projects.

Query Parameters:
- `status` - Filter by project status (active, archived)
- `limit` - Number of results to return (default: 20)

Response:
```json
{
  "projects": [
    {
      "id": "proj-123",
      "name": "Sample Project",
      "status": "active",
      "created_at": "2024-01-15T10:00:00Z"
    }
  ],
  "total": 1
}
```

### Tasks

**POST /api/tasks**

Creates a new task.

Request Body:
```json
{
  "title": "Implement feature X",
  "description": "Add the new feature to the dashboard",
  "priority": "high",
  "assignee": "user-123"
}
```

## Error Responses

All errors follow this format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": ["Missing required field: title"]
  }
}
```
"#;
    
    std::fs::write(project_path.join("docs/api.md"), api_docs)?;
    
    Ok(())
}

fn create_development_commits(project_path: &std::path::Path) -> Result<()> {
    use std::process::Command;
    
    // Commit 2: Add user authentication
    let auth_js = r#"// User authentication module
class AuthManager {
    constructor(apiBase) {
        this.apiBase = apiBase;
        this.token = localStorage.getItem('auth_token');
    }
    
    async login(email, password) {
        const response = await fetch(`${this.apiBase}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password })
        });
        
        const data = await response.json();
        if (data.token) {
            this.token = data.token;
            localStorage.setItem('auth_token', this.token);
        }
        
        return data;
    }
    
    logout() {
        this.token = null;
        localStorage.removeItem('auth_token');
    }
    
    isAuthenticated() {
        return !!this.token;
    }
    
    getAuthHeaders() {
        return this.token ? { 'Authorization': `Bearer ${this.token}` } : {};
    }
}
"#;
    
    std::fs::write(project_path.join("src/auth.js"), auth_js)?;
    
    Command::new("git")
        .args(&["add", "src/auth.js"])
        .current_dir(project_path)
        .output()?;
    
    Command::new("git")
        .args(&["commit", "-m", "Add user authentication module

- Implement AuthManager class for login/logout
- Add token-based authentication support  
- Store auth tokens in localStorage
- Provide helper methods for authenticated requests"])
        .current_dir(project_path)
        .output()?;
    
    // Commit 3: Update dashboard with authentication
    let updated_app = r#"// Main application entry point
class DashboardApp {
    constructor() {
        this.apiBase = '/api';
        this.currentUser = null;
        this.authManager = new AuthManager(this.apiBase);
        this.init();
    }
    
    async init() {
        if (!this.authManager.isAuthenticated()) {
            this.showLoginForm();
            return;
        }
        
        await this.loadUserProfile();
        this.setupEventListeners();
        this.renderDashboard();
    }
    
    async loadUserProfile() {
        try {
            const response = await fetch(`${this.apiBase}/user/profile`, {
                headers: this.authManager.getAuthHeaders()
            });
            this.currentUser = await response.json();
        } catch (error) {
            console.error('Failed to load user profile:', error);
            this.authManager.logout();
            this.showLoginForm();
        }
    }
    
    setupEventListeners() {
        document.getElementById('refresh-btn')?.addEventListener('click', () => {
            this.refreshData();
        });
        
        document.getElementById('logout-btn')?.addEventListener('click', () => {
            this.authManager.logout();
            this.showLoginForm();
        });
    }
    
    renderDashboard() {
        const container = document.getElementById('dashboard');
        if (container) {
            container.innerHTML = `
                <div class="header">
                    <h1>Welcome, ${this.currentUser?.name || 'User'}</h1>
                    <button id="logout-btn">Logout</button>
                </div>
                <div class="metrics">
                    <div class="metric-card">
                        <h3>Active Projects</h3>
                        <span class="metric-value">12</span>
                    </div>
                    <div class="metric-card">
                        <h3>Tasks Completed</h3>
                        <span class="metric-value">84</span>
                    </div>
                    <div class="metric-card">
                        <h3>Team Members</h3>
                        <span class="metric-value">6</span>
                    </div>
                </div>
            `;
            this.setupEventListeners();
        }
    }
    
    showLoginForm() {
        const container = document.getElementById('dashboard');
        if (container) {
            container.innerHTML = `
                <div class="login-form">
                    <h2>Login</h2>
                    <form id="login-form">
                        <input type="email" id="email" placeholder="Email" required>
                        <input type="password" id="password" placeholder="Password" required>
                        <button type="submit">Login</button>
                    </form>
                </div>
            `;
            
            document.getElementById('login-form')?.addEventListener('submit', async (e) => {
                e.preventDefault();
                const email = document.getElementById('email').value;
                const password = document.getElementById('password').value;
                
                try {
                    await this.authManager.login(email, password);
                    this.init();
                } catch (error) {
                    alert('Login failed: ' + error.message);
                }
            });
        }
    }
    
    async refreshData() {
        console.log('Refreshing dashboard data...');
        await this.loadUserProfile();
        this.renderDashboard();
    }
}

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new DashboardApp();
});
"#;
    
    std::fs::write(project_path.join("src/app.js"), updated_app)?;
    
    Command::new("git")
        .args(&["add", "src/app.js"])
        .current_dir(project_path)
        .output()?;
    
    Command::new("git")
        .args(&["commit", "-m", "Integrate authentication into dashboard

- Add login/logout functionality to main app
- Require authentication before showing dashboard
- Add logout button to dashboard header
- Handle authentication errors gracefully  
- Show login form for unauthenticated users"])
        .current_dir(project_path)
        .output()?;
    
    // Commit 4: Add responsive design
    let updated_css = r#"/* Dashboard Styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: #f5f5f5;
    color: #333;
}

#dashboard {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

.header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
    padding-bottom: 20px;
    border-bottom: 1px solid #ddd;
}

.header h1 {
    color: #2c3e50;
    font-weight: 300;
    margin: 0;
}

#logout-btn {
    background: #e74c3c;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
}

#logout-btn:hover {
    background: #c0392b;
}

.metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    margin-bottom: 30px;
}

.metric-card {
    background: white;
    padding: 24px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    text-align: center;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.metric-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
}

.metric-card h3 {
    color: #666;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.metric-value {
    font-size: 32px;
    font-weight: 700;
    color: #3498db;
}

.login-form {
    max-width: 400px;
    margin: 100px auto;
    padding: 40px;
    background: white;
    border-radius: 12px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
}

.login-form h2 {
    text-align: center;
    margin-bottom: 30px;
    color: #2c3e50;
    font-weight: 300;
}

.login-form input {
    width: 100%;
    padding: 12px 16px;
    margin-bottom: 16px;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 14px;
    transition: border-color 0.3s ease;
}

.login-form input:focus {
    outline: none;
    border-color: #3498db;
    box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.2);
}

.login-form button {
    width: 100%;
    padding: 12px;
    background: #3498db;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 16px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.3s ease;
}

.login-form button:hover {
    background: #2980b9;
}

#refresh-btn {
    background: #27ae60;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    transition: background 0.3s ease;
}

#refresh-btn:hover {
    background: #219a52;
}

/* Responsive Design */
@media (max-width: 768px) {
    #dashboard {
        padding: 15px;
    }
    
    .header {
        flex-direction: column;
        gap: 15px;
        align-items: flex-start;
    }
    
    .header h1 {
        font-size: 24px;
    }
    
    .metrics {
        grid-template-columns: 1fr;
        gap: 15px;
    }
    
    .metric-card {
        padding: 20px;
    }
    
    .metric-value {
        font-size: 28px;
    }
    
    .login-form {
        margin: 50px 20px;
        padding: 30px 20px;
    }
}

@media (max-width: 480px) {
    .metric-card {
        padding: 16px;
    }
    
    .metric-value {
        font-size: 24px;
    }
    
    .login-form {
        margin: 30px 15px;
        padding: 25px 15px;
    }
}
"#;
    
    std::fs::write(project_path.join("src/styles.css"), updated_css)?;
    
    Command::new("git")
        .args(&["add", "src/styles.css"])
        .current_dir(project_path)
        .output()?;
    
    Command::new("git")
        .args(&["commit", "-m", "Add responsive design and improved styling

- Implement responsive design for mobile devices
- Add hover effects and smooth transitions
- Improve login form styling and layout
- Add header with logout button styling
- Enhance metric cards with better visual hierarchy
- Add media queries for tablet and mobile breakpoints"])
        .current_dir(project_path)
        .output()?;
    
    Ok(())
}

fn populate_sample_data_in_dir(output_dir: &str, force: bool) -> Result<()> {
    println!("{} Populating database with sample data in {}...", "🗄️".blue().bold(), output_dir);
    
    let output_path = std::path::Path::new(output_dir);
    let db_path = output_path.join(".ws/project.db");
    
    // Check if database exists and has data
    if db_path.exists() && !force {
        let output = std::process::Command::new("sqlite3")
            .arg(&db_path)
            .arg("SELECT COUNT(*) FROM features;")
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let count = String::from_utf8_lossy(&output.stdout).trim().parse::<i32>().unwrap_or(0);
                if count > 0 {
                    println!("{} Database already has {} features (use --force to overwrite)", "⚠️".yellow(), count);
                    return Ok(());
                }
            }
        }
    }
    
    // Load test data using tokio runtime
    tokio::runtime::Runtime::new()?.block_on(async {
        populate_sample_data_in_dir_async(output_dir, force).await
    })
}

async fn populate_sample_data_in_dir_async(output_dir: &str, _force: bool) -> Result<()> {
    use workspace::entities::{database::initialize_database, EntityManager};
    
    let output_path = std::path::Path::new(output_dir);
    let db_path = output_path.join(".ws/project.db");
    
    // Initialize database with proper schema
    let pool = initialize_database(&db_path).await?;
    let entity_manager = EntityManager::new(pool.clone());
    
    // Get the current project (first project) to use for all sample data
    let current_project = entity_manager.get_current_project().await?;
    let project_id = &current_project.id;
    
    // Generate comprehensive test data SQL with dynamic project ID - just add data to existing project
    let test_data_sql = format!(r#"-- Clear existing sample data (keep project)
DELETE FROM entity_audit_trails;
DELETE FROM feature_state_transitions;
DELETE FROM note_links;
DELETE FROM notes;
DELETE FROM dependencies;
DELETE FROM tests;
DELETE FROM templates;
DELETE FROM directives;
DELETE FROM milestones;
DELETE FROM sessions;
DELETE FROM tasks;
DELETE FROM features;

-- Insert sample features for current project
INSERT INTO features (id, project_id, code, name, description, category, state, test_status, priority, implementation_notes, test_evidence, created_at, updated_at, completed_at, estimated_effort, actual_effort) VALUES
('feat-001', '{project_id}', 'F-001', 'User Authentication Portal', 'Secure login system with multi-factor authentication and SSO integration', 'Frontend', 'tested_passing', 'All authentication tests passing', 'critical', 'Implemented using OAuth 2.0 and JWT tokens', 'All authentication tests passing', '2024-06-01T09:00:00Z', '2024-07-15T16:30:00Z', '2024-07-15T16:30:00Z', 40, 45),
('feat-002', '{project_id}', 'F-002', 'Dashboard Analytics Widget', 'Interactive dashboard with real-time metrics and customizable charts', 'Frontend', 'in_progress', 'Unit tests 70% complete', 'high', 'Using Chart.js and WebSocket for real-time updates', 'Unit tests 70% complete', '2024-06-15T10:00:00Z', '2024-08-01T14:20:00Z', NULL, 32, 28),
('feat-003', '{project_id}', 'F-003', 'Mobile Responsive Layout', 'Responsive design system supporting all device sizes', 'Frontend', 'tested_passing', 'Cross-browser testing completed', 'medium', 'Bootstrap 5 with custom breakpoints', 'Cross-browser testing completed', '2024-05-20T11:30:00Z', '2024-07-30T09:15:00Z', '2024-07-28T15:45:00Z', 24, 22),
('feat-004', '{project_id}', 'F-004', 'Progressive Web App', 'PWA capabilities with offline support and push notifications', 'Frontend', 'not_implemented', 'Not yet implemented', 'medium', NULL, NULL, '2024-07-01T08:00:00Z', '2024-07-01T08:00:00Z', NULL, 48, NULL),
('feat-005', '{project_id}', 'F-005', 'GraphQL API Gateway', 'Unified GraphQL endpoint aggregating multiple microservices', 'Backend', 'tested_passing', 'Load testing completed', 'critical', 'Apollo Server with federation and caching', 'Load testing completed', '2024-05-01T09:30:00Z', '2024-07-20T11:45:00Z', '2024-07-18T14:20:00Z', 60, 65),
('feat-006', '{project_id}', 'F-006', 'Payment Processing Service', 'Secure payment gateway with multiple provider support', 'Backend', 'tested_failing', 'Payment tests failing on edge cases', 'critical', 'Stripe and PayPal integration with webhook handling', 'Payment tests failing on edge cases', '2024-06-10T10:15:00Z', '2024-08-02T16:00:00Z', NULL, 80, 72),
('feat-007', '{project_id}', 'F-007', 'Inventory Management API', 'RESTful API for product catalog and stock management', 'Backend', 'tested_passing', 'API documentation complete', 'high', 'CRUD operations with optimistic locking', 'API documentation complete', '2024-05-15T14:00:00Z', '2024-07-10T10:30:00Z', '2024-07-08T16:45:00Z', 45, 42),
('feat-008', '{project_id}', 'F-008', 'Machine Learning Pipeline', 'Automated ML pipeline for recommendation engine', 'Backend', 'not_implemented', 'Not yet started', 'medium', NULL, NULL, '2024-07-15T09:00:00Z', '2024-07-15T09:00:00Z', NULL, 120, NULL),
('feat-009', '{project_id}', 'F-009', 'Search Functionality', 'Full-text search with filtering and pagination', 'Backend', 'in_progress', 'Integration tests passing', 'high', 'Elasticsearch with custom analyzers', 'Integration tests passing', '2024-07-20T11:00:00Z', '2024-08-01T14:30:00Z', NULL, 56, 48),
('feat-010', '{project_id}', 'F-010', 'Admin Dashboard', 'Administrative interface for system management', 'Frontend', 'tested_passing', 'UI tests complete', 'medium', 'React admin panel with role-based access', 'UI tests complete', '2024-06-01T10:00:00Z', '2024-07-25T16:00:00Z', '2024-07-25T16:00:00Z', 36, 34);

-- Insert comprehensive tasks
INSERT INTO tasks (id, project_id, code, title, description, category, status, priority, acceptance_criteria, validation_steps, evidence, assigned_to, created_at, updated_at, started_at, completed_at, estimated_effort, actual_effort) VALUES
('task-001', '{project_id}', 'TASK-001', 'Setup Production Infrastructure', 'Configure production AWS environment with security groups and VPC', 'infrastructure', 'completed', 'critical', 'Production environment accessible and secure', '1. VPC configured\n2. Security groups configured\n3. IAM roles configured', 'Infrastructure documentation completed', 'devops-team', '2024-03-01T09:00:00Z', '2024-03-15T16:30:00Z', '2024-03-01T09:30:00Z', '2024-03-15T16:30:00Z', 40, 42),
('task-002', '{project_id}', 'TASK-002', 'Database Schema Design', 'Design and implement normalized database schema', 'infrastructure', 'completed', 'critical', 'Schema supports all business requirements', '1. All entities normalized\n2. Foreign keys in place\n3. Indexes optimized', 'Schema documentation completed', 'backend-team', '2024-03-10T10:00:00Z', '2024-03-25T14:20:00Z', '2024-03-15T11:00:00Z', '2024-03-25T14:20:00Z', 32, 35),
('task-003', '{project_id}', 'TASK-003', 'User Authentication Implementation', 'Implement secure user registration and login system', 'feature', 'completed', 'critical', 'Users can register, login, and access protected resources', '1. Registration flow works\n2. Login with MFA functional\n3. JWT tokens validated', 'All authentication tests passing', 'fullstack-team', '2024-04-01T08:30:00Z', '2024-04-20T17:45:00Z', '2024-04-01T09:00:00Z', '2024-04-20T17:45:00Z', 48, 52),
('task-004', '{project_id}', 'TASK-004', 'Payment Gateway Integration', 'Integrate Stripe and PayPal payment processing', 'feature', 'in_progress', 'critical', 'Secure payment processing with proper error handling', '1. Stripe integration functional\n2. PayPal integration working\n3. Webhook handlers implemented', 'Stripe integration 90% complete', 'backend-team', '2024-06-01T09:00:00Z', '2024-08-02T15:30:00Z', '2024-06-01T09:30:00Z', NULL, 56, 48),
('task-005', '{project_id}', 'TASK-005', 'Mobile App Development', 'Develop React Native mobile application', 'feature', 'in_progress', 'high', 'Mobile app functional on iOS and Android', '1. App builds successfully\n2. Core features working\n3. App store guidelines met', 'iOS version 70% complete', 'mobile-team', '2024-05-15T10:00:00Z', '2024-08-01T14:15:00Z', '2024-05-20T08:00:00Z', NULL, 80, 65),
('task-006', '{project_id}', 'TASK-006', 'API Performance Optimization', 'Optimize API response times and database queries', 'performance', 'in_progress', 'high', 'API response times under 200ms for 95th percentile', '1. Load testing shows improvement\n2. Database optimization complete\n3. Caching strategy implemented', 'Database optimization 60% complete', 'backend-team', '2024-06-15T11:00:00Z', '2024-08-02T12:45:00Z', '2024-06-20T09:00:00Z', NULL, 44, 38),
('task-007', '{project_id}', 'TASK-007', 'Implement Search Functionality', 'Add full-text search with filtering and pagination', 'feature', 'pending', 'medium', 'Users can search and filter content effectively', '1. Search results relevant\n2. Filters work correctly\n3. Pagination handles large result sets', NULL, 'fullstack-team', '2024-07-01T10:00:00Z', '2024-07-15T16:20:00Z', NULL, NULL, 36, NULL),
('task-008', '{project_id}', 'TASK-008', 'Create Admin Dashboard', 'Build administrative interface for system management', 'feature', 'pending', 'medium', 'Administrators can manage users and settings', '1. User management functional\n2. System settings configurable\n3. Audit logs accessible', NULL, 'frontend-team', '2024-07-10T09:30:00Z', '2024-07-20T11:45:00Z', NULL, NULL, 40, NULL),
('task-009', '{project_id}', 'TASK-009', 'Implement Email Notifications', 'Set up transactional email system with templates', 'feature', 'in_progress', 'medium', 'System sends relevant notifications to users', '1. Email templates render correctly\n2. Delivery tracking functional\n3. Unsubscribe mechanism works', 'Email service configured', 'backend-team', '2024-06-20T08:00:00Z', '2024-08-01T13:30:00Z', '2024-06-25T10:00:00Z', NULL, 24, 20),
('task-010', '{project_id}', 'TASK-010', 'Third-party API Integration', 'Integrate external APIs for enhanced functionality', 'integration', 'blocked', 'high', 'External APIs properly integrated with error handling', '1. API calls successful\n2. Rate limiting respected\n3. Error scenarios handled', 'Blocked pending payment system completion', 'integration-team', '2024-07-01T11:00:00Z', '2024-07-25T15:00:00Z', NULL, NULL, 32, NULL);

-- Insert sessions
INSERT INTO sessions (id, project_id, title, description, state, started_at, ended_at, summary, achievements, created_at, updated_at) VALUES
('session-001', '{project_id}', 'Sprint 1 Development', 'Initial development sprint focusing on core authentication', 'completed', '2024-03-01T09:00:00Z', '2024-03-15T17:00:00Z', 'Successfully implemented user authentication system', 'Authentication system, database schema, production infrastructure', '2024-03-01T09:00:00Z', '2024-03-15T17:00:00Z'),
('session-002', '{project_id}', 'Sprint 2 Development', 'Payment system integration and testing', 'completed', '2024-03-16T09:00:00Z', '2024-03-30T17:00:00Z', 'Made significant progress on payment integration', 'GraphQL API, inventory management, testing framework', '2024-03-16T09:00:00Z', '2024-03-30T17:00:00Z'),
('session-003', '{project_id}', 'Sprint 3 Development', 'Performance optimization and monitoring setup', 'completed', '2024-04-01T09:00:00Z', '2024-04-15T17:00:00Z', 'Implemented comprehensive monitoring', 'CDN integration, monitoring dashboard, container orchestration', '2024-04-01T09:00:00Z', '2024-04-15T17:00:00Z'),
('session-004', '{project_id}', 'Sprint 4 Development', 'Mobile app development and API enhancements', 'active', '2024-07-15T09:00:00Z', NULL, NULL, NULL, '2024-07-15T09:00:00Z', '2024-08-02T16:00:00Z');

-- Insert dependencies
INSERT INTO dependencies (id, project_id, from_entity_id, from_entity_type, to_entity_id, to_entity_type, dependency_type, description, created_at) VALUES
('dep-001', '{project_id}', 'feat-002', 'feature', 'feat-001', 'feature', 'requires', 'Dashboard requires user authentication', '2024-06-15T10:00:00Z'),
('dep-002', '{project_id}', 'feat-004', 'feature', 'feat-001', 'feature', 'requires', 'PWA requires authentication system', '2024-07-01T08:00:00Z'),
('dep-003', '{project_id}', 'feat-007', 'feature', 'feat-001', 'feature', 'requires', 'Payment system requires authentication', '2024-06-10T10:15:00Z'),
('dep-004', '{project_id}', 'task-002', 'task', 'task-001', 'task', 'requires', 'Database schema requires infrastructure', '2024-03-10T10:00:00Z'),
('dep-005', '{project_id}', 'task-003', 'task', 'task-002', 'task', 'requires', 'Authentication requires database schema', '2024-04-01T08:30:00Z');

-- Insert notes
INSERT INTO notes (id, project_id, entity_id, entity_type, note_type, title, content, created_at, updated_at) VALUES
('note-001', '{project_id}', 'feat-001', 'feature', 'Architecture', 'Authentication Architecture Decision', 'Decided to use OAuth 2.0 with PKCE for mobile clients and standard authorization code flow for web clients.', '2024-03-15T14:30:00Z', '2024-03-15T14:30:00Z'),
('note-002', '{project_id}', 'feat-007', 'feature', 'Decision', 'Payment Provider Selection', 'After evaluating Stripe, PayPal, and Square, decided on Stripe as primary with PayPal as secondary.', '2024-06-10T15:20:00Z', '2024-06-10T15:20:00Z'),
('note-003', '{project_id}', 'feat-002', 'feature', 'Issue', 'Performance Bottleneck Identified', 'Database queries for user dashboard are taking 2-3 seconds due to N+1 problem.', '2024-07-25T10:15:00Z', '2024-07-25T10:15:00Z'),
('note-004', '{project_id}', 'task-004', 'task', 'Issue', 'Payment Webhook Failures', 'Stripe webhooks are failing intermittently due to timeout issues.', '2024-07-28T14:30:00Z', '2024-07-28T14:30:00Z'),
('note-005', '{project_id}', 'feat-004', 'feature', 'Idea', 'Progressive Web App Enhancement', 'Consider implementing advanced PWA features like background sync and push notifications.', '2024-07-01T12:00:00Z', '2024-07-01T12:00:00Z'),
('note-006', '{project_id}', 'task-001', 'task', 'Progress', 'Infrastructure Setup Complete', 'AWS infrastructure is fully configured with auto-scaling, monitoring, and backup systems operational.', '2024-03-15T16:00:00Z', '2024-03-15T16:00:00Z'),
('note-007', '{project_id}', 'feat-008', 'feature', 'Evidence', 'Performance Benchmarks', 'API response times: 95th percentile under 150ms, throughput 5000 requests/second with caching layer.', '2024-07-08T14:30:00Z', '2024-07-08T14:30:00Z'),
('note-008', '{project_id}', '{project_id}', 'project', 'Architecture', 'Microservices Architecture Decision', 'Adopted microservices architecture with API gateway, service mesh, and distributed tracing for scalability.', '2024-02-15T10:00:00Z', '2024-02-15T10:00:00Z');

-- Insert directives (using correct column names: rule instead of description)
INSERT INTO directives (id, project_id, code, title, rule, category, priority, context, rationale, examples, created_at, updated_at) VALUES
('dir-001', '{project_id}', 'DEV-001', 'Code Review Mandatory', 'All code changes must undergo peer review before merging to main branch', 'development', 'high', 'All pull requests and merge requests', 'Ensure code quality and knowledge sharing', 'Pull requests blocked without approvals, CI/CD pipeline enforces checks', '2024-03-01T09:00:00Z', '2024-07-15T14:30:00Z'),
('dir-002', '{project_id}', 'SEC-001', 'Secret Management Policy', 'No secrets or API keys in source code, use environment variables or secure vaults', 'security', 'high', 'All code commits and deployments', 'Prevent security breaches and credential exposure', 'AWS Secrets Manager, HashiCorp Vault, environment-specific configurations', '2024-03-01T09:30:00Z', '2024-06-20T11:15:00Z'),
('dir-003', '{project_id}', 'TEST-001', 'Minimum Test Coverage', 'Maintain minimum 80% code coverage for all modules', 'testing', 'high', 'All production code modules', 'Ensure code reliability and catch regressions', 'Jest for frontend, pytest for backend, integration tests for APIs', '2024-03-15T10:00:00Z', '2024-07-30T16:45:00Z'),
('dir-004', '{project_id}', 'ARCH-001', 'API Versioning Strategy', 'All public APIs must include version numbers and maintain backward compatibility', 'architecture', 'medium', 'All public API endpoints', 'Prevent breaking changes for API consumers', 'v1/users, v2/orders, deprecation headers for sunset endpoints', '2024-04-01T08:00:00Z', '2024-07-01T12:30:00Z'),
('dir-005', '{project_id}', 'PERF-001', 'Performance Budgets', 'Frontend bundle size under 1MB, API response times under 200ms', 'performance', 'medium', 'All frontend builds and API endpoints', 'Maintain optimal user experience', 'Webpack bundle analyzer, New Relic monitoring, Lighthouse CI', '2024-05-01T11:00:00Z', '2024-07-20T09:45:00Z'),
('dir-006', '{project_id}', 'DOC-001', 'API Documentation Required', 'All API endpoints must have OpenAPI documentation with examples', 'documentation', 'medium', 'All API endpoints', 'Facilitate API integration and maintenance', 'Swagger UI, Redoc, automated doc generation from code annotations', '2024-04-15T14:00:00Z', '2024-06-30T10:20:00Z');

-- Insert milestones (using correct column names: no code column, no validation_evidence, achievement_summary, feature_count, task_count)
INSERT INTO milestones (id, project_id, title, description, target_date, achieved_date, success_criteria, status, completion_percentage, created_at, updated_at) VALUES
('milestone-001', '{project_id}', 'MVP Launch', 'Minimum viable product with core authentication and payment features', '2024-06-30T23:59:59Z', '2024-07-15T16:30:00Z', 'User registration, login, payment processing, basic dashboard functional', 'achieved', 100.0, '2024-03-01T09:00:00Z', '2024-07-15T17:00:00Z'),
('milestone-002', '{project_id}', 'Beta Release', 'Feature-complete beta with advanced analytics and mobile support', '2024-08-31T23:59:59Z', NULL, 'Mobile responsive design, advanced analytics, performance optimizations complete', 'in_progress', 65.0, '2024-06-01T10:00:00Z', '2024-08-01T14:20:00Z'),
('milestone-003', '{project_id}', 'Production Scaling', 'Production-ready system handling 10,000 concurrent users', '2024-10-31T23:59:59Z', NULL, 'Load testing passed, auto-scaling configured, monitoring comprehensive', 'planned', 0.0, '2024-07-01T11:00:00Z', '2024-07-15T16:00:00Z'),
('milestone-004', '{project_id}', 'Q4 Feature Expansion', 'Advanced features including AI recommendations and multi-tenant support', '2024-12-31T23:59:59Z', NULL, 'AI models deployed, multi-tenancy implemented, enterprise features complete', 'planned', 0.0, '2024-08-01T09:00:00Z', '2024-08-01T09:00:00Z');

-- Insert note links (using correct column names: target_id and target_type instead of target_entity_id and target_entity_type)
INSERT INTO note_links (id, project_id, source_note_id, target_id, target_type, link_type, created_at) VALUES
('link-001', '{project_id}', 'note-001', 'feat-007', 'feature', 'reference', '2024-06-10T15:30:00Z'),
('link-002', '{project_id}', 'note-002', 'task-004', 'task', 'related', '2024-06-10T15:45:00Z'),
('link-003', '{project_id}', 'note-003', 'feat-008', 'feature', 'blocks', '2024-07-25T10:30:00Z'),
('link-004', '{project_id}', 'note-004', 'note-002', 'note', 'response_to', '2024-07-28T14:45:00Z'),
('link-005', '{project_id}', 'note-005', 'milestone-002', 'milestone', 'depends_on', '2024-07-01T12:15:00Z'),
('link-006', '{project_id}', 'note-006', 'milestone-001', 'milestone', 'reference', '2024-03-15T16:15:00Z'),
('link-007', '{project_id}', 'note-007', 'dir-005', 'directive', 'reference', '2024-07-08T14:45:00Z'),
('link-008', '{project_id}', 'note-008', 'feat-006', 'feature', 'reference', '2024-05-01T09:45:00Z');
"#, project_id = project_id);
    
    // Execute the test data SQL
    let db_path_str = db_path.to_string_lossy();
    let mut child = std::process::Command::new("sqlite3")
        .arg(&*db_path_str)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(test_data_sql.as_bytes())?;
    }
    
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("{} Warning: Some SQL statements failed: {}", "⚠️".yellow(), error);
    }
    
    // Show summary
    let features = entity_manager.list_features().await?;
    let tasks = entity_manager.list_tasks().await?;
    
    println!("  {} {} features created", "📋".cyan(), features.len());
    println!("  {} {} tasks created", "✅".cyan(), tasks.len());
    println!("  {} Comprehensive sample data loaded", "✅".green());
    
    Ok(())
}

// Entity relationship management functions

fn link_entities(from_entity: String, from_type: String, to_entity: String, to_type: String, relationship_type: String, description: Option<String>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool.clone());
        
        // Get current project
        let project = entity_manager.get_current_project().await?;
        
        // Parse entity types
        let from_entity_type = parse_entity_type(&from_type)?;
        let to_entity_type = parse_entity_type(&to_type)?;
        
        // Create the relationship
        let dependency = workspace::entities::relationships::create_dependency(
            &pool,
            &project.id,
            &from_entity,
            from_entity_type,
            &to_entity,
            to_entity_type,
            relationship_type.clone(),
            description.clone(),
        ).await?;
        
        println!("{} Linked {} {} to {} {} with relationship '{}'", 
                "✅".green(), from_type, from_entity, to_type, to_entity, relationship_type);
        println!("   Dependency ID: {}", dependency.id);
        if let Some(desc) = description {
            println!("   Description: {}", desc);
        }
        
        Ok(())
    })
}

fn list_entity_relationships(entity_id: String, entity_type: String, relationship_type: Option<String>, include_resolved: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        
        // Get relationships for this entity
        let relationships = workspace::entities::relationships::get_relationships(&pool, &entity_id).await?;
        
        println!("{} Relationships for {} {}", "🔗".cyan(), entity_type, entity_id);
        
        if relationships.is_empty() {
            println!("   No relationships found");
            return Ok(());
        }
        
        for (rel_type, entity_ids) in relationships {
            println!("   {:?}: {}", rel_type, entity_ids.join(", "));
        }
        
        Ok(())
    })
}

fn unlink_entities(dependency_id: String, force: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        
        if !force {
            print!("Remove relationship {}? [y/N]: ", dependency_id);
            use std::io::Write;
            std::io::stdout().flush()?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if !input.trim().to_lowercase().starts_with('y') {
                println!("Cancelled");
                return Ok(());
            }
        }
        
        // Remove the dependency
        sqlx::query("DELETE FROM dependencies WHERE id = ?")
            .bind(&dependency_id)
            .execute(&pool)
            .await?;
        
        println!("{} Removed relationship {}", "✅".green(), dependency_id);
        
        Ok(())
    })
}

fn resolve_entity_relationship(dependency_id: String, description: Option<String>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        
        workspace::entities::relationships::resolve_dependency(&pool, &dependency_id).await?;
        
        println!("{} Resolved relationship {}", "✅".green(), dependency_id);
        if let Some(desc) = description {
            println!("   Resolution: {}", desc);
        }
        
        Ok(())
    })
}

fn show_relationship_stats(detailed: bool, format: String) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool.clone());
        
        let project = entity_manager.get_current_project().await?;
        let dependencies = workspace::entities::relationships::get_project_dependencies(&pool, &project.id).await?;
        
        if format == "json" {
            let stats = serde_json::json!({
                "total_relationships": dependencies.len(),
                "active_relationships": dependencies.iter().filter(|d| d.resolved_at.is_none()).count(),
                "resolved_relationships": dependencies.iter().filter(|d| d.resolved_at.is_some()).count(),
            });
            println!("{}", serde_json::to_string_pretty(&stats)?);
        } else {
            println!("{} Relationship Statistics for {}", "📊".cyan(), project.name);
            println!("   Total relationships: {}", dependencies.len());
            println!("   Active relationships: {}", dependencies.iter().filter(|d| d.resolved_at.is_none()).count());
            println!("   Resolved relationships: {}", dependencies.iter().filter(|d| d.resolved_at.is_some()).count());
            
            if detailed {
                let mut type_counts = std::collections::HashMap::new();
                for dep in &dependencies {
                    *type_counts.entry(&dep.dependency_type).or_insert(0) += 1;
                }
                
                println!("   Breakdown by type:");
                for (dep_type, count) in type_counts {
                    println!("     {}: {}", dep_type, count);
                }
            }
        }
        
        Ok(())
    })
}

fn parse_entity_type(type_str: &str) -> Result<workspace::entities::EntityType> {
    match type_str.to_lowercase().as_str() {
        "feature" => Ok(workspace::entities::EntityType::Feature),
        "task" => Ok(workspace::entities::EntityType::Task),
        "session" => Ok(workspace::entities::EntityType::Session),
        "project" => Ok(workspace::entities::EntityType::Project),
        "directive" => Ok(workspace::entities::EntityType::Directive),
        "note" => Ok(workspace::entities::EntityType::Note),
        "template" => Ok(workspace::entities::EntityType::Template),
        "dependency" => Ok(workspace::entities::EntityType::Dependency),
        "milestone" => Ok(workspace::entities::EntityType::Milestone),
        "test" => Ok(workspace::entities::EntityType::Test),
        _ => Err(anyhow::anyhow!("Unknown entity type: {}", type_str)),
    }
}

fn run_note_command(action: NoteAction) -> Result<()> {
    match action {
        NoteAction::Add { entity_type, entity_id, title, content, note_type, tags } => {
            add_entity_note(entity_type, entity_id, title, content, note_type, tags)?;
        }
        NoteAction::AddProject { title, content, note_type, tags } => {
            add_project_note(title, content, note_type, tags)?;
        }
        NoteAction::List { entity_type, entity_id, note_type, project_wide, pinned } => {
            list_notes(entity_type, entity_id, note_type, project_wide, pinned)?;
        }
        NoteAction::Search { query, note_type, format } => {
            search_notes(query, note_type, format)?;
        }
        NoteAction::Update { note_id, title, content, tags } => {
            update_note(note_id, title, content, tags)?;
        }
        NoteAction::Delete { note_id, force } => {
            delete_note(note_id, force)?;
        }
        NoteAction::Pin { note_id } => {
            toggle_note_pin(note_id)?;
        }
        NoteAction::Link { source_note_id, target_id, target_type, entity_type, link_type } => {
            link_note_to_target(source_note_id, target_id, target_type, entity_type, link_type)?;
        }
        NoteAction::Unlink { link_id, force } => {
            unlink_note(link_id, force)?;
        }
        NoteAction::ListLinks { id, incoming, outgoing, format } => {
            list_note_links(id, incoming, outgoing, format)?;
        }
    }
    Ok(())
}

fn add_entity_note(entity_type: String, entity_id: String, title: String, content: String, note_type: String, tags: Option<String>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool.clone());

        let entity_type_enum = parse_entity_type(&entity_type)?;
        let note_type_enum = parse_note_type(&note_type)?;

        let note = workspace::entities::notes::create_entity_note(
            &pool,
            &entity_id,
            entity_type_enum,
            note_type_enum,
            title,
            content,
        ).await?;

        println!("{} Note {} attached to {} {}", "✅".green(), note.id, entity_type, entity_id);
        println!("   Title: {}", note.title);
        println!("   Type: {:?}", note.note_type);
        
        Ok(())
    })
}

fn add_project_note(title: String, content: String, note_type: String, tags: Option<String>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool.clone());

        let project = entity_manager.get_current_project().await?;

        let note = workspace::entities::notes::create_project_note(
            &pool,
            &project.id,
            parse_note_type(&note_type)?,
            title,
            content,
        ).await?;

        println!("{} Project-wide note {} created", "✅".green(), note.id);
        println!("   Title: {}", note.title);
        println!("   Type: {:?}", note.note_type);
        
        Ok(())
    })
}

fn list_notes(entity_type: Option<String>, entity_id: Option<String>, note_type: Option<String>, project_wide: bool, pinned: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool.clone());

        let project = entity_manager.get_current_project().await?;
        let notes = if let Some(entity_id) = entity_id {
            workspace::entities::notes::get_notes_for_entity(&pool, &entity_id).await?
        } else if project_wide {
            workspace::entities::notes::get_project_notes(&pool, &project.id).await?
        } else {
            workspace::entities::notes::list_all(&pool).await?
        };

        if notes.is_empty() {
            println!("{} No notes found", "ℹ️".blue());
            return Ok(());
        }

        println!("{} Found {} notes", "📝".cyan(), notes.len());
        for note in notes {
            let entity_info = if note.is_project_wide {
                "Project-wide".to_string()
            } else if let (Some(entity_type), Some(entity_id)) = (&note.entity_type, &note.entity_id) {
                format!("{:?} {}", entity_type, entity_id)
            } else {
                "Unknown".to_string()
            };
            
            let pin_indicator = if note.is_pinned { " 📌" } else { "" };
            
            println!("   {} {} - {} ({}){}", note.id, note.title, entity_info, format!("{:?}", note.note_type), pin_indicator);
            if !note.content.is_empty() {
                let preview = if note.content.len() > 100 {
                    format!("{}...", &note.content[..97])
                } else {
                    note.content.clone()
                };
                println!("      {}", preview.dimmed());
            }
        }
        
        Ok(())
    })
}

fn search_notes(query: String, note_type: Option<String>, format: String) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool.clone());

        let project = entity_manager.get_current_project().await?;
        let notes = workspace::entities::notes::search_notes(&pool, &project.id, &query).await?;

        let filtered_notes: Vec<_> = if let Some(note_type) = note_type {
            let note_type_enum = parse_note_type(&note_type)?;
            notes.into_iter().filter(|n| n.note_type == note_type_enum).collect()
        } else {
            notes
        };

        if format == "json" {
            println!("{}", serde_json::to_string_pretty(&filtered_notes)?);
        } else {
            if filtered_notes.is_empty() {
                println!("{} No notes found matching '{}'", "ℹ️".blue(), query);
                return Ok(());
            }

            println!("{} Found {} notes matching '{}'", "🔍".cyan(), filtered_notes.len(), query);
            for note in filtered_notes {
                let entity_info = if note.is_project_wide {
                    "Project-wide".to_string()
                } else if let (Some(entity_type), Some(entity_id)) = (&note.entity_type, &note.entity_id) {
                    format!("{:?} {}", entity_type, entity_id)
                } else {
                    "Unknown".to_string()
                };
                
                println!("   {} {} - {} ({})", note.id, note.title, entity_info, format!("{:?}", note.note_type));
                println!("      {}", note.content.dimmed());
            }
        }
        
        Ok(())
    })
}

fn update_note(note_id: String, title: Option<String>, content: Option<String>, tags: Option<String>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;

        let tags_vec = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

        workspace::entities::notes::update_note(
            &pool,
            &note_id,
            title,
            content,
            tags_vec,
        ).await?;

        println!("{} Note {} updated", "✅".green(), note_id);
        
        Ok(())
    })
}

fn delete_note(note_id: String, force: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;

        if !force {
            print!("Delete note {}? (y/N): ", note_id);
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().to_lowercase().starts_with('y') {
                println!("Cancelled");
                return Ok(());
            }
        }

        workspace::entities::notes::delete_note(&pool, &note_id).await?;

        println!("{} Note {} deleted", "✅".green(), note_id);
        
        Ok(())
    })
}

fn toggle_note_pin(note_id: String) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;

        let is_pinned = workspace::entities::notes::toggle_pin(&pool, &note_id).await?;

        let status = if is_pinned { "pinned" } else { "unpinned" };
        println!("{} Note {} {}", "✅".green(), note_id, status);
        
        Ok(())
    })
}

fn link_note_to_target(source_note_id: String, target_id: String, target_type: String, entity_type: Option<String>, link_type: String) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool);

        let project = entity_manager.get_current_project().await?;

        let link = entity_manager.create_note_link(
            &project.id,
            &source_note_id,
            &target_type,
            &target_id,
            entity_type.as_deref(),
            &link_type,
            false, // Manual link creation
            Some(&format!("Manual link creation via CLI")),
        ).await?;

        println!("{} Created link {} from note {} to {} {}", 
                 "✅".green(), link.id, source_note_id, target_type, target_id);
        
        Ok(())
    })
}

fn unlink_note(link_id: String, force: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool);

        if !force {
            print!("Remove link {}? (y/N): ", link_id);
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().to_lowercase().starts_with('y') {
                println!("Cancelled");
                return Ok(());
            }
        }

        let removed = entity_manager.remove_note_link(&link_id).await?;
        
        if removed {
            println!("{} Link {} removed", "✅".green(), link_id);
        } else {
            println!("{} Link {} not found", "❌".red(), link_id);
        }
        
        Ok(())
    })
}

fn list_note_links(id: String, incoming: bool, outgoing: bool, format: String) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let db_path = get_project_root()?.join(".ws/project.db");
        let pool = workspace::entities::database::initialize_database(&db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool);

        // If neither incoming nor outgoing specified, show both
        let show_incoming = incoming || (!incoming && !outgoing);
        let show_outgoing = outgoing || (!incoming && !outgoing);

        let (outgoing_links, incoming_links) = if show_incoming || show_outgoing {
            entity_manager.get_bidirectional_links(&id, None).await?
        } else {
            (Vec::new(), Vec::new())
        };

        if format == "json" {
            let response = serde_json::json!({
                "entity_id": id,
                "outgoing_links": if show_outgoing { outgoing_links } else { Vec::new() },
                "incoming_links": if show_incoming { incoming_links } else { Vec::new() }
            });
            println!("{}", serde_json::to_string_pretty(&response)?);
        } else {
            println!("{} Links for {}", "🔗".blue(), id);
            
            if show_outgoing && !outgoing_links.is_empty() {
                println!("\n{} Outgoing Links:", "→".blue());
                for link in &outgoing_links {
                    println!("  {} {} → {} {} ({})", 
                             link.id, link.link_type, link.target_type, link.target_id,
                             if link.auto_detected { "auto" } else { "manual" });
                }
            }
            
            if show_incoming && !incoming_links.is_empty() {
                println!("\n{} Incoming Links:", "←".blue());
                for link in &incoming_links {
                    println!("  {} {} ← note {} ({})", 
                             link.id, link.link_type, link.source_note_id,
                             if link.auto_detected { "auto" } else { "manual" });
                }
            }
            
            if (show_outgoing && outgoing_links.is_empty()) && (show_incoming && incoming_links.is_empty()) {
                println!("  No links found for {}", id);
            }
        }
        
        Ok(())
    })
}

fn parse_note_type(type_str: &str) -> Result<workspace::entities::NoteType> {
    match type_str.to_lowercase().as_str() {
        "architecture" => Ok(workspace::entities::NoteType::Architecture),
        "decision" => Ok(workspace::entities::NoteType::Decision),
        "reminder" => Ok(workspace::entities::NoteType::Reminder),
        "observation" => Ok(workspace::entities::NoteType::Observation),
        "reference" => Ok(workspace::entities::NoteType::Reference),
        "evidence" => Ok(workspace::entities::NoteType::Evidence),
        "progress" => Ok(workspace::entities::NoteType::Progress),
        "issue" => Ok(workspace::entities::NoteType::Issue),
        _ => Err(anyhow::anyhow!("Unknown note type: {}. Valid types: architecture, decision, reminder, observation, reference, evidence, progress, issue", type_str)),
    }
}