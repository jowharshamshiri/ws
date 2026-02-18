---
layout: default
title: Examples
---

# Real-World Examples

Learn how to use the Workspace tool suite through practical examples for common development workflows, combining refactoring, project cleanup, and version control.

## Development Workflows

### Project Refactoring with Version Tracking

**Scenario**: Major refactoring of a class while maintaining version history and safely managing temporary files.

```bash
# 1. Set up version tracking
ws git install

# 2. Move unwanted files to local trash can
ws scrap temp_files/ debug_logs/ old_tests/

# 3. Preview and apply refactoring
ws refactor ./src "UserManager" "AccountManager" --verbose
ws refactor ./src "UserManager" "AccountManager" --include "*.rs" --include "*.toml"

# 4. Commit changes (auto-bumps version)
git add .
git commit -m "Refactor: UserManager -> AccountManager"

# 5. Check new version
ws git show
cat version.txt
```

### Safe Experimental Development

**Scenario**: Trying experimental changes with easy rollback.

```bash
# 1. Backup current implementation
ws scrap src/experimental.rs src/core.rs

# 2. Note current version
ws version show

# 3. Make experimental changes
ws refactor ./src "old_algorithm" "new_algorithm" --content-only

# 4. If experiment fails, restore easily
ws unscrap experimental.rs
ws unscrap core.rs

# 5. If experiment succeeds, commit
git add . && git commit -m "Implement new algorithm"
```

### CI/CD Preparation Workflow

**Scenario**: Preparing a release with proper cleanup and versioning.

```bash
# 1. Archive old artifacts
ws scrap find "*.tmp" "*.log" "target/debug/*"
ws scrap archive pre-release-cleanup.tar.gz --remove

# 2. Update configuration for production
ws refactor ./config "dev.api.com" "prod.api.com" --content-only
ws refactor ./config "debug=true" "debug=false" --content-only

# 3. Ensure version tracking is set up
ws git status || ws git install

# 4. Create release commit
git add .
git commit -m "Prepare for production release"

# 5. Tag the release
ws version tag
```

---

## Individual Tool Examples

### Scrap - Local Trash Folder

#### Project Cleanup Instead of Deletion

**Scenario**: Moving files to trash that you don't want but might need later.

```bash
# Move unwanted files to local trash can instead of deleting
ws scrap *.tmp *.log build/ target/debug/

# List what you've scrapped
ws scrap list --sort size

# Find specific items you remember scrapping
ws scrap find "*.log"
ws scrap find "test" --content

# Clean up old items (older than 7 days) permanently
ws scrap clean --days 7
```

#### Safe Code Cleanup

**Scenario**: Removing old implementations and experimental code safely.

```bash
# Scrap old code instead of deleting (in case you need it)
ws scrap old_implementation/ legacy_tests/

# Archive before permanent removal
ws scrap archive "old-code-$(date +%Y%m%d).tar.gz"

# Scrap experimental features that didn't work out
ws scrap experimental_feature/ prototype/
ws scrap archive --output experiments-archive.tar.gz --remove
```

### Unscrap - File Restoration

#### Undo Recent Changes

**Scenario**: Need to restore files after a mistake.

```bash
# Restore the last scrapped item
ws unscrap

# Check what's available to restore
ws scrap list

# Restore specific file to original location
ws unscrap important_config.json

# Restore to a different location
ws unscrap data.sql --to backup/
```

#### Selective File Recovery

**Scenario**: Restore only specific files from a cleanup.

```bash
# After cleaning workspace, need one file back
ws scrap temp_files/ logs/ build/
# ... realize you need a log file
ws unscrap server.log

# Restore to custom location without overwriting
ws unscrap config.json --to ./backup/ --force
```

### Version Management

#### Project Setup and Release Management

**Scenario**: Setting up automatic versioning for a new project.

```bash
# Initialize project with versioning
git init
echo "Initial code" > main.rs
git add .
git commit -m "Initial commit"

# Install hook and set major version
ws git install
ws version major 1

# Normal development with automatic versioning
echo "new feature" >> main.rs
git add .
git commit -m "Add feature"  # Version auto-incremented

# Check version progression
ws version show
cat version.txt
```

#### CI/CD Integration

**Scenario**: Using versions in build scripts.

```bash
VERSION=$(cat version.txt)
echo "Building version: $VERSION"

docker build -t myapp:$VERSION .
cargo build --release
cp target/release/myapp "releases/myapp-$VERSION"
```

### Refactor - String Replacement

#### Rename Variables Throughout Project

**Scenario**: You need to rename a variable across your entire codebase.

```bash
# Preview the changes first
ws refactor ./src "oldVariableName" "newVariableName" --verbose

# Apply the changes
ws refactor ./src "oldVariableName" "newVariableName"
```

### Update API Endpoints

**Scenario**: Your API URL changed and you need to update all references.

```bash
# Update only file contents, don't rename files
ws refactor . "api.old-service.com" "api.new-service.com" --content-only

# Include only relevant file types
ws refactor . "api.old-service.com" "api.new-service.com" \
  --content-only \
  --include "*.js" \
  --include "*.py" \
  --include "*.json"
```

