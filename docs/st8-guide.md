---
layout: default
title: St8 Guide - Version Management & Wstemplate
---

# St8 Guide

St8 is the version management component of Workspace. It integrates with Git to provide automatic semantic versioning, template rendering, and cross-project version stamping via `.wstemplate` files.

## How Version Calculation Works

St8 uses a three-part versioning scheme:

- **Major Version**: Set via `ws version major` (stored in the project database)
- **Minor Version**: Total number of commits in the repository
- **Patch Version**: Total number of line changes (additions + deletions)

**Final Version Format**: `{major}.{minor}.{patch}`

### Example

```bash
# Repository state:
# - Major version set to 2 (via ws version major 2)
# - Total commits: 150
# - Total line changes: 4200

# Result: 2.150.4200
```

## Installation and Setup

### 1. Install Workspace

```bash
./install.sh
```

### 2. Install Git Hook

Navigate to your Git repository and install the pre-commit hook:

```bash
cd your-git-repo
ws git install
```

This creates a pre-commit hook that automatically updates your version file before each commit.

### 3. Configure (Optional)

Create a `.st8.json` configuration file in your repository root:

```json
{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt"
}
```

**Configuration Options:**
- `enabled`: Enable/disable automatic version updates
- `version_file`: Path to the file where version is written (default: `version.txt`)
- `auto_detect_project_files`: Automatically detect and update common project files (default: `true`)
- `project_files`: Array of additional project files to update

## Basic Usage

### Install Hook
```bash
ws git install
ws git install --force    # Force reinstall
```

### Show Version Information
```bash
ws git show
ws version show
ws version show --verbose
ws version show --format json
```

### Manual Version Update
```bash
ws update                 # Update version and render templates
ws update --git-add       # Also stage changed files
ws update --no-git        # Skip git integration
```

### Set Major Version
```bash
ws version major 1        # Set major to 1
ws version major 2        # Bump major to 2
```

### Create Git Tag
```bash
ws version tag                    # Tag with current version
ws version tag --prefix "release-"  # Custom prefix
ws version tag --message "Release"  # With message
```

### Check Status
```bash
ws git status
```

### Uninstall Hook
```bash
ws git uninstall
```

## Workflow Integration

### Automatic Mode (Recommended)

1. Install the git hook once: `ws git install`
2. Work normally — commit as usual
3. Version files are automatically updated before each commit:
   - `version.txt` (or custom version file)
   - All detected project files (Cargo.toml, package.json, etc.)
   - All `.wstemplate` files relevant to this project
4. Updated output files are automatically staged (version.txt, rendered templates)

**Note**: The `.ws` directory (containing `state.json`, logs, databases) is local project state and should be in `.gitignore`. It is never staged by `ws update`.

### Manual Mode

1. Set `"enabled": false` in `.st8.json`
2. Run `ws update` when needed
3. Use `ws update --git-add` to stage updated files
4. Commit version changes manually

## Project File Auto-Detection

St8 automatically detects and updates version fields in common project files:

**Supported File Types:**
- **Cargo.toml** (Rust): `version = "x.y.z"` in `[package]` section
- **package.json** (Node.js): `"version": "x.y.z"`
- **pyproject.toml** (Python): `version = "x.y.z"` in `[tool.poetry]` and `[project]` sections
- **setup.py** (Python): `version="x.y.z"` parameter
- **composer.json** (PHP): `"version": "x.y.z"`
- **pubspec.yaml** (Dart/Flutter): `version: x.y.z`
- **pom.xml** (Maven/Java): `<version>x.y.z</version>`
- **build.gradle** (Gradle): `version = 'x.y.z'`
- **CMakeLists.txt** (C/C++): `VERSION x.y.z` in `project()` declaration

**Disable Auto-Detection:**
```json
{
  "auto_detect_project_files": false
}
```

## Template System (.tera)

Workspace supports Tera templates for generating files with version information.

### Managing Templates

```bash
ws template add version-header \
  --template "Version: {{ project.version }}" \
  --output version.h

ws template list
ws template show version-header
ws template render
ws template delete version-header
```

Templates are rendered automatically during `ws update`.

## Wstemplate System (.wstemplate)

The wstemplate system provides cross-project version stamping. A `.wstemplate` file is a Tera template that renders to the file with the `.wstemplate` suffix stripped.

### Single-Entry Model

Each project has at most one wstemplate entry in its `.ws/state.json`:
- **alias**: A Tera-compatible identifier for this project (auto-derived from directory name)
- **root**: The directory tree to scan for `.wstemplate` files and peer `.ws/state.json` files

Cross-project references are resolved dynamically — no explicit cross-project entries are needed.

### Setup

```bash
# Register this project with the wstemplate system
ws wstemplate add /path/to/workspace

# Verify the entry
ws wstemplate list-entries
# Output:
# 1 wstemplate entries:
#   my_project -> /path/to/workspace
```

The alias is auto-derived from the project directory name:
- `my-project` becomes `my_project`
- `API Service` becomes `api_service`
- `123abc` becomes `p_123abc`

Override with `--alias`:
```bash
ws wstemplate add /path/to/workspace --alias mylib
```

### Template Variables

