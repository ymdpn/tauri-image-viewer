use crate::utils::get_cache_path;
use image::ImageOutputFormat;
use base64::{engine::general_purpose, Engine as _};
use std::fs;

#[tauri::command]
pub async fn generate_thumbnail(path: String) -> Result<String, String> {
    let cache_path = get_cache_path(&path);
    
    if cache_path.exists() {
        let cached_thumbnail = fs::read(&cache_path).map_err(|e| e.to_string())?;
        return Ok(format!("data:image/webp;base64,{}", general_purpose::STANDARD.encode(&cached_thumbnail)));
    }

    let img = image::open(&path).map_err(|e| e.to_string())?;
    let thumbnail = img.thumbnail(100, 100);
    
    let mut buffer = Vec::new();
    thumbnail.write_to(&mut std::io::Cursor::new(&mut buffer), ImageOutputFormat::WebP).map_err(|e| e.to_string())?;
    
    fs::write(&cache_path, &buffer).map_err(|e| e.to_string())?;
    
    Ok(format!("data:image/webp;base64,{}", general_purpose::STANDARD.encode(&buffer)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn initialize() {
        INIT.call_once(|| {
        });
    }

    fn get_test_image_path(filename: &str) -> PathBuf {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        PathBuf::from(manifest_dir)
            .join("tests")
            .join("resources")
            .join(filename)
    }

    #[tokio::test]
    async fn test_generate_thumbnail_jpg() {
        initialize();
        let image_path = get_test_image_path("test_image.jpg");
        let result = generate_thumbnail(image_path.to_str().unwrap().to_string()).await;
        assert!(result.is_ok(), "Thumbnail generation failed for JPEG: {:?}", result.err());
        let thumbnail = result.unwrap();
        assert!(thumbnail.starts_with("data:image/webp;base64,"));
    }

    #[tokio::test]
    async fn test_generate_thumbnail_png() {
        initialize();
        let image_path = get_test_image_path("test_image.png");
        let result = generate_thumbnail(image_path.to_str().unwrap().to_string()).await;
        assert!(result.is_ok(), "Thumbnail generation failed for PNG: {:?}", result.err());
        let thumbnail = result.unwrap();
        assert!(thumbnail.starts_with("data:image/webp;base64,"));
    }
}