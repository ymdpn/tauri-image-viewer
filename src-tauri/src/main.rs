mod file_system;
mod image_processing;
mod config;
mod models;
mod utils;
use log::LevelFilter;
use tauri::Manager;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").expect("Failed to get main window");
            #[cfg(debug_assertions)]
            window.open_devtools();
            Ok(())
        })
        .manage(models::AppState::new())
        .invoke_handler(tauri::generate_handler![
            file_system::get_directory_contents,
            file_system::get_root_folders,
            file_system::get_full_image_list,
            image_processing::generate_thumbnail,
            config::get_startup_info,
            config::save_last_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
