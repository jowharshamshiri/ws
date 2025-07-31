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
- **ws ws unscrap**: File restoration and undo operations  
- **ws git**: Git integration and version management via hooks
- **ws template**: Template management and file generation
- **ws update**: Manual version updates and template rendering
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
ws ws scrap --help
ws ws unscrap --help
ws git --help
ws template --help
ws update --help
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
ws ws scrap temp_file.txt old_directory/

# List what's in trash
ws ws scrap list

# Find and clean up
ws ws scrap find "*.log"
ws ws scrap clean
```

### ↩️ Unws scrap - File Restoration

Restore files from `.scrap` folder:

```bash
# Restore last scrapped item
ws ws unscrap

# Restore specific file
ws ws unscrap filename.txt

# Restore to custom location
ws ws unscrap filename.txt --to /new/path/
```

### 🏷️ Git Integration & Templates

Automatic versioning via git hooks and template management:

```bash
# Install git hook
ws git install

# Show version info
ws git show

# Check status
ws git status

# Add templates for automatic file generation
ws template add version-info --template "Version: {{ project.version }}" --output VERSION.txt

# Manual version update
ws update
```

## Quick Start Walkthrough

Let's create a sample project and try all tools:

### Step 1: Create Test Project

```bash
mkdir demo-project
cd demo-project

# Initialize git (for git integration)
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
ws refactor . "oldFunction" "newFunction" --verbose

# Apply changes
ws refactor . "oldFunction" "newFunction"

# Check results
cat *.js
```

### Step 3: Try Scrap (File Management)

```bash
# Move temporary files to .scrap
ws ws scrap temp.txt debug.log

# List what's in .scrap
ws ws scrap list

# Search for files
ws ws scrap find "*.txt"
```

### Step 4: Try Unws scrap (File Restoration)

```bash
# Restore the last file moved
ws ws unscrap

# Or restore specific file
ws ws unscrap debug.log
```

### Step 5: Try Git Integration & Templates

```bash
# Install git hook for automatic versioning
ws git install

# Add a template for version info
ws template add version-file --template "Version: {{ project.version }}" --output version.txt

# Create a tag for versioning base  
git tag v1.0

# Make some changes
echo "// Updated code" >> main.js
git add .
git commit -m "Update main.js"

# Check version information
ws git show

# The version.txt file is automatically created/updated
cat version.txt
```

## Common Workflows

### Development Workflow

```bash
# 1. Start working on feature
git checkout -b feature-branch

# 2. Move unwanted files to trash instead of deleting
ws scrap temp.txt debug.log old_tests/

# 3. Refactor code as needed
ws refactor ./src "OldClass" "NewClass" --verbose
ws refactor ./src "OldClass" "NewClass"

# 4. Set up automatic versioning
ws git install

# 5. If you need files back later
ws unscrap debug.log
```

### Project Maintenance

```bash
# Clean up old temporary files
ws scrap clean --days 30

# Archive old items for backup
ws scrap archive backup-2024.tar.gz --remove

# Check version status across projects
ws git status

# Update configuration URLs
ws refactor ./config "old.api.com" "new.api.com" --content-only
```

### Refactoring Modes

Refac supports different operation modes:

```bash
# Only rename files/directories
ws refactor . "oldProject" "newProject" --names-only

# Only change file contents  
ws refactor . "api.old.com" "api.new.com" --content-only

# Target specific file types
ws refactor ./src "OldStruct" "NewStruct" --include "*.rs"

# Exclude unwanted areas
ws refactor . "oldname" "newname" --exclude "target/*" --exclude "*.log"
```

## Safety Features

### Always Preview First

```bash
# Preview ws refactor changes
ws refactor . "oldname" "newname" --verbose --verbose

# Test ws scrap operations
ws scrap --help  # Review options before using

# Check ws git status before installation
ws git status
```

### Use Version Control

```bash
# Commit before major changes
git add .
git commit -m "Before refactoring"

# Use ws git to track changes automatically
ws git install

# Apply ws refactor changes
ws refactor . "oldname" "newname"

# Scrap temporary files safely (tracked in metadata)
ws scrap temp_*.txt build/debug/
```

### Backup and Recovery

```bash
# Create backups before ws refactor operations
ws refactor . "oldname" "newname" --backup

# Archive ws scrap contents before cleaning
ws scrap archive monthly-backup.tar.gz

# Restore files if needed
ws unscrap important_file.txt
```

## Common Scenarios

### Project Refactor

```bash
# 1. Move build artifacts and logs out of the way
ws scrap target/ *.log temp/

# 2. Set up versioning for the refactor
ws git install
git tag v1.0  # Mark pre-refactor state

