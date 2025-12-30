# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

render-prompt is a minimalist template rendering tool written in Rust. It performs variable substitution (`{{ var }}`) and file inclusion (`{{> file }}`) on plain text templates using YAML/JSON data files. The tool is intentionally kept simple, explicitly NOT supporting conditionals, loops, filters, or code execution.

## Common Commands

### Build and Run
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with arguments
cargo run -- --template template.txt --data data.yaml

# Run release binary
./target/release/rp -t template.txt -d data.yaml -o output.txt
```

### Testing
```bash
# Run all tests
cargo test

# Run specific module tests
cargo test data::merger
cargo test template::engine
cargo test variable

# Run tests with detailed output
cargo test -- --nocapture

# Release build tests
cargo test --release
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# All warnings
cargo clippy -- -W clippy::all
```

## Architecture

### Processing Pipeline

The tool follows a strict processing order (critical for maintaining correctness):

1. **CLI Parsing** (`cli.rs`) - Parse and validate arguments
2. **Data Loading** (`data/loader.rs`) - Load YAML/JSON files
3. **Data Merging** (`data/merger.rs`) - Deep merge multiple data files (later files override earlier ones)
4. **Template Loading** (`template/engine.rs`) - Read main template file
5. **Include Resolution** (`template/include.rs`) - Recursively expand `{{> file }}` directives
6. **Variable Substitution** (`template/variable.rs`) - Replace `{{ var }}` with values
7. **Output** (`main.rs`) - Write to file or stdout

### Module Structure

```
src/
├── main.rs              # Entry point, orchestrates the pipeline
├── cli.rs               # CLI argument definitions (using clap)
├── error.rs             # Error types and exit codes (2-7)
├── data/
│   ├── loader.rs        # YAML/JSON loading (converts to serde_json::Value)
│   └── merger.rs        # Deep merge logic for combining data files
└── template/
    ├── engine.rs        # Main orchestrator for template rendering
    ├── include.rs       # Include directive processor (with safety checks)
    └── variable.rs      # Variable substitution with dot notation support
```

### Key Design Principles

**Include Resolution (`template/include.rs`)**:
- Processes ALL includes first, before any variable substitution
- Includes are recursive (nested includes are supported)
- Safety features:
  - Circular include detection using `HashSet<PathBuf>` to track visited files
  - Depth limit (default: 20, configurable via `--max-include-depth`)
  - Path traversal prevention using `canonicalize()` and root directory validation
  - All included file paths are resolved relative to the template's directory or `--root`

**Variable Substitution (`template/variable.rs`)**:
- Happens AFTER all includes are resolved
- Supports dot notation: `{{ user.profile.name }}`
- Supports array indexing: `{{ items.0 }}`, `{{ matrix.1.2 }}`
- Escape sequences: `\{{` becomes literal `{{` in output
- Type handling: strings, numbers, booleans rendered as-is; objects/arrays as JSON
- Two modes:
  - Default: undefined variables → empty string
  - Strict (`--strict`): undefined variables → error with exit code 6
  - Warning (`--warn-undefined`): undefined variables → warning to stderr

**Data Merging (`data/merger.rs`)**:
- Multiple `-d` files are deep-merged left-to-right (later wins)
- Objects: recursive merge by key
- Arrays: complete replacement (no element merging)
- Primitives: later value overwrites

### Error Handling

All errors use the `RenderError` enum defined in `error.rs`. Each error type maps to a specific exit code:

| Exit Code | Error Type | Examples |
|-----------|------------|----------|
| 0 | Success | - |
| 2 | Usage error | Missing required args, validation failures |
| 3 | Template error | Template file not found/unreadable |
| 4 | Data error | Data file not found, invalid YAML/JSON |
| 5 | Include error | Include file not found, path traversal |
| 6 | Variable error | Undefined variable in strict mode |
| 7 | Circular/depth | Circular includes, depth limit exceeded |

Errors produce both human-readable and machine-readable output to stderr.

### Important Implementation Notes

**Processing Order is Critical**: The spec explicitly requires includes to be resolved before variable substitution. This means:
- Variables in include directives are NOT supported: `{{> {{ filename }} }}` won't work
- All `{{> path }}` must be literal paths
- Variables are only substituted after the full include tree is expanded

**Regex Patterns**: Variable and include patterns are pre-compiled using `lazy_static` for performance:
- Variable: `\{\{\s*([a-zA-Z_][a-zA-Z0-9_.]*)\s*\}\}`
- Include: `\{\{>\s*([^\}]+)\s*\}\}`
- Escape: `\\\{\{`

**No Template Logic**: The tool explicitly does NOT support:
- Conditionals (if/else)
- Loops (for/each)
- Filters or functions
- Mathematical expressions
- Custom code execution

This is by design. Complex logic should be handled in the data files or in preprocessing steps.

## Testing Strategy

Tests are located in two places:
1. **Unit tests**: In `#[cfg(test)]` modules within each source file
2. **Integration tests**: In `tests/` directory (using `assert_cmd` for CLI testing)
3. **Test data**: Hand-crafted examples in `test_data/` for manual verification

When writing tests:
- Use `tempfile::tempdir()` for creating temporary test files
- Test both success and error cases for each error type
- Verify exit codes match the specification
- Test edge cases: empty files, circular includes, deep nesting, special characters
- Include tests for escape sequences and variable resolution edge cases

## Development Workflow

1. Make changes to source files
2. Run `cargo fmt` to format
3. Run `cargo clippy` to check for issues
4. Run `cargo test` to verify tests pass
5. Test manually with `cargo run -- -t ... -d ...`
6. For changes affecting the spec, verify against `README.md` examples
