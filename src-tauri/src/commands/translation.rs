use tauri::{State, Window};
use crate::state::AppState;
use crate::modes::cli_mode::start_cli_translation;
use crate::modes::archive_mode::start_archive_translation;
use std::path::Path;

#[tauri::command]
pub async fn start_translation(
    window: Window,
    state: State<'_, AppState>, 
    folder_path: String, 
    model: String, 
    mode: Option<String>, 
    target_lang: Option<String>,
    font: Option<String>,
    text_align: Option<String>,
    stroke_disabled: Option<bool>,
    inpaint_only: Option<bool>,
    min_font_size: Option<u32>,
    output_folder: Option<String>,
    included_paths: Option<Vec<String>>
) -> Result<(), String> {
    println!("Starting translation for {} with model {}", folder_path, model);
    
    let api_key = {
        let profile = state.profile.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        profile.api_key.clone().ok_or("API Key not found in settings")?
    };

    let path = Path::new(&folder_path);
    let mode_str = mode.unwrap_or_else(|| "cli".to_string());
    let t_lang = target_lang.unwrap_or_else(|| "en".to_string());
    let font_str = font.unwrap_or_else(|| "wildwords".to_string());
    
    let txt_align = text_align.unwrap_or_else(|| "auto".to_string());
    let stroke = stroke_disabled.unwrap_or(false);
    let inpaint = inpaint_only.unwrap_or(false);
    let min_size = min_font_size.unwrap_or(12);
    
    let profile_state = Some(state.profile.clone());

    match mode_str.as_str() {
        "archive" => {
            start_archive_translation(&window, path, &model, &api_key, &t_lang, &font_str, &txt_align, stroke, inpaint, min_size, profile_state, output_folder, included_paths)
                .await
                .map_err(|e| e.to_string())?;
        },
        _ => {
            start_cli_translation(&window, path, &model, &api_key, &t_lang, &font_str, &txt_align, stroke, inpaint, min_size, profile_state, output_folder, included_paths)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
        
    Ok(())
}
