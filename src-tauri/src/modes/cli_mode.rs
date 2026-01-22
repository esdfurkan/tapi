use crate::core::processor::{process_directory, TranslationOptions};
use crate::core::api::ApiEndpoints;
use crate::utils::logger::ProgressLogger;
use crate::config::profile::Profile;
use std::path::Path;
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub async fn start_cli_translation(
    logger: &impl ProgressLogger, 
    folder: &Path, 
    model: &str, 
    api_key: &str, 
    target_lang: &str, 
    font: &str, 
    text_align: &str,
    stroke_disabled: bool,
    inpaint_only: bool,
    min_font_size: u32,
    profile: Option<Arc<Mutex<Profile>>>,
    db: Option<Arc<Mutex<Option<crate::core::database::DatabaseManager>>>>,
    output_folder: Option<String>,
    included_paths: Option<Vec<String>>
) -> Result<()> {
    println!("Starting CLI translation for {:?}", folder);
    
    let output_dir = if let Some(out) = output_folder {
        Path::new(&out).to_path_buf()
    } else {
        folder.join("translated")
    };
    
    // Get custom endpoints from profile if available
    let endpoints = if let Some(ref p) = profile {
        if let Ok(prof) = p.lock() {
            Some(ApiEndpoints {
                storage: prof.storage_url.clone(),
                storage_headers: Some(prof.storage_headers.clone()),
                ocr: prof.ocr_url.clone(),
                ocr_headers: Some(prof.ocr_headers.clone()),
                translate: prof.translate_url.clone(),
                save_debug_json: prof.save_debug_json,
            })
        } else {
            None
        }
    } else {
        None
    };
    
    let options = TranslationOptions {
        model: model.to_string(),
        api_key: api_key.to_string(),
        target_lang: target_lang.to_string(),
        font: font.to_string(),
        text_align: text_align.to_string(),
        stroke_disabled,
        inpaint_only,
        min_font_size,
        profile,
        endpoints,
        included_paths,
        db,
    };

    process_directory(logger, folder, &output_dir, &options).await?;
    
    Ok(())
}
