---
title: Documentation
layout: default
permalink: /documentation/
---

## Core Concepts

### 1. All-in-One Binary

All tools are subcommands of the `wsb` binary:

```bash
wsb refactor     # String replacement
wsb scrap        # Local trash
wsb unscrap      # File restoration
wsb git          # Git integration
wsb update       # Version update + template rendering
wsb wstemplate   # Cross-project version stamping
wsb version      # Database-driven versioning
wsb template     # Tera template management
wsb ldiff        # Line difference visualization
wsb code         # AST code analysis and codebase tree
wsb test         # Intelligent test runner
wsb status       # Project status
wsb feature      # Feature management
wsb task         # Task management
wsb directive    # Project directive management
wsb note         # Note management
wsb relationship # Entity relationship management
wsb start        # Start development session
wsb end          # End development session
wsb continuity   # Session context management
wsb consolidate  # Documentation consolidation
wsb database     # Database backup and maintenance
wsb mcp-server   # MCP server for Claude AI
wsb sample       # Create sample project
```

### 2. Refactor Operation Modes

- **Full mode**: Process both names and content (default)
- **Names-only**: Rename files/directories only
- **Content-only**: Modify file contents only
- **Files-only/Dirs-only**: Process specific item types

### 3. Safety Features

- **Collision Detection**: Prevents overwriting existing items
- **Binary Protection**: Automatically skips binary files
- **Dry-run Mode**: Preview changes before applying
- **Backup System**: Optional pre-modification backups

### 4. Version Management

Two template systems for version stamping:
- **`.tera` templates**: Managed via `wsb template`, stored in `.wsb/templates/`
- **`.wstemplate` files**: Live alongside project files, support cross-project references

### 5. Pattern Matching

```bash
wsb refactor . "oldname" "newname" --ignore-case
wsb refactor . "old_\\w+" "new_name" --regex
```

[View Full API Reference]({{ '/api-reference/' | relative_url }}){: .btn .btn-outline }
