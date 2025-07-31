# Workspace

A tool suite for developers and system administrators that provides file operations, line analysis, version management, and development workflow automation.

**Current Version**: 0.34.20950  
**Build Status**: Clean compilation with zero warnings  
**Test Status**: 249 tests passing across 8 test suites  

## Tools Overview

| Tool | Purpose | Primary Use Cases |
|------|---------|-------------------|
| **refac** | Recursive string replacement with encoding detection | Code refactoring, API migrations, bulk renames |
| **ldiff** | Line difference visualization for pattern recognition | Log analysis, debug pattern detection, monitoring |
| **scrap** | Local trash can with metadata tracking | Safe file disposal, experimental cleanup |
| **unscrap** | File restoration from scrap folder | Accident recovery, experiment rollback |
| **st8** | Automated version management with templates | Release automation, version consistency, file generation |

## Quick Start Examples

### Code Refactoring with Refac
```bash
# Refac always previews changes and asks for confirmation
refac ./src "OldClassName" "NewClassName" --verbose

# Apply refactoring with backups
refac ./src "OldClassName" "NewClassName" --backup

# Process specific file types
refac ./config "old.api.url" "new.api.url" --content-only --include "*.toml"

# Include hidden files
refac . "st8" "new_st8" --include-hidden
```

### Log Pattern Analysis with Ldiff
```bash
# Real-time log monitoring
tail -f /var/log/system.log | ldiff

# Compare deployment logs
cat deploy-v1.log | ldiff > patterns-v1.txt
cat deploy-v2.log | ldiff > patterns-v2.txt

# Custom substitute character
journalctl -f | ldiff "■"
```

### File Management with Scrap/Unscrap
```bash
# Move files to local storage instead of deleting
scrap experimental_feature/ temp_logs/ *.bak

# Review stored files
scrap list --sort date

# Find specific files in storage
scrap find "*.rs" --content "TODO"

# Restore when needed
unscrap experimental_feature/
unscrap important.txt --to ~/backup/
```

### Version Management with St8 Templates
```bash
# Set up automatic versioning
st8 install

# Add a template for generating version headers
st8 template add version-header.h --content \
"#ifndef VERSION_H
#define VERSION_H
#define VERSION \"{{ project.version }}\"
#define VERSION_MAJOR {{ project.major_version }}
#define VERSION_MINOR {{ project.minor_version }}
#define VERSION_PATCH {{ project.patch_version }}
#define BUILD_DATE \"{{ datetime.date }}\"
#endif"

# Add template for deployment config
st8 template add deploy.yml --file-path ./deploy.template.yml

# List templates
st8 template list

# Templates render automatically when version updates
git add . && git commit -m "Add new feature"  # Auto-increments version and renders templates
```

## Tool Documentation

### Refac - String Replacement

**Core Features:**
- **Encoding Detection**: Handles UTF-8, UTF-16, Windows-1252, and other text encodings
- **Multi-threaded Processing**: Parallel processing for large codebases
- **Safety Features**: Pre-validation prevents mid-operation failures
- **Collision Detection**: Prevents naming conflicts and data loss
- **Binary Protection**: Skips binary files automatically

**Usage Examples:**
```bash
# Multi-mode operations
refac . "oldname" "newname" --names-only     # Rename files/dirs only
refac . "oldname" "newname" --content-only   # Replace content only
refac . "oldname" "newname" --files-only     # Process files only
refac . "oldname" "newname" --dirs-only      # Process directories only

# Pattern filtering
refac . "oldname" "newname" \
  --include "*.rs" --include "*.toml" \
  --exclude "target/*" --exclude "*.log"

# Performance tuning
refac ./large-project "old" "new" \
  --threads 8 \
  --max-depth 5 \
  --verbose

# Backup operations
refac . "oldname" "newname" --force --backup --assume-yes
```

### Ldiff - Pattern Recognition

**Core Features:**
- **Real-time Analysis**: Works with streaming input (`tail -f`)
- **ANSI Preservation**: Maintains terminal colors and formatting
- **Customizable Visualization**: User-defined substitute characters
- **Performance**: Handles large log files

**Usage Patterns:**
```bash
# System administration
systemctl status | ldiff
ps aux | ldiff "*"
df -h | ldiff "░"

# Development workflows
npm test | ldiff
cargo test --verbose | ldiff "█"
git log --oneline | ldiff

# Security monitoring
tail -f /var/log/auth.log | ldiff "⚠"
journalctl -f -u ssh | ldiff "●"
```

