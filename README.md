# brave-rofi-rust

A Rofi menu for switching browser **tabs, bookmarks, and history** — from *any* window,
not just when the browser is focused.

Bind it to a key in your window manager and it becomes a global switcher: hit the key from
your editor, terminal, or anything else, pick a tab / bookmark / history entry, and the
browser is brought forward on the right page. It talks to the browser over the
**Chrome DevTools Protocol (CDP)** and hands window-focusing off to a hook you configure,
so it isn't tied to any one WM.

> **Status: draft / work in progress.** This scratches a personal itch and is rough around
> the edges — there's dead code, a few half-finished ideas, and the name may change if it
> grows into something more consistent and feature-rich. See [Status & limitations](#status--limitations).

## Features

- List and switch between open tabs (via CDP `Target.activateTarget`)
- Open a **bookmark** from anywhere (read from your surfraw bookmarks file)
- Open a **history** entry from anywhere (read from the browser's History database)
- Search (regular and incognito)
- Open a new tab; close selected tabs or all tabs

## How it works

```
  any focused window
        │  (WM keybinding runs `brave-rofi`)
        ▼
   brave-rofi ──curl──▶ http://localhost:9222/json     list open tabs (CDP)
        │
        ▼
     rofi menu ──▶ you pick a tab / bookmark / history entry
        │
        ├─ tab       →  CDP Target.activateTarget      (switch tab inside the browser)
        ├─ bookmark  →  launch $BROWSER <url>          (~/.config/surfraw/bookmarks)
        └─ history   →  launch $BROWSER <url>          (copy of the History sqlite)
        │
        ▼
   $BROWSER_POST_SWITCH_HOOK  e.g. `i3-msg … focus`    raise the browser window
```

CDP switches the *tab inside the browser*, but it can't raise the browser's OS window — that's
what the post-switch hook is for. Splitting those two responsibilities is what lets the whole
thing work no matter which application currently has focus.

## Requirements

- Rust (latest stable)
- [Rofi](https://github.com/davatorium/rofi)
- `curl` in `PATH`
- A supported **Chromium-based** browser: Brave Beta, Brave, or Chromium
- The browser must be running with **remote debugging enabled**, e.g. launch it with
  `--remote-debugging-port=9222`
- For bookmarks: a surfraw-format bookmarks file at `~/.config/surfraw/bookmarks`
  (surfraw itself is not required — only the file format is reused)

## Installation

```bash
cargo install --git https://github.com/antlis/brave-rofi-rust.git
```

This installs `brave-rofi` and its short alias `bbr` to `$HOME/.cargo/bin/`.

## Configuration

Choose your browser with `$BROWSER` (default: `brave-beta`):

```bash
export BROWSER=brave-beta   # default
export BROWSER=brave
export BROWSER=chromium
```

Set `$BROWSER_POST_SWITCH_HOOK` to a shell command that focuses the browser window after a
tab is switched or opened. It runs through `sh -c`, so anything works:

```bash
# i3
export BROWSER_POST_SWITCH_HOOK='i3-msg [class="Brave-browser"] focus'

# sway / Wayland
export BROWSER_POST_SWITCH_HOOK='swaymsg [app_id="brave-browser"] focus'

# generic X11
export BROWSER_POST_SWITCH_HOOK='wmctrl -xa brave-browser'
```

Then bind it in your WM. Example for i3:

```
bindsym $mod+b exec --no-startup-id brave-rofi
```

### Bookmarks file

Bookmarks are read from `~/.config/surfraw/bookmarks`. Each line is `name url`, e.g.:

```
github    https://github.com
astro     https://astro.build
```

## Usage

```bash
brave-rofi     # or the short alias:
bbr

cargo run      # from a source checkout
```

The main menu lists your open tabs, followed by these actions:

| Item | What it does |
|------|--------------|
| `<n>. <title>` | Switch to that tab |
| `Search (<browser>)` | Search with your browser's default engine |
| `- Bookmarks` | Open a bookmark |
| `- Bookmarks incognito` | Open a bookmark in an incognito window |
| `- New Tab` | Open a new blank tab |
| `- Close Tab` | Multi-select tabs to close |
| `- Close ALL Tabs` | Close every tab (with confirmation) |
| `- Search in incognito` | Search in an incognito window |
| `- History` | Open a history entry |
| `- Exit` | Quit |

## Status & limitations

This is a personal, draft-quality tool. Known rough edges:

- **Chromium-family only.** Firefox-based browsers (including Zen) speak a different remote
  protocol and store history in a different schema (`moz_places` in `places.sqlite`), so tab
  switching and history don't work there. Zen support was started but never finished.
- **Remote debugging must be on.** If the browser wasn't launched with
  `--remote-debugging-port=9222`, the tab list will be empty.
- **Dead code / leftovers.** Some code paths are unused or half-migrated from earlier
  approaches and haven't been cleaned up yet.
- **Name is not final.** If this grows more consistent and feature-rich, it may be renamed.

Contributions and issues are welcome, but expect churn.

## License

MIT OR Apache-2.0.
