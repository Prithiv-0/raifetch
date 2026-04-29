use super::InfoModule;

pub struct TerminalModule;

impl TerminalModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for TerminalModule {
    fn key(&self) -> &'static str {
        "Terminal"
    }

    fn value(&self) -> anyhow::Result<String> {
        // TERM_PROGRAM is set by most modern terminals (kitty, wezterm, etc.)
        // Fall back through a priority list, returning just the value not the key name.
        let candidates = [
            ("TERM_PROGRAM", None),
            ("KITTY_WINDOW_ID", Some("kitty")),
            ("KONSOLE_VERSION", Some("konsole")),
            ("XTERM_VERSION", Some("xterm")),
            ("VTE_VERSION", Some("vte-based")),
            ("COLORTERM", None),
            ("TERM", None),
        ];

        for (var, override_name) in candidates {
            if let Ok(val) = std::env::var(var) {
                let name = override_name.map(|s| s.to_string()).unwrap_or(val);
                return Ok(name);
            }
        }

        Ok("unknown".to_string())
    }
}