### Scrap - Local Trash System

**Core Features:**
- **Metadata Tracking**: Preserves original locations and timestamps
- **Conflict Resolution**: Smart naming prevents overwrites
- **Git Integration**: Automatic .gitignore management
- **Search Capabilities**: Find files by name, content, or metadata
- **Archive Support**: Compress and backup scrap contents

**Operations:**
```bash
# Basic operations
scrap file.txt directory/                    # Move to scrap
scrap                                       # List contents (default)
scrap list --sort name                      # Sort by name
scrap list --sort date                      # Sort by date
scrap list --sort size                      # Sort by size

# Search operations
scrap find "test.*"                         # Find by filename pattern
scrap find "TODO" --content                 # Search file contents
scrap find "*.log" --days 7                # Find files from last 7 days

# Maintenance operations
scrap clean                                 # Remove items older than 30 days
scrap clean --days 7                        # Remove items older than 7 days
scrap archive --output backup-$(date +%Y%m%d).tar.gz
scrap archive --remove                     # Archive and remove originals
scrap purge --force                        # Remove all items
```

### Unscrap - File Recovery System

**Core Features:**
- **Automatic Recovery**: Restores to original locations using metadata
- **Custom Destinations**: Flexible restoration paths
- **Conflict Handling**: Safe resolution of destination conflicts
- **Batch Operations**: Restore multiple related files

**Recovery Operations:**
```bash
# Quick recovery
unscrap                                     # Restore last scrapped item
unscrap filename.txt                        # Restore specific file
unscrap project_backup/                     # Restore directory

# Custom recovery
unscrap important.txt --to ~/Documents/     # Custom destination
unscrap config.toml --force                 # Overwrite existing
unscrap --list                              # Show restorable items
```

### St8 - Version Management with Templates

**Core Features:**
- **Git Integration**: Automatic version bumping on commits
- **Template Engine**: Tera template support with variables
- **Multi-format Support**: Cargo.toml, package.json, version.txt, etc.
- **State Management**: Centralized configuration in .ws folder
- **Activity Logging**: All operations logged to .ws/st8/logs/st8.log

**Template System Variables:**
- `{{ project.version }}` - Full version (e.g., "1.2.3")
- `{{ project.name }}` - Project name from repository
- `{{ project.major_version }}`, `{{ project.minor_version }}`, `{{ project.patch_version }}` - Version components
- `{{ datetime.date }}`, `{{ datetime.time }}`, `{{ datetime.iso }}` - Build timestamps

**Template Examples:**

#### C/C++ Version Header
```bash
st8 template add include/version.h --content \
"#ifndef VERSION_H
#define VERSION_H

#define PROJECT_NAME \"{{ project.name }}\"
#define PROJECT_VERSION \"{{ project.version }}\"
#define VERSION_MAJOR {{ project.major_version }}
#define VERSION_MINOR {{ project.minor_version }}
#define VERSION_PATCH {{ project.patch_version }}
#define BUILD_DATE \"{{ datetime.date }}\"
#define BUILD_TIME \"{{ datetime.time }}\"

#endif // VERSION_H"
```

#### JavaScript/Node.js Version Module
```bash
st8 template add src/version.js --content \
"// Auto-generated version file - DO NOT EDIT
export const VERSION = {
  full: '{{ project.version }}',
  major: {{ project.major_version }},
  minor: {{ project.minor_version }},
  patch: {{ project.patch_version }},
  name: '{{ project.name }}',
  buildDate: '{{ datetime.date }}',
  buildTime: '{{ datetime.time }}'
};

export default VERSION;"
```

#### Docker Compose with Version
```bash
st8 template add docker-compose.prod.yml --content \
"version: '3.8'
services:
  app:
    image: {{ project.name }}:{{ project.version }}
    environment:
      - VERSION={{ project.version }}
      - BUILD_DATE={{ datetime.date }}
    labels:
      - \"version={{ project.version }}\"
      - \"build.date={{ datetime.date }}\"
      - \"build.time={{ datetime.time }}\""
```

