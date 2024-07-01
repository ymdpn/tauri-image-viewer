use crate::models::{FileItem, AppState, SortBy, SortOrder};
use crate::utils::is_image;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use sha2::{Sha256, Digest};
use tauri::api::path::home_dir;
use std::sync::Mutex;
use log::{info, debug, error};

#[tauri::command]
pub fn get_directory_contents(path: &str, state: State<'_, AppState>) -> Result<Vec<FileItem>, String> {
    get_directory_contents_impl(path, &state.inner().image_paths)
}

fn get_directory_contents_impl(path: &str, image_paths: &Mutex<std::collections::HashMap<String, String>>) -> Result<Vec<FileItem>, String> {
    let mut items = Vec::new();
    let mut image_paths = image_paths.lock().unwrap();
    
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
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let size = if is_dir { 0 } else { metadata.len() };

                let _id = if is_dir {
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
pub fn get_root_folders() -> Vec<FileItem> {
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
pub fn get_full_image_list(path: &str, sort_by: SortBy, sort_order: SortOrder) -> Result<Vec<String>, String> {
    info!("get_full_image_list called with path: {}", path);
    let dir_path = Path::new(path);
    debug!("Directory path: {:?}", dir_path);
    
    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read directory: {:?}", e);
            return Err(format!("Failed to read directory: {}", e));
        }
    };

    let mut images: Vec<(PathBuf, std::fs::Metadata)> = entries
        .filter_map(|entry| {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    debug!("Checking file: {:?}", path);
                    if path.is_file() {
                        match is_image(&path) {
                            true => {
                                debug!("Found image: {:?}", path);
                                match fs::metadata(&path) {
                                    Ok(metadata) => Some((path, metadata)),
                                    Err(e) => {
                                        error!("Failed to get metadata for {:?}: {:?}", path, e);
                                        None
                                    }
                                }
                            },
                            false => {
                                debug!("Not an image: {:?}", path);
                                None
                            }
                        }
                    } else {
                        debug!("Not a file: {:?}", path);
                        None
                    }
                },
                Err(e) => {
                    error!("Error reading directory entry: {:?}", e);
                    None
                }
            }
        })
        .collect();

    debug!("Collected images: {:?}", images);

    sort_images(&mut images, &sort_by, &sort_order);

    let result: Vec<String> = images.into_iter()
        .map(|(path, _)| path.to_string_lossy().into_owned())
        .collect();
    info!("Returning {} images", result.len());
    Ok(result)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    use std::thread::sleep;
    use std::time::Duration;
    use std::collections::HashMap;
    use env_logger;
    use std::path::PathBuf;

    fn create_test_directory() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        info!("Creating test directory at: {:?}", base_path);

        let file_data = [
            ("file1.txt", "Hello", 0),
            ("file2.jpg", "Hello World", 1),
            ("file3.png", "Hello World!", 2),
        ];

        for (name, content, delay) in file_data.iter() {
            let path = base_path.join(name);
            info!("Creating file: {:?}", path);
            let mut file = File::create(&path).expect("Failed to create file");
            file.write_all(content.as_bytes()).expect("Failed to write content");
            sleep(Duration::from_secs(*delay));
            let metadata = file.metadata().expect("Failed to get metadata");
            let mtime = metadata.modified().expect("Failed to get modification time") + Duration::from_secs(*delay);
            filetime::set_file_mtime(&path, filetime::FileTime::from_system_time(mtime)).expect("Failed to set modification time");
            
            if !path.exists() {
                error!("Failed to create file: {:?}", path);
            } else {
                info!("Successfully created file: {:?}", path);
            }
        }

        (temp_dir, base_path)
    }

    #[test]
    fn test_get_directory_contents() {
        let (_temp_dir, base_path) = create_test_directory();
        let image_paths = Mutex::new(HashMap::new());
        
        let contents = get_directory_contents_impl(base_path.to_str().unwrap(), &image_paths).unwrap();

        assert_eq!(contents.len(), 3, "Expected 3 files, but found {}", contents.len());
        assert!(contents.iter().any(|item| item.name == "file1.txt"));
        assert!(contents.iter().any(|item| item.name == "file2.jpg"));
        assert!(contents.iter().any(|item| item.name == "file3.png"));
    }

    #[test]
    fn test_get_full_image_list() {
        let _ = env_logger::builder().is_test(true).try_init();
    
        let (_temp_dir, dir_path) = create_test_directory();
        info!("Test directory: {:?}", dir_path);
        
        for entry in fs::read_dir(&dir_path).unwrap() {
            let entry = entry.unwrap();
            info!("File in test directory: {:?}", entry.path());
        }
    
        // dir_path を直接使用
        let result = get_full_image_list(dir_path.to_str().unwrap(), SortBy::Name, SortOrder::Asc);
        match result {
            Ok(images) => {
                info!("Result: {:?}", images);
                assert_eq!(images.len(), 2, "Expected 2 image files, but found {}", images.len());
                if images.len() >= 2 {
                    assert!(images[0].ends_with("file2.jpg"), "Expected file2.jpg, got {}", images[0]);
                    assert!(images[1].ends_with("file3.png"), "Expected file3.png, got {}", images[1]);
                }
            },
            Err(e) => {
                error!("Error in get_full_image_list: {:?}", e);
                panic!("get_full_image_list failed: {}", e);
            }
        }
    }
    #[test]
    fn test_sort_images() {
        let (_temp_dir, base_path) = create_test_directory();

        let mut images: Vec<(PathBuf, std::fs::Metadata)> = fs::read_dir(&base_path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let metadata = entry.metadata().unwrap();
                Some((entry.path(), metadata))
            })
            .collect();

        sort_images(&mut images, &SortBy::Name, &SortOrder::Asc);
        assert_eq!(images.len(), 3, "Expected 3 files, but found {}", images.len());
        assert_eq!(images[0].0.file_name().unwrap(), "file1.txt");
        assert_eq!(images[1].0.file_name().unwrap(), "file2.jpg");
        assert_eq!(images[2].0.file_name().unwrap(), "file3.png");
    }

    #[test]
    fn test_sort_images_by_date() {
        let (_temp_dir, base_path) = create_test_directory();

        let mut images: Vec<(PathBuf, std::fs::Metadata)> = fs::read_dir(&base_path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let metadata = entry.metadata().unwrap();
                Some((entry.path(), metadata))
            })
            .collect();

        sort_images(&mut images, &SortBy::Date, &SortOrder::Asc);
        assert_eq!(images[0].0.file_name().unwrap(), "file1.txt", "Expected file1.txt to be oldest");
        assert_eq!(images[1].0.file_name().unwrap(), "file2.jpg", "Expected file2.jpg to be second oldest");
        assert_eq!(images[2].0.file_name().unwrap(), "file3.png", "Expected file3.png to be newest");

        sort_images(&mut images, &SortBy::Date, &SortOrder::Desc);
        assert_eq!(images[0].0.file_name().unwrap(), "file3.png", "Expected file3.png to be newest");
        assert_eq!(images[1].0.file_name().unwrap(), "file2.jpg", "Expected file2.jpg to be second newest");
        assert_eq!(images[2].0.file_name().unwrap(), "file1.txt", "Expected file1.txt to be oldest");
    }

    #[test]
    fn test_sort_images_by_size() {
        let (_temp_dir, base_path) = create_test_directory();

        let mut images: Vec<(PathBuf, std::fs::Metadata)> = fs::read_dir(&base_path)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let metadata = entry.metadata().unwrap();
                Some((entry.path(), metadata))
            })
            .collect();

        sort_images(&mut images, &SortBy::Size, &SortOrder::Asc);
        assert_eq!(images[0].0.file_name().unwrap(), "file1.txt");
        assert_eq!(images[1].0.file_name().unwrap(), "file2.jpg");
        assert_eq!(images[2].0.file_name().unwrap(), "file3.png");

        sort_images(&mut images, &SortBy::Size, &SortOrder::Desc);
        assert_eq!(images[0].0.file_name().unwrap(), "file3.png");
        assert_eq!(images[1].0.file_name().unwrap(), "file2.jpg");
        assert_eq!(images[2].0.file_name().unwrap(), "file1.txt");
    }
}