use super::gtk::read_gtk_setting;
use super::InfoModule;

pub struct ThemeModule;
impl ThemeModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for ThemeModule {
    fn key(&self) -> &'static str {
        "Theme"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(read_gtk_setting("gtk-theme-name").unwrap_or_else(|| "unknown".to_string()))
    }
}
