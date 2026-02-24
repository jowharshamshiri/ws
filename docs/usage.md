---
layout: default
title: Usage Guide
---

# Usage Guide

This guide covers all aspects of using the Workspace tool suite for string replacement, line analysis, project cleanup, version management, and cross-project version stamping.

## Tools Overview

### Refactor - String Replacement
```bash
wsb refactor <ROOT_DIR> <OLD_STRING> <NEW_STRING> [OPTIONS]
```

### Ldiff - Line Difference Visualizer
```bash
wsb ldiff [SUBSTITUTE_CHAR]
```

### Scrap - Local Trash
```bash
wsb scrap [PATH...] [SUBCOMMAND] [OPTIONS]
```

### Unscrap - File Restoration
```bash
wsb unscrap [NAME] [OPTIONS]
```

### Git Integration - Version Management
```bash
wsb git <SUBCOMMAND> [OPTIONS]
```

### Version - Database-Driven Versioning
```bash
wsb version <SUBCOMMAND> [OPTIONS]
```

### Wstemplate - Cross-Project Version Stamping
```bash
wsb wstemplate <SUBCOMMAND> [OPTIONS]
```

## Refactor - String Replacement

### Command Syntax

```bash
wsb refactor <ROOT_DIR> <OLD_STRING> <NEW_STRING> [OPTIONS]
```

- `ROOT_DIR`: Directory to search in (use `.` for current directory)
- `OLD_STRING`: String to find and replace
- `NEW_STRING`: Replacement string

### Simple Examples

```bash
wsb refactor . "oldname" "newname"
wsb refactor ./src "OldClass" "NewClass"
wsb refactor . "oldname" "newname" --verbose
```

## Operation Modes

Refactor operates on two levels by default:

1. **Name Replacement**: Renames files and directories
2. **Content Replacement**: Replaces strings inside text files

### Mode Flags

```bash
wsb refactor . "oldname" "newname" --names-only     # Only rename files/directories
wsb refactor . "oldname" "newname" --content-only   # Only replace content
wsb refactor . "oldname" "newname" --files-only     # Only process files
wsb refactor . "oldname" "newname" --dirs-only      # Only process directories
```

## Safety Features

### Dry Run Mode

```bash
wsb refactor . "oldname" "newname" --verbose
wsb refactor . "oldname" "newname" --verbose --verbose
```

### Backup Files

```bash
wsb refactor . "oldname" "newname" --backup
```

### Force Mode

```bash
wsb refactor . "oldname" "newname" --force
```

## Filtering Options

### Include Patterns

```bash
wsb refactor . "oldname" "newname" --include "*.rs"
wsb refactor . "oldname" "newname" --include "*.rs" --include "*.toml"
```

### Exclude Patterns

```bash
wsb refactor . "oldname" "newname" --exclude "*.log"
wsb refactor . "oldname" "newname" --exclude "target/*" --exclude "*.log"
```

### Combining Filters

```bash
wsb refactor ./src "oldname" "newname" \
  --include "*.rs" \
  --include "*.toml" \
  --exclude "*test*"
```

## Advanced Options

### Depth Control

```bash
wsb refactor . "oldname" "newname" --max-depth 1
wsb refactor . "oldname" "newname" --max-depth 3
```

### Threading

```bash
wsb refactor . "oldname" "newname" --threads 8
wsb refactor . "oldname" "newname" --threads 0    # Auto-detect
```

### Pattern Matching

```bash
wsb refactor . "oldname" "newname" --ignore-case
wsb refactor . "old_\\w+" "new_name" --regex
wsb refactor . "old.*name" "newname" --regex --ignore-case
```

### Output Formats

```bash
wsb refactor . "oldname" "newname"                   # Human-readable (default)
wsb refactor . "oldname" "newname" --format json     # Machine-readable
wsb refactor . "oldname" "newname" --format plain    # No colors
```

## Ldiff - Line Difference Visualizer

The `ldiff` tool processes input lines, replacing repeated tokens with a substitute character to highlight patterns and differences.

### Basic Usage

```bash
echo -e "hello world\nhello universe" | wsb ldiff
# Output:
# hello world
# ░░░░░ universe

echo -e "test line\ntest another" | wsb ldiff "*"
# Output:
# test line
# **** another
```

