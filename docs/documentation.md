---
title: Documentation
layout: default
permalink: /documentation/
---

## Core Concepts

### 1. All-in-One Binary

All tools are subcommands of the `ws` binary:

```bash
ws refactor   # String replacement
ws scrap      # Local trash
ws unscrap    # File restoration
ws git        # Git integration
ws update     # Version update + template rendering
ws wstemplate # Cross-project version stamping
ws version    # Database-driven versioning
ws ldiff      # Line difference visualization
ws status     # Project status
ws feature    # Feature management
ws task       # Task management
ws test       # Intelligent test runner
ws code       # AST code analysis
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
- **`.tera` templates**: Managed via `ws template`, stored in `.ws/templates/`
- **`.wstemplate` files**: Live alongside project files, support cross-project references

### 5. Pattern Matching

```bash
ws refactor . "oldname" "newname" --ignore-case
ws refactor . "old_\\w+" "new_name" --regex
```

[View Full API Reference]({{ '/api-reference/' | relative_url }}){: .btn .btn-outline }
