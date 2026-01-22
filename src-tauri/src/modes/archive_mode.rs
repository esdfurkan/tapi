use crate::core::processor::{process_directory, TranslationOptions};
use crate::core::api::ApiEndpoints;
use crate::core::archive::{extract_zip, create_zip};
use crate::core::pdf::extract_images_from_pdf;
use crate::utils::logger::ProgressLogger;
use crate::config::profile::Profile;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};

pub async fn start_archive_translation(
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
    println!("Starting archive translation in {:?}", folder);
    
    let output_base = if let Some(out) = output_folder {
        Path::new(&out).to_path_buf()
    } else {
        folder.join("archive_outputs")
    };
    fs::create_dir_all(&output_base)?;

    let mut archives_found = 0;
    let mut success_count = 0;

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

    // Find all archives
    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ["zip", "cbz", "pdf"].contains(&ext_str.as_str()) {
                    // Filter by included_paths if provided
                    if let Some(ref includes) = included_paths {
                        let path_str = path.to_string_lossy().to_string();
                        let is_included = includes.iter().any(|inc| path_str.starts_with(inc));
                        if !is_included {
                            continue;
                        }
                    }

                    archives_found += 1;
                    logger.log(format!("Processing: {:?}", path.file_name().unwrap_or_default()));
                    
                    // 1. Extract
                    let temp_dir = folder.join("temp_extract");
                    if temp_dir.exists() { fs::remove_dir_all(&temp_dir)?; }
                    fs::create_dir_all(&temp_dir)?;
                    
                    let extract_res = if ext_str == "pdf" {
                        extract_images_from_pdf(path, &temp_dir)
                    } else {
                        extract_zip(path, &temp_dir).map(|_| 1) // Zip doesn't return count yet, assume > 0
                    };

                    if let Err(e) = extract_res {
                        logger.log(format!("Extraction error for {:?}: {}", path.file_name().unwrap_or_default(), e));
                        let _ = fs::remove_dir_all(&temp_dir);
                        continue;
                    }

                    // 2. Translate
                    let temp_out = folder.join("temp_translated");
                    if temp_out.exists() { fs::remove_dir_all(&temp_out)?; }
                    
                    let options = TranslationOptions {
                        model: model.to_string(),
                        api_key: api_key.to_string(),
                        target_lang: target_lang.to_string(),
                        font: font.to_string(),
                        text_align: text_align.to_string(),
                        stroke_disabled,
                        inpaint_only,
                        min_font_size,
                        profile: profile.clone(),
                        endpoints: endpoints.clone(),
                        included_paths: included_paths.clone(), // Pass down for fine-grained image filtering
                        db: db.clone(),
                    };
                    
                    if let Err(e) = process_directory(logger, &temp_dir, &temp_out, &options).await {
                        logger.log(format!("Translation error for {:?}: {}", path.file_name().unwrap_or_default(), e));
                        let _ = fs::remove_dir_all(&temp_dir);
                        let _ = fs::remove_dir_all(&temp_out);
                        continue;
                    }

                    // 3. Repack
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    let out_name = if ext_str == "pdf" {
                        format!("{}.cbz", file_name)
                    } else {
                        file_name.to_string()
                    };
                    let out_path = output_base.join(out_name);
                    
                    if let Err(e) = create_zip(&temp_out, &out_path) {
                        logger.log(format!("Repack error for {:?}: {}", path.file_name().unwrap_or_default(), e));
                    } else {
                        success_count += 1;
                        logger.log(format!("Successfully translated: {}", out_path.display()));
                    }
                    
                    // Cleanup
                    let _ = fs::remove_dir_all(&temp_dir);
                    let _ = fs::remove_dir_all(&temp_out);
                }
            }
        }
    }

    if archives_found == 0 {
        return Err(anyhow!("No valid archives (zip, cbz, pdf) found in the selected folder."));
    }

    if success_count == 0 {
        return Err(anyhow!("Failed to translate any archives in the folder. Check logs for details."));
    }

    logger.log(format!("Task completed! Processed {}/{} archives successfully.", success_count, archives_found));
    Ok(())
}