## File and Directory Organization

### Rename Project Files

**Scenario**: You're renaming your project from "MyApp" to "AwesomeApp".

```bash
ws refactor . "MyApp" "AwesomeApp" --verbose

ws refactor . "MyApp" "AwesomeApp" \
  --exclude "node_modules/*" \
  --exclude ".git/*" \
  --exclude "target/*"
```

### Reorganize File Naming Convention

**Scenario**: Change file naming from `camelCase` to `snake_case`.

```bash
ws refactor ./src "camelCase" "snake_case" --names-only
ws refactor ./src "([a-z])([A-Z])" "\$1_\$2" --names-only --regex
```

## Language-Specific Refactoring

### Rust Project Refactoring

```bash
ws refactor ./src "OldStruct" "NewStruct" \
  --include "*.rs" \
  --include "*.toml"

ws refactor ./src "OldStruct" "NewStruct" \
  --include "*.rs" \
  --backup
```

### JavaScript/TypeScript Project

```bash
ws refactor ./src "oldFunction" "newFunction" \
  --include "*.js" \
  --include "*.ts" \
  --include "*.jsx" \
  --include "*.tsx"

ws refactor ./src "oldFunction" "newFunction" \
  --include "*.js" \
  --include "*.ts" \
  --exclude "*test*" \
  --exclude "*spec*"
```

### Python Project

```bash
ws refactor ./project "OldClass" "NewClass" \
  --include "*.py" \
  --exclude "__pycache__/*"

ws refactor ./project "old-package" "new-package" \
  --include "*.py" \
  --include "requirements*.txt" \
  --include "setup.py"
```

## Configuration and Deployment

### Update Environment Variables

```bash
ws refactor ./config "OLD_ENV_VAR" "NEW_ENV_VAR" \
  --include "*.env" \
  --include "*.yml" \
  --include "*.yaml" \
  --include "*.json"

ws refactor ./config "staging.server.com" "production.server.com" \
  --content-only \
  --include "*.env" \
  --include "*.config"
```

### Docker and Deployment Scripts

```bash
ws refactor ./deployment "old-service" "new-service" \
  --include "*.yml" \
  --include "*.yaml" \
  --include "Dockerfile*" \
  --include "*.sh"
```

## Database and Schema Changes

### Update Table Names

```bash
ws refactor ./sql "old_table" "new_table" \
  --include "*.sql" \
  --include "*.migration"

ws refactor ./src "old_table" "new_table" \
  --include "*.py" \
  --include "*.js" \
  --include "*.rb"
```

## Patterns

### Using Regular Expressions

```bash
ws refactor ./docs "v1\\.\\d+\\.\\d+" "v2.0.0" \
  --regex \
  --include "*.md" \
  --include "*.txt"

ws refactor ./src "oldfunction" "newFunction" \
  --regex \
  --ignore-case \
  --include "*.js"
```

### Batch Operations with Scripts

```bash
#!/bin/bash
REPLACEMENTS=(
  "OldClass1:NewClass1"
  "OldClass2:NewClass2"
  "old_function:new_function"
)

for replacement in "${REPLACEMENTS[@]}"; do
  IFS=':' read -r old new <<< "$replacement"
  ws refactor ./src "$old" "$new" --include "*.rs" --force
done
```

## Safety and Testing

### Safe Refactoring Workflow

```bash
#!/bin/bash
OLD_NAME="$1"
NEW_NAME="$2"
PROJECT_DIR="$3"

# Preview changes
ws refactor "$PROJECT_DIR" "$OLD_NAME" "$NEW_NAME" --verbose --verbose

read -p "Continue? (y/N): " confirm
[ "$confirm" != "y" ] && exit 0

# Apply with backup
ws refactor "$PROJECT_DIR" "$OLD_NAME" "$NEW_NAME" --backup

# Run tests
cargo test || npm test || python -m pytest
```

## Tool Suite Workflows

### Full-Stack Application Refactoring

```bash
#!/bin/bash
OLD_NAME="$1"
NEW_NAME="$2"

# 1. Clean workspace
ws scrap target/ build/ *.log *.tmp

# 2. Preview and apply
ws refactor . "$OLD_NAME" "$NEW_NAME" --verbose
ws refactor . "$OLD_NAME" "$NEW_NAME" --backup

# 3. Commit (triggers version bump)
git add .
git commit -m "Refactor: $OLD_NAME -> $NEW_NAME"

# 4. Show results
cat version.txt
```

### Release Pipeline Integration

```bash
#!/bin/bash
# 1. Clean build artifacts
ws scrap target/ build/ *.log
ws scrap purge --force

# 2. Update configurations
ws refactor ./config "development" "production" --content-only
ws refactor ./config "debug=true" "debug=false" --content-only

# 3. Create release
git add .
git commit -m "Prepare production release"
ws version tag
echo "Release $(cat version.txt) prepared"
```
