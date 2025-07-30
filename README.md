# Nomion

A tool suite for developers and system administrators for file operations, line analysis, version management, and development workflow automation.

## Tools Overview

| Tool | Purpose | Primary Use Cases |
|------|---------|-------------------|
| **refac** | Code refactoring and string replacement | API migrations, bulk renames, content updates |
| **ldiff** | Log pattern analysis and visualization | Debug analysis, pattern recognition, monitoring |
| **scrap** | Safe file disposal with metadata | Experimental code cleanup, temporary file management |
| **unscrap** | File restoration and recovery | Accident recovery, experiment rollback |
| **verbump** | Version management | Release automation, version consistency |

Built for safety, reliability, and performance with extensive testing (231 tests across 8 test suites).

## Testing & Quality Assurance

### Test Suite
- 231 tests across all tools and scenarios
- 8 test files covering different aspects:
  - `integration_tests.rs` - End-to-end tool integration (15 tests)
  - `refac_concurrency_tests.rs` - Multi-threading safety (9 tests)
  - `refac_edge_cases_tests.rs` - Complex scenarios (14 tests)
  - `refac_empty_directory_tests.rs` - Directory handling edge cases (8 tests)
  - `refac_encoding_tests.rs` - UTF-8 and encoding safety (7 tests)
  - `scrap_advanced_integration_tests.rs` - Scrap workflows (21 tests)
  - `scrap_integration_tests.rs` - Core scrap functionality (18 tests)
  - `verbump_integration_tests.rs` - Version management (25 tests)

### Quality Standards
- Zero compilation warnings across platforms
- Memory safety through Rust's ownership model
- Performance testing with large file sets
- Pre-operation validation to prevent mid-execution failures
- Proper operation ordering to prevent race conditions
- UTF-8 and encoding issue detection

### Edge Case Coverage
- Concurrency: High-thread scenarios, parallel processing
- Encoding: UTF-8, invalid encodings, BOM handling, mixed encodings
- File System: Deep nesting, long filenames, special characters
- Permissions: Read-only files, permission changes, restricted directories
- Edge Cases: Empty directories, symlinks, hidden files, binary detection

## Core Features

### Refac - Code Refactoring

**String Replacement Engine**
- Language-aware processing for code structure
- Multi-mode operations: names-only, content-only, files-only, dirs-only
- Collision detection to prevent overwrites and conflicts
- Binary file protection with automatic detection
- Pattern filtering with glob and regex support
- Parallel processing for large codebases

**Safety & Validation**
- Pre-validation tests all operations before execution
- Dry-run mode to preview changes
- Optional file backups before modifications
- Atomic operations prevent partial failures
- Cross-platform support for Windows, macOS, and Linux

### Ldiff - Log Analysis

**Pattern Recognition Engine**
- Real-time analysis: Process streaming logs with `tail -f` compatibility
- ANSI color preservation: Maintains terminal formatting and colors
- Customizable visualization: User-defined substitute characters
- Performance optimized for large log files

**Use Cases**
- System monitoring: Track patterns in system logs and metrics
- Debug analysis: Identify recurring patterns in application logs
- Security monitoring: Detect suspicious patterns in auth logs
- Development: Analyze test output and build logs for patterns

### Scrap - File Management

**Local Trash System**
- Metadata tracking: Preserves original locations, timestamps, and context
- Conflict resolution: Automatic naming to prevent overwrites
- Search and discovery: Find files by name, content, date, or size
- Git integration: Automatic `.gitignore` management
- Archive support: Compress and backup scrap contents

**Operations**
- Cleanup: Age-based removal with dry-run preview
- Bulk operations: Move multiple files/directories efficiently
- Restoration metadata: Original path and context preservation
- Size management: Track and report storage usage

### Unscrap - File Recovery

**Restoration System**
- Automatic recovery: Restore files to original locations using metadata
- Custom destinations: Flexible restoration to alternative paths
- Conflict handling: Resolution of destination conflicts
- Undo operations: Quick reversal of recent scrap operations

**Recovery Features**
- Last-action undo: Reverse the most recent scrap operation
- Selective restoration: Restore specific files from scrap history
- Batch recovery: Restore multiple related files simultaneously
- Path reconstruction: Automatically recreate directory structures

### Verbump - Version Management

