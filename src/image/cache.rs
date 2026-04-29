/// Image render cache — stores the encoded output (ANSI/sixel/kitty bytes) keyed by
/// image path + backend + dimensions, invalidated by the image file's mtime.
///
/// Format: [8 bytes mtime LE] [2 bytes cols LE] [2 bytes rows LE] [render bytes...]
use std::path::Path;
use std::time::UNIX_EPOCH;

pub struct CachedRender {
    pub data: String,
    pub cols: u16,
    pub rows: u16,
}

/// Try to load a cached render. Returns None on cache miss or stale cache.
pub fn load(image_path: &Path, backend: &str, width: u16, height: u16) -> Option<CachedRender> {
    let raw = std::fs::read(cache_path(image_path, backend, width, height)).ok()?;
    if raw.len() < 12 {
        return None;
    }

    let cached_mtime = u64::from_le_bytes(raw[0..8].try_into().ok()?);
    if cached_mtime != file_mtime(image_path)? {
        return None;
    } // stale

    let cols = u16::from_le_bytes(raw[8..10].try_into().ok()?);
    let rows = u16::from_le_bytes(raw[10..12].try_into().ok()?);
    let data = String::from_utf8(raw[12..].to_vec()).ok()?;
    Some(CachedRender { data, cols, rows })
}

/// Save a render result to cache.
pub fn save(
    image_path: &Path,
    backend: &str,
    width: u16,
    height: u16,
    data: &str,
    cols: u16,
    rows: u16,
) {
    let Some(mtime) = file_mtime(image_path) else {
        return;
    };
    let mut out = Vec::with_capacity(12 + data.len());
    out.extend_from_slice(&mtime.to_le_bytes());
    out.extend_from_slice(&cols.to_le_bytes());
    out.extend_from_slice(&rows.to_le_bytes());
    out.extend_from_slice(data.as_bytes());
    let _ = std::fs::write(cache_path(image_path, backend, width, height), out);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn cache_path(image_path: &Path, backend: &str, width: u16, height: u16) -> std::path::PathBuf {
    let key = format!("{}{}{}{}", image_path.display(), backend, width, height);
    let hash = djb2(&key);
    std::path::PathBuf::from(format!("/tmp/raifetch_img_{hash:016x}.cache"))
}

fn file_mtime(path: &Path) -> Option<u64> {
    std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// djb2 hash — fast, no crypto deps needed for a cache key.
fn djb2(s: &str) -> u64 {
    s.bytes()
        .fold(5381u64, |h, b| h.wrapping_mul(33).wrapping_add(b as u64))
}
