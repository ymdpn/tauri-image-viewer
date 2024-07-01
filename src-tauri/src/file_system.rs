use crate::models::{FileItem, AppState, SortBy, SortOrder};
use crate::utils::is_image;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use sha2::{Sha256, Digest};
use tauri::api::path::{home_dir};

#[tauri::command]
pub fn get_directory_contents(path: &str, state: State<'_, AppState>) -> Result<Vec<FileItem>, String> {
    get_directory_contents_impl(path, state.inner())
}

fn get_directory_contents_impl(path: &str, state: &AppState) -> Result<Vec<FileItem>, String> {
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
    get_root_folders_impl()
}

fn get_root_folders_impl() -> Vec<FileItem> {
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
    get_full_image_list_impl(path, sort_by, sort_order)
}

fn get_full_image_list_impl(path: &str, sort_by: SortBy, sort_order: SortOrder) -> Result<Vec<String>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn create_test_directory() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create some test files and directories
        fs::create_dir(base_path.join("dir1")).unwrap();
        fs::create_dir(base_path.join("dir2")).unwrap();
        File::create(base_path.join("file1.txt")).unwrap().write_all(b"Hello").unwrap();
        File::create(base_path.join("image1.jpg")).unwrap().write_all(b"JPEG").unwrap();
        File::create(base_path.join("image2.png")).unwrap().write_all(b"PNG").unwrap();

        temp_dir
    }

    #[test]
    fn test_get_directory_contents() {
        let temp_dir = create_test_directory();
        let state = AppState::new();
        let contents = get_directory_contents_impl(temp_dir.path().to_str().unwrap(), &state).unwrap();

        assert_eq!(contents.len(), 5);
        assert!(contents.iter().any(|item| item.name == "dir1" && item.is_dir));
        assert!(contents.iter().any(|item| item.name == "file1.txt" && !item.is_dir));
        assert!(contents.iter().any(|item| item.name == "image1.jpg" && !item.is_dir));
    }

    #[test]
    fn test_get_root_folders() {
        let roots = get_root_folders_impl();
        assert!(!roots.is_empty());
        assert!(roots.iter().any(|item| item.name == "Home"));
        assert!(roots.iter().any(|item| item.name == "Root"));
    }

    #[test]
    fn test_get_full_image_list() {
        let temp_dir = create_test_directory();
        let image_list = get_full_image_list_impl(
            temp_dir.path().join("image1.jpg").to_str().unwrap(),
            SortBy::Name,
            SortOrder::Asc
        ).unwrap();

        assert_eq!(image_list.len(), 2);
        assert!(image_list.iter().any(|path| path.ends_with("image1.jpg")));
        assert!(image_list.iter().any(|path| path.ends_with("image2.png")));
    }

    #[test]
    fn test_sort_images() {
        let temp_dir = create_test_directory();
        let base_path = temp_dir.path();

        let mut images: Vec<(PathBuf, std::fs::Metadata)> = vec![
            (base_path.join("image1.jpg"), fs::metadata(base_path.join("image1.jpg")).unwrap()),
            (base_path.join("image2.png"), fs::metadata(base_path.join("image2.png")).unwrap()),
        ];

        sort_images(&mut images, &SortBy::Name, &SortOrder::Asc);
        assert_eq!(images[0].0.file_name().unwrap(), "image1.jpg");
        assert_eq!(images[1].0.file_name().unwrap(), "image2.png");

        sort_images(&mut images, &SortBy::Name, &SortOrder::Desc);
        assert_eq!(images[0].0.file_name().unwrap(), "image2.png");
        assert_eq!(images[1].0.file_name().unwrap(), "image1.jpg");
    }
}