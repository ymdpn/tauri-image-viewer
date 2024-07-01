use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Serialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub date_modified: u64,
    pub size: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageState {
    pub current_index: usize,
    pub images: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub last_folder: Option<String>,
}

pub struct AppState {
    pub image_paths: Mutex<HashMap<String, String>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            image_paths: Mutex::new(HashMap::new()),
        }
    }
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
pub struct StartupInfo {
    pub folder: String,
    pub file: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_item_serialization() {
        let file_item = FileItem {
            name: "test.txt".to_string(),
            path: "/path/to/test.txt".to_string(),
            is_dir: false,
            date_modified: 1234567890,
            size: 1024,
        };

        let serialized = serde_json::to_string(&file_item).unwrap();
        assert!(serialized.contains("test.txt"));
        assert!(serialized.contains("/path/to/test.txt"));
        assert!(serialized.contains("false"));
        assert!(serialized.contains("1234567890"));
        assert!(serialized.contains("1024"));
    }

    #[test]
    fn test_sort_by_serialization() {
        let sort_by = SortBy::Name;
        let serialized = serde_json::to_string(&sort_by).unwrap();
        assert_eq!(serialized, "\"name\"");

        let deserialized: SortBy = serde_json::from_str("\"size\"").unwrap();
        assert!(matches!(deserialized, SortBy::Size));
    }

    #[test]
    fn test_sort_order_serialization() {
        let sort_order = SortOrder::Asc;
        let serialized = serde_json::to_string(&sort_order).unwrap();
        assert_eq!(serialized, "\"asc\"");

        let deserialized: SortOrder = serde_json::from_str("\"desc\"").unwrap();
        assert!(matches!(deserialized, SortOrder::Desc));
    }

    #[test]
    fn test_app_state() {
        let app_state = AppState::new();
        let mut image_paths = app_state.image_paths.lock().unwrap();
        image_paths.insert("key".to_string(), "value".to_string());
        assert_eq!(image_paths.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_startup_info_serialization() {
        let startup_info = StartupInfo {
            folder: "/home/user".to_string(),
            file: Some("/home/user/image.jpg".to_string()),
        };

        let serialized = serde_json::to_string(&startup_info).unwrap();
        assert!(serialized.contains("/home/user"));
        assert!(serialized.contains("/home/user/image.jpg"));

        let deserialized: StartupInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.folder, "/home/user");
        assert_eq!(deserialized.file, Some("/home/user/image.jpg".to_string()));
    }
}