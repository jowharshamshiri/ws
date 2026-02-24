---
layout: default
title: Command Reference
---

# Command Reference

Complete reference for all `wsb` command-line options and usage patterns.

## Commands Overview

All tools are subcommands of the `wsb` binary:

```bash
wsb <COMMAND> [OPTIONS]
```

| Command | Description |
|---------|-------------|
| `refactor` | Recursive string replacement in files and directories |
| `git` | Git integration and version management |
| `template` | Tera template management |
| `update` | Update version and render templates |
| `wsbtemplate` | Cross-project `.wstemplate` version stamping |
| `version` | Database-driven version management |
| `scrap` | Local trash can using `.scrap` folder |
| `unscrap` | Restore files from `.scrap` folder |
| `ldiff` | Line difference visualization |
| `code` | AST-based code analysis and transformation |
| `test` | Intelligent test runner based on project type |
| `status` | Project status with feature metrics |
| `feature` | Feature management with state machine workflow |
| `task` | Feature-centric task management |
| `directive` | Project directive and rule management |
| `note` | Note management for any entity |
| `relationship` | Entity relationship management |
| `start` | Start development session |
| `end` | End development session |
| `continuity` | Session continuity and context management |
| `consolidate` | Documentation consolidation |
| `database` | Database backup, recovery, maintenance |
| `mcp-server` | MCP server for Claude AI integration |
| `sample` | Create sample project with test data |

---

## wsb refactor

Recursive string replacement in file names and contents with collision detection.

### Synopsis
```bash
wsb refactor <ROOT_DIR> <OLD_STRING> <NEW_STRING> [OPTIONS]
```

### Required Arguments

| Argument | Description |
|----------|-------------|
| `ROOT_DIR` | Root directory to search in (use `.` for current directory) |
| `OLD_STRING` | String to find and replace |
| `NEW_STRING` | Replacement string |

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--assume-yes` | `-y` | Skip confirmation prompts | `false` |
| `--force` | `-f` | Skip confirmation prompt | `false` |
| `--verbose` | `-v` | Show detailed output | `false` |
| `--backup` | `-b` | Create backup files before modifying | `false` |
| `--follow-symlinks` | | Follow symbolic links | `false` |
| `--files-only` | | Only process files (skip directories) | `false` |
| `--dirs-only` | | Only process directories (skip files) | `false` |
| `--names-only` | | Skip content replacement, only rename | `false` |
| `--content-only` | | Skip renaming, only replace content | `false` |
| `--include <PATTERN>` | | Include only files matching glob (repeatable) | all |
| `--exclude <PATTERN>` | | Exclude files matching glob (repeatable) | none |
| `--max-depth <N>` | | Maximum depth to search (0 = unlimited) | `0` |
| `--threads <N>` | `-j` | Number of threads (0 = auto) | `0` |
| `--ignore-case` | `-i` | Case-insensitive matching | `false` |
| `--regex` | `-r` | Use regex patterns | `false` |
| `--format <FORMAT>` | | Output format: `human`, `json`, `plain` | `human` |
| `--progress <MODE>` | | Progress display: `auto`, `always`, `never` | `auto` |

Only one mode flag (`--files-only`, `--dirs-only`, `--names-only`, `--content-only`) can be specified at a time.

### Examples
```bash
wsb refactor . "oldname" "newname"                          # Full replacement
wsb refactor . "oldname" "newname" --verbose                # Preview first
wsb refactor . "oldname" "newname" --include "*.rs" --backup
wsb refactor . "old.api.com" "new.api.com" --content-only
wsb refactor . "OldClass" "NewClass" --names-only
wsb refactor . "old_\\w+" "new_name" --regex
wsb refactor . "oldname" "newname" --format json            # Machine-readable output
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid arguments |
| `3` | Permission denied |
| `4` | File not found |
| `5` | Naming collision detected |

---

## wsb git

Git integration and version management via pre-commit hooks.

### Subcommands

| Subcommand | Description | Options |
|------------|-------------|---------|
| `install` | Install pre-commit hook | `--force` |
| `uninstall` | Remove hook | |
| `show` | Display version information | |
| `status` | Show configuration status | |

### Examples
```bash
wsb git install              # Install git hook
wsb git install --force      # Force reinstall
wsb git show                 # Show version info
wsb git status               # Check configuration
wsb git uninstall            # Remove hook
```

---

## wsb template

Tera template management for automatic file generation during `wsb update`.

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `add` | Add a new template |
| `list` | List all templates |
| `show` | Show template details |
| `update` | Update an existing template |
| `delete` | Remove a template |
| `render` | Render all enabled templates |

### Examples
```bash
wsb template add version-header --template "v{{ project.version }}" --output version.h
wsb template list
wsb template show version-header
wsb template render
wsb template delete version-header
```

---

## wsb update

Update version file and render all templates (both `.tera` and `.wstemplate`).

### Synopsis
```bash
wsb update [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--no-git` | Skip git integration | `false` |
| `--git-add` | Auto-add updated files to git staging | `false` |