#### Kubernetes Deployment Manifest
```bash
st8 template add k8s/deployment.yml --content \
"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ project.name }}
  labels:
    app: {{ project.name }}
    version: \"{{ project.version }}\"
spec:
  replicas: 3
  selector:
    matchLabels:
      app: {{ project.name }}
  template:
    metadata:
      labels:
        app: {{ project.name }}
        version: \"{{ project.version }}\"
    spec:
      containers:
      - name: {{ project.name }}
        image: {{ project.name }}:{{ project.version }}
        env:
        - name: VERSION
          value: \"{{ project.version }}\"
        - name: BUILD_DATE
          value: \"{{ datetime.date }}\""
```

#### Python Version Module
```bash
st8 template add __version__.py --content \
"\"\"\"
Auto-generated version information.
This file is automatically updated by st8 on version changes.
\"\"\"

__version__ = '{{ project.version }}'
__project__ = '{{ project.name }}'
__major__ = {{ project.major_version }}
__minor__ = {{ project.minor_version }}
__patch__ = {{ project.patch_version }}
__build_date__ = '{{ datetime.date }}'
__build_time__ = '{{ datetime.time }}'

VERSION_INFO = {
    'version': __version__,
    'project': __project__,
    'major': __major__,
    'minor': __minor__,
    'patch': __patch__,
    'build_date': __build_date__,
    'build_time': __build_time__,
}"
```

**Template Management Workflow:**
```bash
# Set up st8 in your project
st8 install

# Add templates for your project
st8 template add src/version.h --file-path ./templates/version.h.template
st8 template add package.json --content "{ \"version\": \"{{ project.version }}\" }"

# List templates
st8 template list

# Show template details
st8 template show src/version.h

# Test template rendering
st8 template render

# Enable/disable templates
st8 template enable src/version.h
st8 template enable --all

# Version updates render templates automatically
git add . && git commit -m "New feature"
# -> Version auto-increments from 1.2.3 to 1.2.4
# -> Templates re-render with new version

# Manual version update
st8 update --minor  # 1.2.4 -> 1.3.0
st8 update --major  # 1.3.0 -> 2.0.0
```

## Installation

### Quick Install
```bash
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace
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
# Build and install tools
cargo install --path .

# Install specific tools
cargo install --path . --bin refac
cargo install --path . --bin ldiff
cargo install --path . --bin scrap
cargo install --path . --bin unscrap
cargo install --path . --bin st8
```

### Verification
```bash
# Check installation
refac --version    # Should show: refac 0.34.20950
ldiff --version    # Should show: ldiff 0.34.20950
scrap --version    # Should show: scrap 0.34.20950
unscrap --version  # Should show: unscrap 0.34.20950
st8 --version      # Should show: st8 0.34.20950

# Test basic functionality
echo "test" | ldiff
st8 status
refac --help
```

## Quality Assurance & Testing

### Test Suite Coverage
- **249 Tests** across 8 test suites
- **Unit Tests**: Core functionality and edge cases (94 tests)
- **Integration Tests**: End-to-end workflows (155 tests)
- **Edge Case Coverage**: Concurrency, encoding, permissions, deep nesting

### Test Categories
| Test Suite | Tests | Focus Area |
|------------|-------|------------|
| `integration_tests` | 18 | Cross-tool integration |
| `refac_concurrency_tests` | 9 | Multi-threading safety |
| `refac_edge_cases_tests` | 14 | Complex scenarios |
| `refac_empty_directory_tests` | 8 | Directory handling |
| `refac_encoding_tests` | 7 | UTF-8/encoding safety |
| `scrap_advanced_integration_tests` | 21 | Advanced scrap workflows |
| `scrap_integration_tests` | 18 | Core scrap functionality |
| `st8_integration_tests` | 25 | Version management |
| `st8_template_tests` | 15 | Template system |

### Quality Standards
- Zero compilation warnings across platforms
- Memory safety through Rust's ownership model
- Performance testing with large file sets
- Pre-operation validation prevents mid-execution failures
- Operation ordering prevents race conditions
- Encoding detection handles text file encodings

### Running Tests
```bash
# Run test suite
cargo test

# Run specific test categories
cargo test --test integration_tests
cargo test --test refac_encoding_tests
cargo test --test st8_template_tests

# Test with verbose output
cargo test -- --nocapture

# Performance testing
cargo test --release
```

## Command Reference

