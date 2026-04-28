use image::{DynamicImage, Rgba};
use super::{ImageBackend, resize_to_cells};

/// Unicode half-block fallback — works in any terminal.
/// Uses upper-half block "▀" with fg=top-pixel, bg=bottom-pixel.
pub struct BlockBackend;

impl BlockBackend {
    pub fn new() -> Self { Self }
}

impl ImageBackend for BlockBackend {
    fn name(&self) -> &'static str { "block" }

    fn is_supported(&self) -> bool { true } // always available

    fn render(
        &self,
        img: &DynamicImage,
        cols: u16,
        rows: u16,
    ) -> anyhow::Result<(String, u16, u16)> {
        let (resized, actual_cols, actual_rows) = resize_to_cells(img, cols, rows);
        let rgba = resized.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());

        // Each terminal row covers 2 pixel rows (▀ top half / bottom half)
        let cell_h = (h as f32 / actual_rows as f32).max(1.0) as u32;
        let cell_w = (w as f32 / actual_cols as f32).max(1.0) as u32;

        let mut out = String::new();

        let mut py = 0u32;
        while py < h {
            let py2 = (py + cell_h).min(h);

            let mut px = 0u32;
            while px < w {
                let px2 = (px + cell_w).min(w);

                let top    = avg_color(&rgba, px, py,  px2, py.saturating_add(cell_h / 2).min(h));
                let bottom = avg_color(&rgba, px, py2.saturating_sub(cell_h / 2).min(h - 1), px2, py2);

                out.push_str(&truecolor_fg(top));
                out.push_str(&truecolor_bg(bottom));
                out.push('▀');
                out.push_str("\x1b[0m");

                px = px2;
            }
            out.push('\n');
            py = py2;
        }

        Ok((out, actual_cols, actual_rows))
    }
}

fn avg_color(
    img: &image::RgbaImage,
    x0: u32, y0: u32,
    x1: u32, y1: u32,
) -> Rgba<u8> {
    let (x1, y1) = (x1.min(img.width()), y1.min(img.height()));
    if x0 >= x1 || y0 >= y1 {
        return Rgba([0, 0, 0, 255]);
    }
    let (mut r, mut g, mut b, mut a, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = img.get_pixel(x, y);
            r += p[0] as u32;
            g += p[1] as u32;
            b += p[2] as u32;
            a += p[3] as u32;
            n += 1;
        }
    }
    if n == 0 { return Rgba([0, 0, 0, 255]); }
    Rgba([(r/n) as u8, (g/n) as u8, (b/n) as u8, (a/n) as u8])
}

fn truecolor_fg(c: Rgba<u8>) -> String {
    format!("\x1b[38;2;{};{};{}m", c[0], c[1], c[2])
}

fn truecolor_bg(c: Rgba<u8>) -> String {
    format!("\x1b[48;2;{};{};{}m", c[0], c[1], c[2])
}
