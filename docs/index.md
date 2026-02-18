---
layout: default
title: Workspace - AI-Assisted Development Suite
toc: false
---

# Workspace

Multi-tool CLI suite with AI-assisted development capabilities, real-time project dashboard, and entity-driven management system. All tools accessible through a single `ws` binary.

**Build Status**: Clean compilation with structured logging

## Core Capabilities

### Multi-Tool Foundation

**Refactor**: Recursive string replacement with collision detection
```bash
ws refactor ./src "OldClassName" "NewClassName" --verbose
ws refactor ./src "OldApi" "NewApi" --backup --include "*.rs"
```

**Scrap/Unscrap**: Local trash system with metadata tracking
```bash
ws scrap experimental_feature/ temp_logs/ *.bak
ws scrap list --sort date
ws unscrap important.rs
```

**Version Management**: Git-based semantic versioning with template integration
```bash
ws git install
ws update --git-add
```

**Ldiff**: Line difference visualization for pattern analysis
```bash
tail -f /var/log/system.log | ws ldiff
```

### Version Stamping with Wstemplate

**Wstemplate**: Cross-project version stamping using `.wstemplate` files
```bash
ws wstemplate add /path/to/workspace     # Set scan root for this project
ws wstemplate list                        # Show relevant templates
ws wstemplate render                      # Render all relevant templates
```

Templates use Tera syntax with automatic cross-project version resolution:
```
version = "{{ project.version }}"
dependency = "{{ projects.other_lib.version }}"
```

Each project has a single wstemplate entry (alias + scan root). Cross-project
references like `{{ projects.OTHER.version }}` are resolved dynamically by
scanning for sibling `.ws/state.json` files.

### AI-Assisted Development Environment

**MCP Server Integration**: API endpoints for Claude AI assistance
```bash
ws mcp-server                # Start on localhost:3000
ws mcp-server --port 8080    # Custom port
```

**Entity-Driven Management**: Core entity types with relationship tracking
- Projects, Features, Tasks, Sessions, Directives, Notes

### Database-Driven Project Management

**SQLite Backend**: Entity management with relationship tracking
```bash
ws status --include-features
ws feature add "New capability"
ws task add "Implement feature" --feature F00001
```

**Template System**: Tera-based file generation with version integration
```bash
ws template add version-header --template "v{{ project.version }}" --output version.h
```

**Version Management**: Database-driven major version with git-calculated components
```bash
ws version show              # Display current version breakdown
ws version major 2           # Set major version to 2
ws version tag               # Create git tag with current version
```

## All Commands

| Command | Description |
|---------|-------------|
| `ws refactor` | Recursive string replacement in files and directories |
| `ws git` | Git integration (install/uninstall hooks, show version, status) |
| `ws template` | Tera template management (add, list, show, update, delete, render) |
| `ws update` | Update version file and render all templates |
| `ws wstemplate` | Manage `.wstemplate` cross-project version stamping |
| `ws version` | Version management (show, major, tag, info) |
| `ws scrap` | Local trash can with `.scrap` folder |
| `ws unscrap` | Restore files from `.scrap` folder |
| `ws ldiff` | Line difference visualization for pattern analysis |
| `ws code` | AST-based code analysis and transformation |
| `ws test` | Intelligent test runner based on project type |
| `ws status` | Project status with feature metrics and progress |
| `ws feature` | Feature management with state machine workflow |
| `ws task` | Feature-centric task management |
| `ws directive` | Project directive and rule management |
| `ws note` | Note management for any entity |
| `ws relationship` | Entity relationship management |
| `ws start` | Start development session with context loading |
| `ws end` | End development session with documentation |
| `ws artifacts` | Session artifact management |
| `ws continuity` | Session continuity and context management |
| `ws consolidate` | Documentation consolidation with diagrams |
| `ws database` | Database backup, recovery, and maintenance |
| `ws mcp-server` | MCP server for Claude AI integration |
| `ws sample` | Create sample project with test data |

## Installation

```bash
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace
./install.sh
```

**Verification**:
```bash
ws --version
ws --help
```

## Usage Examples

### Multi-Tool Operations
```bash
# File refactoring with safety checks
ws refactor ./src "old_name" "new_name" --backup

# Safe file disposal and recovery
ws scrap temp_files/ experimental/
ws unscrap important_config.toml

# Version management with git integration
ws git install
ws update --git-add
```

### Project Management
```bash
# Entity management
ws feature add "User authentication"
ws task add "Implement login" --feature F00001
ws status --include-features
```

### Cross-Project Version Stamping
```bash
# Set up wstemplate for a project
ws wstemplate add /path/to/workspace

# Render templates (happens automatically on ws update)
ws wstemplate render
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
