---
layout: default
title: Getting Started
---

# Getting Started with Workspace

This guide will help you get up and running with the Workspace tool suite quickly. Learn the core concepts and basic usage patterns for all tools via the unified `ws` binary.

## What is Workspace?

Workspace is a suite of command-line utilities for developers and system administrators:

- **ws refactor**: Recursive string replacement in file names and contents
- **ws scrap**: Local trash can for files you want to delete
- **ws unscrap**: File restoration and undo operations  
- **ws st8**: Automatic version management via git hooks
- **ws ldiff**: Line difference visualization for pattern recognition

## Installation

### Easy Installation (Recommended)

```bash
# Clone and install all tools
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace
./install.sh
```

This installs the unified `ws` binary (containing all tools as subcommands) to `~/.local/bin`.

### Verify Installation

```bash
# Check ws binary is installed
ws --version

# Quick help for all subcommands
ws --help
ws refactor --help
ws scrap --help
ws unscrap --help
ws st8 --help
ws ldiff --help
```

## Tool Overview

### 🔄 Refac - String Replacement

Performs recursive string replacement in file names and contents:

```bash
# Basic usage
ws refactor <DIRECTORY> <OLD_STRING> <NEW_STRING> [OPTIONS]

# Always preview first
ws refactor . "oldFunction" "newFunction" --verbose
```

### 🗑️ Scrap - Local Trash

Local trash can for files you want to delete:

```bash
# Move unwanted files to local trash can instead of deleting
ws scrap temp_file.txt old_directory/

# List what's in trash
ws scrap list

# Find and clean up
ws scrap find "*.log"
ws scrap clean
```

### ↩️ Unscrap - File Restoration

Restore files from `.scrap` folder:

```bash
# Restore last scrapped item
ws unscrap

# Restore specific file
ws unscrap filename.txt

# Restore to custom location
ws unscrap filename.txt --to /new/path/
```

### 🏷️ St8 - Version Management

Automatic versioning via git hooks:

```bash
# Install git hook
ws st8 install

# Show version info
ws st8 show

# Check status
st8 status
```

## Quick Start Walkthrough

Let's create a sample project and try all tools:

### Step 1: Create Test Project

```bash
mkdir demo-project
cd demo-project

# Initialize git (for st8)
git init
git config user.name "Demo User"
git config user.email "demo@example.com"

# Create some files
echo "function oldFunction() { return 'hello'; }" > oldFile.js
echo "oldFunction();" > main.js
echo "This is a temporary file" > temp.txt
echo "Log entry 1" > debug.log

# Initial commit
git add .
git commit -m "Initial commit"
```

### Step 2: Try Refac (String Replacement)

```bash
# Preview changes
refac . "oldFunction" "newFunction" --verbose

# Apply changes
refac . "oldFunction" "newFunction"

# Check results
cat *.js
```

### Step 3: Try Scrap (File Management)

```bash
# Move temporary files to .scrap
scrap temp.txt debug.log

# List what's in .scrap
scrap

# Search for files
scrap find "*.txt"
```

### Step 4: Try Unscrap (File Restoration)

```bash
# Restore the last file moved
unscrap

# Or restore specific file
unscrap debug.log
```

### Step 5: Try St8 (Version Management)

```bash
# Install git hook for automatic versioning
st8 install

# Create a tag for versioning base
git tag v1.0

# Make some changes
echo "// Updated code" >> main.js
git add .
git commit -m "Update main.js"

# Check version information
st8 show

# The version.txt file is automatically created/updated
cat version.txt
```

## Common Workflows

### Development Workflow

```bash
# 1. Start working on feature
git checkout -b feature-branch

# 2. Move unwanted files to trash instead of deleting
scrap temp.txt debug.log old_tests/

# 3. Refactor code as needed
refac ./src "OldClass" "NewClass" --verbose
refac ./src "OldClass" "NewClass"

# 4. Set up automatic versioning
st8 install

# 5. If you need files back later
unscrap debug.log
```

### Project Maintenance

```bash
# Clean up old temporary files
scrap clean --days 30

# Archive old items for backup
scrap archive backup-2024.tar.gz --remove

# Check version status across projects
st8 status

# Update configuration URLs
refac ./config "old.api.com" "new.api.com" --content-only
```

### Refactoring Modes

Refac supports different operation modes:

```bash
# Only rename files/directories
refac . "oldProject" "newProject" --names-only

# Only change file contents  
refac . "api.old.com" "api.new.com" --content-only

# Target specific file types
refac ./src "OldStruct" "NewStruct" --include "*.rs"

# Exclude unwanted areas
refac . "oldname" "newname" --exclude "target/*" --exclude "*.log"
```

## Safety Features

### Always Preview First

```bash
# Preview refac changes
refac . "oldname" "newname" --verbose --verbose

# Test scrap operations
scrap --help  # Review options before using

# Check st8 status before installation
st8 status
```

### Use Version Control

```bash
# Commit before major changes
git add .
git commit -m "Before refactoring"

# Use st8 to track changes automatically
st8 install

# Apply refac changes
refac . "oldname" "newname"

# Scrap temporary files safely (tracked in metadata)
scrap temp_*.txt build/debug/
```

### Backup and Recovery

```bash
# Create backups before refac operations
refac . "oldname" "newname" --backup

# Archive scrap contents before cleaning
scrap archive monthly-backup.tar.gz

# Restore files if needed
unscrap important_file.txt
```

