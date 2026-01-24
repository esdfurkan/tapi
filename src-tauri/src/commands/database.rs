use crate::state::AppState;
use crate::core::database::{HashEntryOutput, DatabaseManager};
use tauri::{command, State};

#[command]
pub async fn save_hash_name(state: State<'_, AppState>, hash: String, name: String, folder: String) -> Result<(), String> {
    let mode = {
        let profile = state.profile.read().await;
        profile.database_mode.clone()
    };

    if mode == "off" {
        return Ok(());
    }

    let db_lock = state.db.read().await;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    
    db.save_hash(hash, name, folder).await.map_err(|e| e.to_string())
}

#[command]
pub async fn get_name_by_hash(state: State<'_, AppState>, hash: String) -> Result<Option<String>, String> {
    let db_lock = state.db.read().await;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    
    db.get_name(&hash).await.map_err(|e| e.to_string())
}

#[command]
pub async fn list_hash_names(state: State<'_, AppState>) -> Result<Vec<HashEntryOutput>, String> {
    let db_lock = state.db.read().await;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    
    db.list_all().await.map_err(|e| e.to_string())
}

#[command]
pub async fn delete_hash_entry(state: State<'_, AppState>, hash: String) -> Result<(), String> {
    let db_lock = state.db.read().await;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    
    db.delete_hash(&hash).await.map_err(|e| e.to_string())
}

#[command]
pub async fn clear_all_database(state: State<'_, AppState>) -> Result<(), String> {
    let db_lock = state.db.read().await;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    
    db.clear_all().await.map_err(|e| e.to_string())
}

#[command]
pub async fn pull_remote_database(state: State<'_, AppState>) -> Result<(), String> {
    let (url, token, user, pass) = {
        let profile = state.profile.read().await;
        if profile.database_mode != "remote" {
            return Err("Not in remote mode".to_string());
        }
        if profile.remote_db_url.is_empty() {
             return Err("Remote URL not set".to_string());
        }
        (profile.remote_db_url.clone(), profile.remote_db_token.clone(), profile.remote_db_user.clone(), profile.remote_db_pass.clone())
    };

    let db_lock = state.db.read().await;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.pull_from_remote(&url, &token, &user, &pass).await.map_err(|e| e.to_string())
}

#[command]
pub async fn push_remote_database(state: State<'_, AppState>) -> Result<(), String> {
    let (url, token, user, pass) = {
        let profile = state.profile.read().await;
        if profile.database_mode != "remote" {
            return Err("Not in remote mode".to_string());
        }
        if profile.remote_db_url.is_empty() {
             return Err("Remote URL not set".to_string());
        }
        (profile.remote_db_url.clone(), profile.remote_db_token.clone(), profile.remote_db_user.clone(), profile.remote_db_pass.clone())
    };

    let db_lock = state.db.read().await;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.push_to_remote(&url, &token, &user, &pass).await.map_err(|e| e.to_string())
}

#[command]
pub async fn test_database_connection(state: State<'_, AppState>) -> Result<String, String> {
    let (url, token, user, pass) = {
        let profile = state.profile.read().await;
        if profile.remote_db_url.is_empty() {
             return Err("Remote URL not set".to_string());
        }
        (profile.remote_db_url.clone(), profile.remote_db_token.clone(), profile.remote_db_user.clone(), profile.remote_db_pass.clone())
    };

    // Use the proper test method - no side effects
    match DatabaseManager::test_remote_connection(&url, &token, &user, &pass).await {
        Ok(_) => Ok("Connection successful!".to_string()),
        Err(e) => Err(format!("Connection failed: {}", e))
    }
}

