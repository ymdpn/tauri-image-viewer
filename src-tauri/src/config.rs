use crate::models::{AppConfig, StartupInfo};
use std::env;
use std::fs;
use std::path::Path;
use tauri::api::path::config_dir;

#[tauri::command]
pub fn get_startup_info() -> Result<StartupInfo, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = Path::new(&args[1]);
        if file_path.is_file() {
            let parent = file_path.parent().ok_or("Invalid file path")?;
            Ok(StartupInfo {
                folder: parent.to_string_lossy().into_owned(),
                file: Some(file_path.to_string_lossy().into_owned()),
            })
        } else if file_path.is_dir() {
            Ok(StartupInfo {
                folder: file_path.to_string_lossy().into_owned(),
                file: None,
            })
        } else {
            Err("Invalid path".to_string())
        }
    } else {
        let config_path = config_dir().unwrap().join("image_viewer_config.json");
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                if let Some(last_folder) = config.last_folder {
                    if Path::new(&last_folder).exists() {
                        return Ok(StartupInfo {
                            folder: last_folder,
                            file: None,
                        });
                    }
                }
            }
        }
        Err("No valid startup folder".to_string())
    }
}
#[tauri::command]
pub fn save_last_folder(folder: String) -> Result<(), String> {
    let config = AppConfig {
        last_folder: Some(folder),
    };
    let config_path = config_dir().unwrap().join("image_viewer_config.json");
    let content = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Add unit tests for configuration functions
}