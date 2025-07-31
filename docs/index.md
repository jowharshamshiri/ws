---
layout: default
title: Workspace - Developer Tool Suite
toc: false
---

# Workspace

A tool suite for developers and system administrators that provides file operations, line analysis, version management, and development workflow automation.

**Current Version**: 0.34.20950  
**Build Status**: Clean compilation with zero warnings  
**Test Status**: 249 tests passing across 8 test suites

## Tools Overview

### Refac - Code Refactoring Engine
Recursive string replacement with automatic encoding detection and safety features.

```bash
# Refac always previews changes and asks for confirmation
refac ./src "OldClassName" "NewClassName" --verbose

# Refactor with backups and specific file types
refac ./src "OldApi" "NewApi" --backup --include "*.rs" --include "*.toml"

# Include hidden files like .ws configurations
refac . "st8" "new_st8" --include-hidden
```

**Key Features**: Encoding detection (UTF-8, UTF-16, Windows-1252), pre-validation, collision detection, multi-threaded processing  
**Developer Focus**: API migrations, bulk renames, safe refactoring, configuration updates  
**[Complete Guide]({{ '/refac-guide/' | relative_url }})**

### Ldiff - Log Analysis Engine
Real-time pattern recognition for logs and command output with ANSI color preservation.

```bash
# Monitor logs in real-time
tail -f /var/log/system.log | ldiff

# Analyze different log sources with distinct markers
journalctl -f | ldiff "■"
systemctl status | ldiff "●"
cargo test --verbose | ldiff "█"
```

**Key Features**: Real-time streaming, ANSI preservation, customizable visualization, pattern recognition  
**Developer Focus**: Debug analysis, monitoring, test output parsing, system administration  
**[Complete Guide]({{ '/ldiff-guide/' | relative_url }})**

### Scrap - Local Trash System
Safe file disposal with metadata tracking, search capabilities, and git integration.

```bash
# Move files to local trash instead of deleting
scrap experimental_feature/ temp_logs/ *.bak old_tests/

# Search and browse stored files
scrap find "*.rs" --content "TODO"
scrap list --sort date

# Archive and clean up old items
scrap archive --output backup-$(date +%Y%m%d).tar.gz --remove
scrap clean --days 14
```

**Key Features**: Metadata tracking, conflict resolution, content search, git integration, archive support  
**Developer Focus**: Experimental code cleanup, safe file disposal, temporary file management  
**[Complete Guide]({{ '/scrap-guide/' | relative_url }})**

### Unscrap - File Recovery System
Restore files from scrap with automatic path reconstruction and conflict resolution.

```bash
# Quick restore operations
unscrap                                    # Restore last scrapped item
unscrap experimental_feature/              # Restore specific directory
unscrap important.rs --to ~/backup/       # Custom destination with conflict handling
```

**Key Features**: Automatic recovery, custom destinations, conflict handling, batch restoration  
**Developer Focus**: Accident recovery, experiment rollback, selective file restoration  
**[Complete Guide]({{ '/unscrap-guide/' | relative_url }})**

### St8 - Version Management with Templates
Git-integrated versioning with template engine for automated file generation.

```bash
# Set up automatic versioning
st8 install

# Add templates that auto-update with version changes
st8 template add src/version.h --content \
"#define VERSION \"{{ project.version }}\"
#define BUILD_DATE \"{{ datetime.date }}\""

# Templates render automatically on commits
git add . && git commit -m "New feature"  # Auto-increments version and renders templates
```

**Key Features**: Git integration, Tera template engine, automatic rendering, multi-format support  
**Developer Focus**: Release automation, version consistency, file generation, CI/CD integration  
**[Complete Guide]({{ '/st8-guide/' | relative_url }})**

## Template System Examples

The st8 template system supports Tera templating with these variables:
- `{{ project.version }}` - Full version (e.g., "1.2.3")
- `{{ project.name }}` - Project name from repository
- `{{ project.major_version }}`, `{{ project.minor_version }}`, `{{ project.patch_version }}` - Version components
- `{{ datetime.date }}`, `{{ datetime.time }}`, `{{ datetime.iso }}` - Build timestamps

### C/C++ Version Header Template
```bash
st8 template add include/version.h --content \
"#ifndef VERSION_H
#define VERSION_H
#define PROJECT_VERSION \"{{ project.version }}\"
#define BUILD_DATE \"{{ datetime.date }}\"
#endif"
```

### JavaScript Version Module Template
```bash
st8 template add src/version.js --content \
"export const VERSION = {
  full: '{{ project.version }}',
  major: {{ project.major_version }},
  buildDate: '{{ datetime.date }}'
};"
```

### Docker Compose Template
```bash
st8 template add docker-compose.prod.yml --content \
"version: '3.8'
services:
  app:
    image: {{ project.name }}:{{ project.version }}
    environment:
      - VERSION={{ project.version }}"
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
# Check installation and versions
refac --version    # Should show: refac 0.34.20950
ldiff --version    # Should show: ldiff 0.34.20950
scrap --version    # Should show: scrap 0.34.20950
unscrap --version  # Should show: unscrap 0.34.20950
st8 --version      # Should show: st8 0.34.20950

# Test basic functionality
echo "hello world" | ldiff
st8 status
refac --help
```

**[Installation Guide]({{ '/installation/' | relative_url }})**

## Usage Workflows

### Development Refactoring Workflow
```bash
# 1. Review changes (refac shows changes before applying)
refac ./src "OldApi" "NewApi" --verbose

# 2. Apply refactoring with backups
refac ./src "OldApi" "NewApi" --backup

# 3. Update configuration files
refac ./config "old.endpoint" "new.endpoint" --content-only --include "*.toml"

# 4. Clean up old test files
scrap legacy_tests/ old_benchmarks/

# 5. Update version and render templates
st8 update --minor  # Templates auto-render with new version
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
refac --help      # Refac documentation
ldiff --help      # Log analysis usage guide
scrap --help      # File management operations
unscrap --help    # Recovery system guide
st8 --help        # Version management and templates
st8 template --help  # Template system commands
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