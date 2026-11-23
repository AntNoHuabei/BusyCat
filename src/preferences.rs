use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static CURRENT_SIZE: AtomicU32 = AtomicU32::new(250.0_f32.to_bits());

pub fn remember_size(size: f32) {
    CURRENT_SIZE.store(size.to_bits(), Ordering::Relaxed);
}

pub fn save_current_size() -> std::io::Result<()> {
    save_size(f32::from_bits(CURRENT_SIZE.load(Ordering::Relaxed)))
}

fn path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|root| PathBuf::from(root).join("BusyCat").join("size.txt"))
}

fn parse_size(text: &str) -> Option<f32> {
    text.trim()
        .parse::<f32>()
        .ok()
        .filter(|size| size.is_finite() && (125.0..=800.0).contains(size))
}

pub fn load_size() -> f32 {
    path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| parse_size(&text))
        .unwrap_or(250.0)
}

pub fn save_size(size: f32) -> std::io::Result<()> {
    if let Some(path) = path() {
        std::fs::create_dir_all(path.parent().expect("settings directory"))?;
        std::fs::write(path, size.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_supported_sizes_and_rejects_corrupt_preferences() {
        for size in [125.0, 250.0, 800.0] {
            assert_eq!(parse_size(&size.to_string()), Some(size));
        }
        for text in ["NaN", "inf", "124", "801", "", "broken"] {
            assert_eq!(parse_size(text), None);
        }
    }
}
