use serde::Deserialize;
use std::path::PathBuf;

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

// ─── General ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_separator")]
    pub separator: String,
    #[serde(default = "default_gap")]
    pub gap: usize,
    #[serde(default = "default_true")]
    pub show_header: bool,
    #[serde(default = "default_true")]
    pub auto_hide_wm: bool,
    #[serde(default = "default_cmd_timeout_ms")]
    pub command_timeout_ms: u64,
}
fn default_separator() -> String {
    " → ".to_string()
}
fn default_gap() -> usize {
    3
}
fn default_cmd_timeout_ms() -> u64 {
    800
}
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            separator: default_separator(),
            gap: default_gap(),
            show_header: true,
            auto_hide_wm: true,
            command_timeout_ms: default_cmd_timeout_ms(),
        }
    }
}

// ─── Image ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ImageConfig {
    /// auto | kitty | iterm2 | sixel | block
    #[serde(default = "default_image_backend")]
    pub backend: String,
    pub path: Option<String>,
    #[serde(default = "default_image_width")]
    pub width: u16,
    #[serde(default)]
    pub height: u16,
    /// auto | image | ascii | none
    #[serde(default = "default_logo_type")]
    pub logo_type: String,
    /// auto | arch | ubuntu | debian | endeavour | nixos | fedora | manjaro | opensuse | pop | mint | generic
    #[serde(default = "default_auto")]
    pub ascii_distro: String,
    /// Path to a custom ASCII art text file (overrides ascii_distro when set)
    pub ascii_file: Option<String>,
}
fn default_image_width() -> u16 {
    32
}
fn default_image_backend() -> String {
    "auto".to_string()
}
fn default_logo_type() -> String {
    "auto".to_string()
}
fn default_auto() -> String {
    "auto".to_string()
}
impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            backend: default_image_backend(),
            path: None,
            width: default_image_width(),
            height: 0,
            logo_type: default_logo_type(),
            ascii_distro: default_auto(),
            ascii_file: None,
        }
    }
}

// ─── Theme ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeConfig {
    #[serde(default = "default_label_color")]
    pub label_color: String,
    #[serde(default = "default_value_color")]
    pub value_color: String,
    #[serde(default = "default_false")]
    pub bold_labels: bool,
    #[serde(default = "default_bar_width")]
    pub bar_width: usize,
    #[serde(default = "default_bar_fill")]
    pub bar_fill: char,
    #[serde(default = "default_bar_empty")]
    pub bar_empty: char,
    /// Show NerdFont icons before labels (requires a Nerd Font)
    #[serde(default = "default_false")]
    pub icons: bool,
    /// Pad all labels to the same width so values align
    #[serde(default = "default_true")]
    pub align_labels: bool,
    // Note: label_color = "auto" is resolved at runtime via distro_auto_color()
}
fn default_label_color() -> String {
    "bright_cyan".to_string()
}
fn default_value_color() -> String {
    "white".to_string()
}
fn default_bar_width() -> usize {
    20
}
fn default_bar_fill() -> char {
    '█'
}
fn default_bar_empty() -> char {
    '░'
}
impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            label_color: default_label_color(),
            value_color: default_value_color(),
            bold_labels: false,
            bar_width: 20,
            bar_fill: '█',
            bar_empty: '░',
            icons: false,
            align_labels: true,
        }
    }
}

// ─── Custom Module ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct CustomModule {
    pub key: String,
    pub command: String,
}

