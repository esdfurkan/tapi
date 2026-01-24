use crate::core::image::{find_all_images,  save_image_with_limit};
use crate::core::api::{ApiClient, ApiEndpoints};


use crate::utils::logger::{ProgressLogger, log_debug};
use std::path::{Path, PathBuf};

use std::fs;
use std::collections::HashSet;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::profile::Profile;

pub struct TranslationOptions {
    pub model: String,
    pub api_key: String,
    pub target_lang: String,
    pub font: String,
    pub text_align: String,
    pub stroke_disabled: bool,
    pub inpaint_only: bool,
    pub min_font_size: u32,
    pub profile: Option<Arc<RwLock<Profile>>>,
    pub endpoints: Option<ApiEndpoints>,
    pub included_paths: Option<Vec<String>>,
    pub db: Option<Arc<RwLock<Option<crate::core::database::DatabaseManager>>>>,
}

fn get_model_cost(model: &str) -> u64 {
    match model {
        "gemini-2.5-flash" | "deepseek" | "grok-4-fast" | "gemini-3-flash" => 1,
        // Assume others are 1 for now unless specified
        _ => 1, 
    }
}

async fn calculate_file_hash(path: &Path) -> Result<String> {
    log_debug(&format!("START HASH: {:?}", path));
    let path = path.to_owned();
    let hash = tokio::task::spawn_blocking(move || {
        // Use a smaller buffer for hashing to reduce per-thread memory footprint? 
        // Default copy is usually efficient (8KB-128KB).
        // The main issue is likely the sheer number of threads spawned previously.
        log_debug(&format!("BLOCKING HASH START: {:?}", path));
        let mut file = std::fs::File::open(&path)?;
        let mut hasher = blake3::Hasher::new();
        std::io::copy(&mut file, &mut hasher)?;
        let h = hasher.finalize().to_hex().to_string();
        log_debug(&format!("BLOCKING HASH END: {:?}", path));
        Ok::<String, anyhow::Error>(h)
    }).await??;
    log_debug(&format!("END HASH: {}", hash));
    Ok(hash)
}

fn load_history(path: &Path) -> HashSet<String> {
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashSet::new()
    }
}

fn save_history(path: &Path, history: &HashSet<String>) {
    if let Ok(content) = serde_json::to_string(history) {
        // Use a temporary file to ensure atomic writes (prevents corruption on crash)
        let tmp_path = path.with_extension("tmp");
        if fs::write(&tmp_path, content).is_ok() {
            let _ = fs::rename(&tmp_path, path);
        }
    }
}

