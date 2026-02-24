---
layout: default
title: Getting Started
---

# Getting Started with Workspace

This guide will help you get up and running with the Workspace tool suite quickly. Learn the core concepts and basic usage patterns for all tools via the unified `wsb` binary.

## What is Workspace?

Workspace is a suite of command-line utilities for developers:

- **wsb refactor**: Recursive string replacement in file names and contents
- **wsb scrap**: Local trash can for files you want to remove safely
- **wsb unscrap**: File restoration from the `.scrap` folder
- **wsb git**: Git integration and version management via hooks
- **wsb template**: Tera template management and file generation
- **wsb update**: Version updates and template rendering
- **wsb wstemplate**: Cross-project version stamping with `.wstemplate` files
- **wsb version**: Database-driven version management
- **wsb ldiff**: Line difference visualization for pattern recognition
- **wsb status**: Project status with feature metrics
- **wsb feature**: Feature management with state machine workflow
- **wsb task**: Feature-centric task management
- **wsb directive**: Project directive and rule management
- **wsb code**: AST-based code analysis
- **wsb test**: Intelligent test runner based on project type
- **wsb mcp-server**: MCP server for Claude AI integration

## Installation

### Easy Installation (Recommended)

```bash
git clone https://github.com/jowharshamshiri/wsb.git
cd workspace
./install.sh
```

This installs the unified `wsb` binary (containing all tools as subcommands) to `~/.local/bin`.

### Verify Installation

```bash
wsb --version
wsb --help
wsb refactor --help
wsb scrap --help
wsb unscrap --help
wsb git --help
wsb template --help
wsb update --help
wsb wstemplate --help
wsb ldiff --help
```

## Tool Overview

### Refactor - String Replacement

Performs recursive string replacement in file names and contents:

```bash
# Basic usage
wsb refactor <DIRECTORY> <OLD_STRING> <NEW_STRING> [OPTIONS]

# Always preview first
wsb refactor . "oldFunction" "newFunction" --verbose
```

### Scrap - Local Trash

Local trash can for files you want to remove safely:

```bash
# Move unwanted files to local trash can instead of deleting
wsb scrap temp_file.txt old_directory/

# List what's in trash
wsb scrap list

# Find and clean up
wsb scrap find "*.log"
wsb scrap clean
```

### Unscrap - File Restoration

Restore files from `.scrap` folder:

```bash
# Restore last scrapped item
wsb unscrap

# Restore specific file
wsb unscrap filename.txt

# Restore to custom location
wsb unscrap filename.txt --to /new/path/
```

### Git Integration & Templates

Automatic versioning via git hooks and template management:

```bash
# Install git hook
wsb git install

# Show version info
wsb git show

# Check status
wsb git status

# Add templates for automatic file generation
wsb template add version-info --template "Version: {{ project.version }}" --output VERSION.txt

# Manual version update
wsb update
```

### Wstemplate - Cross-Project Version Stamping

Manage `.wstemplate` files that stamp versions across projects:

```bash
# Register this project's scan root
wsb wstemplate add /path/to/workspace

# List templates relevant to this project
wsb wstemplate list

# Render all relevant templates
wsb wstemplate render
```

A `.wstemplate` file is a Tera template that produces the corresponding output file (e.g., `Cargo.toml.wstemplate` renders to `Cargo.toml`). Templates can reference any project's version:

```
version = "{{ project.version }}"
dep_version = "{{ projects.other_lib.version }}"
```

### Version Management

Database-driven major version with git-calculated components:

```bash
wsb version show              # Display current version breakdown
wsb version major 2           # Set major version to 2
wsb version tag               # Create git tag with current version
wsb version info              # Show calculation details
```

Version format: `{major}.{minor}.{patch}` where:
- **Major**: Set via `wsb version major` (stored in database)
- **Minor**: Total commits in the repository
- **Patch**: Total line changes (additions + deletions)

## Quick Start Walkthrough

### Step 1: Create Test Project

```bash
mkdir demo-project
cd demo-project

git init
git config user.name "Demo User"
git config user.email "demo@example.com"

echo "function oldFunction() { return 'hello'; }" > oldFile.js
echo "oldFunction();" > main.js
echo "This is a temporary file" > temp.txt

git add .
git commit -m "Initial commit"
```

### Step 2: Try Refactor (String Replacement)

```bash
# Preview changes
wsb refactor . "oldFunction" "newFunction" --verbose

# Apply changes
wsb refactor . "oldFunction" "newFunction"
```

### Step 3: Try Scrap (File Management)

```bash
# Move temporary files to .scrap
wsb scrap temp.txt

# List what's in .scrap
wsb scrap list
```

### Step 4: Try Unscrap (File Restoration)

