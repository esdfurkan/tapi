use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use walkdir::{WalkDir, DirEntry};
use image::{DynamicImage, GenericImageView};
use anyhow::Result;

pub fn find_all_images(path: &Path) -> Vec<PathBuf> {
    let mut images = Vec::new();
    
    let is_ignored = |entry: &DirEntry| -> bool {
        let name = entry.file_name().to_string_lossy();
        // Return true to keep (include), false to skip (exclude)
        !name.starts_with('.') && name != "translated" && name != "error" && !name.ends_with("_output")
    };

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(is_ignored)
        .filter_map(|e| e.ok()) 
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ["jpg", "jpeg", "png", "webp"].contains(&ext_str.as_str()) {
                    images.push(path.to_path_buf());
                }
            }
        }
    }
    images
}

pub fn preprocess_image(path: &Path) -> Result<DynamicImage> {
    let img = image::open(path)?;
    let (width, height) = img.dimensions();
    
    // Resize if too large (e.g., height > 2500px) to save API costs/time
    // This is a heuristic, can be adjusted
    if height > 2500 {
        let new_height = 2500;
        let new_width = (width as f32 * (new_height as f32 / height as f32)) as u32;
        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
        return Ok(resized);
    }
    
    Ok(img)
}

pub fn save_image_with_limit(img: DynamicImage, path: &Path, max_mb: f64) -> Result<()> {
    let target_bytes = (max_mb * 1024.0 * 1024.0) as u64;
    // Convert to RGB8 initially to drop alpha channel for JPEG
    // consume the image since we own it now
    let mut current_img = DynamicImage::ImageRgb8(img.into_rgb8());
    
    // Strategy: High quality (85-90) is priority. 
    // If file is too big, resize the image iteratively
    
    loop {
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        
        // Try with high quality first
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 85);
        encoder.encode(current_img.as_bytes(), current_img.width(), current_img.height(), current_img.color().into())?;
        
        if buffer.len() as u64 <= target_bytes {
            let mut file = File::create(path)?;
            file.write_all(&buffer)?;
            break;
        }
        
        // If too big, resize down by 10%
        let new_w = (current_img.width() as f32 * 0.9) as u32;
        let new_h = (current_img.height() as f32 * 0.9) as u32;
        
        if new_w < 100 || new_h < 100 {
             // Emergency fallback if it gets too small
            let mut file = File::create(path)?;
            file.write_all(&buffer)?;
            break;
        }
        
        // Resize and update current_img
        // filter: Lanczos3 gives best quality
        current_img = current_img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
        // Ensure it is RGB8 again for consistent encoding, though resize usually returns DynamicImage which handles it
        if let DynamicImage::ImageRgb8(_) = current_img {
            // ok
        } else {
            current_img = DynamicImage::ImageRgb8(current_img.into_rgb8());
        }
    }
    Ok(())
}

