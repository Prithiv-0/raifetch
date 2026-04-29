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
![Platform](https://img.shields.io/badge/platform-Linux%20|%20macOS-lightgrey?style=flat-square)

</div>

---

## Features

- **Parallel info collection** via `rayon` — all modules run concurrently for optimal performance.
- **macOS Support** — Native support for macOS via `sysctl`, `system_profiler`, and `df`.
- **Image backends**: Kitty graphics protocol, iTerm2 inline images, Sixel (foot/xterm), and Unicode block fallback. Image renders are cached for instant repeated runs.
- **ASCII distro logos** — bundled for Arch, EndeavourOS, Ubuntu, Debian, NixOS, Fedora, Manjaro, openSUSE, Pop!\_OS, and Linux Mint, with support for custom file loading.
- **Progress bars** on Memory, Swap, Disk, and Battery with threshold coloring.
- **NerdFont icons** (optional, set `icons = true` in config).
- **Label alignment** — all values start at the same column for a clean layout.
- **Distro auto-colors** — `label_color = "auto"` automatically picks your distribution's brand color.
- **Package counts** cached for 5 minutes to ensure fast repeated runs.
- **Wayland-native** resolution detection (Hyprland, sway, kscreen).
- **Fully configurable** via TOML with no required setup.
- **Custom shell modules** — run completely asynchronously in the background so they never block rendering.

## Modules

| Module | Information Displayed |
|---|---|
| OS | Distro name and architecture |
| Host | Hardware model (from DMI / sysctl) |
| Mobo | Motherboard model |
| BIOS | BIOS version |
| Kernel | Linux / Darwin kernel version |
| Boot | System boot time |
| Bootloader | Bootloader (e.g. GRUB) |
| Init | Init system (e.g. systemd) |
| Uptime | System uptime |
| Processes | Number of running processes |
| Users | Number of logged-in users |
| CPU | Model, core count, frequency (Optional: Cache and Temperature) |
| GPU | GPU vendor and model (Optional: VRAM and Temperature) |
| Memory | Used / Total and progress bar |
| Swap | Used / Total and progress bar |
| Disk | Used / Total, progress bar, and filesystem (all mounts optional) |
| Battery | Charge %, status icon, and progress bar |
| Network | Interface name and local IP |
| Resolution | Active display resolution and refresh rate |
| Display | Display server / protocol (Wayland, X11, Aqua) |
| Theme | System/UI theme |
| Icons | Icon theme |
| Font | System font |
| Shell | Shell name and version |
| Terminal | Terminal emulator |
| DE | Desktop environment |
| WM | Window manager (auto-hidden when same as DE) |
| Packages | Package counts (pacman, dpkg, rpm, flatpak, snap, brew, nix, port) |
| Locale | LANG and timezone |
| Entropy | System entropy |
| Colors | 16-color palette swatches |
| Custom | Run any shell command asynchronously as a module |

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


## CLI Flags

```
raifetch [OPTIONS]

Options:
  -i, --image <PATH>          Override image path
      --no-image              Disable image/logo
      --backend <BACKEND>     Force backend: kitty | sixel | block
      --color <WHEN>          Color output: auto | always | never
      --config <PATH>         Use an alternate config file
      --config-path           Print config file location
      --list-modules          List all available module names
      --generate-config       Print default config to stdout
      --install               Copy binary to ~/.local/bin
      --clear-cache           Remove all cached image renders from /tmp
      --module <MODULE>       Print only one module (for status bars)
      --raw                   With --module: print bare value only (no label/color/separator)
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
| `kitty` | Kitty, Ghostty, WezTerm | Kitty graphics protocol |
| `iterm2`| iTerm2, WezTerm | iTerm2 inline images protocol |
| `sixel` | foot, xterm, mlterm | Sixel |
| `block` | Any (fallback) | Unicode half-blocks + TrueColor |

The backend is auto-detected. You can force a specific backend using `--backend kitty` or `--backend iterm2`.

## License

MIT
