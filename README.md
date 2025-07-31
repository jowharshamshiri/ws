# Workspace

A unified tool suite for developers and system administrators that provides file operations, line analysis, version management, and development workflow automation. All tools are accessible through a single `ws` binary with intuitive subcommands.

**Current Version**: 0.38.31859  
**Build Status**: Clean compilation with zero warnings  
**Test Status**: All tests passing across comprehensive test suites  

## Tools Overview

All tools are accessed via `ws <subcommand>` - no separate binaries to manage.

| Subcommand | Purpose | Primary Use Cases |
|------------|---------|-------------------|
| **ws refactor** | Recursive string replacement with encoding detection | Code refactoring, API migrations, bulk renames |
| **ws ldiff** | Line difference visualization for pattern recognition | Log analysis, debug pattern detection, monitoring |
| **ws scrap** | Local trash can with metadata tracking | Safe file disposal, experimental cleanup |
| **ws unscrap** | File restoration from scrap folder | Accident recovery, experiment rollback |
| **ws git** | Git integration and version management | Git hook management, version display, status |
| **ws template** | Template management and rendering | File generation, version headers, deployment configs |
| **ws update** | Manual project state updates | Version updates, template rendering |

## Quick Start Examples

### Code Refactoring with ws refactor
```bash
# Refactor always previews changes and asks for confirmation
ws refactor ./src "OldClassName" "NewClassName" --verbose

# Apply refactoring with backups
ws refactor ./src "OldClassName" "NewClassName" --backup

# Process specific file types
ws refactor ./config "old.api.url" "new.api.url" --content-only --include "*.toml"

# Include hidden files
ws refactor . "old_name" "new_name" --include-hidden
```

### Log Pattern Analysis with ws ldiff
```bash
# Real-time log monitoring
tail -f /var/log/system.log | ws ldiff

# Compare deployment logs
cat deploy-v1.log | ws ldiff > patterns-v1.txt
cat deploy-v2.log | ws ldiff > patterns-v2.txt

# Custom substitute character
journalctl -f | ws ldiff "■"
```

### File Management with Scrap/Unscrap
```bash
# Move files to local storage instead of deleting
ws scrap experimental_feature/ temp_logs/ *.bak

# Review stored files
ws scrap list --sort date

# Find specific files in storage
ws scrap find "*.rs" --content

# Restore when needed
ws unscrap experimental_feature/
ws unscrap important.txt --to ~/backup/
```

### Version Management and Templates
```bash
# Set up automatic versioning
ws git install

# Add a template for generating version headers
ws template add version-header \
  --template "#ifndef VERSION_H
#define VERSION_H
#define VERSION \"{{ project.version }}\"
#define VERSION_MAJOR {{ project.major_version }}
#define VERSION_MINOR {{ project.minor_version }}
#define VERSION_PATCH {{ project.patch_version }}
#define BUILD_DATE \"{{ datetime.date }}\"
#endif" \
  --output "include/version.h"

# Add template for deployment config
ws template add deploy --template deploy.template.yml --output deploy.yml

# List templates
ws template list

# Show git status and version info
ws git status
ws git show

# Manual version update and template rendering
ws update --git-add

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
ws refactor . "oldname" "newname" --names-only     # Rename files/dirs only
ws refactor . "oldname" "newname" --content-only   # Replace content only
ws refactor . "oldname" "newname" --files-only     # Process files only
ws refactor . "oldname" "newname" --dirs-only      # Process directories only

# Pattern filtering
ws refactor . "oldname" "newname" \
  --include "*.rs" --include "*.toml" \
  --exclude "target/*" --exclude "*.log"

# Performance tuning
ws refactor ./large-project "old" "new" \
  --threads 8 \
  --max-depth 5 \
  --verbose

# Backup operations
ws refactor . "oldname" "newname" --force --backup --assume-yes
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
systemctl status | ws ldiff
ps aux | ws ldiff "*"
df -h | ws ldiff "░"

# Development workflows
npm test | ws ldiff
cargo test --verbose | ws ldiff "█"
git log --oneline | ws ldiff

# Security monitoring
tail -f /var/log/auth.log | ws ldiff "⚠"
journalctl -f -u ssh | ws ldiff "●"
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
ws scrap file.txt directory/                    # Move to scrap
ws scrap                                        # List contents (default)
ws scrap list --sort name                       # Sort by name
ws scrap list --sort date                       # Sort by date
ws scrap list --sort size                       # Sort by size

# Search operations
ws scrap find "test.*"                          # Find by filename pattern
ws scrap find "TODO" --content                  # Search file contents

# Maintenance operations
ws scrap clean                                  # Remove items older than 30 days
ws scrap clean --days 7                         # Remove items older than 7 days
ws scrap archive --output backup-$(date +%Y%m%d).tar.gz
ws scrap archive --remove                      # Archive and remove originals
ws scrap purge --force                         # Remove all items
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
ws unscrap                                     # Restore last scrapped item
ws unscrap filename.txt                        # Restore specific file
ws unscrap project_backup/                     # Restore directory

# Custom recovery
ws unscrap important.txt --to ~/Documents/     # Custom destination
ws unscrap config.toml --force                 # Overwrite existing
```

