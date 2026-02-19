---
layout: default
title: Getting Started
---

# Getting Started with Workspace

This guide will help you get up and running with the Workspace tool suite quickly. Learn the core concepts and basic usage patterns for all tools via the unified `ws` binary.

## What is Workspace?

Workspace is a suite of command-line utilities for developers:

- **ws refactor**: Recursive string replacement in file names and contents
- **ws scrap**: Local trash can for files you want to remove safely
- **ws unscrap**: File restoration from the `.scrap` folder
- **ws git**: Git integration and version management via hooks
- **ws template**: Tera template management and file generation
- **ws update**: Version updates and template rendering
- **ws wstemplate**: Cross-project version stamping with `.wstemplate` files
- **ws version**: Database-driven version management
- **ws ldiff**: Line difference visualization for pattern recognition
- **ws status**: Project status with feature metrics
- **ws feature**: Feature management with state machine workflow
- **ws task**: Feature-centric task management
- **ws directive**: Project directive and rule management
- **ws code**: AST-based code analysis
- **ws test**: Intelligent test runner based on project type
- **ws mcp-server**: MCP server for Claude AI integration

## Installation

### Easy Installation (Recommended)

```bash
git clone https://github.com/jowharshamshiri/ws.git
cd workspace
./install.sh
```

This installs the unified `ws` binary (containing all tools as subcommands) to `~/.local/bin`.

### Verify Installation

```bash
ws --version
ws --help
ws refactor --help
ws scrap --help
ws unscrap --help
ws git --help
ws template --help
ws update --help
ws wstemplate --help
ws ldiff --help
```

## Tool Overview

### Refactor - String Replacement

Performs recursive string replacement in file names and contents:

```bash
# Basic usage
ws refactor <DIRECTORY> <OLD_STRING> <NEW_STRING> [OPTIONS]

# Always preview first
ws refactor . "oldFunction" "newFunction" --verbose
```

### Scrap - Local Trash

Local trash can for files you want to remove safely:

```bash
# Move unwanted files to local trash can instead of deleting
ws scrap temp_file.txt old_directory/

# List what's in trash
ws scrap list

# Find and clean up
ws scrap find "*.log"
ws scrap clean
```

### Unscrap - File Restoration

Restore files from `.scrap` folder:

```bash
# Restore last scrapped item
ws unscrap

# Restore specific file
ws unscrap filename.txt

# Restore to custom location
ws unscrap filename.txt --to /new/path/
```

### Git Integration & Templates

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

### Wstemplate - Cross-Project Version Stamping

Manage `.wstemplate` files that stamp versions across projects:

```bash
# Register this project's scan root
ws wstemplate add /path/to/workspace

# List templates relevant to this project
ws wstemplate list

# Render all relevant templates
ws wstemplate render
```

A `.wstemplate` file is a Tera template that produces the corresponding output file (e.g., `Cargo.toml.wstemplate` renders to `Cargo.toml`). Templates can reference any project's version:

```
version = "{{ project.version }}"
dep_version = "{{ projects.other_lib.version }}"
```

### Version Management

Database-driven major version with git-calculated components:

```bash
ws version show              # Display current version breakdown
ws version major 2           # Set major version to 2
ws version tag               # Create git tag with current version
ws version info              # Show calculation details
```

Version format: `{major}.{minor}.{patch}` where:
- **Major**: Set via `ws version major` (stored in database)
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
ws refactor . "oldFunction" "newFunction" --verbose

# Apply changes
ws refactor . "oldFunction" "newFunction"
```

### Step 3: Try Scrap (File Management)

```bash
# Move temporary files to .scrap
ws scrap temp.txt

# List what's in .scrap
ws scrap list
```

### Step 4: Try Unscrap (File Restoration)

```bash
# Restore the last file moved
ws unscrap

# Or restore specific file
ws unscrap temp.txt
```

### Step 5: Try Git Integration & Templates

```bash
# Install git hook for automatic versioning
ws git install

# Add a template for version info
ws template add version-file --template "Version: {{ project.version }}" --output version.txt

# Make some changes
echo "// Updated code" >> main.js
git add .
git commit -m "Update main.js"

# Check version information
ws git show
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
ws scrap archive backup.tar.gz --remove

# Check version status
ws git status

# Update configuration URLs
ws refactor ./config "old.api.com" "new.api.com" --content-only
```

### Refactoring Modes

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
# Preview refactor changes
ws refactor . "oldname" "newname" --verbose

# Check git status before installation
ws git status
```

### Use Version Control

```bash
# Commit before major changes
git add .
git commit -m "Before refactoring"

# Use git hook to track changes automatically
ws git install

# Apply refactor changes
ws refactor . "oldname" "newname"

# Scrap temporary files safely
ws scrap temp_*.txt build/debug/
```

### Backup and Recovery

```bash
# Create backups before refactor operations
ws refactor . "oldname" "newname" --backup

# Archive scrap contents before cleaning
ws scrap archive monthly-backup.tar.gz

# Restore files if needed
ws unscrap important_file.txt
```

## Getting Help

### Tool-Specific Help

```bash
ws refactor --help
ws scrap --help
ws unscrap --help
ws git --help
ws wstemplate --help
ws version --help

# Verbose output for debugging
ws refactor . "old" "new" --verbose --verbose
ws git status
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
ws refactor . "old" "new" --verbose        # Preview changes
ws refactor . "old" "new" --include "*.rs" # Specific files
ws refactor . "old" "new" --names-only     # Rename only

# === SCRAP - File Management ===
ws scrap file.txt dir/                  # Move to .scrap
ws scrap list                           # List contents
ws scrap find "*.log"                   # Search files
ws scrap clean --days 30               # Remove old items

# === UNSCRAP - File Restoration ===
ws unscrap                              # Restore last item
ws unscrap file.txt                     # Restore specific file
ws unscrap file.txt --to /new/path/     # Custom destination

# === VERSION MANAGEMENT ===
ws git install                          # Install git hook
ws git show                             # Display version info
ws git status                           # Check configuration
ws version show                         # Detailed version breakdown
ws version major 1                      # Set major version

# === WSTEMPLATE - Cross-Project Versioning ===
ws wstemplate add /path/to/root         # Set scan root
ws wstemplate list                      # Show relevant templates
ws wstemplate render                    # Render templates

# === PROJECT MANAGEMENT ===
ws status                               # Project status
ws feature add "New feature"            # Add feature
ws task add "Task" "Description"        # Add task
```
