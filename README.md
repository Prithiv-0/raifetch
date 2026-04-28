<div align="center">

```
  ██████╗  █████╗ ██╗███████╗███████╗████████╗ ██████╗██╗  ██╗
  ██╔══██╗██╔══██╗██║██╔════╝██╔════╝╚══██╔══╝██╔════╝██║  ██║
  ██████╔╝███████║██║█████╗  █████╗     ██║   ██║     ███████║
  ██╔══██╗██╔══██║██║██╔══╝  ██╔══╝     ██║   ██║     ██╔══██║
  ██║  ██║██║  ██║██║██║     ███████╗   ██║   ╚██████╗██║  ██║
  ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚═╝     ╚══════╝   ╚═╝    ╚═════╝╚═╝  ╚═╝
```

**A fast, multithreaded system information fetch tool written in Rust.**  
Inspired by [fastfetch](https://github.com/fastfetch-cli/fastfetch) — with a native image backend and full TOML configuration.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey?style=flat-square&logo=linux)

</div>

---

## Features

- **Parallel info collection** via `rayon` — all modules run concurrently for optimal performance.
- **Image backends**: Kitty graphics protocol, Sixel (foot/xterm), and Unicode block fallback.
- **ASCII distro logos** — bundled for Arch, EndeavourOS, Ubuntu, Debian, NixOS, Fedora, Manjaro, openSUSE, Pop!\_OS, and Linux Mint, with support for custom file loading.
- **Progress bars** on Memory, Swap, Disk, and Battery with threshold coloring.
- **NerdFont icons** (optional, set `icons = true` in config).
- **Label alignment** — all values start at the same column for a clean layout.
- **Distro auto-colors** — `label_color = "auto"` automatically picks your distribution's brand color.
- **Package counts** cached for 5 minutes to ensure fast repeated runs.
- **Wayland-native** resolution detection (Hyprland, sway, kscreen).
- **Fully configurable** via TOML with no required setup.

## Modules

| Module | Information Displayed |
|---|---|
| OS | Distro name and architecture |
| Host | Hardware model (from DMI) |
| Kernel | Linux kernel version |
| Uptime | System uptime |
| CPU | Model, core count, frequency, usage % |
| GPU | GPU vendor and model |
| Memory | Used / Total and progress bar |
| Swap | Used / Total and progress bar |
| Disk | Used / Total, progress bar, and filesystem (all mounts optional) |
| Battery | Charge %, status icon, and progress bar |
| Network | Interface name and local IP |
| Resolution | Active display resolution and refresh rate |
| Shell | Shell name and version |
| Terminal | Terminal emulator |
| DE | Desktop environment |
| WM | Window manager (auto-hidden when same as DE) |
| Packages | Package counts (pacman, dpkg, rpm, flatpak, snap) |
| Locale | LANG and timezone |
| Colors | 16-color palette swatches |
| Custom | Run any shell command as a module |

## Installation

### From Source

Ensure you have Rust and Cargo installed, then run the following:

```bash
git clone https://github.com/Prithiv-0/raifetch.git
cd raifetch
cargo build --release
./target/release/raifetch --install
```
This installs the binary to `~/.local/bin/raifetch`.

Make sure `~/.local/bin` is in your `PATH`. For example, in Zsh:
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
```

## Configuration

`raifetch` reads its configuration from `~/.config/raifetch/config.toml`.  
To generate the default configuration file:

```bash
raifetch --generate-config > ~/.config/raifetch/config.toml
```

### Key Options

```toml
[general]
show_header  = true    # Show user@host header
auto_hide_wm = true    # Hide WM when it matches the DE

[image]
logo_type    = "auto"  # Options: auto | image | ascii | none
path         = "~/Pictures/logo.png"
ascii_distro = "auto"  # Auto-detect, or specify: arch | ubuntu | debian | endeavour | nixos | fedora | manjaro | opensuse | pop | mint
ascii_file   = ""      # Path to a custom ASCII art .txt file

[theme]
label_color  = "auto"  # 'auto' uses distro brand color. Other options: bright_cyan, red, green, etc.
value_color  = "white"
bold_labels  = false
icons        = false   # Enable NerdFont icons (requires a Nerd Font)
align_labels = true    # Pad labels to the same width
bar_width    = 20
bar_fill     = "█"
bar_empty    = "░"

[modules]
show_all_disks = false # Set to true to show all mount points

# Custom shell modules
[[modules.custom]]
key     = "Weather"
command = "curl -s 'wttr.in/?format=1'"
```

## CLI Flags

```
raifetch [OPTIONS]

Options:
  -i, --image <PATH>          Override image path
      --no-image              Disable image/logo
      --backend <BACKEND>     Force backend: kitty | sixel | block
      --color <WHEN>          Color output: auto | always | never
      --module <MODULE>       Print a single module (for status bars)
      --config-path           Print config file location
      --generate-config       Print default config to stdout
      --list-modules          List all available module names
      --install               Copy binary to ~/.local/bin
  -h, --help                  Print help
  -V, --version               Print version
```

### Single Module Output
This is particularly useful for integration with tools like `waybar` or `polybar`:

```bash
raifetch --module memory    # Prints only the memory information
raifetch --module cpu       # Prints only the CPU information
```

## Image Backends

| Backend | Supported Terminals | Protocol |
|---|---|---|
| `kitty` | Kitty | Kitty graphics protocol |
| `sixel` | foot, xterm, mlterm | Sixel |
| `block` | Any (fallback) | Unicode half-blocks + TrueColor |

The backend is auto-detected. You can force a specific backend using `--backend kitty`.

## License

MIT
