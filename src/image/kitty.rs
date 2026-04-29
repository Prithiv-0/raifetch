use super::{resize_to_cells, ImageBackend};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::png::PngEncoder, DynamicImage, ImageEncoder};

pub struct KittyBackend;

impl KittyBackend {
    pub fn new() -> Self {
        Self
    }
}

impl ImageBackend for KittyBackend {
    fn name(&self) -> &'static str {
        "kitty"
    }

    fn is_supported(&self) -> bool {
        // Kitty sets TERM=xterm-kitty, or KITTY_WINDOW_ID
        std::env::var("KITTY_WINDOW_ID").is_ok()
            || std::env::var("TERM")
                .map(|t| t.contains("kitty"))
                .unwrap_or(false)
    }

    fn render(
        &self,
        img: &DynamicImage,
        cols: u16,
        rows: u16,
    ) -> anyhow::Result<(String, u16, u16)> {
        let (resized, actual_cols, actual_rows) = resize_to_cells(img, cols, rows);

        // Encode as PNG into a byte buffer
        let mut png_bytes: Vec<u8> = Vec::new();
        let encoder = PngEncoder::new(&mut png_bytes);
        let rgba = resized.to_rgba8();
        encoder
            .write_image(
                &rgba,
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| anyhow::anyhow!("PNG encode error: {e}"))?;

        // Base64-encode and split into ≤4096-char chunks
        let b64 = STANDARD.encode(&png_bytes);
        let chunks: Vec<&str> = b64
            .as_bytes()
            .chunks(4096)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect();

        let total = chunks.len();
        let mut out = String::new();

        for (i, chunk) in chunks.iter().enumerate() {
            let more = if i < total - 1 { 1 } else { 0 };

            if i == 0 {
                // First chunk carries all control parameters
                out.push_str(&format!(
                    "\x1b_Ga=T,f=100,c={actual_cols},r={actual_rows},q=2,m={more};{chunk}\x1b\\"
                ));
            } else {
                out.push_str(&format!("\x1b_Gm={more};{chunk}\x1b\\"));
            }
        }

        Ok((out, actual_cols, actual_rows))
    }
}
