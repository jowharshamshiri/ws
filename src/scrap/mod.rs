pub mod scrap_common;

pub use scrap_common::{ScrapMetadata, ScrapEntry};

use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

/// Run scrap command with the given arguments
pub fn run_scrap(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        // Default action: list contents
        return list_scrap_contents(None);
    }

    let mut args_iter = args.iter();
    let first_arg = args_iter.next().unwrap();

    match first_arg.as_str() {
        "list" => {
            let sort_option = if args.len() > 2 && args[1] == "--sort" {
                Some(&args[2])
            } else {
                None
            };
            list_scrap_contents(sort_option.map(|s| s.as_str()))
        }
        "clean" => {
            let days = if args.len() > 2 && args[1] == "--days" {
                args[2].parse().unwrap_or(30)
            } else {
                30
            };
            let dry_run = args.contains(&"--dry-run".to_string());
            clean_scrap_folder(days, dry_run)
        }
        "purge" => {
            let force = args.contains(&"--force".to_string());
            purge_scrap_folder(force)
        }
        "find" => {
            if args.len() < 2 {
                anyhow::bail!("Find requires a pattern argument");
            }
            let pattern = &args[1];
            let content_search = args.contains(&"--content".to_string());
            find_in_scrap(pattern, content_search)
        }
        "archive" => {
            let output = if args.len() > 2 && args[1] == "--output" {
                Some(&args[2])
            } else {
                None
            };
            let remove = args.contains(&"--remove".to_string());
            archive_scrap_folder(output.map(|s| s.as_str()), remove)
        }
        path => {
            // Treat as file path to scrap
            let path_buf = PathBuf::from(path);
            scrap_file_or_directory(&path_buf)
        }
    }
}

/// Run unscrap command with the given arguments
pub fn run_unscrap(args: Vec<String>) -> Result<()> {
    let scrap_dir = get_scrap_directory()?;
    let mut metadata = ScrapMetadata::load(&scrap_dir)?;

    if args.is_empty() {
        // Restore last scrapped item
        return restore_last_item(&mut metadata, &scrap_dir);
    }

    let mut args_iter = args.iter();
    let name = args_iter.next().unwrap();
    let mut to_path = None;
    let mut force = false;

    // Parse remaining arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--to" => {
                if i + 1 < args.len() {
                    to_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    anyhow::bail!("--to requires a path argument");
                }
            }
            "--force" => {
                force = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    restore_item(&mut metadata, &scrap_dir, name, to_path, force)
}

fn get_scrap_directory() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    Ok(current_dir.join(".scrap"))
}

fn ensure_scrap_directory() -> Result<PathBuf> {
    let scrap_dir = get_scrap_directory()?;
    if !scrap_dir.exists() {
        fs::create_dir_all(&scrap_dir)?;
        update_gitignore(&scrap_dir)?;
    }
    Ok(scrap_dir)
}

fn update_gitignore(scrap_dir: &Path) -> Result<()> {
    let gitignore_path = scrap_dir.parent().unwrap().join(".gitignore");
    let entry = ".scrap/";
    
    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if !content.contains(entry) {
            let mut new_content = content;
            if !new_content.is_empty() && !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(entry);
            new_content.push('\n');
            fs::write(&gitignore_path, new_content)?;
        }
    } else {
        fs::write(&gitignore_path, format!("{}\n", entry))?;
    }
    
    Ok(())
}

fn scrap_file_or_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    let scrap_dir = ensure_scrap_directory()?;
    let mut metadata = ScrapMetadata::load(&scrap_dir)?;

    let file_name = path.file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
        .to_string_lossy();

    // Generate unique name if file already exists in scrap
    let scrapped_name = generate_unique_name(&scrap_dir, &file_name);
    let dest_path = scrap_dir.join(&scrapped_name);

    // Move file/directory to scrap
    fs::rename(path, &dest_path)
        .with_context(|| format!("Failed to move {} to scrap", path.display()))?;

    // Update metadata
    metadata.add_entry(&scrapped_name, path.to_path_buf());
    metadata.save(&scrap_dir)?;

    println!("Moved {} to .scrap/{}", path.display(), scrapped_name);
    Ok(())
}

fn generate_unique_name(scrap_dir: &Path, base_name: &str) -> String {
    let mut name = base_name.to_string();
    let mut counter = 1;
    
    while scrap_dir.join(&name).exists() {
        if let Some(dot_pos) = base_name.rfind('.') {
            let (stem, ext) = base_name.split_at(dot_pos);
            name = format!("{}_{}{}", stem, counter, ext);
        } else {
            name = format!("{}_{}", base_name, counter);
        }
        counter += 1;
    }
    
    name
}

fn list_scrap_contents(sort_option: Option<&str>) -> Result<()> {
    let scrap_dir = get_scrap_directory()?;
    if !scrap_dir.exists() {
        fs::create_dir_all(&scrap_dir)
            .with_context(|| format!("Failed to create scrap directory: {}", scrap_dir.display()))?;
        update_gitignore(&scrap_dir)?;
        println!("Scrap folder is empty");
        return Ok(());
    }

    let metadata = ScrapMetadata::load(&scrap_dir)?;
    if metadata.entries.is_empty() {
        println!("Scrap folder is empty");
        return Ok(());
    }

    let mut entries: Vec<_> = metadata.entries.values().collect();
    
    match sort_option {
        Some("date") => entries.sort_by_key(|e| e.scrapped_at),
        Some("name") => entries.sort_by_key(|e| &e.scrapped_name),
        Some("size") => {
            // For size sorting, we'd need to get actual file sizes
            entries.sort_by_key(|e| &e.scrapped_name); // Fallback to name
        }
        _ => entries.sort_by_key(|e| e.scrapped_at),
    }

    println!("Scrapped files:");
    for entry in entries {
        println!("  {} (from {}) - {}", 
                 entry.scrapped_name, 
                 entry.original_path.display(),
                 entry.scrapped_at.format("%Y-%m-%d %H:%M:%S"));
    }

    Ok(())
}

