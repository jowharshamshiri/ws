# wsb

[![Crates.io](https://img.shields.io/crates/v/wsb.svg)](https://crates.io/crates/wsb)
[![Downloads](https://img.shields.io/crates/d/wsb.svg)](https://crates.io/crates/wsb)
[![License](https://img.shields.io/crates/l/wsb.svg)](https://crates.io/crates/wsb)

Multi-tool CLI suite for file operations, version management, and development workflow automation. All tools accessible through a single `ws` binary.

## Install

```bash
cargo install wsb
```

Or build from source:

```bash
git clone https://github.com/jowharshamshiri/ws.git
cd ws
cargo build --release
```

## Core Tools

| Command | Description |
|---------|-------------|
| `ws refactor` | Recursive string replacement in files and directories |
| `ws scrap` | Local trash can with `.scrap` folder |
| `ws unscrap` | Restore files from `.scrap` folder |
| `ws ldiff` | Line difference visualization for pattern analysis |
| `ws git` | Git integration (install/uninstall hooks, show version, status) |
| `ws update` | Update version file and render all templates |
| `ws template` | Tera template management (add, list, show, render, delete) |
| `ws wstemplate` | Cross-project version stamping with `.wstemplate` files |
| `ws version` | Version management (show, major, tag, info) |
| `ws code` | AST-based code analysis and transformation |
| `ws test` | Intelligent test runner based on project type |
| `ws status` | Project status with feature metrics and progress |
| `ws feature` | Feature management with state machine workflow |
| `ws task` | Feature-centric task management |
| `ws directive` | Project directive and rule management |
| `ws note` | Note management for any entity |
| `ws mcp-server` | MCP server for Claude AI integration |

## Quick Start

```bash
# Recursive string replacement
ws refactor ./src "OldClassName" "NewClassName" --verbose

# Safe file disposal and recovery
ws scrap temp_files/ experimental/
ws unscrap important_config.toml

# Git-based versioning with auto-detection of Cargo.toml, package.json, etc.
ws git install
ws update --git-add

# Cross-project version stamping
ws wstemplate add /path/to/workspace-root
ws wstemplate render

# Line difference visualization
tail -f /var/log/app.log | ws ldiff

# Project management
ws feature add "User authentication"
ws task add "Implement login" --feature F00001
ws status --include-features
```

## Version Management

wsb uses a three-part versioning scheme: `{major}.{minor}.{patch}`

- **Major**: Set via `ws version major` (stored in project database)
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

Cross-project references are resolved dynamically by scanning for `.ws/state.json` files — no explicit dependency declarations needed.

## Documentation

- [Installation Guide](https://jowharshamshiri.github.io/ws/installation/)
- [Getting Started](https://jowharshamshiri.github.io/ws/getting-started/)
- [Usage Guide](https://jowharshamshiri.github.io/ws/usage/)
- [St8 Guide (Versioning & Wstemplate)](https://jowharshamshiri.github.io/ws/st8-guide/)
- [API Reference](https://jowharshamshiri.github.io/ws/api-reference/)

## License

MIT
