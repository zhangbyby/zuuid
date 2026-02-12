# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

```bash
# Build
cargo build --release

# Run
cargo run -- [args]

# Run specific examples
cargo run -- -v7 -us                    # v7, uppercase, simple format
cargo run -- -n 5                       # generate 5 UUIDs
cargo run -- -V 4 -f -U                 # v4, full format, uppercase

# Test
cargo test                              # all tests
cargo test test_generate_default_format # single test
cargo test test_cli_parse               # all CLI parsing tests

# Install locally
cargo install --path .

# Lint/format
cargo fmt
cargo clippy
```

## Architecture

This is a single-binary CLI tool (`src/main.rs`) with no modules. The codebase uses `clap` (derive API) for argument parsing and the `uuid` crate for UUID generation.

### Key Components

**Language Detection (`Language` enum, `Messages` struct):**
- Auto-detects locale from `LANG`, `LC_ALL`, or `LC_MESSAGES` environment variables
- Supports English and Chinese with localized error/warning messages
- Defaults to English if no Chinese locale is detected

**Format Precedence (`determine_format_precedence`):**
- Handles conflicting `-f` (full/hyphens) and `-s` (simple/no hyphens) flags
- Parses raw command-line args to determine flag order
- Uses composite position (arg_index * 1000 + char_offset) for combined flags like `-fs`, `-sf`
- Returns `(prefer_full, conflict_detected)` tuple

**UUID Generation (`generate_uuid`):**
- Uses `Uuid::new_v4()` for random UUIDs
- Uses `Uuid::now_v7()` for time-ordered UUIDs
- Applies formatting (uppercase, hyphens) after generation

**CLI Arguments (`Cli` struct):**
- clap derive with short/long aliases (e.g., `-U`/`-u` for uppercase, `-s`/`-S` for simple)
- Version accepts: `4`, `7`, `v4`, `v7`, `V4`, `V7`
- `count` parameter generates multiple UUIDs in a loop

### Dependencies

- `uuid`: v4 and v7 UUID generation
- `clap`: CLI argument parsing with derive feature
