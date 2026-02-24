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
wsb git install

# 2. Move unwanted files to local trash can
wsb scrap temp_files/ debug_logs/ old_tests/

# 3. Preview and apply refactoring
wsb refactor ./src "UserManager" "AccountManager" --verbose
wsb refactor ./src "UserManager" "AccountManager" --include "*.rs" --include "*.toml"

# 4. Commit changes (auto-bumps version)
git add .
git commit -m "Refactor: UserManager -> AccountManager"

# 5. Check new version
wsb git show
cat version.txt
```

### Safe Experimental Development

**Scenario**: Trying experimental changes with easy rollback.

```bash
# 1. Backup current implementation
wsb scrap src/experimental.rs src/core.rs

# 2. Note current version
wsb version show

# 3. Make experimental changes
wsb refactor ./src "old_algorithm" "new_algorithm" --content-only

# 4. If experiment fails, restore easily
wsb unscrap experimental.rs
wsb unscrap core.rs

# 5. If experiment succeeds, commit
git add . && git commit -m "Implement new algorithm"
```

### CI/CD Preparation Workflow

**Scenario**: Preparing a release with proper cleanup and versioning.

```bash
# 1. Archive old artifacts
wsb scrap find "*.tmp" "*.log" "target/debug/*"
wsb scrap archive pre-release-cleanup.tar.gz --remove

# 2. Update configuration for production
wsb refactor ./config "dev.api.com" "prod.api.com" --content-only
wsb refactor ./config "debug=true" "debug=false" --content-only

# 3. Ensure version tracking is set up
wsb git status || wsb git install

# 4. Create release commit
git add .
git commit -m "Prepare for production release"

# 5. Tag the release
wsb version tag
```

---

## Individual Tool Examples

### Scrap - Local Trash Folder

#### Project Cleanup Instead of Deletion

**Scenario**: Moving files to trash that you don't want but might need later.

```bash
# Move unwanted files to local trash can instead of deleting
wsb scrap *.tmp *.log build/ target/debug/

# List what you've scrapped
wsb scrap list --sort size

# Find specific items you remember scrapping
wsb scrap find "*.log"
wsb scrap find "test" --content

# Clean up old items (older than 7 days) permanently
wsb scrap clean --days 7
```

#### Safe Code Cleanup

**Scenario**: Removing old implementations and experimental code safely.

```bash
# Scrap old code instead of deleting (in case you need it)
wsb scrap old_implementation/ legacy_tests/

# Archive before permanent removal
wsb scrap archive "old-code-$(date +%Y%m%d).tar.gz"

# Scrap experimental features that didn't work out
wsb scrap experimental_feature/ prototype/
wsb scrap archive --output experiments-archive.tar.gz --remove
```

### Unscrap - File Restoration

#### Undo Recent Changes

**Scenario**: Need to restore files after a mistake.

```bash
# Restore the last scrapped item
wsb unscrap

# Check what's available to restore
wsb scrap list

# Restore specific file to original location
wsb unscrap important_config.json

# Restore to a different location
wsb unscrap data.sql --to backup/
```

#### Selective File Recovery

**Scenario**: Restore only specific files from a cleanup.

```bash
# After cleaning workspace, need one file back
wsb scrap temp_files/ logs/ build/
# ... realize you need a log file
wsb unscrap server.log

# Restore to custom location without overwriting
wsb unscrap config.json --to ./backup/ --force
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
wsb git install
wsb version major 1

# Normal development with automatic versioning
echo "new feature" >> main.rs
git add .
git commit -m "Add feature"  # Version auto-incremented

# Check version progression
wsb version show
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
wsb refactor ./src "oldVariableName" "newVariableName" --verbose

# Apply the changes
wsb refactor ./src "oldVariableName" "newVariableName"
```

### Update API Endpoints

**Scenario**: Your API URL changed and you need to update all references.

```bash
# Update only file contents, don't rename files
wsb refactor . "api.old-service.com" "api.new-service.com" --content-only

