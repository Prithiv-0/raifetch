use image::{DynamicImage, RgbaImage};
use super::{ImageBackend, resize_to_cells};

/// Basic Sixel encoder — works in foot, xterm, mlterm.
pub struct SixelBackend;
impl SixelBackend { pub fn new() -> Self { Self } }

impl ImageBackend for SixelBackend {
    fn name(&self) -> &'static str { "sixel" }

    fn is_supported(&self) -> bool {
        // foot sets TERM=foot; xterm with sixel reports VT340
        let term = std::env::var("TERM").unwrap_or_default();
        term.contains("foot") || term.contains("xterm")
            || std::env::var("TERM_PROGRAM").map(|t| t == "iTerm.app").unwrap_or(false)
    }

    fn render(&self, img: &DynamicImage, cols: u16, rows: u16) -> anyhow::Result<(String, u16, u16)> {
        let (resized, actual_cols, actual_rows) = resize_to_cells(img, cols, rows);
        let rgba = resized.to_rgba8();
        let s = encode_sixel(&rgba)?;
        Ok((s, actual_cols, actual_rows))
    }
}

/// Encode an RGBA image as sixel data using a 8×8×4 = 256-color palette.
fn encode_sixel(img: &RgbaImage) -> anyhow::Result<String> {
    let w = img.width();
    let h = img.height();

    // Quantize each pixel to the 256-color palette index
    let indices: Vec<u8> = img.pixels()
        .map(|p| quantize(p[0], p[1], p[2]))
        .collect();

    // Only emit color definitions that are actually used
    let mut used = [false; 256];
    for &i in &indices { used[i as usize] = true; }

    let mut out = String::with_capacity(w as usize * h as usize * 2);
    // DCS header: pixel aspect=0 (square), transparent bg=0, pixel size=1
    out.push_str("\x1bP0;1q");
    // Emit color definitions
    for (i, _) in used.iter().enumerate().filter(|(_, &u)| u) {
        let (r, g, b) = palette_color(i as u8);
        let r100 = r as u32 * 100 / 255;
        let g100 = g as u32 * 100 / 255;
        let b100 = b as u32 * 100 / 255;
        out.push_str(&format!("#{i};2;{r100};{g100};{b100}"));
    }

    // Encode in 6-row bands
    let mut y = 0u32;
    while y < h {
        let band_h = 6.min(h - y);
        // For each colour used in this band, emit its sixel row

        for col_idx in 0u8..=255 {
            if !used[col_idx as usize] { continue; }
            // Check if this colour appears in this band
            let mut any = false;
            let mut sixels = String::with_capacity(w as usize);
            for x in 0..w {
                let mut bits = 0u8;
                for dy in 0..band_h {
                    let idx = ((y + dy) * w + x) as usize;
                    if indices[idx] == col_idx { bits |= 1 << dy; any = true; }
                }
                sixels.push((bits + 63) as char);
            }
            if any {
                out.push_str(&format!("#{col_idx}"));
                out.push_str(&sixels);
                out.push('$'); // carriage return within band
            }
        }
        out.push('-'); // next sixel band
        y += 6;
    }
    out.push_str("\x1b\\"); // String Terminator
    Ok(out)
}

/// Map RGB to 8×8×4 palette index (256 entries).
fn quantize(r: u8, g: u8, b: u8) -> u8 {
    let ri = (r as u16 * 7 / 255) as u8; // 0-7
    let gi = (g as u16 * 7 / 255) as u8; // 0-7
    let bi = (b as u16 * 3 / 255) as u8; // 0-3
    ri * 32 + gi * 4 + bi
}

/// Reverse map palette index → RGB.
fn palette_color(idx: u8) -> (u8, u8, u8) {
    let bi = (idx % 4) as u16;
    let gi = ((idx / 4) % 8) as u16;
    let ri = (idx / 32) as u16;
    ((ri * 255 / 7) as u8, (gi * 255 / 7) as u8, (bi * 255 / 3) as u8)
}
