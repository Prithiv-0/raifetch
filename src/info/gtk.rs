use std::path::PathBuf;

pub fn read_gtk_setting(key: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    let gtk3 = home.join(".config/gtk-3.0/settings.ini");
    if let Some(v) = read_key_value(&gtk3, key) {
        return Some(v);
    }

    let gtk2 = home.join(".gtkrc-2.0");
    if let Some(v) = read_key_value(&gtk2, key) {
        return Some(v);
    }

    None
}

fn read_key_value(path: &PathBuf, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(val) = rest.strip_prefix('=') {
                let v = val.trim().trim_matches('"');
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}