# Include only relevant file types
wsb refactor . "api.old-service.com" "api.new-service.com" \
  --content-only \
  --include "*.js" \
  --include "*.py" \
  --include "*.json"
```

## File and Directory Organization

### Rename Project Files

**Scenario**: You're renaming your project from "MyApp" to "AwesomeApp".

```bash
wsb refactor . "MyApp" "AwesomeApp" --verbose

wsb refactor . "MyApp" "AwesomeApp" \
  --exclude "node_modules/*" \
  --exclude ".git/*" \
  --exclude "target/*"
```

### Reorganize File Naming Convention

**Scenario**: Change file naming from `camelCase` to `snake_case`.

```bash
wsb refactor ./src "camelCase" "snake_case" --names-only
wsb refactor ./src "([a-z])([A-Z])" "\$1_\$2" --names-only --regex
```

## Language-Specific Refactoring

### Rust Project Refactoring

```bash
wsb refactor ./src "OldStruct" "NewStruct" \
  --include "*.rs" \
  --include "*.toml"

wsb refactor ./src "OldStruct" "NewStruct" \
  --include "*.rs" \
  --backup
```

### JavaScript/TypeScript Project

```bash
wsb refactor ./src "oldFunction" "newFunction" \
  --include "*.js" \
  --include "*.ts" \
  --include "*.jsx" \
  --include "*.tsx"

wsb refactor ./src "oldFunction" "newFunction" \
  --include "*.js" \
  --include "*.ts" \
  --exclude "*test*" \
  --exclude "*spec*"
```

### Python Project

```bash
wsb refactor ./project "OldClass" "NewClass" \
  --include "*.py" \
  --exclude "__pycache__/*"

wsb refactor ./project "old-package" "new-package" \
  --include "*.py" \
  --include "requirements*.txt" \
  --include "setup.py"
```

## Configuration and Deployment

### Update Environment Variables

```bash
wsb refactor ./config "OLD_ENV_VAR" "NEW_ENV_VAR" \
  --include "*.env" \
  --include "*.yml" \
  --include "*.yaml" \
  --include "*.json"

wsb refactor ./config "staging.server.com" "production.server.com" \
  --content-only \
  --include "*.env" \
  --include "*.config"
```

### Docker and Deployment Scripts

```bash
wsb refactor ./deployment "old-service" "new-service" \
  --include "*.yml" \
  --include "*.yaml" \
  --include "Dockerfile*" \
  --include "*.sh"
```

## Database and Schema Changes

### Update Table Names

```bash
wsb refactor ./sql "old_table" "new_table" \
  --include "*.sql" \
  --include "*.migration"

wsb refactor ./src "old_table" "new_table" \
  --include "*.py" \
  --include "*.js" \
  --include "*.rb"
```

## Patterns

### Using Regular Expressions

```bash
wsb refactor ./docs "v1\\.\\d+\\.\\d+" "v2.0.0" \
  --regex \
  --include "*.md" \
  --include "*.txt"

wsb refactor ./src "oldfunction" "newFunction" \
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
  wsb refactor ./src "$old" "$new" --include "*.rs" --force
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
wsb refactor "$PROJECT_DIR" "$OLD_NAME" "$NEW_NAME" --verbose --verbose

read -p "Continue? (y/N): " confirm
[ "$confirm" != "y" ] && exit 0

# Apply with backup
wsb refactor "$PROJECT_DIR" "$OLD_NAME" "$NEW_NAME" --backup

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
wsb scrap target/ build/ *.log *.tmp

# 2. Preview and apply
wsb refactor . "$OLD_NAME" "$NEW_NAME" --verbose
wsb refactor . "$OLD_NAME" "$NEW_NAME" --backup

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
wsb scrap target/ build/ *.log
wsb scrap purge --force

# 2. Update configurations
wsb refactor ./config "development" "production" --content-only
wsb refactor ./config "debug=true" "debug=false" --content-only

# 3. Create release
git add .
git commit -m "Prepare production release"
wsb version tag
echo "Release $(cat version.txt) prepared"
```