**Git-Integrated Versioning**
- Automatic bumping: Version increments on commits via git hooks
- Multi-format support: Handles Cargo.toml, package.json, version.txt, and more
- Semantic versioning: Version increment strategies
- Project detection: Automatic configuration for different project types

**Version Control**
- Audit logging: History of version changes
- Configuration management: Per-project settings
- Status monitoring: Version and configuration status
- Integration ready: Designed for CI/CD and release automation

## Installation

### Quick Install
```bash
git clone https://github.com/jowharshamshiri/nomion
cd nomion
./install.sh
```

**Installation Options:**
```bash
./install.sh --help                    # See all options
./install.sh -d /usr/local/bin         # System-wide installation
./install.sh --force                   # Force reinstall
./install.sh --verbose                 # Detailed output
```

### Manual Installation
```bash
# Build from source
cargo build --release

# Install all tools
cargo install --path .

# Install specific tools
cargo install --path . --bin refac
cargo install --path . --bin ldiff
cargo install --path . --bin scrap
cargo install --path . --bin unscrap
cargo install --path . --bin verbump
```

### Uninstallation
```bash
./uninstall.sh                         # Remove all tools
./uninstall.sh -d /usr/local/bin       # Remove from custom directory
```

## Quick Start Examples

### Developer Refactoring Workflow
```bash
# 1. Preview changes safely (recommended first step)
refac ./src "OldClassName" "NewClassName" --dry-run --verbose

# 2. Apply the refactoring with backups
refac ./src "OldClassName" "NewClassName" --backup

# 3. Update configuration files separately
refac ./config "old.api.url" "new.api.url" --content-only --include "*.toml"
```

### Log Analysis & Monitoring
```bash
# Real-time log pattern analysis
tail -f /var/log/system.log | ldiff

# Compare deployment logs
cat deploy-1.log | ldiff > patterns-1.txt
cat deploy-2.log | ldiff > patterns-2.txt

# Custom substitute character for different log sources
journalctl -f | ldiff "■"
```

### File Management
```bash
# Move experimental code to safe storage
scrap experimental_feature/ temp_logs/ *.bak

# Review what you've stored
scrap list --sort date

# Find specific files later
scrap find "*.rs" --content "TODO"

# Restore when needed
unscrap experimental_feature/
```

### Version Management
```bash
# Set up automatic versioning
verbump install

# Check version status
verbump show

# Version automatically updates on commits
git add . && git commit -m "Add new feature"  # Auto-increments version
```

## Command Reference

### Refac - String Replacement & Refactoring

**Basic Syntax:**
```bash
refac <directory> <old_string> <new_string> [OPTIONS]
```

**Essential Options:**
| Option | Description |
|--------|-------------|
| `--dry-run` | Preview changes without applying them |
| `--backup` | Create backup files before modification |
| `--verbose` | Show detailed operation information |
| `--include <pattern>` | Include only files matching pattern |
| `--exclude <pattern>` | Exclude files matching pattern |
| `--names-only` | Only rename files/directories, skip content |
| `--content-only` | Only replace content, skip renaming |
| `--threads <n>` | Number of threads for parallel processing |

### Ldiff - Log Pattern Analysis

**Basic Syntax:**
```bash
ldiff [substitute_char]
```

**Usage Patterns:**
```bash
command | ldiff              # Default substitute character
command | ldiff "*"          # Custom substitute character
tail -f logfile | ldiff      # Real-time log monitoring
```

### Scrap - File Management

**Operations:**
```bash
scrap [files...]             # Move files to .scrap (default: list contents)
scrap list [--sort name|date|size]  # List scrap contents
scrap find <pattern> [--content]    # Search in scrap
scrap clean [--days N]       # Remove old items
scrap archive [--output file] [--remove]  # Archive contents
scrap purge [--force]        # Remove all items
```

### Unscrap - File Restoration

**Operations:**
```bash
unscrap                      # Restore last scrapped item
unscrap <filename>           # Restore specific file
unscrap <filename> --to <path>  # Restore to custom location
unscrap --force              # Overwrite existing files
```

### Verbump - Version Management

**Operations:**
```bash
verbump install [--force]    # Install git hook
verbump show                 # Display version information
verbump status               # Check configuration
verbump update [--patch|--minor|--major]  # Manual version bump
verbump uninstall            # Remove git hook
```

