---
layout: default
title: Testing & Quality Assurance
---

# Testing & Quality Assurance

Workspace maintains reliability and safety through comprehensive testing.

## Test Suite Overview

### Running Tests

```bash
cargo test                    # Run all tests
cargo test --lib              # Run unit tests only
cargo test --test <SUITE>     # Run specific integration suite
cargo test -- --nocapture     # With output
```

### Test Suites

| Test Suite | Focus Area |
|------------|------------|
| **Unit tests** (in `src/`) | Core logic: version calculation, alias derivation, template rendering, state management, entity models |
| **integration_tests.rs** | End-to-end workflows across tools |
| **refac_concurrency_tests.rs** | Multi-threading safety, race conditions |
| **refac_edge_cases_tests.rs** | Deep nesting, special characters, unicode |
| **refac_empty_directory_tests.rs** | Directory handling, permissions |
| **refac_encoding_tests.rs** | UTF-8, BOM, invalid encodings |
| **scrap_advanced_integration_tests.rs** | Archive, search, metadata |
| **scrap_integration_tests.rs** | Basic scrap/unscrap operations |
| **st8_integration_tests.rs** | Version management, git hooks |
| **st8_template_tests.rs** | Template rendering and management |
| **code_analysis_tests.rs** | AST parsing and code analysis |
| **entity_manager_tests.rs** | Entity CRUD operations |
| **database_system_tests.rs** | Database operations |

## Safety Features

### Pre-Operation Validation

Every refactor operation undergoes validation before execution:
- File accessibility and permissions
- Encoding compatibility
- Path validation
- Collision detection

### Race Condition Prevention

Files are processed before directories to prevent path invalidation. Deepest paths are processed first when renaming.

### Encoding Safety

Binary files are automatically detected and skipped. UTF-8 validation prevents crashes on invalid encodings.

## Quality Standards

### Memory Safety

Rust's ownership model provides compile-time guarantees:
- No buffer overflows
- No use-after-free
- No data races
- Automatic memory management

## Test Writing Guidelines

When adding new features:

1. **Write tests that expose real issues** — not tautological tests that pass by definition
2. **Cover edge cases** — boundary conditions, error scenarios, missing files
3. **Use descriptive names** — test names should explain the scenario
4. **Use tempfile** — all tests should use temporary directories for isolation

```rust
#[test]
fn test_specific_scenario() {
    let temp_dir = TempDir::new().unwrap();
    // Arrange: Set up test environment
    // Act: Execute the operation
    // Assert: Verify expected outcomes
    // Cleanup handled automatically by TempDir
}
```
