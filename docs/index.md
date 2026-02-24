---
layout: default
title: Workspace - Development Tool Suite
toc: false
---

# Workspace

Multi-tool CLI suite with real-time project dashboard and entity-driven management system. All tools accessible through a single `wsb` binary.

**Build Status**: Clean compilation with structured logging

## Core Capabilities

### Multi-Tool Foundation

**Refactor**: Recursive string replacement with collision detection
```bash
wsb refactor ./src "OldClassName" "NewClassName" --verbose
wsb refactor ./src "OldApi" "NewApi" --backup --include "*.rs"
```

**Scrap/Unscrap**: Local trash system with metadata tracking
```bash
wsb scrap experimental_feature/ temp_logs/ *.bak
wsb scrap list --sort date
wsb unscrap important.rs
```

**Version Management**: Git-based semantic versioning with template integration
```bash
wsb git install
wsb update --git-add
```

**Ldiff**: Line difference visualization for pattern analysis
```bash
tail -f /var/log/system.log | wsb ldiff
```

### Version Stamping with Wstemplate

**Wstemplate**: Cross-project version stamping using `.wstemplate` files
```bash
wsb wstemplate add /path/to/workspace     # Set scan root for this project
wsb wstemplate list                        # Show relevant templates
wsb wstemplate render                      # Render all relevant templates
```

Templates use Tera syntax with automatic cross-project version resolution:
```
version = "{{ project.version }}"
dependency = "{{ projects.other_lib.version }}"
```

Each project has a single wstemplate entry (alias + scan root). Cross-project
references like `{{ projects.OTHER.version }}` are resolved dynamically by
scanning for sibling `.wsb/state.json` files.

### MCP Server Integration

**MCP Server**: API endpoints for Claude integration
```bash
wsb mcp-server                # Start on localhost:3000
wsb mcp-server --port 8080    # Custom port
```

**Entity-Driven Management**: Core entity types with relationship tracking
- Projects, Features, Tasks, Sessions, Directives, Notes

### Database-Driven Project Management

**SQLite Backend**: Entity management with relationship tracking
```bash
wsb status --include-features
wsb feature add "New capability"
wsb task add "Implement feature" --feature F00001
```

**Template System**: Tera-based file generation with version integration
```bash
wsb template add version-header --template "v{{ project.version }}" --output version.h
```

**Version Management**: Database-driven major version with git-calculated components
```bash
wsb version show              # Display current version breakdown
wsb version major 2           # Set major version to 2
wsb version tag               # Create git tag with current version
```

## All Commands

| Command | Description |
|---------|-------------|
| `wsb refactor` | Recursive string replacement in files and directories |
| `wsb git` | Git integration (install/uninstall hooks, show version, status) |
| `wsb template` | Tera template management (add, list, show, update, delete, render) |
| `wsb update` | Update version file and render all templates |
| `wsb wstemplate` | Manage `.wstemplate` cross-project version stamping |
| `wsb version` | Version management (show, major, tag, info) |
| `wsb scrap` | Local trash can with `.scrap` folder |
| `wsb unscrap` | Restore files from `.scrap` folder |
| `wsb ldiff` | Line difference visualization for pattern analysis |
| `wsb code` | AST-based code analysis and transformation |
| `wsb test` | Intelligent test runner based on project type |
| `wsb status` | Project status with feature metrics and progress |
| `wsb feature` | Feature management with state machine workflow |
| `wsb task` | Feature-centric task management |
| `wsb directive` | Project directive and rule management |
| `wsb note` | Note management for any entity |
| `wsb relationship` | Entity relationship management |
| `wsb start` | Start development session with context loading |
| `wsb end` | End development session with documentation |
| `wsb continuity` | Session continuity and context management |
| `wsb consolidate` | Documentation consolidation with diagrams |
| `wsb database` | Database backup, recovery, and maintenance |
| `wsb mcp-server` | MCP server for Claude AI integration |
| `wsb sample` | Create sample project with test data |

## Installation

```bash
git clone https://github.com/jowharshamshiri/wsb.git
cd workspace
./install.sh
```

**Verification**:
```bash
wsb --version
wsb --help
```

## Usage Examples

### Multi-Tool Operations
```bash
# File refactoring with safety checks
wsb refactor ./src "old_name" "new_name" --backup

# Safe file disposal and recovery
wsb scrap temp_files/ experimental/
wsb unscrap important_config.toml

# Version management with git integration
wsb git install
wsb update --git-add
```

### Project Management
```bash
# Entity management
wsb feature add "User authentication"
wsb task add "Implement login" --feature F00001
wsb status --include-features
```

### Cross-Project Version Stamping
```bash
# Set up wstemplate for a project
wsb wstemplate add /path/to/workspace

# Render templates (happens automatically on wsb update)
wsb wstemplate render
```

## Documentation

**Guides**:
- **[Installation Guide]({{ '/installation/' | relative_url }})** - Setup instructions
- **[Getting Started]({{ '/getting-started/' | relative_url }})** - Basic usage
- **[Usage Guide]({{ '/usage/' | relative_url }})** - Common workflows
- **[API Reference]({{ '/api-reference/' | relative_url }})** - Command reference

**Tool-Specific**:
- **[Refac Guide]({{ '/refac-guide/' | relative_url }})** - File refactoring
- **[Scrap Guide]({{ '/scrap-guide/' | relative_url }})** - File management
- **[St8 Guide]({{ '/st8-guide/' | relative_url }})** - Version management and wstemplate
- **[Ldiff Guide]({{ '/ldiff-guide/' | relative_url }})** - Log analysis
