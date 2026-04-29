use super::gtk::read_gtk_setting;
use super::InfoModule;

pub struct FontModule;
impl FontModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for FontModule {
    fn key(&self) -> &'static str {
        "Font"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(read_gtk_setting("gtk-font-name").unwrap_or_else(|| "unknown".to_string()))
    }
}
