---
layout: default
title: Workspace - Developer Tool Suite
toc: false
---

# Workspace

A unified tool suite for developers and system administrators that provides file operations, line analysis, version management, and development workflow automation. All tools accessible through a single `ws` binary with intuitive subcommands.

**Current Version**: 0.38.31859  
**Build Status**: Clean compilation with zero warnings  
**Test Status**: 249 tests passing across 8 test suites

## Tools Overview

### Refac - Code Refactoring Engine
Recursive string replacement with automatic encoding detection and safety features.

```bash
# Refactor always previews changes and asks for confirmation
ws refactor ./src "OldClassName" "NewClassName" --verbose

# Refactor with backups and specific file types
ws refactor ./src "OldApi" "NewApi" --backup --include "*.rs" --include "*.toml"

# Include hidden files like .ws configurations  
ws refactor . "old_name" "new_name" --include-hidden
```

**Key Features**: Encoding detection (UTF-8, UTF-16, Windows-1252), pre-validation, collision detection, multi-threaded processing  
**Developer Focus**: API migrations, bulk renames, safe refactoring, configuration updates  
**[Complete Guide]({{ '/refac-guide/' | relative_url }})**

### Ldiff - Log Analysis Engine
Real-time pattern recognition for logs and command output with ANSI color preservation.

```bash
# Monitor logs in real-time
tail -f /var/log/system.log | ws ldiff

# Analyze different log sources with distinct markers
journalctl -f | ws ldiff "■"
systemctl status | ws ldiff "●"
cargo test --verbose | ws ldiff "█"
```

**Key Features**: Real-time streaming, ANSI preservation, customizable visualization, pattern recognition  
**Developer Focus**: Debug analysis, monitoring, test output parsing, system administration  
**[Complete Guide]({{ '/ldiff-guide/' | relative_url }})**

### Scrap - Local Trash System
Safe file disposal with metadata tracking, search capabilities, and git integration.

```bash
# Move files to local trash instead of deleting
ws scrap experimental_feature/ temp_logs/ *.bak old_tests/

# Search and browse stored files
ws scrap find "*.rs" --content
ws scrap list --sort date

# Archive and clean up old items
ws scrap archive --output backup-$(date +%Y%m%d).tar.gz --remove
ws scrap clean --days 14
```

**Key Features**: Metadata tracking, conflict resolution, content search, git integration, archive support  
**Developer Focus**: Experimental code cleanup, safe file disposal, temporary file management  
**[Complete Guide]({{ '/scrap-guide/' | relative_url }})**

### Unscrap - File Recovery System
Restore files from scrap with automatic path reconstruction and conflict resolution.

```bash
# Quick restore operations
ws unscrap                                    # Restore last scrapped item
ws unscrap experimental_feature/              # Restore specific directory
ws unscrap important.rs --to ~/backup/       # Custom destination with conflict handling
```

**Key Features**: Automatic recovery, custom destinations, conflict handling, batch restoration  
**Developer Focus**: Accident recovery, experiment rollback, selective file restoration  
**[Complete Guide]({{ '/unscrap-guide/' | relative_url }})**

### Git Integration & Template System
Git-integrated versioning with template engine for automated file generation.

```bash
# Set up automatic versioning
ws git install

# Add templates that auto-update with version changes
ws template add version-header \
  --template "#define VERSION \"{{ project.version }}\"
#define BUILD_DATE \"{{ datetime.date }}\"" \
  --output "src/version.h"

# Manual version update and template rendering
ws update --git-add

# Templates render automatically on commits
git add . && git commit -m "New feature"  # Auto-increments version and renders templates
```

**Key Features**: Git integration, Tera template engine, automatic rendering, multi-format support  
**Developer Focus**: Release automation, version consistency, file generation, CI/CD integration  
**[Complete Guide]({{ '/st8-guide/' | relative_url }})**

## Template System Examples

The template system supports Tera templating with these variables:
- `{{ project.version }}` - Full version (e.g., "1.2.3")
- `{{ project.name }}` - Project name from repository
- `{{ project.major_version }}`, `{{ project.minor_version }}`, `{{ project.patch_version }}` - Version components
- `{{ datetime.date }}`, `{{ datetime.time }}`, `{{ datetime.iso }}` - Build timestamps

### C/C++ Version Header Template
```bash
ws template add version-header \
  --template \
"#ifndef VERSION_H
#define VERSION_H
#define PROJECT_VERSION \"{{ project.version }}\"
#define BUILD_DATE \"{{ datetime.date }}\"
#endif" \
  --output "include/version.h"
```

### JavaScript Version Module Template
```bash
ws template add version-js \
  --template \
"export const VERSION = {
  full: '{{ project.version }}',
  major: {{ project.major_version }},
  buildDate: '{{ datetime.date }}'
};" \
  --output "src/version.js"
```

### Docker Compose Template
```bash
ws template add docker-compose \
  --template \
"version: '3.8'
services:
  app:
    image: {{ project.name }}:{{ project.version }}
    environment:
      - VERSION={{ project.version }}" \
  --output "docker-compose.prod.yml"
```

## Quality Assurance & Testing

