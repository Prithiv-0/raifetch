/// Side-by-side renderer: image on the left, info lines on the right.
///
/// For the Kitty backend, the image is rendered via terminal escape codes.
/// We use ANSI cursor movement to overlay the info text alongside it.
///
/// For the block backend, the image is already a multi-line string, so we
/// zip image lines with info lines directly.

use crate::theme::Theme;


pub fn render_side_by_side(
    image_str: String,
    image_cols: u16,
    _image_rows: u16,
    image_is_inline: bool,
    info_pairs: &[(String, String)], // (key, value) already formatted
    theme: &Theme,
    gap: usize,
) -> Vec<String> {
    // Build styled info lines
    let info_lines: Vec<String> = {
        // Header: user@host
        let host  = whoami::devicename();
        let user  = whoami::username();
        let header = format!(
            "  {}@{}",
            theme.apply_label(&user),
            theme.apply_label(&host)
        );
        let sep_line = format!(
            "  {}",
            theme.apply_label(&"─".repeat(user.len() + host.len() + 1))
        );

        let mut lines = vec![header, sep_line];
        for (k, v) in info_pairs {
            lines.push(theme.format_line(k, v));
        }
        lines
    };

    if image_is_inline {
        // Kitty path: print image, move cursor back up, overlay text
        // We return a special sentinel so main knows to do cursor acrobatics
        // (handled in main.rs print_combined)
        // Here we just return info_lines; image_str is printed separately
        info_lines
    } else {
        // Block path: zip image rows with info lines
        let img_lines: Vec<&str> = image_str.lines().collect();
        let total = img_lines.len().max(info_lines.len());
        let pad   = " ".repeat(image_cols as usize);
        let gap_s = " ".repeat(gap);

        (0..total)
            .map(|i| {
                let img_part  = img_lines.get(i).copied().unwrap_or(pad.as_str());
                let info_part = info_lines.get(i).cloned().unwrap_or_default();
                format!("{img_part}{gap_s}{info_part}")
            })
            .collect()
    }
}
