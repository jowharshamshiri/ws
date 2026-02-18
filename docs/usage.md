---
layout: default
title: Usage Guide
---

# Usage Guide

This guide covers all aspects of using the Workspace tool suite for string replacement, line analysis, project cleanup, version management, and cross-project version stamping.

## Tools Overview

### Refactor - String Replacement
```bash
ws refactor <ROOT_DIR> <OLD_STRING> <NEW_STRING> [OPTIONS]
```

### Ldiff - Line Difference Visualizer
```bash
ws ldiff [SUBSTITUTE_CHAR]
```

### Scrap - Local Trash
```bash
ws scrap [PATH...] [SUBCOMMAND] [OPTIONS]
```

### Unscrap - File Restoration
```bash
ws unscrap [NAME] [OPTIONS]
```

### Git Integration - Version Management
```bash
ws git <SUBCOMMAND> [OPTIONS]
```

### Version - Database-Driven Versioning
```bash
ws version <SUBCOMMAND> [OPTIONS]
```

### Wstemplate - Cross-Project Version Stamping
```bash
ws wstemplate <SUBCOMMAND> [OPTIONS]
```

## Refactor - String Replacement

### Command Syntax

```bash
ws refactor <ROOT_DIR> <OLD_STRING> <NEW_STRING> [OPTIONS]
```

- `ROOT_DIR`: Directory to search in (use `.` for current directory)
- `OLD_STRING`: String to find and replace
- `NEW_STRING`: Replacement string

### Simple Examples

```bash
ws refactor . "oldname" "newname"
ws refactor ./src "OldClass" "NewClass"
ws refactor . "oldname" "newname" --verbose
```

## Operation Modes

Refactor operates on two levels by default:

1. **Name Replacement**: Renames files and directories
2. **Content Replacement**: Replaces strings inside text files

### Mode Flags

```bash
ws refactor . "oldname" "newname" --names-only     # Only rename files/directories
ws refactor . "oldname" "newname" --content-only   # Only replace content
ws refactor . "oldname" "newname" --files-only     # Only process files
ws refactor . "oldname" "newname" --dirs-only      # Only process directories
```

## Safety Features

### Dry Run Mode

```bash
ws refactor . "oldname" "newname" --verbose
ws refactor . "oldname" "newname" --verbose --verbose
```

### Backup Files

```bash
ws refactor . "oldname" "newname" --backup
```

### Force Mode

```bash
ws refactor . "oldname" "newname" --force
```

## Filtering Options

### Include Patterns

```bash
ws refactor . "oldname" "newname" --include "*.rs"
ws refactor . "oldname" "newname" --include "*.rs" --include "*.toml"
```

### Exclude Patterns

```bash
ws refactor . "oldname" "newname" --exclude "*.log"
ws refactor . "oldname" "newname" --exclude "target/*" --exclude "*.log"
```

### Combining Filters

```bash
ws refactor ./src "oldname" "newname" \
  --include "*.rs" \
  --include "*.toml" \
  --exclude "*test*"
```

## Advanced Options

### Depth Control

```bash
ws refactor . "oldname" "newname" --max-depth 1
ws refactor . "oldname" "newname" --max-depth 3
```

### Threading

```bash
ws refactor . "oldname" "newname" --threads 8
ws refactor . "oldname" "newname" --threads 0    # Auto-detect
```

### Pattern Matching

```bash
ws refactor . "oldname" "newname" --ignore-case
ws refactor . "old_\\w+" "new_name" --regex
ws refactor . "old.*name" "newname" --regex --ignore-case
```

### Output Formats

```bash
ws refactor . "oldname" "newname"                   # Human-readable (default)
ws refactor . "oldname" "newname" --format json     # Machine-readable
ws refactor . "oldname" "newname" --format plain    # No colors
```

## Ldiff - Line Difference Visualizer

The `ldiff` tool processes input lines, replacing repeated tokens with a substitute character to highlight patterns and differences.

### Basic Usage

```bash
echo -e "hello world\nhello universe" | ws ldiff
# Output:
# hello world
# ░░░░░ universe

echo -e "test line\ntest another" | ws ldiff "*"
# Output:
# test line
# **** another
```

### Log Analysis

```bash
tail -f /var/log/syslog | ws ldiff
cat /var/log/nginx/access.log | ws ldiff
journalctl -u myapp | ws ldiff
```

### Real-time Monitoring

```bash
tail -f /var/log/app1.log /var/log/app2.log | ws ldiff
dmesg -w | ws ldiff
```