| Variable | Description |
|----------|-------------|
| `{{ project.version }}` | Owning project's full version |
| `{{ project.major_version }}` | e.g., `v0` |
| `{{ project.minor_version }}` | Commit count |
| `{{ project.patch_version }}` | Line changes |
| `{{ project.name }}` | Project name |
| `{{ projects.ALIAS.version }}` | Any discoverable project's version |
| `{{ projects.ALIAS.* }}` | Same fields as `project.*` |
| `{{ datetime.iso }}` | RFC 3339 timestamp |
| `{{ datetime.date }}` | YYYY-MM-DD |
| `{{ datetime.time }}` | HH:MM:SS |
| `{{ datetime.year }}` | Year |
| `{{ datetime.month }}` | Month |
| `{{ datetime.day }}` | Day |

### Example: Cargo.toml.wstemplate

```toml
[package]
name = "my-app"
version = "{{ project.version }}"

[dependencies]
my-lib = { path = "../my-lib", version = "{{ projects.my_lib.version }}" }
```

When `ws update` runs in this project (or in `my-lib`), this template renders to `Cargo.toml` with actual version numbers.

### Example: package.json.wstemplate

```json
{
  "name": "@scope/my-app",
  "version": "{{ project.version }}",
  "dependencies": {
    "@scope/tagged-urn": "{{ projects.tagged_urn_js.version }}"
  }
}
```

### How Template Selection Works

When `ws update` (or `ws wstemplate render`) runs for project X:

1. All `.wstemplate` files in the scan root are discovered
2. A template is rendered if:
   - It lives under project X's root (own templates), OR
   - Its text contains `{{ projects.X_ALIAS.* }}` (references to X)
3. For each template, `{{ project.* }}` resolves to the **owning** project's version
4. `{{ projects.ALIAS.* }}` resolves dynamically from the alias's `version.txt`

### Dynamic Cross-Project Resolution

The engine discovers all projects by scanning for `.ws/state.json` files in the scan root. When a template references `{{ projects.other_lib.version }}`:

1. The engine finds `other_lib`'s project root from its `state.json`
2. Reads `{project_root}/version.txt`
3. Parses the version and populates the template variable

### Error Handling

All errors are hard failures — no silent fallbacks:

- **Unresolvable alias**: Lists all known aliases so you can fix the reference
- **Missing `version.txt`**: Tells you to run `ws update` in the dependency project
- **Multiple entries in state.json**: Reports the count and explains single-entry constraint
- **Duplicate aliases across projects**: Reports both projects claiming the same alias

### Batch Setup Script

For a multi-project workspace, create a setup script:

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"

register() {
    local project="$1"
    echo "=== $project ==="
    cd "$ROOT/$project"
    ws wstemplate add "$ROOT"
    echo
}

register my-lib
register my-app
register my-tests

echo "Done. Run 'ws update' in any project to render its templates."
```

### Wstemplate Commands

```bash
ws wstemplate add <PATH> [--alias <ALIAS>]  # Set scan root
ws wstemplate remove <ALIAS>                 # Remove entry
ws wstemplate list-entries                    # Show this project's entry
ws wstemplate list                            # List relevant templates
ws wstemplate render                          # Render all relevant templates
```

## Troubleshooting

### Hook Not Running

1. Check if hook is installed: `ws git status`
2. Verify hook file exists: `ls -la .git/hooks/pre-commit`
3. Ensure hook is executable: `chmod +x .git/hooks/pre-commit`

### Version Not Updating

1. Check git repository status: `git status`
2. Test manually: `ws git show`
3. Check configuration: `ws git status`

### Wstemplate Not Rendering

1. Check entry exists: `ws wstemplate list-entries`
2. Check for relevant templates: `ws wstemplate list`
3. Try explicit render: `ws wstemplate render`
4. Check that dependency projects have `version.txt` (run `ws update` in them)

### Removing Version Management

```bash
ws git uninstall        # Remove git hook
rm .st8.json            # Remove configuration (optional)
rm version.txt          # Remove version file (optional)
ws wstemplate remove my_alias  # Remove wstemplate entry (optional)
```

## Logging

St8 logs all actions to `.ws/logs/ws.log`:

```bash
tail -f .ws/logs/ws.log
```

## Best Practices

1. **Install Early**: Set up versioning when creating a new repository
2. **Use `ws version major`**: Set major version via database, not git tags
3. **Consistent Workflow**: Let the hook handle versioning automatically
4. **Shared Scan Root**: For multi-project workspaces, point all projects' wstemplate to the common root
5. **Run `ws update` After Setup**: Ensure `version.txt` exists before other projects reference it
6. **Add `.ws` to `.gitignore`**: The `.ws` directory is local state — do not commit it

## Integration Examples

### CI/CD Pipeline

```bash
VERSION=$(cat version.txt)
echo "Building version: $VERSION"
docker build -t myapp:$VERSION .
```

### GitHub Actions

```yaml
- name: Get Version
  id: version
  run: echo "version=$(cat version.txt)" >> $GITHUB_OUTPUT

- name: Create Release
  uses: actions/create-release@v1
  with:
    tag_name: v${{ steps.version.outputs.version }}
    release_name: Release ${{ steps.version.outputs.version }}
```

### Makefile Integration

```makefile
VERSION := $(shell cat version.txt)

build:
	@echo "Building version $(VERSION)"
	cargo build --release
```

## See Also

- [Installation Guide]({{ '/installation/' | relative_url }})
- [Getting Started]({{ '/getting-started/' | relative_url }})
- [API Reference]({{ '/api-reference/' | relative_url }})
