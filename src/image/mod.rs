pub mod block;
pub mod cache;
pub mod kitty;
pub mod sixel;
pub mod iterm2;

use image::DynamicImage;
pub use block::BlockBackend;
pub use kitty::KittyBackend;
pub use sixel::SixelBackend;
pub use iterm2::ITerm2Backend;

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
    let iterm = ITerm2Backend::new();
    if iterm.is_supported() { return Box::new(iterm); }
    let sixel = SixelBackend::new();
    if sixel.is_supported() { return Box::new(sixel); }
    Box::new(BlockBackend::new())
}

// ─── Terminal Cell Size ────────────────────────────────────────────────────────

pub fn terminal_cell_size() -> Option<(f64, f64)> {
    #[cfg(unix)]
    {
        use std::io::{Read, Write};
        use std::mem;
        use libc::{tcgetattr, tcsetattr, cfmakeraw, poll, pollfd, POLLIN, TCSANOW, STDIN_FILENO, termios};

        unsafe {
            let mut orig: termios = mem::zeroed();
            if tcgetattr(STDIN_FILENO, &mut orig) != 0 { return None; }
            let mut raw = orig;
            cfmakeraw(&mut raw);
            if tcsetattr(STDIN_FILENO, TCSANOW, &raw) != 0 { return None; }

            print!("\x1b[16t");
            std::io::stdout().flush().ok();

            let mut fds = [pollfd { fd: STDIN_FILENO, events: POLLIN, revents: 0 }];
            let mut w = 0.0;
            let mut h = 0.0;

            // 1ms timeout. Zero impact.
            if poll(fds.as_mut_ptr(), 1, 1) > 0 {
                let mut buf = [0u8; 32];
                // Non-blocking read just in case
                if let Ok(n) = std::io::stdin().read(&mut buf) {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    if let Some(rest) = s.strip_prefix("\x1b[6;") {
                        let parts: Vec<&str> = rest.split(';').collect();
                        if parts.len() == 2 {
                            if let Ok(height) = parts[0].parse::<f64>() {
                                if let Ok(width) = parts[1].trim_end_matches('t').parse::<f64>() {
                                    if width > 0.0 && height > 0.0 {
                                        w = width;
                                        h = height;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            tcsetattr(STDIN_FILENO, TCSANOW, &orig);
            if w > 0.0 && h > 0.0 { return Some((w, h)); }
        }
    }
    None
}

// ─── Resize helper ───────────────────────────────────────────────────────────

pub fn resize_to_cells(img: &DynamicImage, cols: u16, rows_hint: u16) -> (DynamicImage, u16, u16) {
    let (cell_w, cell_h) = terminal_cell_size().unwrap_or((8.0, 16.0));
    let cell_ratio = cell_w / cell_h;
    
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

    let px_w = (target_cols * cell_w).round() as u32;
    let px_h = (target_rows * cell_h).round() as u32;
    let resized = img.resize(px_w, px_h, image::imageops::FilterType::Lanczos3);
    (resized, target_cols as u16, target_rows as u16)
}
