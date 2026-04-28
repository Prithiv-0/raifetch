use serde::Deserialize;
use std::path::PathBuf;

fn default_true()  -> bool   { true }
fn default_false() -> bool   { false }

// ─── General ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_separator")] pub separator:     String,
    #[serde(default = "default_gap")]       pub gap:           usize,
    #[serde(default = "default_true")]      pub show_header:   bool,
    #[serde(default = "default_true")]      pub auto_hide_wm:  bool,
}
fn default_separator() -> String { " → ".to_string() }
fn default_gap()       -> usize  { 3 }
impl Default for GeneralConfig {
    fn default() -> Self {
        Self { separator: default_separator(), gap: default_gap(), show_header: true, auto_hide_wm: true }
    }
}

// ─── Image ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ImageConfig {
    #[serde(default = "default_true")]         pub enabled:      bool,
    pub path:                                                     Option<String>,
    #[serde(default = "default_image_width")]  pub width:        u16,
    #[serde(default)]                          pub height:       u16,
    /// auto | image | ascii | none
    #[serde(default = "default_logo_type")]    pub logo_type:    String,
    /// auto | arch | ubuntu | debian | endeavour | nixos | fedora | manjaro | opensuse | pop | mint | generic
    #[serde(default = "default_auto")]         pub ascii_distro: String,
    /// Path to a custom ASCII art text file (overrides ascii_distro when set)
    pub ascii_file: Option<String>,
}
fn default_image_width() -> u16    { 32 }
fn default_logo_type()   -> String { "auto".to_string() }
fn default_auto()        -> String { "auto".to_string() }
impl Default for ImageConfig {
    fn default() -> Self {
        Self { enabled: true, path: None, width: default_image_width(),
               height: 0, logo_type: default_logo_type(), ascii_distro: default_auto(),
               ascii_file: None }
    }
}

// ─── Theme ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeConfig {
    #[serde(default = "default_label_color")] pub label_color: String,
    #[serde(default = "default_value_color")] pub value_color: String,
    #[serde(default = "default_false")]        pub bold_labels: bool,
    #[serde(default = "default_bar_width")]    pub bar_width:   usize,
    #[serde(default = "default_bar_fill")]     pub bar_fill:    char,
    #[serde(default = "default_bar_empty")]    pub bar_empty:   char,
    /// Show NerdFont icons before labels (requires a Nerd Font)
    #[serde(default = "default_false")]        pub icons:       bool,
    /// Pad all labels to the same width so values align
    #[serde(default = "default_true")]         pub align_labels: bool,
    // Note: label_color = "auto" is resolved at runtime via distro_auto_color()
}
fn default_label_color() -> String { "bright_cyan".to_string() }
fn default_value_color() -> String { "white".to_string() }
fn default_bar_width()   -> usize  { 20 }
fn default_bar_fill()    -> char   { '█' }
fn default_bar_empty()   -> char   { '░' }
impl Default for ThemeConfig {
    fn default() -> Self {
        Self { label_color: default_label_color(), value_color: default_value_color(),
               bold_labels: false, bar_width: 20, bar_fill: '█', bar_empty: '░',
               icons: false, align_labels: true }
    }
}

// ─── Custom Module ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct CustomModule {
    pub key:     String,
    pub command: String,
}

// ─── Modules ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ModulesConfig {
    #[serde(default = "default_order")] pub order: Vec<String>,

    #[serde(default = "default_true")]  pub show_os:         bool,
    #[serde(default = "default_true")]  pub show_host:       bool,
    #[serde(default = "default_true")]  pub show_kernel:     bool,
    #[serde(default = "default_true")]  pub show_uptime:     bool,
    #[serde(default = "default_true")]  pub show_cpu:        bool,
    #[serde(default = "default_false")] pub show_gpu:        bool,
    #[serde(default = "default_true")]  pub show_memory:     bool,
    #[serde(default = "default_true")]  pub show_swap:       bool,
    #[serde(default = "default_true")]  pub show_disk:       bool,
    #[serde(default = "default_false")] pub show_all_disks:  bool,
    #[serde(default = "default_true")]  pub show_battery:    bool,
    #[serde(default = "default_true")]  pub show_network:    bool,
    #[serde(default = "default_true")]  pub show_resolution: bool,
    #[serde(default = "default_true")]  pub show_shell:      bool,
    #[serde(default = "default_true")]  pub show_terminal:   bool,
    #[serde(default = "default_true")]  pub show_de:         bool,
    #[serde(default = "default_true")]  pub show_wm:         bool,
    #[serde(default = "default_true")]  pub show_packages:   bool,
    #[serde(default = "default_true")]  pub show_locale:     bool,
    #[serde(default = "default_true")]  pub show_colors:     bool,

    #[serde(default)] pub custom: Vec<CustomModule>,
}
fn default_order() -> Vec<String> {
    ["os","host","kernel","uptime","cpu","gpu","memory","swap","disk","battery","network",
     "resolution","shell","terminal","de","wm","packages","locale","colors"]
        .iter().map(|s| s.to_string()).collect()
}
impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            order: default_order(),
            show_os: true, show_host: true, show_kernel: true, show_uptime: true,
            show_cpu: true, show_gpu: false, show_memory: true, show_swap: true, show_disk: true,
            show_all_disks: false, show_battery: true, show_network: true,
            show_resolution: true, show_shell: true, show_terminal: true,
            show_de: true, show_wm: true, show_packages: true,
            show_locale: true, show_colors: true,
            custom: vec![],
        }
    }
}

// ─── Root ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)] pub general: GeneralConfig,
    #[serde(default)] pub image:   ImageConfig,
    #[serde(default)] pub theme:   ThemeConfig,
    #[serde(default)] pub modules: ModulesConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::path();
        if !path.exists() { return Ok(Config::default()); }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text).map_err(|e| anyhow::anyhow!("Config parse error: {e}"))
    }
    pub fn path() -> PathBuf {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("raifetch/config.toml")
    }
    pub fn expand_path(p: &str) -> PathBuf {
        if let Some(rest) = p.strip_prefix("~/") {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")).join(rest)
        } else { PathBuf::from(p) }
    }
    pub fn load_from(path: &std::path::Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read config '{}': {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| anyhow::anyhow!("Config parse error: {e}"))
    }
    pub fn default_toml() -> &'static str { include_str!("../config/default.toml") }
}
