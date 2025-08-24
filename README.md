# Workspace - AI-Assisted Development Suite

Multi-tool CLI suite with AI-assisted development capabilities and real-time project dashboard.

## Core Tools

**Multi-tool Foundation**:
- `refac`: Recursive string replacement with collision detection
- `scrap/unscrap`: Local trash system with metadata tracking  
- `st8`: Git-based semantic versioning with template integration
- `ldiff`: Line difference visualization for pattern analysis

**Entity System**: 10 core types (Project, Feature, Task, Session, Directive, Note, Template, Test, Dependency, Milestone) with relationship tracking

**ADE Interface**: Professional development environment with 9 functional sections and Appwrite-style design

## Quick Start

```bash
# Multi-tool operations
ws refactor ./src "old_name" "new_name"
ws scrap old_files/
ws unscrap important_file.txt

# Project management
ws status
ws feature add "New capability"
ws task add "Implement feature" --feature F00001

# Start dashboard
ws mcp-server
# Open http://localhost:3000
```

## Features

- **MCP server integration**: API endpoints for Claude AI assistance
- **Database-driven**: SQLite backend for entity management
- **Template system**: Tera-based file generation
- **Real-time dashboard**: Entity tracking and feature management
- **Safety-first operations**: Dry-run validation and collision detection

**Current Status**: 99.7% implementation complete (301/302 features)