### What It Does

1. Calculates the current version from git history
2. Writes `version.txt`
3. Updates project files (Cargo.toml, package.json, etc.) with the new version
4. Renders `.tera` templates via the template manager
5. Renders `.wstemplate` files via the wstemplate engine
6. With `--git-add`: stages `version.txt`, rendered `.tera` outputs, and rendered `.wstemplate` outputs

### Examples
```bash
wsb update                   # Basic update
wsb update --git-add         # Update and stage files
wsb update --no-git          # Update without git integration
```

---

## wsb wstemplate

Manage `.wstemplate` file rendering for cross-project version stamping.

Each project has a single wstemplate entry consisting of an **alias** (Tera identifier)
and a **scan root** (directory tree to search). Cross-project references like
`{{ projects.OTHER.version }}` are resolved dynamically by scanning the root for
sibling `.wsb/state.json` files.

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `add <PATH>` | Set the scan root (replaces any existing entry) |
| `remove <ALIAS>` | Remove this project's wstemplate entry |
| `list-entries` | Show this project's wstemplate entry |
| `list` | List all `.wstemplate` files relevant to this project |
| `render` | Render all relevant `.wstemplate` files |

### Options for `add`

| Option | Description |
|--------|-------------|
| `--alias <ALIAS>` | Override auto-derived alias (must be valid Tera identifier) |

### Template Variables

| Variable | Description |
|----------|-------------|
| `{{ project.version }}` | Owning project's full version (e.g., `0.5.12`) |
| `{{ project.major_version }}` | e.g., `v0` |
| `{{ project.minor_version }}` | Commit count |
| `{{ project.patch_version }}` | Line changes |
| `{{ project.name }}` | From manifest or directory name |
| `{{ projects.ALIAS.version }}` | Any discoverable project's version |
| `{{ projects.ALIAS.major_version }}` | Any project's major version |
| `{{ projects.ALIAS.minor_version }}` | Any project's minor version |
| `{{ projects.ALIAS.patch_version }}` | Any project's patch version |
| `{{ projects.ALIAS.name }}` | Any project's name |
| `{{ datetime.iso }}` | RFC 3339 timestamp |
| `{{ datetime.date }}` | YYYY-MM-DD |
| `{{ datetime.time }}` | HH:MM:SS |
| `{{ datetime.year }}` | Year |
| `{{ datetime.month }}` | Month |
| `{{ datetime.day }}` | Day |

### Template Selection

When rendering, only templates satisfying at least one condition are rendered:
1. The template lives under the current project's root (own templates)
2. The template's text contains a reference to `{{ projects.SELF_ALIAS.* }}`

### Error Handling

- **Unresolvable alias**: Hard error listing all known aliases
- **Missing `version.txt`**: Hard error with instructions to run `wsb update` in the dependency
- **Multiple wstemplate entries in state.json**: Hard error (single-entry model enforced)

### Examples
```bash
wsb wstemplate add /path/to/workspace              # Set scan root
wsb wstemplate add /path/to/workspace --alias mylib # Custom alias
wsb wstemplate list-entries                         # Show current entry
wsb wstemplate list                                 # Show relevant templates
wsb wstemplate render                               # Render templates
wsb wstemplate remove mylib                         # Remove entry
```

---

## wsb version

Version management with database-driven major version and git-calculated components.

### Version Format

`{major}.{minor}.{patch}` where:
- **Major**: Set via `wsb version major` (stored in database)
- **Minor**: Total commits in the repository
- **Patch**: Total line changes (additions + deletions)

### Subcommands

| Subcommand | Description | Options |
|------------|-------------|---------|
| `show` | Display version breakdown | `--verbose`, `--format` |
| `major <N>` | Set major version number | |
| `tag` | Create git tag with current version | `--prefix`, `--message` |
| `info` | Show calculation details | `--include-history` |

### Examples
```bash
wsb version show                          # Display version
wsb version show --verbose --format json  # Detailed JSON output
wsb version major 2                       # Set major to 2
wsb version tag                           # Create git tag
wsb version tag --prefix "release-"       # Custom tag prefix
wsb version info --include-history        # Show git history analysis
```

---

## wsb scrap

Local trash can using a `.scrap` folder for files you want to remove safely.

### Synopsis
```bash
wsb scrap [PATH...] [SUBCOMMAND] [OPTIONS]
```

### Subcommands

| Subcommand | Description | Options |
|------------|-------------|---------|
| `list` | List `.scrap` contents | `--sort name\|date\|size` |
| `clean` | Remove old items | `--days N` |
| `purge` | Remove all items | `--force` |
| `find` | Search for patterns | `--content` |
| `archive` | Create archive | `--output FILE`, `--remove` |

### Examples
```bash
wsb scrap temp.txt logs/                    # Move to .scrap
wsb scrap list --sort size                  # List contents
wsb scrap find "*.log"                      # Find files
wsb scrap clean --days 30                   # Remove old items
wsb scrap archive backup.tar.gz --remove    # Archive and remove
wsb scrap purge --force                     # Empty completely
```

