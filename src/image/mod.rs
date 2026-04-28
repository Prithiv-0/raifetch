pub mod block;
pub mod kitty;
pub mod sixel;

use image::DynamicImage;
pub use block::BlockBackend;
pub use kitty::KittyBackend;
pub use sixel::SixelBackend;

// ─── Trait ───────────────────────────────────────────────────────────────────

pub trait ImageBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_supported(&self) -> bool;
    fn render(&self, img: &DynamicImage, cols: u16, rows: u16) -> anyhow::Result<(String, u16, u16)>;
}

// ─── Auto-detect ─────────────────────────────────────────────────────────────

pub fn auto_detect() -> Box<dyn ImageBackend> {
    let kitty = KittyBackend::new();
    if kitty.is_supported() { return Box::new(kitty); }
    let sixel = SixelBackend::new();
    if sixel.is_supported() { return Box::new(sixel); }
    Box::new(BlockBackend::new())
}

// ─── Resize helper ───────────────────────────────────────────────────────────

pub fn resize_to_cells(img: &DynamicImage, cols: u16, rows_hint: u16) -> (DynamicImage, u16, u16) {
    let cell_ratio: f64 = 0.5; // approx cell_width_px / cell_height_px (8/16)
    let src_w = img.width()  as f64;
    let src_h = img.height() as f64;
    let img_aspect = src_w / src_h;

    let target_cols = cols as f64;
    let target_rows = if rows_hint > 0 {
        rows_hint as f64
    } else {
        // Maintain image aspect: cols * cell_ratio / img_aspect
        (target_cols * cell_ratio / img_aspect).round().max(1.0)
    };

    let px_w = (target_cols * 8.0).round() as u32;
    let px_h = (target_rows * 16.0).round() as u32;
    let resized = img.resize(px_w, px_h, image::imageops::FilterType::Lanczos3);
    (resized, target_cols as u16, target_rows as u16)
}
