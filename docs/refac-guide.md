---
layout: default
title: Refac - Code Refactoring Guide
---

# Refac - Code Refactoring Guide

The `refac` tool is a string replacement engine designed for safe code refactoring and content modification across codebases.

## Overview

**Refac** (short for "refactor") provides string replacement capabilities with:
- **Pre-validation**: Tests all operations before execution
- **Collision Detection**: Prevents overwrites and conflicts
- **Multi-threading**: Parallel processing for large codebases
- **Language Awareness**: Handling of code structures
- **Safety Features**: Atomic operations with rollback capability

## Basic Usage

### Command Syntax
```bash
refac <directory> <old_string> <new_string> [OPTIONS]
```

### First Steps
```bash
# Refac always previews changes and asks for confirmation
refac ./src "OldClassName" "NewClassName" --verbose

# Apply with backup for safety
refac ./src "OldClassName" "NewClassName" --backup

# Target specific file types
refac ./src "old_api" "new_api" --include "*.rs" --include "*.toml"
```

## Core Features

### Pre-Operation Validation
Refac validates every operation before making changes:

```bash
# Validation process (automatic):
# 1. File accessibility check
# 2. Encoding validation
# 3. Permission verification
# 4. Collision detection
# 5. Path length validation
# 6. Disk space check
```

**Validation Output Example**:
```
INFO: Phase 1: Discovering files and directories...
INFO: Phase 2: Checking for naming collisions...
INFO: Phase 3: Validating all operations...
INFO: Validation passed: All operations can be performed safely.
```

### 🔍 Built-in Change Preview
Refac always shows changes before applying them and asks for confirmation:

```bash
# Basic operation (shows preview automatically)
refac . "oldname" "newname"

# Verbose output with detailed information
refac . "oldname" "newname" --verbose

# JSON output for scripting (still shows preview)
refac . "oldname" "newname" --format json
```

**Preview Output Example**:
```
=== CHANGE SUMMARY ===
Content modifications: 15 file(s)
File renames:         8 file(s)
Directory renames:    3 directory(ies)
Total changes:        26

FILES TO BE MODIFIED:
  src/main.rs: 3 occurrence(s)
  src/lib.rs: 1 occurrence(s)
  tests/integration.rs: 2 occurrence(s)

FILES TO BE RENAMED:
  oldname_config.toml → newname_config.toml
  src/oldname_module.rs → src/newname_module.rs
```

### ⚡ Multi-Threading and Performance
Optimized parallel processing for large datasets:

```bash
# Use multiple threads for faster processing
refac . "oldname" "newname" --threads 8

# Auto-detect optimal thread count
refac . "oldname" "newname" --threads 0

# Progress tracking for long operations
refac . "oldname" "newname" --progress always
```

**Performance Features**:
- **Parallel Content Processing**: Multi-threaded file content replacement
- **Streaming I/O**: Efficient handling of large files
- **Smart Filtering**: Pre-filter files to reduce processing overhead
- **Progress Tracking**: Visual progress bars with ETA estimates

## Operation Modes

### 🎯 Targeted Operations
Control exactly what gets modified:

```bash
# Only rename files and directories (skip content)
refac . "oldproject" "newproject" --names-only

# Only replace content (skip renaming)
refac . "old.api.com" "new.api.com" --content-only

# Only process files (skip directories)
refac . "oldname" "newname" --files-only

# Only process directories (skip files)
refac . "oldname" "newname" --dirs-only
```

### 📁 Pattern Filtering
Precise control over which files are processed:

```bash
# Include specific file types
refac . "oldname" "newname" \
  --include "*.rs" \
  --include "*.toml" \
  --include "*.md"

# Exclude unwanted areas
refac . "oldname" "newname" \
  --exclude "target/*" \
  --exclude "*.log" \
  --exclude ".git/*"

# Complex filtering
refac ./src "OldStruct" "NewStruct" \
  --include "*.rs" \
  --exclude "*/tests/*" \
  --exclude "*/examples/*"
```

### 🏗️ Directory Depth Control
Manage traversal depth for large projects:

```bash
# Limit to current directory only
refac . "oldname" "newname" --max-depth 1

# Search 3 levels deep
refac . "oldname" "newname" --max-depth 3

# Unlimited depth (default)
refac . "oldname" "newname" --max-depth 0
```

## Advanced Features

### 💾 Backup and Recovery
Safe modification with automatic backups:

```bash
# Create backups before modifying files
refac . "oldname" "newname" --backup

# Backups are created with .refac_backup extension
# Example: config.toml → config.toml.refac_backup
```

**Backup Features**:
- **Atomic Backup Creation**: Backups created before any modifications
- **Conflict Resolution**: Unique backup names if backups already exist
- **Selective Backup**: Only backs up files that will be modified
- **Manual Cleanup**: Backups remain until manually removed