## Scrap - Local Trash Can

The scrap tool provides a local trash can using a `.scrap` folder.

### Basic Operations

```bash
ws scrap temp_file.txt old_directory/
ws scrap list
ws scrap find "*.log"
ws scrap clean --days 30
ws scrap archive --remove
```

For detailed information, see the [Scrap Tool Guide]({{ '/scrap-guide/' | relative_url }}).

## Unscrap - File Restoration

```bash
ws unscrap                              # Restore last item
ws unscrap filename.txt                 # Restore specific file
ws unscrap file.txt --to /new/location/ # Custom destination
ws unscrap file.txt --force             # Overwrite existing
```

For detailed information, see the [Unscrap Tool Guide]({{ '/unscrap-guide/' | relative_url }}).

## Version Management

### Git Hook Integration

```bash
ws git install           # Install pre-commit hook
ws git show              # Show current version
ws git status            # Check configuration
ws git uninstall         # Remove hook
```

### Manual Version Update

```bash
ws update                # Update version and render templates
ws update --git-add      # Also stage changed files
ws update --no-git       # Skip git integration
```

### Database-Driven Versioning

```bash
ws version show          # Display current version breakdown
ws version major 2       # Set major version to 2
ws version tag           # Create git tag
ws version info          # Show calculation details
```

### Version Format

`{major}.{minor}.{patch}` where:
- **Major**: Set via `ws version major` (stored in database)
- **Minor**: Total commits in the repository
- **Patch**: Total line changes (additions + deletions)

### Configuration

Create `.st8.json` in your repository root:

```json
{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt"
}
```

## Wstemplate - Cross-Project Version Stamping

### Overview

`.wstemplate` files are Tera templates that render to the file with the `.wstemplate` suffix stripped (e.g., `Cargo.toml.wstemplate` renders to `Cargo.toml`). They're rendered automatically during `ws update`.

### Setup

Each project needs a single wstemplate entry: its alias and a scan root.

```bash
ws wstemplate add /path/to/workspace     # Set scan root for this project
ws wstemplate list-entries               # Verify entry
```

The alias is auto-derived from the project directory name (e.g., `my-project` becomes `my_project`). Override with `--alias`:

```bash
ws wstemplate add /path/to/workspace --alias mylib
```

### Template Syntax

Templates use Tera syntax. Available variables:

```
{{ project.version }}                  # This project's version
{{ project.name }}                     # This project's name
{{ projects.other_lib.version }}       # Another project's version
{{ datetime.date }}                    # Current date (YYYY-MM-DD)
```

### Dynamic Cross-Project Resolution

When rendering, the engine scans the root for all `.ws/state.json` files to discover peer projects. No explicit cross-project entries are needed. If a template references `{{ projects.tagged_urn_js.version }}`, the engine finds `tagged_urn_js`'s project root and reads its `version.txt`.

### Error Handling

- **Unresolvable alias**: Hard error listing all known aliases
- **Missing `version.txt`**: Hard error — run `ws update` in the dependency first
- **Multiple entries in state.json**: Hard error — single-entry model enforced

### Commands

```bash
ws wstemplate add /path/to/root              # Set scan root
ws wstemplate add /path/to/root --alias lib  # With custom alias
ws wstemplate list-entries                    # Show entry
ws wstemplate list                            # List relevant templates
ws wstemplate render                          # Render all relevant
ws wstemplate remove my_alias                 # Remove entry
```

## Tool Integration

### Combined Workflows

```bash
# Development workflow
ws git install                              # Set up versioning
ws scrap temp_* *.log build/                # Clear workspace
ws refactor . "OldClass" "NewClass" --verbose # Preview changes
ws refactor . "OldClass" "NewClass"          # Apply changes
git add . && git commit -m "Refactor class"  # Auto-version bump

# Cross-project version update
ws wstemplate add /path/to/workspace         # Set scan root
ws update --git-add                          # Update + render + stage
```

## Next Steps

- [Command Reference]({{ '/api-reference/' | relative_url }}) for full option details
- [Scrap Guide]({{ '/scrap-guide/' | relative_url }}) for file management
- [Unscrap Guide]({{ '/unscrap-guide/' | relative_url }}) for restoration workflows
- [St8 Guide]({{ '/st8-guide/' | relative_url }}) for version management and wstemplate setup
- [Examples]({{ '/examples/' | relative_url }}) for more real-world scenarios
