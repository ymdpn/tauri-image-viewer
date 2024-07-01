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

    // Add unit tests for image processing functions
}