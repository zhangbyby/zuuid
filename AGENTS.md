# AGENTS.md

Guidelines for agentic coding agents working in the zuuid repository.

## Project Overview

zuuid is a single-binary CLI tool for generating UUID v4 and v7 identifiers. It supports multiple output formats (uppercase/lowercase, with/without hyphens) and internationalization (English/Chinese).

## Build/Lint/Test Commands

```bash
# Build (release mode)
cargo build --release

# Run the CLI
cargo run -- [args]
cargo run -- -v7 -us           # v7, uppercase, simple format
cargo run -- -n 5              # generate 5 UUIDs
cargo run -- -V 4 -f -U        # v4, full format, uppercase

# Run all tests
cargo test

# Run a single test
cargo test test_generate_default_format
cargo test test_cli_parse      # runs all tests matching pattern

# Run tests with output
cargo test -- --nocapture

# Install locally
cargo install --path .

# Lint and format
cargo fmt
cargo clippy
cargo clippy -- -D warnings    # treat warnings as errors
```

## Code Style Guidelines

### Imports

- Group imports at the top of the file
- External crates first, then std imports within function bodies as needed
- Use `use super::*;` in test modules

```rust
use clap::Parser;
use uuid::Uuid;
```

### Formatting

- Use `cargo fmt` before committing
- Max line length: 100 characters (rustfmt default)
- Indent with 4 spaces
- Match arms align with `=>` operator

### Types and Enums

- Use `#[derive(...)]` attributes for common traits
- Implement `Default` for enums with a natural default value
- Use `#[default]` attribute for default enum variant

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum UuidVersion {
    #[default]
    V4,
    V7,
}
```

### Naming Conventions

- **Types/Enums/Structs**: PascalCase (`UuidVersion`, `Language`)
- **Functions/Variables**: snake_case (`generate_uuid`, `prefer_full`)
- **Constants**: SCREAMING_SNAKE_CASE (if any)
- **CLI short flags**: uppercase primary, lowercase alias (e.g., `-U`/`-u`)
- Boolean variables: use `is_` prefix for predicates, or descriptive adjectives (`uppercase`, `simple`, `full`)

### Error Handling

- Return `Result<T, String>` for fallible operations
- Use `String` as error type for simple error messages
- Provide localized error messages when appropriate

```rust
impl std::str::FromStr for UuidVersion {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ... Err(msgs.invalid_version(s))
    }
}
```

### Documentation

- Use `///` doc comments for public items
- Include brief description and examples in doc comments
- Document CLI arguments with help text

```rust
/// UUID version to generate
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum UuidVersion {
    /// Version 4: Random UUID (default)
    #[default]
    V4,
}
```

### CLI Arguments (clap)

- Use clap derive API with `#[derive(Parser)]`
- Provide both short (`-V`) and long (`--uuid-version`) forms
- Use `visible_short_alias` for additional short flags
- Set sensible defaults with `default_value`

```rust
#[arg(short = 'V', long = "uuid-version", visible_short_alias = 'v', default_value = "4")]
version: UuidVersion,
```

### Testing

- Place tests in `#[cfg(test)] mod tests { use super::*; ... }`
- Name tests descriptively: `test_<function>_<scenario>`
- Use `Cli::try_parse_from(["zuuid", ...])` for CLI parsing tests
- Use `assert!`, `assert_eq!`, `assert_ne!` for assertions
- Test both positive and negative cases

```rust
#[test]
fn test_generate_default_format() {
    let uuid = generate_uuid(UuidVersion::V4, false, false, false, false);
    assert!(uuid.len() == 36);
    assert!(uuid.contains('-'));
}
```

### Internationalization

- Use a `Language` enum to detect and store language preference
- Create a `Messages` struct with methods returning localized strings
- Detect language from `LANG`, `LC_ALL`, `LC_MESSAGES` environment variables
- Default to English for unsupported languages

### Pattern Matching

- Use `match` for exhaustive handling of enum variants
- Use `_ => {}` for ignored cases in flag parsing

```rust
match (full_pos, simple_pos) {
    (Some(f), Some(s)) => { /* ... */ }
    _ => (true, false),
}
```

### Boolean Parameters

- When functions have multiple boolean flags, document their meaning
- Use tuples to return multiple boolean states

```rust
/// Returns (prefer_full, conflict_detected)
fn determine_format_precedence() -> (bool, bool)
```

### ANSI Escape Codes

- Use `\x1b[33m` for yellow warnings, `\x1b[0m` to reset
- Print warnings to stderr with `eprintln!`

## Architecture Notes

- Single-file binary: all code in `src/main.rs`
- No internal modules or separate library crate
- Dependencies: `uuid` (with `v4`, `v7` features), `clap` (with `derive` feature)
- Rust edition: 2024; minimum Rust: 1.89

## Common Patterns

### Format Precedence

Use clap `ArgMatches` occurrence indices to preserve first-format-flag precedence, including combined short flags. Never scan raw command-line strings: long options like `--uuid-version` contain characters that are not flags.

```rust
matches.index_of("full") < matches.index_of("simple")
```

Only compare positions when both flags are explicitly set. Locale selection uses the first nonempty value in `LC_ALL`, `LC_MESSAGES`, `LANG` order. Use child-process environments for tests instead of mutating the test process environment.

### UUID Generation

Resolve `OutputFormat` once. `write_uuids` generates UUIDs and passes them to `encode_uuid`, reusing `Uuid::encode_buffer()` and the lower/uppercase encoder. Write through a single buffered stdout lock; propagate write/flush errors and treat `BrokenPipe` as successful early termination.

`--version` reports the program version; `-V` / `-v` still selects UUID v4/v7. Integration tests in `tests/cli.rs` exercise the real executable.
