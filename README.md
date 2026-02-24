# wsb

[![Crates.io](https://img.shields.io/crates/v/wsb.svg)](https://crates.io/crates/wsb)
[![Downloads](https://img.shields.io/crates/d/wsb.svg)](https://crates.io/crates/wsb)
[![License](https://img.shields.io/crates/l/wsb.svg)](https://crates.io/crates/wsb)

Multi-tool CLI suite for file operations, version management, and development workflow automation. All tools accessible through a single `wsb` binary.

## Install

```bash
cargo install wsb
```

Or build from source:

```bash
git clone https://github.com/jowharshamshiri/wsb.git
cd wsb
cargo build --release
```

## Core Tools

| Command | Description |
|---------|-------------|
| `wsb refactor` | Recursive string replacement in files and directories |
| `wsb scrap` | Local trash can with `.scrap` folder |
| `wsb unscrap` | Restore files from `.scrap` folder |
| `wsb ldiff` | Line difference visualization for pattern analysis |
| `wsb git` | Git integration (install/uninstall hooks, show version, status) |
| `wsb update` | Update version file and render all templates |
| `wsb template` | Tera template management (add, list, show, render, delete) |
| `wsb wstemplate` | Cross-project version stamping with `.wstemplate` files |
| `wsb version` | Version management (show, major, tag, info) |
| `wsb code` | AST-based code analysis and transformation |
| `wsb test` | Intelligent test runner based on project type |
| `wsb status` | Project status with feature metrics and progress |
| `wsb feature` | Feature management with state machine workflow |
| `wsb task` | Feature-centric task management |
| `wsb directive` | Project directive and rule management |
| `wsb note` | Note management for any entity |
| `wsb mcp-server` | MCP server for Claude AI integration |

## Quick Start

```bash
# Recursive string replacement
wsb refactor ./src "OldClassName" "NewClassName" --verbose

# Safe file disposal and recovery
wsb scrap temp_files/ experimental/
wsb unscrap important_config.toml

# Git-based versioning with auto-detection of Cargo.toml, package.json, etc.
wsb git install
wsb update --git-add

# Cross-project version stamping
wsb wstemplate add /path/to/workspace-root
wsb wstemplate render

# Line difference visualization
tail -f /var/log/app.log | wsb ldiff

# Project management
wsb feature add "User authentication"
wsb task add "Implement login" --feature F00001
wsb status --include-features
```

## Version Management

wsb uses a three-part versioning scheme: `{major}.{minor}.{patch}`

- **Major**: Set via `wsb version major` (stored in project database)
- **Minor**: Total commits in the repository
- **Patch**: Total line changes (additions + deletions)

Version is automatically stamped into project files (Cargo.toml, package.json, pyproject.toml, etc.) via a git pre-commit hook.

## Cross-Project Version Stamping

`.wstemplate` files are Tera templates that render to the file with the suffix stripped (e.g., `Cargo.toml.wstemplate` renders to `Cargo.toml`). Templates can reference any sibling project's version:

```toml
[package]
name = "my-app"
version = "{{ project.version }}"

[dependencies]
my-lib = { path = "../my-lib", version = "{{ projects.my_lib.version }}" }
```

Cross-project references are resolved dynamically by scanning for `.wsb/state.json` files — no explicit dependency declarations needed.

## Documentation

- [Installation Guide](https://jowharshamshiri.github.io/wsb/installation/)
- [Getting Started](https://jowharshamshiri.github.io/wsb/getting-started/)
- [Usage Guide](https://jowharshamshiri.github.io/wsb/usage/)
- [St8 Guide (Versioning & Wstemplate)](https://jowharshamshiri.github.io/wsb/st8-guide/)
- [API Reference](https://jowharshamshiri.github.io/wsb/api-reference/)

## License

MIT