### Log Analysis

```bash
tail -f /var/log/syslog | wsb ldiff
cat /var/log/nginx/access.log | wsb ldiff
journalctl -u myapp | wsb ldiff
```

### Real-time Monitoring

```bash
tail -f /var/log/app1.log /var/log/app2.log | wsb ldiff
dmesg -w | wsb ldiff
```

## Scrap - Local Trash Can

The scrap tool provides a local trash can using a `.scrap` folder.

### Basic Operations

```bash
wsb scrap temp_file.txt old_directory/
wsb scrap list
wsb scrap find "*.log"
wsb scrap clean --days 30
wsb scrap archive --remove
```

For detailed information, see the [Scrap Tool Guide]({{ '/scrap-guide/' | relative_url }}).

## Unscrap - File Restoration

```bash
wsb unscrap                              # Restore last item
wsb unscrap filename.txt                 # Restore specific file
wsb unscrap file.txt --to /new/location/ # Custom destination
wsb unscrap file.txt --force             # Overwrite existing
```

For detailed information, see the [Unscrap Tool Guide]({{ '/unscrap-guide/' | relative_url }}).

## Version Management

### Git Hook Integration

```bash
wsb git install           # Install pre-commit hook
wsb git show              # Show current version
wsb git status            # Check configuration
wsb git uninstall         # Remove hook
```

### Manual Version Update

```bash
wsb update                # Update version and render templates
wsb update --git-add      # Also stage changed files
wsb update --no-git       # Skip git integration
```

### Database-Driven Versioning

```bash
wsb version show          # Display current version breakdown
wsb version major 2       # Set major version to 2
wsb version tag           # Create git tag
wsb version info          # Show calculation details
```

### Version Format

`{major}.{minor}.{patch}` where:
- **Major**: Set via `wsb version major` (stored in database)
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

`.wstemplate` files are Tera templates that render to the file with the `.wstemplate` suffix stripped (e.g., `Cargo.toml.wstemplate` renders to `Cargo.toml`). They're rendered automatically during `wsb update`.

### Setup

Each project needs a single wstemplate entry: its alias and a scan root.

```bash
wsb wstemplate add /path/to/workspace     # Set scan root for this project
wsb wstemplate list-entries               # Verify entry
```

The alias is auto-derived from the project directory name (e.g., `my-project` becomes `my_project`). Override with `--alias`:

```bash
wsb wstemplate add /path/to/workspace --alias mylib
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

When rendering, the engine scans the root for all `.wsb/state.json` files to discover peer projects. No explicit cross-project entries are needed. If a template references `{{ projects.tagged_urn_js.version }}`, the engine finds `tagged_urn_js`'s project root and reads its `version.txt`.

### Error Handling

- **Unresolvable alias**: Hard error listing all known aliases
- **Missing `version.txt`**: Hard error — run `wsb update` in the dependency first
- **Multiple entries in state.json**: Hard error — single-entry model enforced

### Commands

```bash
wsb wstemplate add /path/to/root              # Set scan root
wsb wstemplate add /path/to/root --alias lib  # With custom alias
wsb wstemplate list-entries                    # Show entry
wsb wstemplate list                            # List relevant templates
wsb wstemplate render                          # Render all relevant
wsb wstemplate remove my_alias                 # Remove entry
```

## Tool Integration

### Combined Workflows

```bash
# Development workflow
wsb git install                              # Set up versioning
wsb scrap temp_* *.log build/                # Clear workspace
wsb refactor . "OldClass" "NewClass" --verbose # Preview changes
wsb refactor . "OldClass" "NewClass"          # Apply changes
git add . && git commit -m "Refactor class"  # Auto-version bump

# Cross-project version update
wsb wstemplate add /path/to/workspace         # Set scan root
wsb update --git-add                          # Update + render + stage
```

## Next Steps

- [Command Reference]({{ '/api-reference/' | relative_url }}) for full option details
- [Scrap Guide]({{ '/scrap-guide/' | relative_url }}) for file management
- [Unscrap Guide]({{ '/unscrap-guide/' | relative_url }}) for restoration workflows
- [St8 Guide]({{ '/st8-guide/' | relative_url }}) for version management and wstemplate setup
- [Examples]({{ '/examples/' | relative_url }}) for more real-world scenarios
