---
layout: default
title: Unscrap Tool Guide
---

# Unscrap Tool Guide

The `unscrap` tool restores files and directories from the `.scrap` folder back to their original locations or custom destinations. It works in conjunction with the `scrap` tool to provide a trash can workflow.

## Core Concepts

### Restoration Types

- **Undo Last**: Restore the most recently scrapped item
- **Specific Item**: Restore a named file or directory
- **Custom Destination**: Restore to a different location than the original

### Metadata Integration

- Uses metadata stored by `scrap` tool to know original locations
- Maintains file history for intelligent restoration
- Handles missing metadata gracefully

## Basic Operations

### Quick Restore (Undo Last)

```bash
# Restore the most recently scrapped item
wsb unscrap

# This finds the item with the latest timestamp and restores it
# to its original location
```

Example workflow:

```bash
# Accidentally scrap an important file
wsb scrap important.txt

# Realize the mistake and quickly undo
wsb unscrap
# → Restored 'important.txt' to '/path/to/original/important.txt'
```

### Restore Specific Items

```bash
# Restore a specific file
wsb unscrap filename.txt

# Restore a directory
wsb unscrap old_project/

# Restore with the exact name as it appears in .scrap
wsb unscrap file_1.txt  # If there were naming conflicts
```

## Features

### Custom Destinations

```bash
# Restore to a different directory
wsb unscrap file.txt --to /new/location/

# Restore to a specific file path
wsb unscrap data.json --to ~/backup/recovered-data.json

# Restore to current directory
wsb unscrap file.txt --to .
```

### Force Overwrite

```bash
# Overwrite existing files at destination
wsb unscrap file.txt --force

# Restore to custom location with force
wsb unscrap file.txt --to /existing/location/ --force
```

## Detailed Examples

### Basic Restoration Workflow

```bash
# 1. Check what's in .scrap
wsb scrap list

# Output:
# 📄 temp.log               1.2 KB  2 hours ago     from: /home/user/project/temp.log
# 📁 old_code              15.3 MB  1 day ago       from: /home/user/project/old_code
# 📄 debug.txt              524 B   3 hours ago     from: /home/user/debug.txt

# 2. Restore specific item
wsb unscrap temp.log
# → Restored 'temp.log' to '/home/user/project/temp.log'

# 3. Check it's gone from .scrap
wsb scrap list
# temp.log no longer appears in the list
```

### Undo Last Operations

```bash
# Work session with multiple scrap operations
wsb scrap old_file1.txt        # 10:00 AM
wsb scrap temp_directory/      # 10:30 AM
wsb scrap debug.log           # 11:00 AM

# Undo the last action (debug.log)
wsb unscrap
# → Restoring last scrapped item: debug.log (from /path/to/debug.log)
# → Restored 'debug.log' to '/path/to/debug.log'

# Undo the previous action (temp_directory/)
wsb unscrap
# → Restoring last scrapped item: temp_directory (from /path/to/temp_directory)
```

### Custom Destination Scenarios

```bash
# Restore to a backup location
wsb unscrap important_config.json --to ~/backups/

# Restore with a new name
wsb unscrap old_script.sh --to ~/scripts/new_script.sh

# Restore to current project directory
wsb unscrap library_code/ --to ./vendor/
```

### Handling Conflicts

```bash
# Try to restore when destination exists
wsb unscrap file.txt
# Error: Destination '/path/to/file.txt' already exists. Use --force to overwrite.

# Force the restoration
wsb unscrap file.txt --force
# → Restored 'file.txt' to '/path/to/file.txt' (overwrote existing file)

# Or restore to a different location
wsb unscrap file.txt --to ./recovered_file.txt
# → Restored 'file.txt' to './recovered_file.txt'
```

## Restoration Behavior

### With Metadata

When metadata exists (file was scrapped with current version):

- Restores to exact original location
- Recreates directory structure if needed
- Preserves original timestamps and permissions

### Without Metadata

When metadata is missing (older .scrap folder or manual additions):

- Restores to current directory by default
- Uses custom destination if specified
- Still preserves file attributes

### Directory Structure Recreation

