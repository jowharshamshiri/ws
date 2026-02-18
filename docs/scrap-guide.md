---
layout: default
title: Scrap Tool Guide
---

# Scrap Tool Guide

The `scrap` tool provides a local trash can using a `.scrap` folder for files you want to delete. It's designed to safely handle files you want to remove from your workspace by moving them to a trash folder, with the ability to easily restore them later.

## Core Concepts

### The .scrap Folder
- Automatically created in your current directory
- Stores temporarily moved files and directories
- Maintains metadata about original file locations
- Automatically added to `.gitignore` if it exists

### Metadata Tracking
- Records original file paths for easy restoration
- Timestamps when files were scrapped
- Enables intelligent restoration and history tracking

## Basic Operations

### Moving Files to .scrap

```bash
# Move a single file
ws scrap temp.log

# Move multiple files (run multiple commands)
ws scrap file1.txt file2.txt old_directory/

# Move with absolute paths
ws scrap /path/to/file.txt

# Automatic conflict resolution
ws scrap file.txt  # Creates .scrap/file.txt
ws scrap file.txt  # Creates .scrap/file_1.txt (automatic rename)
```

### Listing Contents

```bash
# List all items (default behavior)
ws scrap

# Explicit list command
ws scrap list

# Sort by different criteria
ws scrap list --sort name    # Alphabetical
ws scrap list --sort date    # Most recent first
ws scrap list --sort size    # Largest first
```

Example output:
```
Contents of .scrap folder:
--------------------------------------------------------------------------------
📄 temp.log               1.2 KB  2 hours ago     from: /home/user/project/temp.log
📁 old_code              15.3 MB  1 day ago       from: /home/user/project/old_code
📄 debug.txt              524 B   3 hours ago     from: /home/user/debug.txt
```

## Features

### Search and Find

```bash
# Search by filename (supports regex)
ws scrap find ".*\.log"           # Find all .log files
ws scrap find "test.*"            # Find files starting with "test"
ws scrap find "backup"            # Simple string search

# Search in file contents too
ws scrap find "TODO" --content    # Search for "TODO" in filenames and content
ws scrap find "bug.*fix" --content # Regex search in content
```

### Cleaning and Maintenance

```bash
# Remove items older than 30 days (default)
ws scrap clean

# Remove items older than specific number of days
ws scrap clean --days 7          # Remove items older than 1 week
ws scrap clean --days 1          # Remove items older than 1 day

# Preview what would be removed (dry run)
ws scrap clean --days 30 --verbose

# Remove ALL items from .scrap folder
ws scrap purge

# Skip confirmation prompt
ws scrap purge --force
```

### Archive and Backup

```bash
# Archive .scrap contents to compressed file
ws scrap archive

# Archive with custom filename
ws scrap archive --output backup-2024.tar.gz
ws scrap archive --output "backup-$(date +%Y%m%d).tar.gz"

# Archive and remove original files from .scrap
ws scrap archive --remove

# Archive with custom name and remove
ws scrap archive --output monthly-backup.tar.gz --remove
```

## Workflow Examples

### Daily Workspace Cleanup

```bash
# Move temporary files to .scrap
ws scrap *.tmp *.log debug_output/

# List what's in .scrap
ws scrap

# Clean items older than a week
ws scrap clean --days 7

# Archive old items monthly
ws scrap archive --output "archive-$(date +%Y-%m).tar.gz" --remove
```

### Project Development

```bash
# Scrap old experimental code
ws scrap experimental_feature/ old_tests/

# Search for specific files later
ws scrap find "experimental.*"

# Restore if needed (using ws unscrap)
ws unscrap experimental_feature/

# Or clean up completely
ws scrap purge --force
```

### Code Refactoring

```bash
# Before major refactoring, scrap old implementations
ws scrap old_implementation.rs legacy_tests/

# Check what you've scrapped
ws scrap list --sort date

# If refactoring fails, restore old code
ws unscrap old_implementation.rs

# If successful, clean up
ws scrap clean --days 0  # Remove everything
```

## Safety Features

### Conflict Resolution
- **Automatic renaming**: If a file with the same name exists in `.scrap`, it's automatically renamed (e.g., `file_1.txt`, `file_2.txt`)
- **No overwrites**: Never overwrites existing files
- **Atomic operations**: File moves are atomic to prevent corruption

### Confirmation Prompts
- **Destructive operations**: `ws scrap purge` asks for confirmation unless `--force` is used
- **Preview mode**: `ws scrap clean --verbose` shows what would be removed
- **Clear feedback**: Always shows what actions were taken

### Git Integration
- **Automatic .gitignore**: Adds `.scrap/` to `.gitignore` if the file exists
- **Repository safety**: Never commits temporary files to version control
- **Seamless workflow**: Works transparently in Git repositories

## Metadata and History

The scrap tool maintains detailed metadata about all operations:

### What's Tracked
- **Original paths**: Full path to where files came from
- **Timestamps**: When files were moved to .scrap
- **Restore information**: Data needed for intelligent restoration

### Metadata File
- Stored as `.scrap/.metadata.json`
- JSON format for easy parsing
- Automatically managed (no manual editing needed)
- Used by `ws unscrap` for restoration

## Tips and Best Practices

### Organization
```bash
# Use consistent naming for temporary files
ws scrap temp_* debug_* test_*

# Regular cleanup schedule
ws scrap clean --days 14  # Weekly cleanup of old items
```

### Backup Strategy
```bash
# Monthly archives
ws scrap archive --output "archive-$(date +%Y-%m).tar.gz" --remove

# Project-specific backups
ws scrap archive --output "project-backup-v1.0.tar.gz"
```

### Performance
- The `.scrap` folder is excluded from most file operations
- Metadata file is small and efficiently parsed
- Large directories are moved atomically (not copied)

### Integration with Other Tools
```bash
# Combine with find
find . -name "*.tmp" -exec ws scrap {} \;

# Use in scripts
if [ -f "old_config.conf" ]; then
    ws scrap old_config.conf
fi
```

## Error Handling

### Common Errors
- **File not found**: Clear error message if trying to scrap non-existent files
- **Permission denied**: Helpful error if lacking permissions
- **Disk space**: Warning if `.scrap` folder becomes very large

### Recovery
- **Metadata corruption**: Tool regenerates metadata from existing files
- **Partial operations**: Atomic moves prevent partial corruption
- **Emergency restore**: Files can be manually moved out of `.scrap` if needed

## See Also

- [Unscrap Tool Guide]({{ '/unscrap-guide/' | relative_url }}) - Restoring files from .scrap
- [Installation Guide]({{ '/installation/' | relative_url }}) - Setting up the tools
- [API Reference]({{ '/api-reference/' | relative_url }}) - command reference