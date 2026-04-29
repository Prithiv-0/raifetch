use super::ImageBackend;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::DynamicImage;
use std::env;
use std::io::Cursor;

pub struct ITerm2Backend;

impl ITerm2Backend {
    pub fn new() -> Self {
        Self
    }
}

impl ImageBackend for ITerm2Backend {
    fn name(&self) -> &'static str {
        "iTerm2"
    }

    fn is_supported(&self) -> bool {
        if let Ok(prog) = env::var("TERM_PROGRAM") {
            if prog == "iTerm.app"
                || prog == "WezTerm"
                || prog == "vscode"
                || prog == "mintty"
                || prog == "Apple_Terminal"
            {
                return true;
            }
        }
        false
    }

    fn render(
        &self,
        img: &DynamicImage,
        cols: u16,
        rows_hint: u16,
    ) -> anyhow::Result<(String, u16, u16)> {
        let (resized, t_cols, t_rows) = super::resize_to_cells(img, cols, rows_hint);

        let mut buf = Cursor::new(Vec::new());
        resized.write_to(&mut buf, image::ImageFormat::Png)?;
        let b64 = STANDARD.encode(buf.into_inner());

        let seq = format!("\x1b]1337;File=inline=1;width={t_cols};height={t_rows}:{b64}\x07");
        Ok((seq, t_cols, t_rows))
    }
}