---

## wsb unscrap

Restore files from `.scrap` folder to their original locations.

### Synopsis
```bash
wsb unscrap [NAME] [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--to PATH` | Custom restoration path |
| `--force` | Overwrite existing files |

### Examples
```bash
wsb unscrap                           # Restore last item
wsb unscrap important_file.txt        # Restore specific file
wsb unscrap config.json --to backup/  # Restore to directory
wsb unscrap data.txt --force          # Overwrite existing
```

---

## wsb ldiff

Process input lines, replacing repeated tokens with a substitute character to highlight differences.

### Synopsis
```bash
wsb ldiff [SUBSTITUTE_CHAR]
```

Default substitute character: `░`

### Examples
```bash
echo -e "hello world\nhello universe" | wsb ldiff
tail -f /var/log/syslog | wsb ldiff
cat access.log | wsb ldiff "*"
```

---

## wsb status

Display project status with feature metrics and progress tracking.

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--debug-mode` | Enable diagnostic output | `false` |
| `--include-features` | Include feature breakdown | `false` |
| `--include-metrics` | Include detailed metrics | `false` |
| `--format` | Output format: `human`, `json`, `summary` | `human` |

### Examples
```bash
wsb status
wsb status --include-features
wsb status --include-metrics --format json
```

---

## wsb feature

Feature management with state machine workflow and validation.

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `add` | Add a new feature |
| `list` | List features with filters |
| `show` | Show feature details |
| `update` | Update feature status/properties |

### Examples
```bash
wsb feature add "User authentication"
wsb feature list --state implemented
wsb feature show F00001
```

---

## wsb task

Feature-centric task management with automatic feature detection.

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `add` | Add a new task |
| `list` | List tasks with filters |
| `show` | Show task details |
| `update` | Update task status/properties |
| `complete` | Mark task as completed |
| `start` | Start working on a task |
| `block` | Mark task as blocked |
| `unblock` | Remove blocked status |

### Examples
```bash
wsb task add "Implement login" "Build the login page" --feature F00001
wsb task list --status pending
wsb task show T000001
wsb task start T000001
wsb task complete T000001 --evidence "Tests passing"
```

---

## wsb directive

Project directive and rule management.

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `add` | Add a new directive |
| `list` | List directives |
| `show` | Show directive details |
| `update` | Update directive |
| `activate` | Activate a directive |
| `deactivate` | Deactivate a directive |
| `check` | Check directive compliance |

---

## wsb code

AST-based code analysis and codebase exploration.

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `tree` | Visual tree of codebase structure (default) |
| `search` | Search for AST patterns in source code |

### Options for `tree`

| Option | Description | Default |
|--------|-------------|---------|
| `--depth N` | Maximum depth to display | `3` |
| `--hidden` | Show hidden files and directories | `false` |
| `--sizes` | Show file sizes | `false` |
| `--extensions` | Filter by file extensions (e.g., `rs,py,js`) | all |
| `--no-ignore` | Ignore .gitignore rules | `false` |

### Examples
```bash
wsb code                                # Show tree (default)
wsb code tree --depth 5 --sizes         # Deep tree with file sizes
wsb code tree --extensions rs,toml      # Only Rust files
wsb code search "fn main" --language rust
```

---

## wsb test

Intelligent test runner that detects project type and runs appropriate tests.

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--dry-run` | Show test command without executing | `false` |
| `--install` | Install missing test runners | `false` |

### Examples
```bash
wsb test                     # Run tests for detected project type
wsb test --dry-run           # Show what would run
wsb test -- --nocapture      # Pass args to test runner
```

---

## wsb start / wsb end

Session management for development workflow.

### wsb start
```bash
wsb start                                # Start new session
wsb start --continue-from T000001        # Continue from task
wsb start --project-setup                # Initialize new project
wsb start "Implement auth"               # Start with first task description
```

### wsb end
```bash
wsb end                                  # End session
wsb end --summary "Completed auth"       # End with summary
wsb end --skip-docs                      # Skip documentation updates
```

---

## wsb mcp-server

MCP server for Claude AI integration with automatic session management.

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--port` | HTTP server port | `3000` |
| `--debug` | Enable debug logging | `false` |
| `--migrate` | Migrate features from features.md to database | `false` |

### Examples
```bash
wsb mcp-server                    # Start on localhost:3000
wsb mcp-server --port 8080        # Custom port
wsb mcp-server --debug            # With debug logging
```

---

## wsb database

Database backup, recovery, and maintenance operations.

---

## wsb continuity

Session continuity and context management.

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `list` | List continuity records |
| `snapshot` | Create context snapshot |

---

## wsb consolidate

Documentation consolidation with diagram management.

### Options

| Option | Description |
|--------|-------------|
| `--generate-diagrams` | Generate architectural diagrams in DOT format |
| `--preserve-complexity` | Preserve complexity information |
| `--force` | Force consolidation |

---

## Getting Help

```bash
wsb --help              # Show all commands
wsb <COMMAND> --help    # Help for specific command
```