### Git Integration & Template System

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
ws template add version-header \
  --template \
"#ifndef VERSION_H
#define VERSION_H

#define PROJECT_NAME \"{{ project.name }}\"
#define PROJECT_VERSION \"{{ project.version }}\"
#define VERSION_MAJOR {{ project.major_version }}
#define VERSION_MINOR {{ project.minor_version }}
#define VERSION_PATCH {{ project.patch_version }}
#define BUILD_DATE \"{{ datetime.date }}\"
#define BUILD_TIME \"{{ datetime.time }}\"

#endif // VERSION_H" \
  --output "include/version.h"
```

#### JavaScript/Node.js Version Module
```bash
ws template add version-js \
  --template \
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

export default VERSION;" \
  --output "src/version.js"
```

#### Docker Compose with Version
```bash
ws template add docker-compose \
  --template \
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
      - \"build.time={{ datetime.time }}\"" \
  --output "docker-compose.prod.yml"
```

#### Kubernetes Deployment Manifest
```bash
ws template add k8s-deployment \
  --template \
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
          value: \"{{ datetime.date }}\"" \
  --output "k8s/deployment.yml"
```

#### Python Version Module
```bash
ws template add python-version \
  --template \
"\"\"\"
Auto-generated version information.
This file is automatically updated on version changes.
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
}" \
  --output "__version__.py"
```

**Template Management Workflow:**
```bash
# Set up git integration in your project
ws git install

# Add templates for your project
ws template add version-header --template ./templates/version.h.template --output src/version.h
ws template add package-version --template "{ \"version\": \"{{ project.version }}\" }" --output package.json

# List templates
ws template list

# Show template details
ws template show version-header

# Test template rendering
ws template render

# Enable/disable templates
ws template enable version-header --disable  # Disable template
ws template enable version-header            # Enable template

# Version updates render templates automatically
git add . && git commit -m "New feature"
# -> Version auto-increments from 1.2.3 to 1.2.4
# -> Templates re-render with new version

# Manual version update and template rendering
ws update --git-add  # Update version and stage files
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
# Build and install ws binary
cargo install --path .

# The ws binary includes all tools as subcommands
```

### Verification
```bash
# Check installation
ws --version       # Should show: ws 0.38.31859

# Test subcommands
ws refactor --help
ws ldiff --help
ws scrap --help
ws unscrap --help
ws git --help

# Test basic functionality
echo "test" | ws ldiff
ws git status
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
| `st8_integration_tests` | 25 | Git integration & version management |
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

### Template Commands
| Command | Description |
|---------|-------------|
| `ws template add <name>` | Add new template |
| `ws template list` | List templates |
| `ws template show <name>` | Show template details |
| `ws template enable <name>` | Enable/disable template |
| `ws template remove <name>` | Remove template |
| `ws template render` | Render templates |

### Git Integration Commands
| Command | Description |
|---------|-------------|
| `ws git install` | Install git pre-commit hook |
| `ws git uninstall` | Remove git pre-commit hook |
| `ws git show` | Show current version info |
| `ws git status` | Show git integration status |

### Update Commands  
| Command | Description |
|---------|-------------|
| `ws update` | Update version and render templates |
| `ws update --git-add` | Update and stage files |
| `ws update --no-git` | Update without git checks |

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
# 1. Review the migration (refactor shows changes before applying)
ws refactor ./src "oldapi.v1" "newapi.v2" --verbose

# 2. Update source code
ws refactor ./src "oldapi.v1" "newapi.v2" --backup --include "*.rs"

# 3. Update configuration
ws refactor ./config "oldapi.v1" "newapi.v2" --content-only --include "*.toml"

# 4. Update documentation
ws refactor ./docs "oldapi.v1" "newapi.v2" --include "*.md"

# 5. Clean up old files
ws scrap old_api_tests/ legacy_configs/

# 6. Update version and generate deployment files
ws update --git-add
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