## Documentation

**Complete Documentation:** [https://jowharshamshiri.github.io/nomion/](https://jowharshamshiri.github.io/nomion/)

**Quick Links:**
- [Installation Guide](https://jowharshamshiri.github.io/nomion/installation/) - Setup instructions
- [Getting Started](https://jowharshamshiri.github.io/nomion/getting-started/) - Tutorial walkthrough  
- [Usage Guide](https://jowharshamshiri.github.io/nomion/usage/) - Examples
- [API Reference](https://jowharshamshiri.github.io/nomion/api-reference/) - Command documentation
- [Examples](https://jowharshamshiri.github.io/nomion/examples/) - Real-world use cases

**Tool-Specific Guides:**
- [Scrap Guide](https://jowharshamshiri.github.io/nomion/scrap-guide/) - File management
- [Ldiff Guide](https://jowharshamshiri.github.io/nomion/ldiff-guide/) - Log analysis techniques
- [Verbump Guide](https://jowharshamshiri.github.io/nomion/verbump-guide/) - Version management setup

## Detailed Usage Examples

### Basic Syntax

```bash
refac <root_dir> <old_string> <new_string> [options]
```

### Examples

#### Basic Replacement

```bash
# Replace "oldname" with "newname" in current directory
refac . "oldname" "newname"

# Process specific directory
refac /path/to/project "OldClass" "NewClass"
```

#### Dry Run (Preview Changes)

```bash
# See what would be changed without making modifications
refac . "oldname" "newname" --dry-run
```

#### Operation Modes

```bash
# Only rename files and directories (skip content)
refac . "oldname" "newname" --names-only

# Only replace content (skip renaming)
refac . "oldname" "newname" --content-only

# Only process files (skip directories)
refac . "oldname" "newname" --files-only

# Only process directories (skip files)
refac . "oldname" "newname" --dirs-only
```

#### Features

```bash
# Force operation without confirmation
refac . "oldname" "newname" --force

# Create backups before modifying files
refac . "oldname" "newname" --backup

# Verbose output with detailed information
refac . "oldname" "newname" --verbose

# Limit directory traversal depth
refac . "oldname" "newname" --max-depth 3

# Use multiple threads for faster processing
refac . "oldname" "newname" --threads 8
```

#### Pattern Filtering

```bash
# Include only specific file types
refac . "oldname" "newname" --include "*.rs" --include "*.toml"

# Exclude specific patterns
refac . "oldname" "newname" --exclude "*.log" --exclude "target/*"

# Include hidden files
refac . "oldname" "newname" --include ".*"
```

#### Output Formats

```bash
# JSON output for scripting
refac . "oldname" "newname" --format json

# Plain text output
refac . "oldname" "newname" --format plain

# Human-readable output (default)
refac . "oldname" "newname" --format human
```

## Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--dry-run` | `-d` | Show what would be changed without making changes |
| `--force` | `-f` | Skip confirmation prompt |
| `--verbose` | `-v` | Show detailed output |
| `--backup` | `-b` | Create backup files before modifying content |
| `--files-only` | | Only process files (skip directories) |
| `--dirs-only` | | Only process directories (skip files) |
| `--names-only` | | Skip content replacement, only rename files/directories |
| `--content-only` | | Skip file/directory renaming, only replace content |
| `--follow-symlinks` | | Follow symbolic links |
| `--max-depth <N>` | | Maximum depth to search (0 = unlimited) |
| `--threads <N>` | `-j` | Number of threads to use (0 = auto) |
| `--include <PATTERN>` | | Include only files matching pattern |
| `--exclude <PATTERN>` | | Exclude files matching pattern |
| `--format <FORMAT>` | | Output format: human, json, plain |
| `--progress <MODE>` | | Progress display: auto, always, never |
| `--ignore-case` | `-i` | Ignore case when matching patterns |
| `--regex` | `-r` | Use regex patterns instead of literal strings |
| `--help` | `-h` | Show help information |
| `--version` | `-V` | Show version information |

## Safety Considerations

### What Gets Modified

- **File contents**: Text files only (binary files are automatically skipped)
- **File names**: Any file containing the target string
- **Directory names**: Any directory containing the target string

### What Doesn't Get Modified

- **Binary files**: Automatically detected and skipped for content replacement
- **The tool itself**: Self-modification is prevented
- **Symlink targets**: Unless `--follow-symlinks` is specified

### Collision Prevention

The tool checks for potential naming conflicts before making changes:

- Files/directories that would overwrite existing items
- Multiple sources trying to rename to the same target
- Case-only differences on case-insensitive filesystems

### Best Practices

1. **Always use dry-run first**: `--dry-run` to preview changes
2. **Use backups for important files**: `--backup` option
3. **Test on a copy**: Work on a backup of important directories
4. **Use version control**: Ensure your files are committed before running
5. **Be specific with patterns**: Use include/exclude patterns to limit scope

### Ldiff Tool

The `ldiff` tool processes input lines, replacing repeated tokens with a substitute character to highlight patterns and differences. Perfect for log analysis and command output examination.

#### Core Features

- **Pattern Recognition**: Automatically identifies repeated tokens between lines
- **ANSI Preservation**: Maintains terminal colors and formatting
- **Customizable**: Use any character as a substitute
- **Real-time**: Works with streaming input like `tail -f`

#### Examples

```bash
# Basic usage with default substitute character
echo -e "hello world\nhello universe" | ldiff
# Output:
# hello world
# ░░░░░ universe

# Log analysis
tail -f /var/log/system.log | ldiff

# Custom substitute character
find /usr/local -type f | ldiff "*"

# Analyze web server logs
cat /var/log/nginx/access.log | ldiff "■"

# Monitor processes
ps aux | ldiff
```

#### Use Cases

- **Log Analysis**: Find patterns in system and application logs
- **Security Monitoring**: Identify repeated patterns in auth logs
- **Performance Monitoring**: Track recurring patterns in metrics
- **System Administration**: Analyze command output for patterns
- **Development**: Debug application output patterns

### Scrap Tool

The `scrap` tool provides a local trash can using a `.scrap` folder for files you want to delete, with features for listing, searching, and cleaning up deleted files.

#### Core Features

- **Local trash can**: Move files/directories to `.scrap` with automatic conflict resolution
- **Automatic setup**: Creates `.scrap` directory and updates `.gitignore` automatically
- **Metadata tracking**: Remembers original file locations and timestamps
- **Multiple operation modes**: List, clean, search, archive, and restore capabilities
- **Git integration**: Automatically excludes `.scrap/` from version control
- **Safety features**: Never overwrites existing files, provides confirmation prompts

#### Basic Usage

```bash
# List contents of .scrap folder (default behavior)
scrap

# Move a file to .scrap folder
scrap file.txt

# Move a directory to .scrap folder  
scrap old_code/
```

#### Commands

##### List and Browse

```bash
# List contents (default when no arguments)
scrap
scrap list

# Sort by different criteria
scrap list --sort name
scrap list --sort date
scrap list --sort size
```

##### Search and Find

```bash
# Search by filename (supports regex)
scrap find ".*\.log"
scrap find "test.*"

# Search in file contents as well
scrap find "TODO" --content
```

##### Cleaning and Maintenance

```bash
# Remove items older than 30 days (default)
scrap clean

# Remove items older than 7 days
scrap clean --days 7

# Preview what would be removed
scrap clean --days 30 --dry-run

# Remove all items from .scrap folder
scrap purge

# Skip confirmation prompt
scrap purge --force
```

##### Archive and Backup

```bash
# Archive .scrap contents to compressed file
scrap archive

# Archive with custom filename
scrap archive --output backup-2024.tar.gz

# Archive and remove original files
scrap archive --remove
```

#### Examples

```bash
# Move files to local trash can
scrap temp.log debug.txt old_backup/

# Clean up workspace
scrap clean --days 14              # Remove items older than 2 weeks
scrap find "\.tmp$" | head -5       # Find temporary files

# Archive old items
scrap archive --output "archive-$(date +%Y%m%d).tar.gz" --remove

# Handle name conflicts automatically
scrap file.txt  # Creates .scrap/file.txt
scrap file.txt  # Creates .scrap/file_1.txt
scrap file.txt  # Creates .scrap/file_2.txt
```

#### Metadata and History

The scrap tool automatically tracks:

- **Original locations**: Where files came from
- **Timestamps**: When files were scrapped
- **Restore information**: Easy recovery to original locations

```bash
# View items with their original locations
scrap list

# Example output:
# 📄 file.txt              1.2 KB  2 hours ago     from: /path/to/original/file.txt
# 📁 old_project          15.3 MB  1 day ago       from: /home/user/old_project
```

#### Safety Features

- **No overwrites**: Automatically renames to avoid conflicts
- **Confirmation prompts**: Interactive confirmation for destructive operations
- **Dry-run mode**: Preview changes before applying them
- **Atomic operations**: Safe file system operations
- **Error handling**: Clear messages for common issues
- **Backup capability**: Archive functionality for long-term storage

#### Integration with Git

- Automatically adds `.scrap/` to `.gitignore`
- Never commits temporary files to version control
- Works seamlessly in Git repositories

### Unscrap Tool

The `unscrap` tool restores files from the `.scrap` folder back to their original locations or custom destinations.

#### Basic Usage

```bash
# Restore last scrapped item
unscrap

# Restore specific file/directory
unscrap filename.txt

# Restore to custom location
unscrap filename.txt --to /new/location/

# Force overwrite if destination exists
unscrap filename.txt --force
```

#### Examples

```bash
# Quick restore of last action
unscrap

# Restore specific items
unscrap project_backup/
unscrap important.txt --to ~/Documents/

# Handle conflicts
unscrap file.txt --force  # Overwrite existing file
```

#### How Restoration Works

1. **Metadata lookup**: Finds original location from stored metadata
2. **Path reconstruction**: Recreates directory structure if needed
3. **Conflict detection**: Warns if destination already exists
4. **Safe restoration**: Atomic move operations with error handling

## Error Handling

The tool provides error handling and reporting:

- **Input validation**: Checks for invalid arguments and paths
- **Permission errors**: Clear messages for insufficient permissions
- **File system errors**: Handles locked files, missing directories, etc.
- **Collision detection**: Prevents data loss from naming conflicts
- **Graceful degradation**: Continues processing after non-critical errors

## Performance Optimization Tips

- Use `--threads` to increase parallelism for large datasets
- Use `--files-only` or `--dirs-only` when appropriate
- Use include/exclude patterns to limit processing scope
- Consider `--max-depth` for deep directory structures

## Output Examples

### Human-Readable Output

```
=== REFAC TOOL ===
Root directory: /path/to/project
Old string: 'oldname'
New string: 'newname'
Mode: Full

Phase 1: Discovering files and directories...
Phase 2: Checking for naming collisions...

=== CHANGE SUMMARY ===
Content modifications: 15 file(s)
File renames:         8 file(s)
Directory renames:    3 directory(ies)
Total changes:        26

Do you want to proceed? (y/N) y

Replacing content in files...
Renaming files and directories...

=== OPERATION COMPLETE ===
Operation completed successfully!
Total changes applied: 26
```

### JSON Output

```json
{
  "summary": {
    "content_changes": 15,
    "file_renames": 8,
    "directory_renames": 3,
    "total_changes": 26
  },
  "result": "success",
  "dry_run": false
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Permission denied |
| 4 | File not found |
| 5 | Naming collision detected |

## Troubleshooting

### Common Issues

**"Permission denied" errors**

- Run with appropriate permissions
- Check file/directory ownership
- Ensure files are not locked by other processes

**"No changes found" when changes expected**

- Verify the search string is correct (case-sensitive)
- Check include/exclude patterns
- Use `--verbose` to see what's being processed

**"Naming collision detected"**

- Review the collision report
- Rename conflicting files manually
- Use different target names

**Binary files not being processed**

- This is by design for safety
- Use `--verbose` to see which files are skipped
- Manually verify file types if needed

### Debug Mode

For detailed debugging information:

```bash
refac . "old" "new" --verbose --dry-run
```

## Contributing

Contributions are welcome! Please read the contributing guidelines and submit pull requests for any improvements.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/nomion
cd nomion-tool/refac_rs

# Run tests
cargo test

# Run with test coverage
cargo test --all-features

# Build for release
cargo build --release
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Changelog

### Version 0.1.0

- Initial release
- Basic rename functionality
- Safety features and collision detection
- Multi-threading support
- test suite
