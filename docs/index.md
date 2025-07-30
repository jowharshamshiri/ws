---
layout: default
title: Nomion - Command-Line Tool Suite
toc: false
---

# Nomion

A tool suite for developers and system administrators, built for safety, reliability, and performance in daily development workflows.

## Testing & Quality Assurance

### Test Coverage
- 231 tests across 8 test suites
- Pre-operation validation prevents mid-execution failures
- Edge case coverage: Concurrency, encoding, permissions, deep nesting scenarios
- Zero compilation warnings across platforms
- Memory safety through Rust's ownership model

### Safety Features
- Race condition prevention through proper operation ordering
- UTF-8 and encoding issue detection
- Atomic operations prevent partial failures
- Collision detection prevents overwrites and conflicts
- Binary file protection with automatic detection

## Tools Overview

### Refac - Code Refactoring
String replacement engine with language-aware processing and safety features.

```bash
refac . "oldname" "newname" --dry-run --verbose
refac ./src "OldClass" "NewClass" --backup --include "*.rs"
```

**Key Features**: Pre-validation, collision detection, multi-threaded processing, backup support, binary protection  
**Developer Focus**: API migrations, bulk renames, content updates, safe refactoring  
**[Full Guide]({{ '/refac-guide/' | relative_url }})**

### Ldiff - Log Analysis
Real-time pattern recognition engine for logs and command output with ANSI color preservation.

```bash
tail -f /var/log/system.log | ldiff
journalctl -f | ldiff "■"
ps aux | ldiff
```

**Key Features**: Real-time analysis, ANSI color preservation, customizable visualization, streaming support  
**Developer Focus**: Debug analysis, pattern recognition, monitoring, test output parsing  
**[Full Guide]({{ '/ldiff-guide/' | relative_url }})**

### Scrap - File Management
Local trash system with metadata tracking, search capabilities, and git integration.

```bash
scrap experimental_feature/ temp_logs/ *.bak
scrap find "*.rs" --content "TODO"
scrap archive backup-$(date +%Y%m%d).tar.gz --remove
```

**Key Features**: Metadata tracking, conflict resolution, search and discovery, git integration, archive support  
**Developer Focus**: Experimental code cleanup, temporary file management, safe deletion  
**[Full Guide]({{ '/scrap-guide/' | relative_url }})**

### Unscrap - File Recovery
Restoration system with automatic path reconstruction and conflict resolution.

```bash
unscrap                                    # Restore last scrapped item
unscrap experimental_feature/              # Restore specific directory
unscrap important.rs --to ~/backup/       # Custom destination
```

**Key Features**: Automatic recovery, custom destinations, conflict handling, batch restoration, undo operations  
**Developer Focus**: Accident recovery, experiment rollback, selective restoration  
**[Full Guide]({{ '/unscrap-guide/' | relative_url }})**

### Verbump - Version Management
Git-integrated versioning system with automatic bumping and multi-format support.

```bash
verbump install                           # Set up git hook
verbump show                              # Display version info
git commit -m "Add feature"               # Auto-increments version
```

**Key Features**: Git integration, automatic bumping, multi-format support, semantic versioning, audit logging  
**Developer Focus**: Release automation, version consistency, CI/CD integration  
**[Full Guide]({{ '/verbump-guide/' | relative_url }})**

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