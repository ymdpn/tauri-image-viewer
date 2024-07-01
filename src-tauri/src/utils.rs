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
    let hash_str = hex::encode(hash);
    let cache_filename = format!("{}.webp", hash_str);
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
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_get_cache_dir() {
        let cache_dir = get_cache_dir();
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());
        assert!(cache_dir.ends_with("image-viewer-cache"));
    }

    #[test]
    fn test_get_cache_path() {
        let original_path = "/tests/resources/image.jpg";
        let cache_path = get_cache_path(original_path);
        assert!(cache_path.extension().unwrap() == "webp", "Expected .webp extension, got {:?}", cache_path.extension());
        assert!(cache_path.file_stem().unwrap().len() == 64, "Expected 64 character hash, got {} characters", cache_path.file_stem().unwrap().len());
    }

    #[test]
    fn test_is_image() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = temp_dir.path().join("test.jpg");
        File::create(&image_path).unwrap();
        assert!(is_image(&image_path));

        let non_image_path = temp_dir.path().join("test.txt");
        File::create(&non_image_path).unwrap();
        assert!(!is_image(&non_image_path));
    }
}