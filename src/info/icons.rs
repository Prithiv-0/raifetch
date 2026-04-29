use super::gtk::read_gtk_setting;
use super::InfoModule;

pub struct IconsModule;
impl IconsModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for IconsModule {
    fn key(&self) -> &'static str {
        "Icons"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(read_gtk_setting("gtk-icon-theme-name").unwrap_or_else(|| "unknown".to_string()))
    }
}