# 3. Rename classes and update imports
ws refactor ./src "UserController" "AccountController" --verbose
ws refactor ./src "UserController" "AccountController" --include "*.rs"

# 4. Update configuration files  
ws refactor ./config "old.server.com" "new.server.com" --content-only

# 5. Restore any needed artifacts
ws unscrap target/some-important-file

# Version is automatically updated due to git hook
```

### Cleanup and Maintenance

```bash
# Find and manage temporary files
ws scrap find "*.tmp" "*.log" "*~"

# Archive old test data
ws scrap old_test_data/ legacy_configs/
ws scrap archive test-archive-2024.tar.gz --remove

# Update project URLs across all configs
ws refactor . "old.company.com" "new.company.com" \
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
ws git install

# Normal development - versions update automatically
echo "new feature" >> src/main.rs
git add .
git commit -m "Add new feature"  # Version bumped automatically

# Check current version
ws git show
cat version.txt
```

## Performance and Efficiency

### Refac Performance

```bash
# Use multiple threads for large projects
ws refactor . "oldname" "newname" --threads 8

# Limit search depth to avoid deep traversal
ws refactor . "oldname" "newname" --max-depth 3

# Target specific areas
ws refactor ./src "oldname" "newname"
```

### Scrap Efficiency

```bash
# Batch operations for multiple files
ws scrap file1.txt file2.txt dir1/ dir2/

# Use patterns for bulk operations
ws scrap find "*.tmp" | xargs scrap

# Regular cleanup to maintain performance
ws scrap clean --days 7  # Remove old items
```

### St8 Optimization

```bash
# Configure once per repository
ws git install --force  # Update existing hook

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
- Regular cleanup with `ws scrap clean` to remove old items
- Archive before purging if you want long-term backup

**St8:**
- Install hooks early in project lifecycle
- Create meaningful git tags for major versions
- Monitor logs for troubleshooting

### 2. Integrated Workflow

```bash
# Safe development cycle
git checkout -b feature-branch
ws scrap temp_files/ debug_logs/         # Clear workspace
ws refactor ./src "OldAPI" "NewAPI" --verbose  # Preview changes
ws refactor ./src "OldAPI" "NewAPI"         # Apply changes
ws git install                       # Track versions
git add . && git commit -m "Refactor API"  # Auto-version
```

### 3. Project Organization

- Use `.gitignore` for ws scrap folder (automatically handled)
- Configure ws git early in project setup
- Establish naming conventions before bulk refactoring
- Keep restoration metadata for important files

## Getting Help

### Tool-Specific Help

```bash
# Detailed help for each tool
ws refactor --help
ws scrap --help  
ws unscrap --help
ws git --help

# Verbose output for debugging
ws refactor . "old" "new" --verbose --verbose
ws scrap find "pattern" --verbose
ws git status
```

### Common Issues

**Refac not finding files:**
- Use `--verbose` to see what's processed
- Check include/exclude patterns
- Verify file permissions

**Scrap operations failing:**
- Check disk space for .scrap folder
- Verify file permissions
- Review metadata with `ws scrap list`

**Git integration not working:**
- Ensure you're in a git repository
- Check if hook is executable: `ls -la .git/hooks/pre-commit`
- Verify ws git is in PATH

## Next Steps

### Learn More

1. **Tool-Specific Guides:**
   - [Scrap Guide]({{ '/scrap-guide/' | relative_url }}) - file management
   - [Unws scrap Guide]({{ '/ws unscrap-guide/' | relative_url }}) - File restoration techniques
   - [St8 Guide]({{ '/st8-guide/' | relative_url }}) - Version management setup

2. **Resources:**
   - [Usage Guide]({{ '/usage/' | relative_url }}) - Detailed examples for all tools
   - [API Reference]({{ '/api-reference/' | relative_url }}) - command documentation
   - [Examples]({{ '/examples/' | relative_url }}) - Real-world scenarios

### Quick Reference Card

```bash
# === REFAC - String Replacement ===
ws refactor . "old" "new" --verbose        # Preview changes
ws refactor . "old" "new" --include "*.rs" # Specific files
ws refactor . "old" "new" --names-only     # Rename only

# === SCRAP - File Management ===
ws scrap file.txt dir/                  # Move to .scrap
ws scrap                                # List contents
ws scrap find "*.log"                   # Search files
ws scrap clean --days 30               # Remove old items

# === UNSCRAP - File Restoration ===
ws unscrap                              # Restore last item
ws unscrap file.txt                     # Restore specific file
ws unscrap file.txt --to /new/path/     # Custom destination

# === ST8 - Version Management ===
ws git install                      # Install git hook
ws git show                         # Display version info
ws git status                       # Check configuration
```