### 🔒 Safety and Error Handling
Mission-critical safety features:

```bash
# Force operation without confirmation
refac . "oldname" "newname" --force

# Case-sensitive matching (default)
refac . "OldName" "NewName"

# Show detailed error information
refac . "oldname" "newname" --verbose
```

**Safety Guarantees**:
- **Collision Prevention**: Detects and prevents overwrite conflicts
- **Binary File Protection**: Automatically skips binary files for content operations
- **Atomic Operations**: Either all operations succeed or none are applied
- **Permission Respect**: Respects file system permissions and ownership

### 🌐 Cross-Platform Compatibility
Consistent behavior across all platforms:

```bash
# Works identically on Windows, macOS, and Linux
refac . "oldname" "newname"

# Handles platform-specific path separators
refac . "old\\path" "new/path"  # Windows
refac . "old/path" "new/path"   # Unix-like
```

**Platform Features**:
- **Path Normalization**: Automatic path separator handling
- **Case Sensitivity**: Respects filesystem case sensitivity settings
- **Permission Handling**: Platform-appropriate permission management
- **Character Encoding**: UTF-8 support with platform-specific handling

## Real-World Use Cases

### 🔄 API Migration
Migrate from old API to new API across entire codebase:

```bash
# 1. Review the migration (refac shows changes before applying)
refac ./src "old_api::Client" "new_api::Client" --verbose

# 2. Update import statements
refac ./src "use old_api" "use new_api" --content-only --include "*.rs"

# 3. Update function calls
refac ./src "old_api::connect" "new_api::connect" --content-only

# 4. Update configuration files
refac ./config "old_api_endpoint" "new_api_endpoint" --content-only --include "*.toml"
```

### 🏢 Project Rebranding
Rename a project throughout the codebase:

```bash
# 1. Update package names
refac . "oldproject" "newproject" --include "Cargo.toml" --include "package.json"

# 2. Update file names and directory structure
refac . "oldproject" "newproject" --names-only

# 3. Update content references
refac . "oldproject" "newproject" --content-only --exclude "target/*"

# 4. Update documentation
refac ./docs "OldProject" "NewProject" --include "*.md"
```

### 🏗️ Refactoring Code Structure
Reorganize and rename code components:

```bash
# 1. Rename a module throughout the codebase
refac ./src "user_service" "account_service" --include "*.rs"

# 2. Update struct names
refac ./src "UserData" "AccountData" --content-only --include "*.rs"

# 3. Update configuration keys
refac ./config "user_" "account_" --content-only --include "*.toml" --include "*.yaml"
```

### 🌍 Configuration Updates
Update configuration across multiple environments:

```bash
# Update API endpoints across all configs
refac ./config "api.old.com" "api.new.com" \
  --content-only \
  --include "*.toml" \
  --include "*.yaml" \
  --include "*.json" \
  --include "*.env"

# Update database connection strings
refac ./config "old_database" "new_database" \
  --content-only \
  --backup
```

## Output Formats

### 📊 Human-Readable Output
Default format with clear progress information:

```
=== REFAC TOOL ===
Root directory: /path/to/project
Old string: 'oldname'
New string: 'newname'
Mode: Full

Phase 1: Discovering files and directories...
Found 1,234 files and 56 directories to process

Phase 2: Checking for naming collisions...
No collisions detected

Phase 3: Validating all operations...
Validation passed: All operations can be performed safely.

=== CHANGE SUMMARY ===
Content modifications: 15 file(s)
File renames:         8 file(s)
Directory renames:    3 directory(ies)
Total changes:        26

Do you want to proceed? (y/N) y

Replacing content in files...
[████████████████████████████████████████] 15/15 files

Renaming files and directories...
[████████████████████████████████████████] 11/11 items

=== OPERATION COMPLETE ===
Operation completed successfully!
Total changes applied: 26
```

### 🤖 JSON Output
Machine-readable format for automation:

```bash
refac . "oldname" "newname" --format json
```

```json
{
  "summary": {
    "content_changes": 15,
    "file_renames": 8,
    "directory_renames": 3,
    "total_changes": 26
  },
  "operations": [
    {
      "type": "content_replace",
      "file": "src/main.rs",
      "occurrences": 3
    },
    {
      "type": "file_rename",
      "from": "oldname_config.toml",
      "to": "newname_config.toml"
    }
  ],
  "result": "success",
  "interactive": false,
  "execution_time_ms": 1234
}
```

### 📝 Plain Text Output
Minimal output for scripting:

```bash
refac . "oldname" "newname" --format plain
```

```
Content changes: 15
File renames: 8
Directory renames: 3
Total changes: 26
```

## Command Line Reference

### Essential Options
| Option | Short | Description |
|--------|-------|-------------|
| `--assume-yes` | `-y` | Skip confirmation prompts (non-interactive mode) |
| `--verbose` | `-v` | Show detailed output |
| `--backup` | `-b` | Create backup files before modification |