```bash
# Original file: /deep/nested/path/file.txt
# After scrapping and restoring:
wsb unscrap file.txt
# → Creates /deep/nested/path/ if it doesn't exist
# → Restores file.txt to /deep/nested/path/file.txt
```

## Error Handling

### Common Errors and Solutions

#### File Not Found

```bash
wsb unscrap nonexistent.txt
# Error: 'nonexistent.txt' not found in .scrap folder

# Solution: Check what's available
wsb scrap list
```

#### Destination Exists

```bash
wsb unscrap file.txt
# Error: Destination '/path/to/file.txt' already exists. Use --force to overwrite.

# Solutions:
wsb unscrap file.txt --force                    # Overwrite
wsb unscrap file.txt --to ./recovered_file.txt  # Different location
```

#### Permission Denied

```bash
wsb unscrap system_file.conf
# Error: Failed to restore 'system_file.conf' to '/etc/system_file.conf'
# Context: Permission denied

# Solution: Use sudo or restore to accessible location
wsb unscrap system_file.conf --to ~/recovered_system_file.conf
```

#### Missing Parent Directory

```bash
# If original path no longer exists, wsb unscrap creates it
wsb unscrap file.txt
# → Creating parent directory: /path/that/was/deleted
# → Restored 'file.txt' to '/path/that/was/deleted/file.txt'
```

## Integration with Scrap

### Typical Workflow

```bash
# 1. Scrap files during cleanup
wsb scrap temp_* debug_* old_*/

# 2. Continue working...

# 3. Realize you need something back
wsb scrap find "important"  # Find the file
wsb unscrap important_backup.txt  # Restore it

# 4. Or quickly undo last scrap
wsb unscrap  # Restores most recent item
```

### Metadata Consistency

- `unscrap` updates metadata when restoring files
- Removes entries for restored items
- Keeps metadata clean and accurate

## Usage

### Scripting with Unscrap

```bash
#!/bin/bash
# Restore script for project files

# Check if file exists in .scrap
if wsb scrap find "project.config" > /dev/null 2>&1; then
    echo "Restoring project configuration..."
    wsb unscrap project.config --force
else
    echo "No project configuration found in .scrap"
fi
```

### Batch Restoration

```bash
# Restore multiple files (run multiple commands)
for file in config.json settings.ini database.db; do
    if [ -f ".scrap/$file" ]; then
        wsb unscrap "$file" --force
    fi
done
```

### Selective Restoration

```bash
# Restore only specific types of files
wsb scrap find "\.txt$" | while read -r file; do
    echo "Restore $file? (y/n)"
    read -r response
    if [ "$response" = "y" ]; then
        wsb unscrap "$file"
    fi
done
```

## Safety Features

### Conflict Prevention

- **Existence checking**: Always checks if destination exists
- **Clear warnings**: Explicit error messages for conflicts
- **Force option**: Controlled overwriting with `--force` flag

### Atomic Operations

- **Safe restoration**: Files are moved atomically
- **No partial states**: Either fully restored or operation fails
- **Metadata consistency**: Updates metadata only on successful restore

### Path Validation

- **Security**: Prevents path traversal attacks
- **Existence**: Creates parent directories as needed
- **Permissions**: Clear error messages for permission issues

## Tips and Best Practices

### Quick Recovery

```bash
# Always try undo first for recent mistakes
wsb unscrap

# Use wsb scrap list to see what's available
wsb scrap list | grep important
```

### Backup Strategy

```bash
# Before major restore operations, archive current .scrap
wsb scrap archive --output "before-restore-$(date +%s).tar.gz"

# Then restore safely
wsb unscrap critical_file.txt
```

### Project Management

```bash
# Restore project files to a staging area first
mkdir staging/
wsb unscrap project_files/ --to staging/

# Review and then move to final location
mv staging/project_files/* ./
```

## See Also

- [Scrap Tool Guide]({{ '/scrap-guide/' | relative_url }}) - Moving files to .scrap
- [Installation Guide]({{ '/installation/' | relative_url }}) - Setting up the tools
- [API Reference]({{ '/api-reference/' | relative_url }}) - command reference