pub async fn process_directory(logger: &impl ProgressLogger, input_dir: &Path, output_dir: &Path, options: &TranslationOptions) -> Result<()> {
    let mut all_images = find_all_images(input_dir);
    
    // Filter by included_paths if provided
    if let Some(ref includes) = options.included_paths {
        let include_set: HashSet<String> = includes.iter().cloned().collect();
        all_images.retain(|img| {
            // Check if the image itself is in the set OR any of its parent directories are in the set
            let img_str = img.to_string_lossy().to_string();
            if include_set.contains(&img_str) {
                return true;
            }
            
            // Check parent components relative to input_dir? 
            // Better: Check if any part of the path is in include_set
            // Actually, the UI usually sends the Absolute Path of the checked items.
            // Let's check if the path starts with any of the included paths.
            includes.iter().any(|inc| img_str.starts_with(inc))
        });
    }

    if all_images.is_empty() {
        logger.log("No images found in directory (or none selected).".to_string());
        return Ok(());
    }

    fs::create_dir_all(output_dir)?;
    
    // Load history from input directory (local to the folder being processed)
    let history_path = input_dir.join(".f_history");
    log_debug(&format!("HISTORY PATH: {:?}", history_path));
    let mut history = load_history(&history_path);
    
    // Filter images
    let mut images_to_process = Vec::new();
    let total_start = all_images.len();
    
    // Log start of filtering
    logger.log(format!("Dosyalar taranıyor ({})...", total_start));

    let mut skipped_count = 0;
    
    // Limit concurrency for hashing to prevent resource exhaustion
    // Using a JoinSet for async tasks which wrap the blocking hash calls
    let mut join_set: tokio::task::JoinSet<Result<(PathBuf, String), anyhow::Error>> = tokio::task::JoinSet::new();
    // CRITICAL: Reduced concurrency to prevent system crash/freeze
    // Hashing is IO and CPU heavy. Too many parallel tasks kill the OS scheduler and disk cache.
    let max_concurrent = 3; 

    // We need to keep track of original paths vs results to check existence first
    // Actually, checking file existence is fast (sync). Hashing is slow.
    // We should filter existence first, THEN hash.

    let mut pending_images = Vec::with_capacity(all_images.len());
    
    // Pre-filter by checking output existence (fast)
    for img_path in all_images {
        // Calculate relative output path to preserve folder structure
        let relative_path = img_path.strip_prefix(input_dir).unwrap_or_else(|_| img_path.file_name().map(Path::new).unwrap_or(Path::new("unknown")));
        let out_path = output_dir.join(relative_path);
        
        if out_path.exists() {
            skipped_count += 1;
        } else {
            pending_images.push(img_path);
        }
    }

    if skipped_count > 0 {
        logger.log(format!("... {} dosya zaten var, atlandı.", skipped_count));
    }
    
    // Now process hashes for the remaining
    skipped_count = 0; // Reset for hash skips
    let _total_to_hash = pending_images.len();
    let mut processed_hash_count = 0;

    // Check DB for existing hashes if available
    let mut db_existing_hashes = HashSet::new();
    let db_manager = if let Some(ref db_rwlock) = options.db {
        let db_lock = db_rwlock.read().await;
        db_lock.clone()
    } else {
        None
    };

    if let Some(db) = db_manager {
        if let Ok(entries) = db.list_all().await {
            for entry in entries {
                db_existing_hashes.insert(entry.hash);
            }
        }
    }

    for img_path in pending_images {
        // Manage concurrency
        log_debug(&format!("Queueing hash for: {:?}", img_path));
        while join_set.len() >= max_concurrent {
            if let Some(res) = join_set.join_next().await {
                if let Ok(Ok((path, hash))) = res {
                     log_debug(&format!("Joined hash task: {:?}", path));
                     processed_hash_count += 1;
                     if history.contains(&hash) || db_existing_hashes.contains(&hash) {
                         skipped_count += 1;
                     } else {
                         // Calculate output path again here to pass it down
                         let relative_path = path.strip_prefix(input_dir).unwrap_or_else(|_| path.file_name().map(Path::new).unwrap_or(Path::new("unknown"))).to_path_buf();
                         let out_path = output_dir.join(relative_path);
                         images_to_process.push((path, out_path, hash));
                     }
                } else {
                     log_debug("Failed to join hash task");
                }
            }
            // Give breathing room to the runtime
            tokio::task::yield_now().await;
        }
        
        // Spawn
        let p = img_path.clone();
        join_set.spawn(async move {
            let h = calculate_file_hash(&p).await?;
            Ok::<_, anyhow::Error>((p, h))
        });
        
        // UI updates occasionally
        if processed_hash_count > 0 && processed_hash_count % 100 == 0 {
             tokio::task::yield_now().await;
        }
    }
    
    log_debug("Finished queuing all hash tasks. Waiting for remaining...");

    // Wait for remaining tasks
    while let Some(res) = join_set.join_next().await {
        if let Ok(Ok((path, hash))) = res {
             log_debug(&format!("Joined remaining hash task: {:?}", path));
             //processed_hash_count += 1;
             if history.contains(&hash) || db_existing_hashes.contains(&hash) {
                  skipped_count += 1;
             } else {
                  let relative_path = path.strip_prefix(input_dir).unwrap_or_else(|_| path.file_name().map(Path::new).unwrap_or(Path::new("unknown"))).to_path_buf();
                  let out_path = output_dir.join(relative_path);
                  images_to_process.push((path, out_path, hash));
             }
        }
    }

    if skipped_count > 0 {
         logger.log(format!("... {} dosya tarihçeye göre atlandı.", skipped_count));
    }
    
    if images_to_process.is_empty() {
        logger.log("Tüm dosyalar zaten işlenmiş.".to_string());
        return Ok(());
    }
    
    logger.log(format!("İşlenecek dosya sayısı: {}", images_to_process.len()));

    let client = if let Some(endpoints) = &options.endpoints {
        ApiClient::new_with_endpoints(options.api_key.clone(), endpoints.clone())
    } else {
        ApiClient::new(options.api_key.clone())
    };

        process_individual(logger, images_to_process, output_dir, &client, options, &mut history, &history_path).await?;

    Ok(())
}

