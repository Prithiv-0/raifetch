use owo_colors::OwoColorize;

#[derive(Debug, Clone)]
pub struct Theme {
    pub label_color: String,
    pub value_color: String,
    pub separator:   String,
    pub bold_labels: bool,
    pub bar_width:   usize,
    pub bar_fill:    char,
    pub bar_empty:   char,
    pub icons:       bool,
    pub align_labels: bool,
}

impl Theme {
    pub fn apply_label(&self, s: &str) -> String {
        let colored = colorize(s, &self.label_color);
        if self.bold_labels { format!("\x1b[1m{colored}\x1b[22m") } else { colored }
    }
    pub fn apply_value(&self, s: &str) -> String { colorize(s, &self.value_color) }

    /// Format a label+value pair, optionally padding the key to `key_width`.
    pub fn format_line_aligned(&self, key: &str, value: &str, key_width: usize) -> String {
        let icon = if self.icons { icon_for(key) } else { "" };
        let padded = format!("{icon}{key:<width$}", width = key_width);
        format!("  {}{}{}",
            self.apply_label(&padded),
            self.apply_label(&self.separator),
            self.apply_value(value))
    }

    /// Format without alignment (for single-module output).
    pub fn format_line(&self, key: &str, value: &str) -> String {
        self.format_line_aligned(key, value, key.len())
    }

    /// Coloured progress bar: green < 60%, yellow < 80%, red >= 80%.
    pub fn bar(&self, pct: f64) -> String {
        let n = ((pct.clamp(0.0, 100.0) / 100.0) * self.bar_width as f64).round() as usize;
        let n = n.min(self.bar_width);
        let e = self.bar_width - n;
        let color = if pct >= 80.0 { "\x1b[31m" } else if pct >= 60.0 { "\x1b[33m" } else { "\x1b[32m" };
        format!("{}{}{}\x1b[90m{}\x1b[0m",
            color,
            self.bar_fill.to_string().repeat(n),
            "\x1b[0m",
            self.bar_empty.to_string().repeat(e))
    }

    #[allow(dead_code)]
    pub fn plain_width(key: &str, sep: &str, value: &str) -> usize {
        2 + key.len() + sep.len() + value.len()
    }
}

// ─── NerdFont icons ───────────────────────────────────────────────────────────

pub fn icon_for(key: &str) -> &'static str {
    match key {
        "OS"         => "󰣇 ",
        "Host"       => "󰌢 ",
        "Kernel"     => "󰒔 ",
        "Uptime"     => "󱦟 ",
        "CPU"        => "󰻠 ",
        "Memory"     => "󰍛 ",
        "Swap"       => "󱛜 ",
        "Disk"       => "󰋊 ",
        "Battery"    => "󰂄 ",
        "Network"    => "󰤨 ",
        "Resolution" => "󰍹 ",
        "Shell"      => " ",
        "Terminal"   => " ",
        "DE"         => "󰇄 ",
        "WM"         => "󱂬 ",
        "Packages"   => "󰏗 ",
        "Locale"     => "󰗊 ",
        _            => "  ",
    }
}

// ─── Colorize helper ─────────────────────────────────────────────────────────

pub fn colorize(s: &str, color: &str) -> String {
    match color.to_lowercase().as_str() {
        "red"            => s.red().to_string(),
        "green"          => s.green().to_string(),
        "yellow"         => s.yellow().to_string(),
        "blue"           => s.blue().to_string(),
        "magenta"        => s.magenta().to_string(),
        "cyan"           => s.cyan().to_string(),
        "white"          => s.white().to_string(),
        "bright_red"     => s.bright_red().to_string(),
        "bright_green"   => s.bright_green().to_string(),
        "bright_yellow"  => s.bright_yellow().to_string(),
        "bright_blue"    => s.bright_blue().to_string(),
        "bright_magenta" => s.bright_magenta().to_string(),
        "bright_cyan"    => s.bright_cyan().to_string(),
        "bright_white"   => s.bright_white().to_string(),
        _                => s.to_string(),
    }
}

// ─── Distro auto-color ────────────────────────────────────────────────────────

/// Detect distro from /etc/os-release and return its brand color name.
pub fn distro_auto_color() -> String {
    let release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    for line in release.lines() {
        if line.starts_with("ID=") {
            let id = line.splitn(2, '=').nth(1).unwrap_or("")
                .trim_matches('"').to_lowercase();
            return match id.as_str() {
                "arch" | "archlinux"                             => "bright_cyan",
                "endeavouros"                                    => "bright_magenta",
                "ubuntu"                                         => "yellow",
                "debian"                                         => "red",
                "nixos"                                          => "bright_blue",
                "fedora"                                         => "blue",
                "manjaro"                                        => "bright_green",
                "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => "green",
                "pop" | "pop-os"                                 => "cyan",
                "linuxmint"                                      => "green",
                "gentoo"                                         => "magenta",
                "void"                                           => "green",
                "alpine"                                         => "blue",
                _                                                => "bright_cyan",
            }.to_string();
        }
    }
    "bright_cyan".to_string()
}

// ─── Color palette blocks ─────────────────────────────────────────────────────

pub fn color_blocks() -> String {
    let normal: String = (40u8..=47).map(|c| format!("\x1b[{}m   \x1b[0m", c)).collect();
    let bright: String = (100u8..=107).map(|c| format!("\x1b[{}m   \x1b[0m", c)).collect();
    format!("  {}\n  {}", normal, bright)
}
