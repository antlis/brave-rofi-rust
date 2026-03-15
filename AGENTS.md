# AGENTS.md - Developer Guide

This file contains guidelines and instructions for agentic coding agents operating in this repository.

## Project Overview

`brave-rofi-rust` is a Rofi-based menu for managing Brave browser tabs, bookmarks, and history. It uses the Chrome DevTools Protocol (CDP) to communicate with the browser.

## Build Commands

```bash
# Build the project
cargo build

# Build in release mode (for production)
cargo build --release

# Run the application
cargo run

# Run with specific browser (overrides $BROWSER env var)
ROFI_BROWSER=brave-beta cargo run
```

## Lint and Code Quality

```bash
# Run clippy for linting
cargo clippy

# Fix automatically fixable clippy warnings
cargo clippy --fix --allow-dirty

# Format code with rustfmt
cargo fmt

# Check formatting without modifying files
cargo fmt --check

# Run all checks (fmt + clippy + build)
cargo check
```

## Testing

```bash
# Run all tests
cargo test

# Run a specific test by name
cargo test test_name

# Run tests with output visible
cargo test -- --nocapture

# Run doc tests
cargo test --doc
```

## Code Style Guidelines

### Error Handling

- Use `anyhow::Result<T>` for application code (see `src/main.rs:24`)
- Use the `?` operator for error propagation
- Use `anyhow!()` macro for creating errors with context (e.g., `anyhow!("No debugger URL")`)
- Use `ok_or_else(|| anyhow!(...))` for converting `Option` to `Result`

### Naming Conventions

- **Functions/variables**: `snake_case` (e.g., `get_tabs`, `config.history_path`)
- **Types/structs**: `PascalCase` (e.g., `Tab`, `BrowserConfig`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `CDP_PORT`)
- **Module names**: `snake_case` (e.g., `search`, `bookmarks`)

### Imports

- Group imports by external crate, then standard library, then local
- Sort alphabetically within each group
- Use `use` for bringing items into scope (not full paths in function signatures)

Example (from `src/main.rs`):
```rust
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::process::{Command, Stdio};
use std::io::Write;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use config::BrowserConfig;
```

### Struct Definitions

- Use `#[derive(Debug, Clone)]` for structs that need debug output and cloning
- Group related fields logically
- Document public fields with doc comments

Example (from `src/config.rs`):
```rust
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub name: String,
    pub executable: String,
    pub history_path: String,
    pub window_class: String,
    pub cdp_port: u16,
}
```

### Async Code

- Use `#[tokio::main]` for the main async entry point
- Use `tokio::task::spawn_blocking` for CPU-bound operations (database, file I/O)
- Prefer `async` functions over blocking where I/O is involved

### Database Access

- Use `rusqlite` with `OpenFlags::SQLITE_OPEN_READ_ONLY` for reading browser history
- Copy database to `/tmp` before opening to avoid locking issues (see `src/history.rs:10`)

### Module Organization

- Use submodules with `mod module_name;` in parent
- Use `pub mod` for publicly accessible submodules
- Put shared helpers in the parent module (e.g., `prompt` in `src/search/mod.rs`)

Example (`src/search/mod.rs`):
```rust
pub mod regular;
pub mod incognito;

pub fn prompt(query_label: &str) -> String { ... }
```

### String Handling

- Use `String` for owned strings, `&str` for borrowed strings
- Use `format!` for building strings
- Use `String::new()` for empty string initialization

### Option Handling

- Use `unwrap_or()` / `unwrap_or_default()` for default values
- Use `ok_or_else()` to convert to Result
- Use `map_or()` for conditional transformations

### Command Execution

- Use `std::process::Command` for external commands
- Always handle potential failures appropriately (use `?` or explicit error handling)
- Use `.spawn()` for non-blocking execution with `&` or `spawn()?`

### Configuration

- Browser is configured via `$BROWSER` environment variable
- Falls back to `brave-beta` if not set
- Supported values: `brave-beta`, `brave`, `zen`, `chromium`
- Configuration is read via `BrowserConfig::from_env()`

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, CDP communication, rofi menu |
| `src/config.rs` | Browser configuration |
| `src/bookmarks.rs` | Bookmark management |
| `src/history.rs` | History viewing |
| `src/search/` | Search functionality (regular + incognito) |

## Common Patterns

### CDP (Chrome DevTools Protocol)

```rust
let version: serde_json::Value = reqwest_blocking(&cdp_url)?;
let ws_url = version["webSocketDebuggerUrl"]
    .as_str()
    .ok_or_else(|| anyhow!("No debugger URL"))?;
let (mut ws, _) = connect_async(Url::parse(ws_url)?).await?;
send_cdp(&mut ws, json!({ "id": 1, "method": "Method.name", "params": {} })).await?;
```

### Rofi Menu

```rust
fn show_menu(prompt: &str, items: &str) -> Result<String> {
    let mut child = Command::new("rofi")
        .args(["-dmenu", "-i", "-p", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(items.as_bytes())?;
        stdin.flush()?;
    }
    
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

## Dependencies

- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket client for CDP
- `futures-util` - Async utilities
- `serde_json` - JSON serialization
- `anyhow` - Error handling
- `url` / `urlencoding` - URL handling
- `rusqlite` - SQLite database access
