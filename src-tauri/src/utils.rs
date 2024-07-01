use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use tauri::api::path::cache_dir;
use std::fs;

pub fn get_cache_dir() -> PathBuf {
    let cache_dir = cache_dir().expect("Failed to get cache directory");
    let app_cache_dir = cache_dir.join("image-viewer-cache");
    fs::create_dir_all(&app_cache_dir).expect("Failed to create cache directory");
    app_cache_dir
}

pub fn get_cache_path(original_path: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(original_path);
    let hash = hasher.finalize();
    let cache_filename = format!("{:x}.webp", hash);
    get_cache_dir().join(cache_filename)
}
pub fn is_image(path: &Path) -> bool {
    let extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}
#[cfg(test)]
mod tests {
    use super::*;

    // Add unit tests for utility functions
}