// ─── Modules ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ModulesConfig {
    #[serde(default = "default_order")]
    pub order: Vec<String>,

    #[serde(default = "default_true")]
    pub show_os: bool,
    #[serde(default = "default_true")]
    pub show_host: bool,
    #[serde(default = "default_false")]
    pub show_mobo: bool,
    #[serde(default = "default_false")]
    pub show_bios: bool,
    #[serde(default = "default_true")]
    pub show_kernel: bool,
    #[serde(default = "default_false")]
    pub show_boot: bool,
    #[serde(default = "default_false")]
    pub show_bootloader: bool,
    #[serde(default = "default_false")]
    pub show_init: bool,
    #[serde(default = "default_true")]
    pub show_uptime: bool,
    #[serde(default = "default_false")]
    pub show_processes: bool,
    #[serde(default = "default_false")]
    pub show_users: bool,
    #[serde(default = "default_true")]
    pub show_cpu: bool,
    #[serde(default = "default_false")]
    pub cpu_show_temp: bool,
    #[serde(default = "default_false")]
    pub cpu_show_cache: bool,
    #[serde(default = "default_false")]
    pub show_gpu: bool,
    #[serde(default = "default_false")]
    pub gpu_show_temp: bool,
    #[serde(default = "default_false")]
    pub gpu_show_vram: bool,
    #[serde(default = "default_true")]
    pub show_memory: bool,
    #[serde(default = "default_true")]
    pub show_swap: bool,
    #[serde(default = "default_true")]
    pub show_disk: bool,
    #[serde(default = "default_false")]
    pub show_all_disks: bool,
    #[serde(default = "default_true")]
    pub show_battery: bool,
    #[serde(default = "default_true")]
    pub show_network: bool,
    #[serde(default = "default_true")]
    pub show_resolution: bool,
    #[serde(default = "default_false")]
    pub show_display: bool,
    #[serde(default = "default_false")]
    pub show_theme: bool,
    #[serde(default = "default_false")]
    pub show_icons: bool,
    #[serde(default = "default_false")]
    pub show_font: bool,
    #[serde(default = "default_true")]
    pub show_shell: bool,
    #[serde(default = "default_true")]
    pub show_terminal: bool,
    #[serde(default = "default_true")]
    pub show_de: bool,
    #[serde(default = "default_true")]
    pub show_wm: bool,
    #[serde(default = "default_true")]
    pub show_packages: bool,
    #[serde(default = "default_true")]
    pub show_locale: bool,
    #[serde(default = "default_false")]
    pub show_entropy: bool,
    #[serde(default = "default_true")]
    pub show_colors: bool,

    /// Extra package managers to include (opt-in)
    #[serde(default)]
    pub packages_extra: Vec<String>,

    #[serde(default)]
    pub custom: Vec<CustomModule>,
}
fn default_order() -> Vec<String> {
    [
        "os",
        "host",
        "mobo",
        "bios",
        "kernel",
        "boot",
        "bootloader",
        "init",
        "uptime",
        "processes",
        "users",
        "cpu",
        "gpu",
        "memory",
        "swap",
        "disk",
        "battery",
        "network",
        "resolution",
        "display",
        "theme",
        "icons",
        "font",
        "shell",
        "terminal",
        "de",
        "wm",
        "packages",
        "locale",
        "entropy",
        "colors",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            order: default_order(),
            show_os: true,
            show_host: true,
            show_mobo: false,
            show_bios: false,
            show_kernel: true,
            show_boot: false,
            show_bootloader: false,
            show_init: false,
            show_uptime: true,
            show_processes: false,
            show_users: false,
            show_cpu: true,
            cpu_show_temp: false,
            cpu_show_cache: false,
            show_gpu: false,
            gpu_show_temp: false,
            gpu_show_vram: false,
            show_memory: true,
            show_swap: true,
            show_disk: true,
            show_all_disks: false,
            show_battery: true,
            show_network: true,
            show_resolution: true,
            show_display: false,
            show_theme: false,
            show_icons: false,
            show_font: false,
            show_shell: true,
            show_terminal: true,
            show_de: true,
            show_wm: true,
            show_packages: true,
            show_locale: true,
            show_entropy: false,
            show_colors: true,
            packages_extra: vec![],
            custom: vec![],
        }
    }
}

// ─── Root ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub image: ImageConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub modules: ModulesConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text).map_err(|e| anyhow::anyhow!("Config parse error: {e}"))
    }
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("raifetch/config.toml")
    }
    pub fn expand_path(p: &str) -> PathBuf {
        if let Some(rest) = p.strip_prefix("~/") {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/"))
                .join(rest)
        } else {
            PathBuf::from(p)
        }
    }
    pub fn load_from(path: &std::path::Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read config '{}': {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| anyhow::anyhow!("Config parse error: {e}"))
    }
    pub fn default_toml() -> &'static str {
        include_str!("../config/default.toml")
    }
}
