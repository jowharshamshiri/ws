---
layout: default
title: Refactor - Code Refactoring Guide
---

# Refactor - Code Refactoring Guide

The `ws refactor` tool is a string replacement engine designed for safe code refactoring and content modification across codebases.

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
ws refactor <directory> <old_string> <new_string> [OPTIONS]
```

### First Steps
```bash
# Refac always previews changes and asks for confirmation
ws refactor ./src "OldClassName" "NewClassName" --verbose

# Apply with backup for safety
ws refactor ./src "OldClassName" "NewClassName" --backup

# Target specific file types
ws refactor ./src "old_api" "new_api" --include "*.rs" --include "*.toml"
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
ws refactor . "oldname" "newname"

# Verbose output with detailed information
ws refactor . "oldname" "newname" --verbose

# JSON output for scripting (still shows preview)
ws refactor . "oldname" "newname" --format json
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
ws refactor . "oldname" "newname" --threads 8

# Auto-detect optimal thread count
ws refactor . "oldname" "newname" --threads 0

# Progress tracking for long operations
ws refactor . "oldname" "newname" --progress always
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
ws refactor . "oldproject" "newproject" --names-only

# Only replace content (skip renaming)
ws refactor . "old.api.com" "new.api.com" --content-only

# Only process files (skip directories)
ws refactor . "oldname" "newname" --files-only

# Only process directories (skip files)
ws refactor . "oldname" "newname" --dirs-only
```

### 📁 Pattern Filtering
Precise control over which files are processed:

```bash
# Include specific file types
ws refactor . "oldname" "newname" \
  --include "*.rs" \
  --include "*.toml" \
  --include "*.md"

# Exclude unwanted areas
ws refactor . "oldname" "newname" \
  --exclude "target/*" \
  --exclude "*.log" \
  --exclude ".git/*"

# Complex filtering
ws refactor ./src "OldStruct" "NewStruct" \
  --include "*.rs" \
  --exclude "*/tests/*" \
  --exclude "*/examples/*"
```

### 🏗️ Directory Depth Control
Manage traversal depth for large projects:

```bash
# Limit to current directory only
ws refactor . "oldname" "newname" --max-depth 1

# Search 3 levels deep
ws refactor . "oldname" "newname" --max-depth 3

# Unlimited depth (default)
ws refactor . "oldname" "newname" --max-depth 0
```

## Advanced Features

### 💾 Backup and Recovery
Safe modification with automatic backups:

```bash
# Create backups before modifying files
ws refactor . "oldname" "newname" --backup

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
ws refactor . "oldname" "newname" --force

# Case-sensitive matching (default)
ws refactor . "OldName" "NewName"

# Show detailed error information
ws refactor . "oldname" "newname" --verbose
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
ws refactor . "oldname" "newname"

# Handles platform-specific path separators
ws refactor . "old\\path" "new/path"  # Windows
ws refactor . "old/path" "new/path"   # Unix-like
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
# 1. Review the migration (ws refactor shows changes before applying)
ws refactor ./src "old_api::Client" "new_api::Client" --verbose

# 2. Update import statements
ws refactor ./src "use old_api" "use new_api" --content-only --include "*.rs"

# 3. Update function calls
ws refactor ./src "old_api::connect" "new_api::connect" --content-only

# 4. Update configuration files
ws refactor ./config "old_api_endpoint" "new_api_endpoint" --content-only --include "*.toml"
```

### 🏢 Project Rebranding
Rename a project throughout the codebase:

```bash
# 1. Update package names
ws refactor . "oldproject" "newproject" --include "Cargo.toml" --include "package.json"

# 2. Update file names and directory structure
ws refactor . "oldproject" "newproject" --names-only

# 3. Update content references
ws refactor . "oldproject" "newproject" --content-only --exclude "target/*"

# 4. Update documentation
ws refactor ./docs "OldProject" "NewProject" --include "*.md"
```

### 🏗️ Refactoring Code Structure
Reorganize and rename code components:

```bash
# 1. Rename a module throughout the codebase
ws refactor ./src "user_service" "account_service" --include "*.rs"

# 2. Update struct names
ws refactor ./src "UserData" "AccountData" --content-only --include "*.rs"

# 3. Update configuration keys
ws refactor ./config "user_" "account_" --content-only --include "*.toml" --include "*.yaml"
```

### 🌍 Configuration Updates
Update configuration across multiple environments:

```bash
# Update API endpoints across all configs
ws refactor ./config "api.old.com" "api.new.com" \
  --content-only \
  --include "*.toml" \
  --include "*.yaml" \
  --include "*.json" \
  --include "*.env"

# Update database connection strings
ws refactor ./config "old_database" "new_database" \
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
ws refactor . "oldname" "newname" --format json
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
ws refactor . "oldname" "newname" --format plain
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
sudo ws refactor . "oldname" "newname"  # Use carefully
```

**"No changes found" when changes expected**
```bash
# Use verbose mode to see what's being processed
ws refactor . "oldname" "newname" --verbose

# Check case sensitivity
ws refactor . "OldName" "NewName"  # vs "oldname" "newname"

# Verify include/exclude patterns
ws refactor . "oldname" "newname" --include "*.txt" --verbose
```

**"Naming collision detected"**
```bash
# Review the collision report (shown automatically)
ws refactor . "oldname" "newname" --verbose

# Resolve conflicts manually before proceeding
mv conflicting_file.txt conflicting_file_backup.txt
ws refactor . "oldname" "newname"
```

**Binary files not being processed**
```bash
# This is by design for safety
# Use verbose mode to see which files are skipped
ws refactor . "oldname" "newname" --verbose --content-only
```

### Debug Mode
For detailed debugging information:

```bash
# Maximum verbosity (shows preview automatically)
ws refactor . "oldname" "newname" --verbose --progress always

# Check specific file processing
ws refactor specific_file.txt "oldname" "newname" --verbose
```

## Integration Examples

### With Git Workflows
```bash
# Safe refactoring workflow
git checkout -b refactor-api-names
git add .
git commit -m "Checkpoint before refactoring"

ws refactor ./src "old_api" "new_api" --verbose
ws refactor ./src "old_api" "new_api" --backup

git add .
git commit -m "Refactor API names from old_api to new_api"
```

### With Build Systems
```bash
# Update build configurations
ws refactor . "old_target_name" "new_target_name" \
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
if ws refactor ./src "$OLD_NAME" "$NEW_NAME" --assume-yes --format json > refac_plan.json; then
    echo "Refactoring plan validated"
    
    # Apply changes
    ws refactor ./src "$OLD_NAME" "$NEW_NAME" --format json > refac_result.json
    
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

The `ws refactor` tool provides comprehensive, safe, and efficient string replacement capabilities for any scale of refactoring operation, from small tweaks to large-scale codebase transformations.