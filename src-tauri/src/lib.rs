pub mod commands;
pub mod config;
pub mod core;
pub mod modes;
pub mod state;
pub mod utils;

use state::AppState;
use tauri::Manager;
use std::path::PathBuf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
                #[cfg(not(target_os = "android"))]
                crate::utils::monitor::start_stats_thread(app.handle());
                
                // Load settings from config dir
                let handle = app.handle();
                let config_dir = handle.path().app_config_dir().unwrap_or(PathBuf::from("."));
                let profile_path = config_dir.join("profile.json");
                
                if profile_path.exists() {
                    if let Ok(profile) = crate::config::profile::Profile::load(&profile_path) {
                         let state = handle.state::<AppState>();
                         let mut lock = state.profile.lock().unwrap_or_else(|e: std::sync::PoisonError<_>| e.into_inner());
                         *lock = profile;
                    }
                }

                // Create translations directory and extract defaults
                let translations_dir = config_dir.join("translations");
                if !translations_dir.exists() {
                    let _ = std::fs::create_dir_all(&translations_dir);
                    
                    // Default translations to extract
                    let en_json = include_str!("../../src/lib/translations/en.json");
                    let tr_json = include_str!("../../src/lib/translations/tr.json");
                    
                    let _ = std::fs::write(translations_dir.join("en.json"), en_json);
                    let _ = std::fs::write(translations_dir.join("tr.json"), tr_json);
                }

                // Initialize database
                let db_path = config_dir.join("tapi_db"); // Changed name to avoid conflict with sqlite
                let handle_clone = handle.clone();
                tauri::async_runtime::block_on(async move {
                    if let Ok(db_manager) = crate::core::database::DatabaseManager::new(db_path).await {
                        let state = handle_clone.state::<AppState>();
                        let mut db_lock = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string()).unwrap();
                        *db_lock = Some(db_manager);
                    }
                });

                Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::translation::start_translation,
            commands::file_ops::open_folder,
            commands::file_ops::open_translations_folder,
            commands::settings::save_settings,
            commands::settings::load_settings,
            commands::mangadex::download_mangadex_chapter,
            commands::mangadex::save_mangadex_history,
            commands::mangadex::load_mangadex_history,
            commands::downloader::wrapper_download_url,
            commands::downloader::save_base64_image,
            commands::file_ops::get_directory_structure,
            commands::file_ops::list_subdirectories,
            commands::database::save_hash_name,
            commands::database::get_name_by_hash,
            commands::database::list_hash_names,
            commands::database::delete_hash_entry,
            commands::database::clear_all_database,
            commands::database::pull_remote_database,
            commands::database::push_remote_database,
            commands::database::test_database_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