## Common Scenarios

### Project Refactor

```bash
# 1. Move build artifacts and logs out of the way
scrap target/ *.log temp/

# 2. Set up versioning for the refactor
st8 install
git tag v1.0  # Mark pre-refactor state

# 3. Rename classes and update imports
refac ./src "UserController" "AccountController" --verbose
refac ./src "UserController" "AccountController" --include "*.rs"

# 4. Update configuration files  
refac ./config "old.server.com" "new.server.com" --content-only

# 5. Restore any needed artifacts
unscrap target/some-important-file

# Version is automatically updated due to git hook
```

### Cleanup and Maintenance

```bash
# Find and manage temporary files
scrap find "*.tmp" "*.log" "*~"

# Archive old test data
scrap old_test_data/ legacy_configs/
scrap archive test-archive-2024.tar.gz --remove

# Update project URLs across all configs
refac . "old.company.com" "new.company.com" \
  --content-only \
  --include "*.env" \
  --include "*.yaml" \
  --include "*.toml"
```

### Version Management Workflow

```bash
# Set up versioning for new project
git init
git add .
git commit -m "Initial commit"
git tag v0.1.0
st8 install

# Normal development - versions update automatically
echo "new feature" >> src/main.rs
git add .
git commit -m "Add new feature"  # Version bumped automatically

# Check current version
st8 show
cat version.txt
```

## Performance and Efficiency

### Refac Performance

```bash
# Use multiple threads for large projects
refac . "oldname" "newname" --threads 8

# Limit search depth to avoid deep traversal
refac . "oldname" "newname" --max-depth 3

# Target specific areas
refac ./src "oldname" "newname"
```

### Scrap Efficiency

```bash
# Batch operations for multiple files
scrap file1.txt file2.txt dir1/ dir2/

# Use patterns for bulk operations
scrap find "*.tmp" | xargs scrap

# Regular cleanup to maintain performance
scrap clean --days 7  # Remove old items
```

### St8 Optimization

```bash
# Configure once per repository
st8 install --force  # Update existing hook

# Use custom version files for different tools
echo '{"version_file": "src/version.rs"}' > .st8.json
```

## Best Practices

### 1. Tool-Specific Guidelines

**Refac:**
- Always use `--verbose` first
- Be specific with include/exclude patterns
- Use version control before major changes

**Scrap:**
- Use instead of deleting files you might need later
- Regular cleanup with `scrap clean` to remove old items
- Archive before purging if you want long-term backup

**St8:**
- Install hooks early in project lifecycle
- Create meaningful git tags for major versions
- Monitor logs for troubleshooting

### 2. Integrated Workflow

```bash
# Safe development cycle
git checkout -b feature-branch
scrap temp_files/ debug_logs/         # Clear workspace
refac ./src "OldAPI" "NewAPI" --verbose  # Preview changes
refac ./src "OldAPI" "NewAPI"         # Apply changes
st8 install                       # Track versions
git add . && git commit -m "Refactor API"  # Auto-version
```

### 3. Project Organization

- Use `.gitignore` for scrap folder (automatically handled)
- Configure st8 early in project setup
- Establish naming conventions before bulk refactoring
- Keep restoration metadata for important files

## Getting Help

### Tool-Specific Help

```bash
# Detailed help for each tool
refac --help
scrap --help  
unscrap --help
st8 --help

# Verbose output for debugging
refac . "old" "new" --verbose --verbose
scrap find "pattern" --verbose
st8 status
```

### Common Issues

**Refac not finding files:**
- Use `--verbose` to see what's processed
- Check include/exclude patterns
- Verify file permissions

**Scrap operations failing:**
- Check disk space for .scrap folder
- Verify file permissions
- Review metadata with `scrap list`

**St8 not working:**
- Ensure you're in a git repository
- Check if hook is executable: `ls -la .git/hooks/pre-commit`
- Verify st8 is in PATH

## Next Steps

### Learn More

1. **Tool-Specific Guides:**
   - [Scrap Guide]({{ '/scrap-guide/' | relative_url }}) - file management
   - [Unscrap Guide]({{ '/unscrap-guide/' | relative_url }}) - File restoration techniques
   - [St8 Guide]({{ '/st8-guide/' | relative_url }}) - Version management setup

2. **Resources:**
   - [Usage Guide]({{ '/usage/' | relative_url }}) - Detailed examples for all tools
   - [API Reference]({{ '/api-reference/' | relative_url }}) - command documentation
   - [Examples]({{ '/examples/' | relative_url }}) - Real-world scenarios

### Quick Reference Card

```bash
# === REFAC - String Replacement ===
refac . "old" "new" --verbose        # Preview changes
refac . "old" "new" --include "*.rs" # Specific files
refac . "old" "new" --names-only     # Rename only

# === SCRAP - File Management ===
scrap file.txt dir/                  # Move to .scrap
scrap                                # List contents
scrap find "*.log"                   # Search files
scrap clean --days 30               # Remove old items

# === UNSCRAP - File Restoration ===
unscrap                              # Restore last item
unscrap file.txt                     # Restore specific file
unscrap file.txt --to /new/path/     # Custom destination

# === ST8 - Version Management ===
st8 install                      # Install git hook
st8 show                         # Display version info
st8 status                       # Check configuration
```