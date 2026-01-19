use tauri::{AppHandle, Emitter};
use std::path::Path;
use tokio::fs;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, REFERER};
use serde::{Deserialize, Serialize};
use regex::Regex;
use base64::{Engine as _, engine::general_purpose};

#[derive(Clone, Serialize)]
struct DownloadProgress {
    current: usize,
    total: usize,
    status: String,
    filename: String,
}

#[tauri::command]
pub async fn save_base64_image(
    target_path: String,
    base64_data: String,
) -> Result<String, String> {
    // Remove header if present (e.g., "data:image/png;base64,...")
    let base64_clean = if let Some(index) = base64_data.find(',') {
        &base64_data[index + 1..]
    } else {
        &base64_data
    };

    let bytes = general_purpose::STANDARD
        .decode(base64_clean)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    let path = Path::new(&target_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }

    fs::write(path, bytes).await.map_err(|e| e.to_string())?;
    Ok(format!("Saved fallback image to {}", target_path))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NHentaiGallery {
    id: u32,
    media_id: String,
    images: NHentaiImages,
    title: NHentaiTitle,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NHentaiTitle {
    english: String,
    pretty: String,
}

#[derive(Debug, Deserialize)]
struct NHentaiImages {
    pages: Vec<NHentaiImage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NHentaiImage {
    t: String, // "j" for jpg, "p" for png, "w" for webp
    w: u32,
    h: u32,
}

#[tauri::command]
pub async fn wrapper_download_url(
    app_handle: AppHandle,
    url: String,
    _folder_name: Option<String>,
    target_dir: String,
    user_agent: Option<String>,
) -> Result<String, String> {
    
    // Use provided UA or fallback to a generic recent Chrome
    let ua = user_agent.unwrap_or_else(|| "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36".to_string());

    let result = if url.contains("nhentai") {
        download_nhentai(app_handle.clone(), url.clone(), target_dir.clone(), ua.clone()).await
    } else if url.contains("pixiv.net") {
        download_pixiv(app_handle.clone(), url.clone(), target_dir.clone(), ua.clone()).await
    } else {
        download_generic(app_handle.clone(), url.clone(), target_dir.clone(), ua.clone()).await
    };

    match result {
        Ok(msg) => Ok(msg),
        Err(e) => {
            // Check for specific blocking errors to trigger fallback
            if e.contains("403") || e.contains("503") || e.contains("Cloudflare") || e.contains("Login Required") {
                // Return a special signal code instead of an error.
                // The frontend should catch this "FALLBACK_REQUIRED" string and switch to WebView mode.
                Ok("FALLBACK_REQUIRED".to_string())
            } else {
                Err(e)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Generic Scraper: Scans ANY page for .jpg, .png, .webp links and downloads them.
// Useful for direct links or simple gallery sites.
// ---------------------------------------------------------------------------
async fn download_generic(app_handle: AppHandle, url: String, target_dir: String, ua: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    // 1. Check if the URL itself is an image
    if url.ends_with(".jpg") || url.ends_with(".jpeg") || url.ends_with(".png") || url.ends_with(".webp") {
        let filename = url.split('/').last().unwrap_or("image.png").to_string();
        let file_path = Path::new(&target_dir).join(&filename);
        
        // Single file download
        app_handle.emit("download_progress", DownloadProgress {
            current: 0,
            total: 1,
            status: "downloading_single".to_string(),
            filename: filename.clone(),
        }).map_err(|e| e.to_string())?;

        let bytes = client.get(&url).header(USER_AGENT, &ua).send().await.map_err(|e| e.to_string())?
            .bytes().await.map_err(|e| e.to_string())?;
        
        fs::write(&file_path, bytes).await.map_err(|e| e.to_string())?;
        
        return Ok(format!("Saved single image to {}", file_path.display()));
    }

    // 2. It's a webpage, fetch HTML and scan for images
    let resp = client.get(&url).header(USER_AGENT, &ua).send().await.map_err(|e| e.to_string())?;
    let base_url = resp.url().clone(); // For resolving relative links
    let html = resp.text().await.map_err(|e| e.to_string())?;

    // Regex to find potential image links (naive approach, but effective for simple sites)
    // Matches href="...jpg" or src="...png" etc.
    let re = Regex::new(r#"(?:href|src)=["']([^"']+\.(?:jpg|jpeg|png|webp))["']"#).map_err(|e| e.to_string())?;
    
    let mut image_links = Vec::new();
    for cap in re.captures_iter(&html) {
        if let Some(link) = cap.get(1) {
            let link_str = link.as_str();
            // Resolve relative URLs
            if let Ok(absolute_url) = base_url.join(link_str) {
                 image_links.push(absolute_url);
            }
        }
    }
    
    // Deduplicate
    image_links.sort();
    image_links.dedup();

    if image_links.is_empty() {
        return Err("No images found on this page via generic scraper.".to_string());
    }

    // Create a folder based on page title or domain
    let folder_name = base_url.host_str().unwrap_or("generic_download");
    let safe_folder = sanitize_filename::sanitize(folder_name);
    let final_dir = Path::new(&target_dir).join(&safe_folder);
    if !final_dir.exists() {
        fs::create_dir_all(&final_dir).await.map_err(|e| e.to_string())?;
    }

    let total = image_links.len();

    // Download loop
    for (i, img_url) in image_links.iter().enumerate() {
        let filename = img_url.path_segments().and_then(|s| s.last()).unwrap_or("image.png");
        let file_path = final_dir.join(filename);

        app_handle.emit("download_progress", DownloadProgress {
            current: i + 1,
            total,
            status: "downloading".to_string(),
            filename: filename.to_string(),
        }).map_err(|e| e.to_string())?;

        // Download with tolerance for failures
        if let Ok(response) = client.get(img_url.clone()).header(USER_AGENT, &ua).send().await {
            if let Ok(bytes) = response.bytes().await {
                let _ = fs::write(&file_path, bytes).await;
            }
        }
    }

    app_handle.emit("download_progress", DownloadProgress {
        current: total,
        total,
        status: "completed".to_string(),
        filename: "Done".to_string(),
    }).map_err(|e| e.to_string())?;

    Ok(format!("Generic scrape downloaded {} images to {}", total, final_dir.display()))
}

async fn download_nhentai(app_handle: AppHandle, url: String, target_dir: String, ua: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    // Extract ID (Gallery ID) from URL
    // URL: nhentai.net/g/{id}/...
    let re = Regex::new(r"g/(\d+)").map_err(|e| e.to_string())?;
    let captures = re.captures(&url).ok_or("Invalid nHentai URL")?;
    let id = &captures[1];
    
    let api_url = format!("https://nhentai.net/api/gallery/{}", id);
    
    // Fetch Metadata
    let resp = client.get(&api_url)
        .header(USER_AGENT, &ua)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch gallery metadata: {}", e))?;
        
    if !resp.status().is_success() {
        return Err(format!("Failed to connect to nHentai. Status: {}. (Cloudflare blocked?)", resp.status()));
    }
    
    let gallery: NHentaiGallery = resp.json().await.map_err(|e| format!("Get JSON failed: {}", e))?;
    
    // Note on Logic:
    // Gallery ID (in URL) != Media ID (image server path).
    // The API returns 'media_id' which is used for constructing the image URL.
    // Example: Gallery 622461 -> Media 3733533
    // Image URL: https://i.nhentai.net/galleries/3733533/{page}.{ext}
    
    let safe_title = sanitize_filename::sanitize(&gallery.title.pretty);
    let final_dir = Path::new(&target_dir).join(&safe_title);
    
    if !final_dir.exists() {
        fs::create_dir_all(&final_dir).await.map_err(|e| e.to_string())?;
    }
    
    let total_pages = gallery.images.pages.len();
    
    for (i, img) in gallery.images.pages.iter().enumerate() {
        let ext = match img.t.as_str() {
            "j" => "jpg",
            "p" => "png",
            "w" => "webp",
            _ => "jpg",
        };
        
        // We use i.nhentai.net as the primary CDN. 
        // Subdomains like i4, i7 exist but i.nhentai.net usually handles routing or is the canonical public hostname.
        let img_url = format!("https://i.nhentai.net/galleries/{}/{}.{}", gallery.media_id, i + 1, ext);
        
        let filename = format!("{:03}.{}", i + 1, ext);
        let file_path = final_dir.join(&filename);
        
        app_handle.emit("download_progress", DownloadProgress {
            current: i + 1,
            total: total_pages,
            status: "downloading".to_string(),
            filename: filename.clone(),
        }).map_err(|e| e.to_string())?;

        let img_bytes = client.get(&img_url)
             .header(USER_AGENT, &ua)
             .send()
             .await
             .map_err(|e| e.to_string())?
             .bytes()
             .await
             .map_err(|e| e.to_string())?;
             
        fs::write(&file_path, img_bytes).await.map_err(|e| e.to_string())?;
    }
    
    app_handle.emit("download_progress", DownloadProgress {
        current: total_pages,
        total: total_pages,
        status: "completed".to_string(),
        filename: "Done".to_string(),
    }).map_err(|e| e.to_string())?;

    Ok(format!("Downloaded to {}", final_dir.display()))
}

// Minimal Pixiv implementation
async fn download_pixiv(app_handle: AppHandle, url: String, target_dir: String, ua: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let re = Regex::new(r"artworks/(\d+)").map_err(|e| e.to_string())?;
    let captures = re.captures(&url).ok_or("Invalid Pixiv URL")?;
    let id = &captures[1];
    
    let api_url = format!("https://www.pixiv.net/ajax/illust/{}/pages", id);
    
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(&ua).map_err(|_| "Invalid UA string")?);
    headers.insert(REFERER, HeaderValue::from_static("https://www.pixiv.net/"));

    let resp = client.get(&api_url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Pixiv metadata: {}", e))?;

    #[derive(Deserialize)]
    struct PixivPage {
        urls: PixivUrls,
    }
    #[derive(Deserialize)]
    struct PixivUrls {
        original: String, // We target original quality
    }
    #[derive(Deserialize)]
    struct PixivResponse {
        body: Vec<PixivPage>,
        error: bool,
    }

    let json: PixivResponse = resp.json().await.map_err(|e| format!("Failed to parse Pixiv JSON: {}", e))?;
    
    if json.error {
        return Err("Pixiv returned an error (Deleted or Login Required)".to_string());
    }

    let folder_name = format!("pixiv_{}", id);
    let final_dir = Path::new(&target_dir).join(&folder_name);
     if !final_dir.exists() {
        fs::create_dir_all(&final_dir).await.map_err(|e| e.to_string())?;
    }
    
    let total = json.body.len();
    
    for (i, page) in json.body.iter().enumerate() {
        let img_url = &page.urls.original;
        let filename = img_url.split('/').last().unwrap_or("image.png");
        let file_path = final_dir.join(filename);
        
        app_handle.emit("download_progress", DownloadProgress {
            current: i + 1,
            total: total,
            status: "downloading".to_string(),
            filename: filename.to_string(),
        }).map_err(|e| e.to_string())?;

        let img_bytes = client.get(img_url)
             .headers(headers.clone())
             .send()
             .await
             .map_err(|e| e.to_string())?
             .bytes()
             .await
             .map_err(|e| e.to_string())?;
             
        fs::write(&file_path, img_bytes).await.map_err(|e| e.to_string())?;
    }

    app_handle.emit("download_progress", DownloadProgress {
        current: total,
        total: total,
        status: "completed".to_string(),
        filename: "Done".to_string(),
    }).map_err(|e| e.to_string())?;

    Ok(format!("Downloaded to {}", final_dir.display()))
}