### Test Suite Coverage
- **249 Tests** across 8 test suites
- **Pre-operation Validation** prevents mid-execution failures
- **Edge Case Coverage**: Concurrency, encoding, permissions, deep nesting
- **Memory Safety** through Rust's ownership model
- **Zero Compilation Warnings** across platforms

### Safety Features
- **Race Condition Prevention** through proper operation ordering
- **Encoding Detection** handles UTF-8, UTF-16, Windows-1252, and other text encodings
- **Atomic Operations** prevent partial failures
- **Collision Detection** prevents overwrites and conflicts
- **Binary File Protection** with automatic detection

### Test Categories
| Test Suite | Tests | Focus Area |
|------------|-------|------------|
| `integration_tests` | 18 | Cross-tool integration |
| `refac_concurrency_tests` | 9 | Multi-threading safety |
| `refac_edge_cases_tests` | 14 | Complex scenarios |
| `refac_encoding_tests` | 7 | Encoding safety |
| `scrap_advanced_integration_tests` | 21 | Workflows |
| `st8_template_tests` | 15 | Template system |

## Installation

### Quick Install
```bash
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace
./install.sh
```

### Verification
```bash
# Check installation and version
ws --version       # Should show: ws 0.34.20950

# Test basic functionality
echo "hello world" | ws ldiff
ws git status
ws refactor --help
```

**[Installation Guide]({{ '/installation/' | relative_url }})**

## Usage Workflows

### Development Refactoring Workflow
```bash
# 1. Review changes (refactor shows changes before applying)
ws refactor ./src "OldApi" "NewApi" --verbose

# 2. Apply refactoring with backups
ws refactor ./src "OldApi" "NewApi" --backup

# 3. Update configuration files
ws refactor ./config "old.endpoint" "new.endpoint" --content-only --include "*.toml"

# 4. Clean up old test files
ws scrap legacy_tests/ old_benchmarks/

# 5. Update version and render templates
ws update --git-add  # Templates auto-render with new version
```

### Log Analysis Workflow
```bash
# 1. Monitor application logs in real-time
tail -f /var/log/app.log | ldiff

# 2. Analyze historical patterns
cat /var/log/app.log.1 | ldiff > patterns-yesterday.txt

# 3. Compare different deployments
cat deployment-v1.log | ldiff "v1:" > patterns-v1.txt
cat deployment-v2.log | ldiff "v2:" > patterns-v2.txt
diff patterns-v1.txt patterns-v2.txt
```

### File Management Workflow
```bash
# 1. Move questionable files to safe storage
scrap experimental_code/ temp_data/ *.backup

# 2. Review and search stored files
scrap list --sort date
scrap find "important" --content

# 3. Archive old items or restore needed files
scrap archive --output archive-$(date +%Y%m%d).tar.gz
unscrap important_config.toml --to ./config/
```

## Key Features

- **Cross-Platform**: Native support for Windows, macOS, and Linux
- **Safety Features**: Collision detection, confirmation prompts, and atomic operations
- **Performance**: Multi-threaded processing and algorithms
- **Encoding Aware**: Automatic detection and preservation of text file encodings
- **CLI Design**: Clear error messages, help text, and intuitive commands
- **Integration Ready**: Designed for scripts, automation, and CI/CD pipelines

## Getting Help

Each tool provides help documentation:
```bash
ws refactor --help   # Refac documentation
ws ldiff --help      # Log analysis usage guide
ws scrap --help      # File management operations
ws unscrap --help    # Recovery system guide
ws git --help        # Git integration and version management
ws template --help   # Template system commands
ws update --help     # Version update commands
```

## Documentation

- **[Installation Guide]({{ '/installation/' | relative_url }})** - Installation and setup
- **[Getting Started]({{ '/getting-started/' | relative_url }})** - Step-by-step tutorial
- **[Usage Guide]({{ '/usage/' | relative_url }})** - Usage examples
- **[API Reference]({{ '/api-reference/' | relative_url }})** - Command-line reference
- **[Examples]({{ '/examples/' | relative_url }})** - Use cases and workflows
- **[Testing Guide]({{ '/testing/' | relative_url }})** - Test suite documentation

## Tool-Specific Guides

- **[Refac Guide]({{ '/refac-guide/' | relative_url }})** - Refactoring techniques
- **[Ldiff Guide]({{ '/ldiff-guide/' | relative_url }})** - Log analysis and pattern recognition
- **[Scrap Guide]({{ '/scrap-guide/' | relative_url }})** - File management practices
- **[Unscrap Guide]({{ '/unscrap-guide/' | relative_url }})** - Recovery workflows
- **[St8 Guide]({{ '/st8-guide/' | relative_url }})** - Version management and templates

## Support & Community

- **Issues & Bug Reports**: [GitHub Issues](https://github.com/jowharshamshiri/workspace/issues)
- **Source Code**: [GitHub Repository](https://github.com/jowharshamshiri/workspace)
- **Documentation**: [Documentation Site](https://jowharshamshiri.github.io/workspace/)

## License

MIT License - see the [LICENSE](https://github.com/jowharshamshiri/workspace/blob/main/LICENSE) file for details.