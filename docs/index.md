---
layout: default
title: Workspace - AI-Assisted Development Suite
toc: false
---

# Workspace

Multi-tool CLI suite with AI-assisted development capabilities, real-time project dashboard, and entity-driven management system. All tools accessible through a single `ws` binary.

**Current Version**: 0.52.111061  
**Build Status**: Clean compilation with enterprise logging  
**Test Status**: 89.0% test coverage with 301/302 features implemented

## Core Capabilities

### Multi-Tool Foundation
Four specialized tools unified under single binary:

**Refac**: Recursive string replacement with collision detection
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

**St8**: Git-based semantic versioning with template integration
```bash
ws git install
ws update --git-add
```

**Ldiff**: Line difference visualization for pattern analysis
```bash
tail -f /var/log/system.log | ws ldiff
```

### AI-Assisted Development Environment

**MCP Server Integration**: API endpoints for Claude AI assistance
```bash
ws mcp-server  # Start on localhost:3000
```

**Entity-Driven Management**: 10 core entity types with relationship tracking
- Projects, Features, Tasks, Sessions, Directives
- Notes, Templates, Tests, Dependencies, Milestones

**Real-Time Dashboard**: Professional ADE interface with 9 functional sections
- Overview, Sessions, Issues, Features, Workspace
- Testing, Entities, Analytics, Settings

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

**Structured Logging**: Enterprise-grade logging with rotation and archiving

## Feature Status

**Current Implementation**: 301/302 features complete (99.7%)
**Test Coverage**: 89.0% with passing validation
**Architecture**: Entity-based with database persistence

**Entity Types**:
- **189 Backend Features**: Core tools, database system, API layer
- **115 ADE Interface Features**: Dashboard, visualization, user interaction

**Quality Assurance**:
- Zero compilation warnings maintained
- Structured logging with file rotation
- Safety-first operations with collision detection

## Installation

```bash
git clone https://github.com/jowharshamshiri/workspace.git
cd workspace
./install.sh
```

**Verification**:
```bash
ws --version  # Current: 0.52.111061
ws status     # Check installation
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

# Dashboard access
ws mcp-server  # Start on http://localhost:3000
```

### Development Workflow
```bash
# Template-based file generation
ws template add version --template "v{{ project.version }}" --output version.h

# Real-time log analysis
tail -f app.log | ws ldiff

# Structured project tracking
ws feature list --state implemented
ws task complete T000001 --evidence "Tests passing"
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
- **[St8 Guide]({{ '/st8-guide/' | relative_url }})** - Version management
- **[Ldiff Guide]({{ '/ldiff-guide/' | relative_url }})** - Log analysis