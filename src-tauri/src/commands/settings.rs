use tauri::{State, AppHandle, Manager};
use crate::state::AppState;
use crate::config::profile::Profile;
use std::path::PathBuf;
use std::fs;

fn get_config_path(app: &AppHandle) -> PathBuf {
    // Android: /data/user/0/com.furkan.tapi/files/config/profile.json
    // Desktop: ~/.config/com.furkan.tapi/profile.json (or similar)
    app.path().app_config_dir().unwrap_or(PathBuf::from(".")).join("profile.json")
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<AppState>, settings: Profile) -> Result<(), String> {
    // Update in-memory state
    {
        let mut profile = state.profile.lock().map_err(|e| format!("Lock error: {}", e))?;
        *profile = settings.clone();
    }
    
    let path = get_config_path(&app);
    
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    // Save to disk (with encryption)
    settings.save(&path).map_err(|e| format!("Failed to save profile: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn load_settings(state: State<AppState>) -> Result<Profile, String> {
    // Just return what's in memory (which was populated at startup)
    let profile = state.profile.lock().map_err(|e| e.to_string())?;
    Ok(profile.clone())
}
