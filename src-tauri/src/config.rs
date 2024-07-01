use std::env;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub last_folder: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StartupInfo {
    pub folder: String,
    pub file: Option<String>,
}

#[tauri::command]
pub fn get_startup_info() -> Result<StartupInfo, String> {
    if let Ok(test_file) = env::var("TEST_FILE_PATH") {
        // テスト環境
        let file_path = PathBuf::from(test_file);
        let parent = file_path.parent().ok_or("Invalid file path")?;
        Ok(StartupInfo {
            folder: parent.to_string_lossy().into_owned(),
            file: Some(file_path.to_string_lossy().into_owned()),
        })
    } else {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let file_path = PathBuf::from(&args[1]);
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
            // 前回のフォルダを読み込む
            let config = load_config()?;
            Ok(StartupInfo {
                folder: config.last_folder.unwrap_or_else(|| ".".to_string()),
                file: None,
            })
        }
    }
}

fn load_config() -> Result<AppConfig, String> {
    let config_dir = if cfg!(target_os = "windows") {
        PathBuf::from(env::var("APPDATA").map_err(|e| e.to_string())?)
    } else {
        PathBuf::from(env::var("HOME").map_err(|e| e.to_string())?).join(".config")
    };
    let config_path = config_dir.join("image_viewer_config.json");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(AppConfig { last_folder: None })
    }
}


#[tauri::command]
pub fn save_last_folder(folder: String) -> Result<(), String> {
    let config = AppConfig {
        last_folder: Some(folder),
    };
    let config_dir = if cfg!(target_os = "windows") {
        PathBuf::from(env::var("APPDATA").map_err(|e| e.to_string())?)
    } else {
        PathBuf::from(env::var("HOME").map_err(|e| e.to_string())?).join(".config")
    };
    let config_path = config_dir.join("image_viewer_config.json");
    let content = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_get_startup_info() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // テスト用の環境変数を設定
        env::set_var("TEST_FILE_PATH", test_file.to_str().unwrap());

        let result = get_startup_info();
        assert!(result.is_ok(), "get_startup_info failed: {:?}", result.err());
        let startup_info = result.unwrap();
        assert_eq!(startup_info.folder, temp_dir.path().to_str().unwrap());
        assert_eq!(startup_info.file, Some(test_file.to_str().unwrap().to_string()));
    }

    #[test]
    fn test_save_last_folder() {
        let temp_dir = TempDir::new().unwrap();

        // テスト用の環境変数を設定
        if cfg!(target_os = "windows") {
            env::set_var("APPDATA", temp_dir.path());
        } else {
            env::set_var("HOME", temp_dir.path());
        }

        let test_folder = temp_dir.path().join("test_folder").to_str().unwrap().to_string();
        let result = save_last_folder(test_folder.clone());
        assert!(result.is_ok(), "save_last_folder failed: {:?}", result.err());

        let config_path = if cfg!(target_os = "windows") {
            temp_dir.path().join("image_viewer_config.json")
        } else {
            temp_dir.path().join(".config").join("image_viewer_config.json")
        };

        assert!(config_path.exists(), "Config file does not exist: {:?}", config_path);
        let content = fs::read_to_string(&config_path).unwrap();
        let config: AppConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(config.last_folder, Some(test_folder));
    }
}