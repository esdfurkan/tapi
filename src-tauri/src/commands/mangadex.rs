use tauri::{AppHandle, Emitter};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct MangadexServerResponse {
    result: Option<String>,
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
    chapter: Option<MangadexChapterData>,
}

#[derive(Debug, Deserialize)]
struct MangadexChapterData {
    hash: String,
    data: Vec<String>,
    #[serde(default, rename = "dataSaver")]
    data_saver: Vec<String>,
}

#[derive(Clone, Serialize)]
struct DownloadProgress {
    current: usize,
    total: usize,
    status: String,
    filename: String,
}

#[tauri::command]
pub async fn download_mangadex_chapter(
    app_handle: AppHandle,
    url: String,
    folder_name: String,
    target_dir: String,
    use_data_saver: bool,
    user_agent: String,
    series_name: Option<String>,
) -> Result<String, String> {
    // 1. Extract UUID from URL
    let chapter_id = extract_uuid_from_url(&url).ok_or("Invalid MangaDex URL")?;

    // 2. Prepare directories
    let base_path = Path::new(&target_dir);
    if !base_path.exists() {
        return Err("Target directory does not exist".to_string());
    }

    // Determine final download path
    // Format: target_dir / [series_name] / folder_name
    let mut current_path = base_path.to_path_buf();
    
    if let Some(series) = series_name {
        if !series.trim().is_empty() {
             let safe_series = series.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != ' ', "_");
             current_path = current_path.join(safe_series);
        }
    }

    let safe_folder_name = folder_name.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != ' ', "_");
    let download_path = current_path.join(&safe_folder_name);
    
    if !download_path.exists() {
        fs::create_dir_all(&download_path).await.map_err(|e| e.to_string())?;
    }

    // 3. Call MangaDex API
    let client = Client::builder()
        .user_agent(user_agent)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let api_url = format!("https://api.mangadex.org/at-home/server/{}?forcePort443=false", chapter_id);
    
    let response = client.get(&api_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch chapter metadata: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API Request Failed. Status: {}", response.status()));
    }

    let text_content = response.text().await.map_err(|e| format!("Failed to read response body: {}", e))?;

    let resp: MangadexServerResponse = serde_json::from_str(&text_content)
        .map_err(|e| format!("Failed to parse API response: {}. Response excerpt: {:.500}", e, text_content))?;

    if resp.result.as_deref() != Some("ok") {
        return Err("MangaDex API returned an error or invalid response".to_string());
    }

    let base_url = resp.base_url.ok_or("Missing baseUrl")?;
    let chapter = resp.chapter.ok_or("Missing chapter data")?;

    let (images, mode_segment) = if use_data_saver {
        (&chapter.data_saver, "data-saver")
    } else {
        (&chapter.data, "data")
    };

    let total_images = images.len();
    let base_image_url = format!("{}/{}/{}", base_url, mode_segment, chapter.hash);

    // 4. Download images
    for (index, filename) in images.iter().enumerate() {
        let image_url = format!("{}/{}", base_image_url, filename);
        let file_path = download_path.join(filename);

        // Emit progress start for this file
        let _ = app_handle.emit("mangadex-progress", DownloadProgress {
            current: index + 1,
            total: total_images,
            status: "downloading".to_string(),
            filename: filename.clone(),
        });

        match download_file(&client, &image_url, &file_path).await {
            Ok(_) => {
                // Success for this file
            },
            Err(e) => {
                return Err(format!("Failed to download {}: {}", filename, e));
            }
        }
    }

    // Final success emit
    let _ = app_handle.emit("mangadex-progress", DownloadProgress {
        current: total_images,
        total: total_images,
        status: "completed".to_string(),
        filename: "".to_string(),
    });

    Ok(format!("Successfully downloaded {} images to {}", total_images, download_path.display()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangadexHistory {
    pub url: String,
    pub last_chapter: f32,
    pub series_name: Option<String>,
    pub language: Option<String>,
}

#[tauri::command]
pub async fn save_mangadex_history(
    target_dir: String,
    url: String,
    chapter: f32,
    series_name: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    let path = Path::new(&target_dir).join("mangadex_info.json");
    let history = MangadexHistory {
        url,
        last_chapter: chapter,
        series_name,
        language,
    };
    
    let content = serde_json::to_string_pretty(&history)
        .map_err(|e| e.to_string())?;

    fs::write(path, content).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn load_mangadex_history(target_dir: String) -> Result<MangadexHistory, String> {
    let path = Path::new(&target_dir).join("mangadex_info.json");
    
    if !path.exists() {
        return Err("History not found".to_string());
    }

    let content = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    let history: MangadexHistory = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    Ok(history)
}

fn extract_uuid_from_url(url: &str) -> Option<String> {
    // Supports: 
    // https://mangadex.org/chapter/UUID
    // https://mangadex.org/chapter/UUID/1
    if let Some(pos) = url.find("chapter/") {
        let remainder = &url[pos + "chapter/".len()..];
        // Take until next shash or end of string
        let uuid = remainder.split('/').next().unwrap_or("");
        // Simple validation: UUID is usually 36 chars
        if uuid.len() >= 36 {
            return Some(uuid.to_string());
        }
    }
    None
}

async fn download_file(client: &Client, url: &str, path: &Path) -> Result<(), String> {
    let resp = client.get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Download failed with status: {}", resp.status()));
    }

    let content = resp.bytes().await.map_err(|e| e.to_string())?;
    
    let mut file = fs::File::create(path).await.map_err(|e| e.to_string())?;
    file.write_all(&content).await.map_err(|e| e.to_string())?;
    
    Ok(())
}
