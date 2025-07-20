---
layout: default
title: Nomion - Command-Line Tool Suite
toc: false
---

# Nomion

A suite of robust, cross-platform command-line tools for developers and system administrators. Built for safety, reliability, and performance, making them suitable for mission-critical operations and daily development workflows.

## Tools Overview

### 🔄 Refac - String Replacement Tool
Replace strings in file/directory names and file contents with safety features and high performance.

```bash
refac . "oldname" "newname" --dry-run
```

**Key Features**: Collision detection, multi-threaded processing, binary file protection  
**[📖 Full Guide]({{ '/refac-guide/' | relative_url }})**

### 📊 Ldiff - Line Difference Visualizer
Process input lines, replacing repeated tokens with a substitute character for easy pattern recognition.

```bash
cat /var/log/system.log | tail -n 100 | ldiff
find / | ldiff
```

**Key Features**: ANSI color preservation, pattern recognition, customizable substitute characters  
**[📖 Full Guide]({{ '/ldiff-guide/' | relative_url }})**

### 🗑️ Scrap - Local Trash Folder
Move unwanted files to a local `.scrap` folder instead of permanent deletion.

```bash
scrap old_file.txt deprecated_feature/
scrap list
scrap clean --days 30
```

**Key Features**: Metadata tracking, search capabilities, cleanup operations, archiving  
**[📖 Full Guide]({{ '/scrap-guide/' | relative_url }})**

### ↩️ Unscrap - File Restoration
Restore files from the `.scrap` folder to their original locations or custom destinations.

```bash
unscrap filename.txt
unscrap --undo
unscrap filename.txt --to /new/location/
```

**Key Features**: Smart recovery, undo operations, conflict handling, custom destinations  
**[📖 Full Guide]({{ '/unscrap-guide/' | relative_url }})**

### 🏷️ Verbump - Automatic Version Management
Automatic version bumping via git hooks with smart versioning and configuration support.

```bash
verbump install
verbump show
verbump update
```

**Key Features**: Git integration, smart versioning, customizable patterns, audit logging  
**[📖 Full Guide]({{ '/verbump-guide/' | relative_url }})**

## Quick Start Examples

### Log Analysis with Ldiff
```bash
# Find repeated patterns in log files
tail -f /var/log/access.log | ldiff

# Use custom substitute character
journalctl -f | ldiff "*"

# Analyze command output
ps aux | ldiff
find /usr/local -type f | ldiff
```

### Project Refactoring with Refac
```bash
# Preview changes first (recommended)
refac . "oldname" "newname" --dry-run

# Rename class throughout codebase
refac ./src "OldClassName" "NewClassName"

# Update only file contents, keep names
refac ./config "old.example.com" "new.example.com" --content-only
```

### Safe File Management with Scrap/Unscrap
```bash
# Move files to .scrap instead of deleting
scrap old_file.txt deprecated_feature/

# See what you've scrapped
scrap list --sort date

# Restore the last thing you scrapped
unscrap --undo

# Find and restore specific files
scrap find "*.log"
unscrap access.log
```

### Automatic Versioning with Verbump
```bash
# Set up automatic versioning in your project
verbump install

# Check current version info
verbump show

# Manually bump version
verbump update --patch
```

## Key Features

- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Safety First**: Collision detection, confirmation prompts, and atomic operations
- **Performance Optimized**: Multi-threaded processing and efficient algorithms
- **User Friendly**: Clear error messages, help text, and intuitive commands
- **Integration Ready**: Designed to work well in scripts and automation

## Installation

### Quick Install
```bash
git clone https://github.com/jowharshamshiri/nomion
cd nomion
./install.sh
```

### Individual Tools
```bash
# Install specific tools only
cargo install --path . --bin refac
cargo install --path . --bin ldiff
cargo install --path . --bin scrap
cargo install --path . --bin unscrap
cargo install --path . --bin verbump
```

**[📖 Detailed Installation Guide]({{ '/installation/' | relative_url }})**

## Common Workflows

### Development Workflow
```bash
# 1. Refactor code safely
refac ./src "OldApi" "NewApi" --dry-run
refac ./src "OldApi" "NewApi"

# 2. Clean up old files
scrap legacy_code/ old_tests/

# 3. Update version automatically
verbump update
```

### Log Analysis Workflow
```bash
# 1. Monitor logs for patterns
tail -f /var/log/app.log | ldiff

# 2. Analyze historical logs
cat /var/log/app.log.1 | ldiff > patterns.txt

# 3. Compare different log sources
cat /var/log/nginx/access.log | ldiff "█"
```

### File Cleanup Workflow
```bash
# 1. Move questionable files to scrap
scrap temp_files/ *.bak

# 2. Review what was moved
scrap list

# 3. Restore if needed, or clean up
unscrap important.bak
scrap clean --days 7
```

## Getting Help

Each tool has comprehensive help:
```bash
refac --help
ldiff --help
scrap --help
unscrap --help
verbump --help
```

**Get versions:**
```bash
refac --version
ldiff --version
scrap --version
unscrap --version
verbump --version
```

## Documentation

- **[📖 Installation Guide]({{ '/installation/' | relative_url }})** - Detailed installation instructions
- **[📖 Usage Guide]({{ '/usage/' | relative_url }})** - Comprehensive usage examples
- **[📖 API Reference]({{ '/api-reference/' | relative_url }})** - Complete command-line reference
- **[📖 Examples]({{ '/examples/' | relative_url }})** - Real-world usage examples
- **[📖 Contributing]({{ '/contributing/' | relative_url }})** - How to contribute to the project

## Support

- **Issues**: [GitHub Issues](https://github.com/jowharshamshiri/nomion/issues)
- **Documentation**: [nomion.dev](https://nomion.dev)
- **Source Code**: [GitHub Repository](https://github.com/jowharshamshiri/nomion)

## License

MIT License - see the [LICENSE](https://github.com/jowharshamshiri/nomion/blob/main/LICENSE) file for details.