use anyhow::Result;
use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        rolling_file::{
            policy::compound::{
                roll::fixed_window::FixedWindowRoller,
                trigger::size::SizeTrigger,
                CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use std::path::{Path, PathBuf};

/// Initialize the logging system with rotation and archiving
pub fn init_logging(workspace_root: &Path, debug_mode: bool) -> Result<()> {
    let ws_dir = workspace_root.join(".ws");
    std::fs::create_dir_all(&ws_dir)?;
    
    let log_dir = ws_dir.join("logs");
    std::fs::create_dir_all(&log_dir)?;
    
    let log_file = log_dir.join("ws.log");
    let archive_pattern = log_dir.join("ws.{}.log");
    
    // Log pattern with timestamp, level, target, and message
    let log_pattern = "[{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l:5.5})} {t}] {m}{n}";
    
    // Console appender for errors and warnings only (unless debug mode)
    let console_level = if debug_mode { 
        LevelFilter::Debug 
    } else { 
        LevelFilter::Warn 
    };
    
    let console = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new("{h({l:5.5})}: {m}{n}")))
        .build();
    
    // Rolling file appender with size-based rotation
    let file_appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_pattern)))
        .build(
            log_file,
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(10 * 1024 * 1024)), // 10MB per file
                Box::new(
                    FixedWindowRoller::builder()
                        .build(&archive_pattern.to_string_lossy(), 10)? // Keep 10 archived files
                ),
            )),
        )?;
    
    // Build configuration
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(console_level)))
                .build("console", Box::new(console)),
        )
        .appender(
            Appender::builder()
                .build("file", Box::new(file_appender)),
        )
        .logger(
            Logger::builder()
                .appender("file")
                .appender("console")
                .build("workspace", LevelFilter::Debug),
        )
        .logger(
            Logger::builder()
                .appender("file")
                .build("sqlx", LevelFilter::Warn), // Reduce SQL query noise
        )
        .build(
            Root::builder()
                .appender("file")
                .appender("console")
                .build(LevelFilter::Info),
        )?;
    
    log4rs::init_config(config)?;
    
    log::info!("Logging initialized: {}", log_dir.join("ws.log").display());
    log::debug!("Debug logging enabled, console level: {:?}", console_level);
    
    Ok(())
}

/// Initialize simple logging fallback if workspace detection fails
pub fn init_simple_logging(debug_mode: bool) -> Result<()> {
    let level = if debug_mode {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(level)
        .format_timestamp_secs()
        .init();
    
    log::warn!("Using simple console logging (workspace directory not detected)");
    
    Ok(())
}

/// Get the current workspace root directory
pub fn detect_workspace_root() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    
    // Look for .git, .ws, or common project files
    let mut dir = current_dir.as_path();
    loop {
        if dir.join(".git").exists() 
            || dir.join(".ws").exists()
            || dir.join("Cargo.toml").exists()
            || dir.join("package.json").exists() {
            return Some(dir.to_path_buf());
        }
        
        dir = dir.parent()?;
    }
}

/// Initialize logging with automatic workspace detection
pub fn setup_logging(debug_mode: bool) -> Result<()> {
    if let Some(workspace_root) = detect_workspace_root() {
        init_logging(&workspace_root, debug_mode)
    } else {
        init_simple_logging(debug_mode)
    }
}

/// Log an operation start
pub fn log_operation_start(operation: &str, details: &str) {
    log::info!("Starting {}: {}", operation, details);
}

/// Log an operation completion
pub fn log_operation_complete(operation: &str, duration: std::time::Duration) {
    log::info!("Completed {} in {:.2}s", operation, duration.as_secs_f64());
}

/// Log an operation failure
pub fn log_operation_error(operation: &str, error: &anyhow::Error) {
    log::error!("Failed {}: {}", operation, error);
    
    // Log error chain at debug level
    let mut cause = error.source();
    while let Some(err) = cause {
        log::debug!("  Caused by: {}", err);
        cause = err.source();
    }
}

/// Log a warning with context
pub fn log_warning(context: &str, message: &str) {
    log::warn!("{}: {}", context, message);
}

/// Log file operation
pub fn log_file_operation(operation: &str, path: &Path, result: Result<(), &anyhow::Error>) {
    match result {
        Ok(()) => log::debug!("File {}: {}", operation, path.display()),
        Err(e) => log::error!("File {} failed for {}: {}", operation, path.display(), e),
    }
}

/// Log command execution
pub fn log_command_execution(command: &str, args: &[&str], success: bool) {
    let cmd_str = format!("{} {}", command, args.join(" "));
    if success {
        log::debug!("Command executed: {}", cmd_str);
    } else {
        log::error!("Command failed: {}", cmd_str);
    }
}

/// Log performance metrics
pub fn log_performance(operation: &str, items_processed: usize, duration: std::time::Duration) {
    let rate = items_processed as f64 / duration.as_secs_f64();
    log::info!("{}: processed {} items in {:.2}s ({:.1} items/sec)", 
               operation, items_processed, duration.as_secs_f64(), rate);
}

/// Log configuration changes
pub fn log_config_change(component: &str, setting: &str, old_value: &str, new_value: &str) {
    log::info!("Config change in {}: {} = {} -> {}", component, setting, old_value, new_value);
}

/// Log database operations
pub fn log_database_operation(operation: &str, table: &str, affected_rows: usize) {
    log::debug!("DB {}: {} rows affected in {}", operation, affected_rows, table);
}

/// Log version information at startup
pub fn log_version_info(version: &str, git_hash: Option<&str>) {
    log::info!("Workspace v{}", version);
    if let Some(hash) = git_hash {
        log::debug!("Git commit: {}", hash);
    }
}