### Refac Options
| Option | Short | Description |
|--------|-------|-------------|
| `--assume-yes` | `-y` | Skip confirmation prompts (non-interactive mode) |
| `--verbose` | `-v` | Show detailed output |
| `--backup` | `-b` | Create backup files |
| `--files-only` | | Process files only |
| `--dirs-only` | | Process directories only |
| `--names-only` | | Rename only, skip content |
| `--content-only` | | Replace content only |
| `--include-hidden` | | Include hidden files |
| `--threads <N>` | `-j` | Number of threads |
| `--include <PATTERN>` | | Include files matching pattern |
| `--exclude <PATTERN>` | | Exclude files matching pattern |

### St8 Template Commands
| Command | Description |
|---------|-------------|
| `st8 template add <name>` | Add new template |
| `st8 template list` | List templates |
| `st8 template show <name>` | Show template details |
| `st8 template enable <name>` | Enable template |
| `st8 template remove <name>` | Remove template |
| `st8 template render` | Render templates |

## Safety Features

### Built-in Protections
- **Collision Detection**: Prevents overwriting existing files
- **Pre-validation**: Tests operations before execution
- **Binary File Protection**: Skips binary files automatically
- **Atomic Operations**: Prevents partial failures
- **Encoding Safety**: Handles text encodings without corruption
- **Race Condition Prevention**: Proper operation ordering

### Best Practices
1. **Review changes carefully**: Refac shows all changes before applying them
2. **Use backups for important files**: `--backup` option
3. **Test on copies**: Work on backups of important directories
4. **Use version control**: Commit files before running operations
5. **Be specific with patterns**: Use include/exclude to limit scope

## Usage Examples

### API Migration Workflow
```bash
# 1. Review the migration (refac shows changes before applying)
refac ./src "oldapi.v1" "newapi.v2" --verbose

# 2. Update source code
refac ./src "oldapi.v1" "newapi.v2" --backup --include "*.rs"

# 3. Update configuration
refac ./config "oldapi.v1" "newapi.v2" --content-only --include "*.toml"

# 4. Update documentation
refac ./docs "oldapi.v1" "newapi.v2" --include "*.md"

# 5. Clean up old files
scrap old_api_tests/ legacy_configs/

# 6. Update version and generate deployment files
st8 update --minor
# Templates render automatically with new version
```

### Log Analysis Workflow
```bash
# Monitor application startup
tail -f /var/log/myapp.log | ldiff

# Compare two deployment logs
cat deployment-1.log | ldiff > patterns-1.txt
cat deployment-2.log | ldiff > patterns-2.txt
diff patterns-1.txt patterns-2.txt

# Analyze different log sources with distinct markers
tail -f /var/log/nginx/access.log | ldiff "█" &
tail -f /var/log/nginx/error.log | ldiff "░" &
```

### Development Cleanup Workflow
```bash
# Move experimental code to storage
scrap experiment-v1/ temp-tests/ *.backup

# Review what was moved
scrap list --sort date

# Archive old experiments
scrap archive --output experiments-$(date +%Y%m%d).tar.gz --remove

# Clean up old items
scrap clean --days 30

# Restore if needed
scrap find "experiment" --content "important"
unscrap experiment-v1/critical_file.rs --to ./src/
```

## Documentation

**Documentation Site**: [https://jowharshamshiri.github.io/workspace/](https://jowharshamshiri.github.io/workspace/)

**Links:**
- [Installation Guide](https://jowharshamshiri.github.io/workspace/installation/) - Setup instructions
- [Getting Started](https://jowharshamshiri.github.io/workspace/getting-started/) - Step-by-step tutorial
- [Usage Guide](https://jowharshamshiri.github.io/workspace/usage/) - Usage examples
- [API Reference](https://jowharshamshiri.github.io/workspace/api-reference/) - Command documentation
- [Examples](https://jowharshamshiri.github.io/workspace/examples/) - Use cases

## Support & Contributing

- **Issues**: [GitHub Issues](https://github.com/jowharshamshiri/workspace/issues)
- **Documentation**: [workspace.dev](https://workspace.dev)
- **Source Code**: [GitHub Repository](https://github.com/jowharshamshiri/workspace)

## License

MIT License - see the [LICENSE](https://github.com/jowharshamshiri/workspace/blob/main/LICENSE) file for details.