async fn process_individual(
    logger: &impl ProgressLogger, 
    images: Vec<(PathBuf, PathBuf, String)>, 
    _output_dir_parent: &Path, // Not strictly needed as we have full output paths in images
    client: &ApiClient, 
    options: &TranslationOptions,
    history: &mut HashSet<String>,
    history_path: &Path
) -> Result<()> {
    let mut processed_for_save = 0;
    let total_images = images.len();

    for (idx, (img_path, out_path, hash)) in images.into_iter().enumerate() {
        let current_num = idx + 1;
        log_debug(&format!("START PROCESS INDIVIDUAL: {:?}", img_path));
        let msg = format!("Processing {}/{} - {:?}", current_num, total_images, img_path.file_name().unwrap_or_default());
        logger.progress(current_num, total_images, msg);
        
        // Check file size and compress if needed
        let mut path_to_send = img_path.clone();
        let mut temp_file_created = false;
        
        if let Ok(metadata) = fs::metadata(&img_path) {
            if metadata.len() > 15 * 1024 * 1024 {
                log_debug(&format!("COMPRESSING LARGE FILE: {:?}", img_path));
                let msg = format!("Compressing large file: {:?}", img_path.file_name().unwrap_or_default());
                logger.log(msg);

                let img_path_clone = img_path.clone(); // Clone for closure
                let compress_result = tokio::task::spawn_blocking(move || {
                     log_debug(&format!("BLOCKING COMPRESS START: {:?}", img_path_clone));
                     if let Ok(img) = image::open(&img_path_clone) {
                        let temp_path = img_path_clone.with_extension("tmp.jpg");
                        if save_image_with_limit(img, &temp_path, 14.8).is_ok() {
                            log_debug(&format!("BLOCKING COMPRESS SUCCESS: {:?}", temp_path));
                            return Some(temp_path);
                        }
                     }
                     log_debug(&format!("BLOCKING COMPRESS FAIL: {:?}", img_path_clone));
                     None
                }).await.unwrap_or(None);

                if let Some(tp) = compress_result {
                    path_to_send = tp;
                    temp_file_created = true;
                }
            }
        }

        log_debug(&format!("SENDING API REQUEST: {:?}", path_to_send));
        match client.translate_file(&path_to_send, &options.model, &options.target_lang, &options.font, &options.text_align, options.stroke_disabled, options.inpaint_only, options.min_font_size).await {
            Ok(image_bytes) => {
                log_debug(&format!("API SUCCESS, BYTES: {}", image_bytes.len()));
                
                // Ensure parent directory exists
                if let Some(parent) = out_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                
                // Save the received image bytes directly
                if let Err(e) = fs::write(&out_path, image_bytes) {
                     log_debug(&format!("SAVE ERROR: {}", e));
                     logger.log(format!("Kaydetme Hatası: {}", e));
                } else {
                    log_debug(&format!("SAVED: {:?}", out_path));

                    // Update credits used
                    if let Some(profile_rwlock) = &options.profile {
                        let mut profile = profile_rwlock.write().await;
                        profile.total_credits_used += get_model_cost(&options.model);
                        let _ = profile.save(Path::new("profile.json"));
                    }

                    // Update history
                    if !hash.is_empty() {
                        history.insert(hash.clone());
                        
                        // ALSO Save to Database for centralized history
                        if let Some(ref db_rwlock) = options.db {
                            let db_lock = db_rwlock.read().await;
                            if let Some(db) = db_lock.as_ref() {
                                let name = img_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                // Get the immediate folder name for grouping
                                let folder_name = img_path.parent()
                                    .and_then(|p| p.file_name())
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "Root".to_string());
                                
                                let db_clone = db.clone();
                                let hash_clone = hash.clone();
                                tokio::spawn(async move {
                                    let _ = db_clone.save_hash(hash_clone, name, folder_name).await;
                                });
                            }
                        }

                        processed_for_save += 1;
                        // Batch saves to disk to prevent IO overload
                        if processed_for_save >= 10 {
                            save_history(history_path, history);
                            processed_for_save = 0;
                        }
                    }
                }
            },
            Err(e) => {
                let err_msg = format!("Failed to translate {:?}: {}", img_path.file_name().unwrap_or_default(), e);
                logger.log(err_msg);
            },
        }
        
        if temp_file_created {
            let _ = fs::remove_file(path_to_send);
        }

        // Rate limit delay (Individual mode)
        logger.log("Waiting 3 seconds...".to_string());
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
    
    // Save any pending history updates
    if processed_for_save > 0 {
        save_history(history_path, history);
    }
    
    Ok(())
}
