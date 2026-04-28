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

- ⚡ **Parallel info collection** via `rayon` — all modules run concurrently
- 🖼️ **Image backends**: Kitty graphics protocol, Sixel (foot/xterm), Unicode block fallback
- 🎨 **ASCII distro logos** — bundled for Arch, EndeavourOS, Ubuntu, Debian, NixOS, Fedora, Manjaro, openSUSE, Pop!\_OS, Linux Mint; or load your own file
- 📊 **Progress bars** on Memory, Swap, Disk, and Battery with threshold colouring
- 🔤 **NerdFont icons** (optional, `icons = true` in config)
- 📐 **Label alignment** — all values start at the same column
- 🎨 **Distro auto-colours** — `label_color = "auto"` picks your distro's brand colour
- 📦 **Package counts** cached for 5 minutes — fast repeated runs
- 🖥️ **Wayland-native** resolution detection (Hyprland, sway, kscreen)
- 🔧 **Fully configurable** via TOML with no required setup

## Modules

| Module | Info shown |
|---|---|
| OS | Distro name + architecture |
| Host | Hardware model (from DMI) |
| Kernel | Linux kernel version |
| Uptime | System uptime |
| CPU | Model, core count, frequency, usage % |
| Memory | Used / Total + progress bar |
| Swap | Used / Total + progress bar |
| Disk | Used / Total + progress bar + filesystem (all mounts optional) |
| Battery | Charge %, status icon + progress bar |
| Network | Interface name + local IP |
| Resolution | Active display resolution + refresh rate |
| Shell | Shell name + version |
| Terminal | Terminal emulator |
| DE | Desktop environment |
| WM | Window manager (auto-hidden when same as DE) |
| Packages | Package counts (pacman, dpkg, rpm, flatpak, snap) |
| Locale | LANG + timezone |
| Colors | 16-color palette swatches |
| Custom | Run any shell command as a module |

## Installation

### From source

```bash
git clone https://github.com/yourname/raifetch
cd raifetch
cargo build --release
./target/release/raifetch --install    # installs to ~/.local/bin/raifetch
```

Make sure `~/.local/bin` is in your `PATH`:
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
```

## Configuration

raifetch reads `~/.config/raifetch/config.toml`.  
Generate the default config:

```bash
raifetch --generate-config > ~/.config/raifetch/config.toml
```

### Key options

```toml
[general]
show_header  = true    # user@host header
auto_hide_wm = true    # hide WM when it matches DE

[image]
logo_type    = "auto"  # auto | image | ascii | none
path         = "~/Pictures/logo.png"
ascii_distro = "auto"  # auto-detect, or: arch | ubuntu | debian | endeavour | nixos | fedora | manjaro | opensuse | pop | mint
ascii_file   = ""      # path to a custom ASCII art .txt file

[theme]
label_color  = "auto"  # auto = distro brand colour, or: bright_cyan, red, green, ...
value_color  = "white"
bold_labels  = false
icons        = false   # NerdFont icons (requires Nerd Font)
align_labels = true    # pad labels to same width
bar_width    = 20
bar_fill     = "█"
bar_empty    = "░"

[modules]
show_all_disks = false  # true = show all mount points

# Custom shell modules
[[modules.custom]]
key     = "Weather"
command = "curl -s 'wttr.in/?format=1'"
```

## CLI flags

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

### Single module output (for waybar / polybar)

```bash
raifetch --module memory    # prints just the memory line
raifetch --module cpu       # prints just the cpu line
```

## Image backends

| Backend | Terminal | Protocol |
|---|---|---|
| `kitty` | Kitty | Kitty graphics protocol |
| `sixel` | foot, xterm, mlterm | Sixel |
| `block` | Any (fallback) | Unicode half-blocks + TrueColor |

Backend is auto-detected. Force with `--backend kitty`.

## License

MIT