### Operation Modes
| Option | Description |
|--------|-------------|
| `--names-only` | Only rename files/directories, skip content |
| `--content-only` | Only replace content, skip renaming |
| `--files-only` | Only process files, skip directories |
| `--dirs-only` | Only process directories, skip files |

### Filtering Options
| Option | Description |
|--------|-------------|
| `--include <pattern>` | Include only files matching pattern |
| `--exclude <pattern>` | Exclude files matching pattern |
| `--max-depth <n>` | Maximum depth to search (0 = unlimited) |
| `--follow-symlinks` | Follow symbolic links |

### Performance Options
| Option | Short | Description |
|--------|-------|-------------|
| `--threads <n>` | `-j` | Number of threads (0 = auto) |
| `--progress <mode>` | | Progress display: auto, always, never |

### Output Options
| Option | Description |
|--------|-------------|
| `--format <format>` | Output format: human, json, plain |
| `--quiet` | Suppress non-essential output |

## Best Practices

### 🛡️ Safety First
1. **Review Changes Carefully**: Refac shows all changes before applying them
2. **Use Backups**: Enable `--backup` for important files
3. **Test on Copies**: Work on a copy of important directories
4. **Version Control**: Ensure files are committed before major refactoring
5. **Incremental Changes**: Make small, targeted changes rather than large ones

### ⚡ Performance Optimization
1. **Use Filters**: Limit scope with `--include` and `--exclude` patterns
2. **Adjust Threading**: Use `--threads` for optimal performance
3. **Limit Depth**: Use `--max-depth` for deep directory structures
4. **Target Specific Modes**: Use operation modes to limit processing scope

### 🎯 Effective Patterns
1. **Be Specific**: Use precise patterns to avoid unintended matches
2. **Case Sensitivity**: Be aware of case-sensitive matching behavior
3. **Escape Special Characters**: Quote strings with special characters
4. **Test Patterns**: Use verbose mode to verify pattern matching behavior

### 📋 Workflow Integration
1. **CI/CD Integration**: Use JSON output for automated workflows
2. **Script Integration**: Use exit codes for error handling
3. **Monitoring**: Use verbose output for debugging and auditing
4. **Documentation**: Document large refactoring operations for team awareness

## Troubleshooting

### Common Issues

**"Permission denied" errors**
```bash
# Check file permissions
ls -la affected_file.txt

# Run with appropriate permissions
sudo refac . "oldname" "newname"  # Use carefully
```

**"No changes found" when changes expected**
```bash
# Use verbose mode to see what's being processed
refac . "oldname" "newname" --verbose

# Check case sensitivity
refac . "OldName" "NewName"  # vs "oldname" "newname"

# Verify include/exclude patterns
refac . "oldname" "newname" --include "*.txt" --verbose
```

**"Naming collision detected"**
```bash
# Review the collision report (shown automatically)
refac . "oldname" "newname" --verbose

# Resolve conflicts manually before proceeding
mv conflicting_file.txt conflicting_file_backup.txt
refac . "oldname" "newname"
```

**Binary files not being processed**
```bash
# This is by design for safety
# Use verbose mode to see which files are skipped
refac . "oldname" "newname" --verbose --content-only
```

### Debug Mode
For detailed debugging information:

```bash
# Maximum verbosity (shows preview automatically)
refac . "oldname" "newname" --verbose --progress always

# Check specific file processing
refac specific_file.txt "oldname" "newname" --verbose
```

## Integration Examples

### With Git Workflows
```bash
# Safe refactoring workflow
git checkout -b refactor-api-names
git add .
git commit -m "Checkpoint before refactoring"

refac ./src "old_api" "new_api" --verbose
refac ./src "old_api" "new_api" --backup

git add .
git commit -m "Refactor API names from old_api to new_api"
```

### With Build Systems
```bash
# Update build configurations
refac . "old_target_name" "new_target_name" \
  --include "Makefile" \
  --include "*.cmake" \
  --include "*.toml" \
  --content-only
```

### With CI/CD Pipelines
```bash
#!/bin/bash
# Automated refactoring script
set -e

# Validate refactoring first (using assume-yes for non-interactive)
if refac ./src "$OLD_NAME" "$NEW_NAME" --assume-yes --format json > refac_plan.json; then
    echo "Refactoring plan validated"
    
    # Apply changes
    refac ./src "$OLD_NAME" "$NEW_NAME" --format json > refac_result.json
    
    # Verify success
    if [ $? -eq 0 ]; then
        echo "Refactoring completed successfully"
        exit 0
    else
        echo "Refactoring failed"
        exit 1
    fi
else
    echo "Refactoring validation failed"
    exit 1
fi
```

The refac tool provides comprehensive, safe, and efficient string replacement capabilities for any scale of refactoring operation, from small tweaks to large-scale codebase transformations.