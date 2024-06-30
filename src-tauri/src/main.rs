// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::{PathBuf, Path};
use tauri::Manager;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use image::ImageOutputFormat;
use base64::{engine::general_purpose, Engine as _};
use tauri::api::path::{home_dir, cache_dir, config_dir};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::env;

#[derive(Clone, Serialize)]
struct FileItem {
    name: String,
    path: String,
    is_dir: bool,
    date_modified: u64,
    size: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct ImageState {
    current_index: usize,
    images: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    last_folder: Option<String>,
}

struct AppState {
    image_paths: Mutex<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Name,
    Type,
    Date,
    Size,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Serialize, Deserialize)]
struct StartupInfo {
    folder: String,
    file: Option<String>,
}

fn get_cache_dir() -> PathBuf {
    let cache_dir = cache_dir().expect("Failed to get cache directory");
    let app_cache_dir = cache_dir.join("image-viewer-cache");
    fs::create_dir_all(&app_cache_dir).expect("Failed to create cache directory");
    app_cache_dir
}

fn get_cache_path(original_path: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(original_path);
    let hash = hasher.finalize();
    let cache_filename = format!("{:x}.webp", hash);
    get_cache_dir().join(cache_filename)
}

#[tauri::command]
fn get_directory_contents(path: &str, state: tauri::State<AppState>) -> Result<Vec<FileItem>, String> {
    let mut items = Vec::new();
    let mut image_paths = state.image_paths.lock().unwrap();
    
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                let metadata = match fs::metadata(&path) {
                    Ok(meta) => meta,
                    Err(_) => continue,
                };
                
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                let is_dir = metadata.is_dir();
                let date_modified = metadata.modified()
                    .unwrap_or(SystemTime::UNIX_EPOCH)
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let size = if is_dir { 0 } else { metadata.len() };

                let id = if is_dir {
                    name.clone()
                } else {
                    let file_id = format!("{:x}", Sha256::digest(path.to_string_lossy().as_bytes()));
                    image_paths.insert(file_id.clone(), path.to_string_lossy().into_owned());
                    file_id
                };

                items.push(FileItem {
                    name,
                    path: path.to_string_lossy().into_owned(),
                    is_dir,
                    date_modified,
                    size,
                });
            }
            Ok(items)
        },
        Err(e) => Err(format!("Failed to read directory: {}", e)),
    }
}

#[tauri::command]
fn get_root_folders() -> Vec<FileItem> {
    let mut roots = Vec::new();

    if let Some(home) = home_dir() {
        roots.push(FileItem {
            name: "Home".to_string(),
            path: home.to_string_lossy().into_owned(),
            is_dir: true,
            date_modified: 0,
            size: 0,
        });
    }

    roots.push(FileItem {
        name: "Root".to_string(),
        path: "/".to_string(),
        is_dir: true,
        date_modified: 0,
        size: 0,
    });

    #[cfg(target_os = "windows")]
    {
        for drive in 'A'..='Z' {
            let drive_path = format!("{}:\\", drive);
            if fs::metadata(&drive_path).is_ok() {
                roots.push(FileItem {
                    name: format!("Drive ({}:)", drive),
                    path: drive_path,
                    is_dir: true,
                    date_modified: 0,
                    size: 0,
                });
            }
        }
    }

    roots
}

#[tauri::command]
async fn generate_thumbnail(path: String) -> Result<String, String> {
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

#[tauri::command]
fn get_full_image_list(path: &str, sort_by: SortBy, sort_order: SortOrder) -> Result<Vec<String>, String> {
    let parent_dir = Path::new(path).parent().ok_or_else(|| "Invalid path".to_string())?;
    let mut images: Vec<(PathBuf, std::fs::Metadata)> = fs::read_dir(parent_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && is_image(&path) {
                let metadata = fs::metadata(&path).ok()?;
                Some((path, metadata))
            } else {
                None
            }
        })
        .collect();

    sort_images(&mut images, &sort_by, &sort_order);

    Ok(images.into_iter().map(|(path, _)| path.to_string_lossy().into_owned()).collect())
}

fn sort_images(images: &mut [(PathBuf, std::fs::Metadata)], sort_by: &SortBy, sort_order: &SortOrder) {
    images.sort_by(|a, b| {
        let ordering = match sort_by {
            SortBy::Name => a.0.file_name().cmp(&b.0.file_name()),
            SortBy::Type => {
                let ext_a = a.0.extension().and_then(|s| s.to_str()).unwrap_or("");
                let ext_b = b.0.extension().and_then(|s| s.to_str()).unwrap_or("");
                ext_a.cmp(ext_b)
            },
            SortBy::Date => a.1.modified().unwrap_or(UNIX_EPOCH)
                .cmp(&b.1.modified().unwrap_or(UNIX_EPOCH)),
            SortBy::Size => a.1.len().cmp(&b.1.len()),
        };
        match sort_order {
            SortOrder::Asc => ordering,
            SortOrder::Desc => ordering.reverse(),
        }
    });
}

#[tauri::command]
async fn select_folder() -> Result<String, String> {
    let folder = tauri::api::dialog::blocking::FileDialogBuilder::new().pick_folder();
    match folder {
        Some(path) => Ok(path.to_string_lossy().into_owned()),
        None => Err("No folder selected".to_string()),
    }
}

#[tauri::command]
fn get_startup_info() -> Result<StartupInfo, String> {
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
fn save_last_folder(folder: String) -> Result<(), String> {
    let config = AppConfig {
        last_folder: Some(folder),
    };
    let config_path = config_dir().unwrap().join("image_viewer_config.json");
    let content = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn is_image(path: &Path) -> bool {
    let extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").expect("Failed to get main window");
            #[cfg(debug_assertions)]
            window.open_devtools();
            Ok(())
        })
        .manage(AppState {
            image_paths: Mutex::new(HashMap::new()),
        })
        .manage(Mutex::new(ImageState {
            current_index: 0,
            images: Vec::new(),
        }))
        .invoke_handler(tauri::generate_handler![
            get_directory_contents,
            get_root_folders,
            generate_thumbnail,
            select_folder,
            get_startup_info,
            save_last_folder,
            get_full_image_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}