fn clean_scrap_folder(days: u32, dry_run: bool) -> Result<()> {
    let scrap_dir = get_scrap_directory()?;
    if !scrap_dir.exists() {
        println!("No .scrap directory found");
        return Ok(());
    }

    let mut metadata = ScrapMetadata::load(&scrap_dir)?;
    let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);
    let mut removed_count = 0;

    let entries_to_remove: Vec<_> = metadata.entries.iter()
        .filter(|(_, entry)| entry.scrapped_at < cutoff_date)
        .map(|(name, _)| name.clone())
        .collect();

    for name in entries_to_remove {
        let file_path = scrap_dir.join(&name);
        if dry_run {
            println!("Would remove: {}", name);
        } else {
            if file_path.exists() {
                if file_path.is_dir() {
                    fs::remove_dir_all(&file_path)?;
                } else {
                    fs::remove_file(&file_path)?;
                }
            }
            metadata.remove_entry(&name);
            println!("Removed: {}", name);
        }
        removed_count += 1;
    }

    if !dry_run && removed_count > 0 {
        metadata.save(&scrap_dir)?;
    }

    if dry_run {
        println!("Would remove {} items older than {} days", removed_count, days);
    } else {
        println!("Removed {} items older than {} days", removed_count, days);
    }

    Ok(())
}

fn purge_scrap_folder(force: bool) -> Result<()> {
    let scrap_dir = get_scrap_directory()?;
    if !scrap_dir.exists() {
        println!("No .scrap directory found");
        return Ok(());
    }

    if !force {
        anyhow::bail!("Use --force to confirm purging all scrapped files");
    }

    // Remove all files and subdirectories in .scrap except .metadata.json
    let entries = fs::read_dir(&scrap_dir)?;
    let mut removed_count = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        
        if file_name != ".metadata.json" {
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            removed_count += 1;
        }
    }

    // Clear metadata
    let empty_metadata = ScrapMetadata::new();
    empty_metadata.save(&scrap_dir)?;

    println!("Purged {} items from scrap folder", removed_count);
    Ok(())
}

fn find_in_scrap(pattern: &str, content_search: bool) -> Result<()> {
    let scrap_dir = get_scrap_directory()?;
    if !scrap_dir.exists() {
        println!("No .scrap directory found");
        return Ok(());
    }

    let metadata = ScrapMetadata::load(&scrap_dir)?;
    let mut found_count = 0;

    for (name, entry) in &metadata.entries {
        let matches = if content_search {
            // For content search, we'd need to read file contents
            // For now, just match filename
            name.contains(pattern) || entry.original_path.to_string_lossy().contains(pattern)
        } else {
            name.contains(pattern) || entry.original_path.to_string_lossy().contains(pattern)
        };

        if matches {
            println!("{} (from {}) - {}", 
                     name, 
                     entry.original_path.display(),
                     entry.scrapped_at.format("%Y-%m-%d %H:%M:%S"));
            found_count += 1;
        }
    }

    if found_count == 0 {
        println!("No matching files found");
    } else {
        println!("Found {} matching files", found_count);
    }

    Ok(())
}

fn archive_scrap_folder(output: Option<&str>, remove: bool) -> Result<()> {
    let scrap_dir = get_scrap_directory()?;
    if !scrap_dir.exists() {
        println!("No .scrap directory found");
        return Ok(());
    }

    let archive_name = output.unwrap_or("scrap-archive.tar.gz");
    
    // Create tar.gz archive
    let tar_gz = fs::File::create(archive_name)?;
    let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Add all files from scrap directory
    tar.append_dir_all("scrap", &scrap_dir)?;
    tar.finish()?;

    println!("Created archive: {}", archive_name);

    if remove {
        purge_scrap_folder(true)?;
        println!("Removed all files from scrap folder");
    }

    Ok(())
}

fn restore_last_item(metadata: &mut ScrapMetadata, scrap_dir: &Path) -> Result<()> {
    let last_entry = metadata.entries.values()
        .max_by_key(|entry| entry.scrapped_at);

    match last_entry {
        Some(entry) => {
            let name = entry.scrapped_name.clone();
            restore_item(metadata, scrap_dir, &name, None, false)
        }
        None => {
            println!("No items in scrap folder to restore");
            Ok(())
        }
    }
}

fn restore_item(metadata: &mut ScrapMetadata, scrap_dir: &Path, name: &str, to_path: Option<PathBuf>, force: bool) -> Result<()> {
    let entry = metadata.get_entry(name)
        .ok_or_else(|| anyhow::anyhow!("Item not found in scrap: {}", name))?;

    let source_path = scrap_dir.join(name);
    let dest_path = to_path.unwrap_or_else(|| entry.original_path.clone());

    if dest_path.exists() && !force {
        anyhow::bail!("Destination already exists: {} (use --force to overwrite)", dest_path.display());
    }

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Move file back
    fs::rename(&source_path, &dest_path)
        .with_context(|| format!("Failed to restore {} to {}", name, dest_path.display()))?;

    // Remove from metadata
    metadata.remove_entry(name);
    metadata.save(scrap_dir)?;

    println!("Restored {} to {}", name, dest_path.display());
    Ok(())
}