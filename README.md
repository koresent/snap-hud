# snap-hud

Lightweight Wayland screenshot mode selector and HUD overlay built with GTK4 and Cairo.

Provides an instant, keyboard-driven popover to select between regional capture, window capture, and full screen capture, styled with the Gruvbox Dark palette.

## Features

- **Keyboard First:** Select capture mode with single keypresses (`1`/`2`/`3` or `a`/`w`/`s`), with transparent Cyrillic layout support (`ф`/`ц`/`ы`).
- **Low Latency:** Forces Cairo GSK rendering at startup to avoid cold-start shader compilation delays on small overlays under NVIDIA/Vulkan drivers.
- **Pretty Mode:** Optional `--pretty` flag for screenshots with rounded corners and drop shadows.
- **Clean Handoff:** Dismisses itself and introduces a short delay for compositor fade animations before invoking the capture tool (`snap-capture`).

## Keyboard Controls

| Mode | Number Key | Latin Key | Cyrillic Key | Action |
| :--- | :---: | :---: | :---: | :--- |
| **Area** | `1` | `a` | `ф` | Interactive region selection via slurp/flameshot |
| **Window** | `2` | `w` | `ц` | Window picker targeting active Wayland surfaces |
| **Screen** | `3` | `s` | `ы` | Active monitor capture via grim |
| **Cancel** | `Esc` | `q` | `й` | Dismiss HUD without taking action |

## Usage

```bash
# Launch default HUD
snap-hud

# Launch HUD with rounded drop-shadow styling enabled
snap-hud --pretty
```

When a mode is chosen, `snap-hud` executes `snap-capture <mode> [--pretty]` (falling back to `$HOME/.local/bin/snap-capture` or `snap-capture` in PATH).

## Building from Source

### Dependencies

On Arch Linux / CachyOS:
```bash
sudo pacman -S gtk4 pkg-config rust
```

On Debian / Ubuntu:
```bash
sudo apt-get install -y pkg-config libgtk-4-dev
```

### Compilation

```bash
cargo build --release
install -m 755 target/release/snap-hud ~/.local/bin/snap-hud
```
