---
title: Contributing
layout: default
---

## Development Setup

1. Fork the repository
2. Clone your fork:

```bash
git clone https://github.com/your-username/workspace
```

3. Build the project:

```bash
cargo build
```

## Testing

Run the full test suite:

```bash
cargo test
```

## Contribution Guidelines

- Follow Rust coding conventions
- Include tests for new features
- Update documentation accordingly
- Use descriptive commit messages
- Open an issue before major changes

## Code Structure

```
src/
├── bin/
│   └── ws.rs               # CLI entry point (clap parser, command dispatch)
├── lib.rs                   # Library root
├── refac/                   # Refactor tool (string replacement)
│   ├── mod.rs
│   └── binary_detector.rs
├── st8/                     # Version management
│   ├── mod.rs
│   ├── st8_common.rs        # Version calculation, project file updates
│   ├── templates.rs         # Tera template manager
│   └── wstemplate.rs        # Cross-project .wstemplate engine
├── workspace_state.rs       # Per-project state (WstemplateEntry, .ws/state.json)
├── entities/                # Entity system (features, tasks, directives, etc.)
├── ldiff/                   # Line difference visualizer
├── logging.rs               # Structured logging
├── code_analysis/           # AST-based code analysis
├── mcp_protocol.rs          # MCP server protocol handler
└── interactive_tree.rs      # Interactive tree display
```
