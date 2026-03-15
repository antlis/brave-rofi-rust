# brave-rofi-rust

A Rofi-based menu for managing Brave browser tabs, bookmarks, and history. Uses the Chrome DevTools Protocol (CDP) to communicate with the browser.

## Features

- List and switch between browser tabs
- Search (regular and incognito)
- Browse and manage bookmarks
- View browsing history
- Open new tabs
- Close individual tabs or all tabs

## Requirements

- Rust (latest stable)
- A Wayland/i3-wm environment with Rofi
- A supported browser (Brave Beta, Brave, Zen Browser, or Chromium)
- `curl`, `i3-msg`, and `surfraw` in PATH

## Installation

```bash
cargo install --git https://github.com/antlis/brave-rofi-rust.git
```

This installs `brave-rofi` and `bbr` to `$HOME/.cargo/bin/`.

## Configuration

Set the `$BROWSER` environment variable to choose your browser:

```bash
export BROWSER=brave-beta  # Default
export BROWSER=brave
export BROWSER=zen
export BROWSER=chromium
```

## Usage

```bash
# Run the application
cargo run

# Or if installed
brave-rofi
bbr  # Short alias
```

## Keybindings

From the main menu:
- Select a tab number to switch to that tab
- `Search` - Open Brave Search
- `Bookmarks` - Browse bookmarks
- `Bookmarks incognito` - Open bookmarks in incognito mode
- `New Tab` - Open a new blank tab
- `Close Tab` - Close selected tabs
- `Close ALL Tabs` - Close all browser tabs
- `Search in incognito` - Search in incognito mode
- `History` - Browse browsing history
- `Exit` - Exit the application
