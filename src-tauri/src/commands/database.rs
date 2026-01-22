use crate::state::AppState;
use crate::core::database::HashEntryOutput;
use tauri::{command, State};

#[command]
pub async fn save_hash_name(state: State<'_, AppState>, hash: String, name: String) -> Result<(), String> {
    let mode = {
        let profile = state.profile.lock().unwrap();
        profile.database_mode.clone()
    };

    if mode == "off" {
        return Ok(());
    }

    let db = {
        let db_lock = state.db.lock().unwrap();
        db_lock.clone().ok_or("Database not initialized")?
    };
    db.save_hash(hash, name).await.map_err(|e: anyhow::Error| e.to_string())
}

#[command]
pub async fn get_name_by_hash(state: State<'_, AppState>, hash: String) -> Result<Option<String>, String> {
    let db = {
        let db_lock = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        db_lock.clone().ok_or("Database not initialized")?
    };
    db.get_name(&hash).await.map_err(|e: anyhow::Error| e.to_string())
}

#[command]
pub async fn list_hash_names(state: State<'_, AppState>) -> Result<Vec<HashEntryOutput>, String> {
    let db = {
        let db_lock = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        db_lock.clone().ok_or("Database not initialized")?
    };
    db.list_all().await.map_err(|e: anyhow::Error| e.to_string())
}

#[command]
pub async fn delete_hash_entry(state: State<'_, AppState>, hash: String) -> Result<(), String> {
    let db = {
        let db_lock = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        db_lock.clone().ok_or("Database not initialized")?
    };
    db.delete_hash(&hash).await.map_err(|e: anyhow::Error| e.to_string())
}

#[command]
pub async fn clear_all_database(state: State<'_, AppState>) -> Result<(), String> {
    let db = {
        let db_lock = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        db_lock.clone().ok_or("Database not initialized")?
    };
    db.clear_all().await.map_err(|e: anyhow::Error| e.to_string())
}

#[command]
pub async fn pull_remote_database(state: State<'_, AppState>) -> Result<(), String> {
    let (url, token, user, pass) = {
        let profile = state.profile.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        if profile.database_mode != "remote" {
            return Err("Not in remote mode".to_string());
        }
        if profile.remote_db_url.is_empty() {
             return Err("Remote URL not set".to_string());
        }
        (profile.remote_db_url.clone(), profile.remote_db_token.clone(), profile.remote_db_user.clone(), profile.remote_db_pass.clone())
    };

    let db = {
        let db_lock = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        db_lock.clone().ok_or("Database not initialized")?
    };

    db.pull_from_remote(&url, &token, &user, &pass).await.map_err(|e: anyhow::Error| e.to_string())
}

#[command]
pub async fn push_remote_database(state: State<'_, AppState>) -> Result<(), String> {
    let (url, token, user, pass) = {
        let profile = state.profile.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        if profile.database_mode != "remote" {
            return Err("Not in remote mode".to_string());
        }
        if profile.remote_db_url.is_empty() {
             return Err("Remote URL not set".to_string());
        }
        (profile.remote_db_url.clone(), profile.remote_db_token.clone(), profile.remote_db_user.clone(), profile.remote_db_pass.clone())
    };

    let db = {
        let db_lock = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        db_lock.clone().ok_or("Database not initialized")?
    };

    db.push_to_remote(&url, &token, &user, &pass).await.map_err(|e: anyhow::Error| e.to_string())
}

#[command]
pub async fn test_database_connection(state: State<'_, AppState>) -> Result<String, String> {
    let (url, token, user, pass) = {
        let profile = state.profile.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        if profile.remote_db_url.is_empty() {
             return Err("Remote URL not set".to_string());
        }
        (profile.remote_db_url.clone(), profile.remote_db_token.clone(), profile.remote_db_user.clone(), profile.remote_db_pass.clone())
    };

    let db = {
        let db_lock = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        db_lock.clone().ok_or("Database not initialized")?
    };

    // We call pull but with a query that does nothing just to verify auth/connection
    match db.pull_from_remote(&url, &token, &user, &pass).await {
        Ok(_) => Ok("Connection successful!".to_string()),
        Err(e) => Err(format!("Connection failed: {}", e))
    }
}