```bash
# Restore the last file moved
wsb unscrap

# Or restore specific file
wsb unscrap temp.txt
```

### Step 5: Try Git Integration & Templates

```bash
# Install git hook for automatic versioning
wsb git install

# Add a template for version info
wsb template add version-file --template "Version: {{ project.version }}" --output version.txt

# Make some changes
echo "// Updated code" >> main.js
git add .
git commit -m "Update main.js"

# Check version information
wsb git show
cat version.txt
```

## Common Workflows

### Development Workflow

```bash
# 1. Start working on feature
git checkout -b feature-branch

# 2. Move unwanted files to trash instead of deleting
wsb scrap temp.txt debug.log old_tests/

# 3. Refactor code as needed
wsb refactor ./src "OldClass" "NewClass" --verbose
wsb refactor ./src "OldClass" "NewClass"

# 4. Set up automatic versioning
wsb git install

# 5. If you need files back later
wsb unscrap debug.log
```

### Project Maintenance

```bash
# Clean up old temporary files
wsb scrap clean --days 30

# Archive old items for backup
wsb scrap archive backup.tar.gz --remove

# Check version status
wsb git status

# Update configuration URLs
wsb refactor ./config "old.api.com" "new.api.com" --content-only
```

### Refactoring Modes

```bash
# Only rename files/directories
wsb refactor . "oldProject" "newProject" --names-only

# Only change file contents
wsb refactor . "api.old.com" "api.new.com" --content-only

# Target specific file types
wsb refactor ./src "OldStruct" "NewStruct" --include "*.rs"

# Exclude unwanted areas
wsb refactor . "oldname" "newname" --exclude "target/*" --exclude "*.log"
```

## Safety Features

### Always Preview First

```bash
# Preview refactor changes
wsb refactor . "oldname" "newname" --verbose

# Check git status before installation
wsb git status
```

### Use Version Control

```bash
# Commit before major changes
git add .
git commit -m "Before refactoring"

# Use git hook to track changes automatically
wsb git install

# Apply refactor changes
wsb refactor . "oldname" "newname"

# Scrap temporary files safely
wsb scrap temp_*.txt build/debug/
```

### Backup and Recovery

```bash
# Create backups before refactor operations
wsb refactor . "oldname" "newname" --backup

# Archive scrap contents before cleaning
wsb scrap archive monthly-backup.tar.gz

# Restore files if needed
wsb unscrap important_file.txt
```

## Getting Help

### Tool-Specific Help

```bash
wsb refactor --help
wsb scrap --help
wsb unscrap --help
wsb git --help
wsb wstemplate --help
wsb version --help

# Verbose output for debugging
wsb refactor . "old" "new" --verbose --verbose
wsb git status
```

## Next Steps

1. **Tool-Specific Guides:**
   - [Scrap Guide]({{ '/scrap-guide/' | relative_url }}) - File management
   - [Unscrap Guide]({{ '/unscrap-guide/' | relative_url }}) - File restoration
   - [St8 Guide]({{ '/st8-guide/' | relative_url }}) - Version management and wstemplate

2. **Resources:**
   - [Usage Guide]({{ '/usage/' | relative_url }}) - Detailed examples for all tools
   - [API Reference]({{ '/api-reference/' | relative_url }}) - Command documentation
   - [Examples]({{ '/examples/' | relative_url }}) - Real-world scenarios

### Quick Reference Card

```bash
# === REFACTOR - String Replacement ===
wsb refactor . "old" "new" --verbose        # Preview changes
wsb refactor . "old" "new" --include "*.rs" # Specific files
wsb refactor . "old" "new" --names-only     # Rename only

# === SCRAP - File Management ===
wsb scrap file.txt dir/                  # Move to .scrap
wsb scrap list                           # List contents
wsb scrap find "*.log"                   # Search files
wsb scrap clean --days 30               # Remove old items

# === UNSCRAP - File Restoration ===
wsb unscrap                              # Restore last item
wsb unscrap file.txt                     # Restore specific file
wsb unscrap file.txt --to /new/path/     # Custom destination

# === VERSION MANAGEMENT ===
wsb git install                          # Install git hook
wsb git show                             # Display version info
wsb git status                           # Check configuration
wsb version show                         # Detailed version breakdown
wsb version major 1                      # Set major version

# === WSTEMPLATE - Cross-Project Versioning ===
wsb wstemplate add /path/to/root         # Set scan root
wsb wstemplate list                      # Show relevant templates
wsb wstemplate render                    # Render templates

# === PROJECT MANAGEMENT ===
wsb status                               # Project status
wsb feature add "New feature"            # Add feature
wsb task add "Task" "Description"        # Add task
```
