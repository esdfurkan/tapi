use std::path::Path;
use anyhow::{Result, anyhow};
use lopdf::{Document, Object, Dictionary};
use std::fs::File;
use std::io::Write;

pub fn extract_images_from_pdf(pdf_path: &Path, output_dir: &Path) -> Result<usize> {
    let doc = Document::load(pdf_path).map_err(|e| anyhow!("Failed to load PDF: {}", e))?;
    let mut image_count = 0;

    for (object_id, _) in doc.objects.iter() {
        if let Ok(stream) = doc.get_object(*object_id).and_then(|obj| obj.as_stream()) {
            if is_image_dict(&stream.dict) {
                let extension = get_extension(&stream.dict);
                image_count += 1;
                let file_name = format!("page_{:04}.{}", image_count, extension);
                let file_path = output_dir.join(file_name);
                
                let mut file = File::create(file_path)?;
                file.write_all(&stream.content)?;
            }
        }
    }

    if image_count == 0 {
        return Err(anyhow!("No images found in PDF"));
    }

    Ok(image_count)
}

fn is_image_dict(dict: &Dictionary) -> bool {
    dict.get(b"Subtype")
        .and_then(|obj| obj.as_name())
        .map(|name| name == b"Image")
        .unwrap_or(false)
}

fn get_extension(dict: &Dictionary) -> &'static str {
    if let Ok(filter) = dict.get(b"Filter") {
        match filter {
            Object::Name(name) => {
                if name == b"DCTDecode" { return "jpg"; }
                if name == b"JPXDecode" { return "jp2"; }
                if name == b"FlateDecode" { return "png"; }
            }
            Object::Array(arr) => {
                for obj in arr {
                    if let Ok(name) = obj.as_name() {
                        if name == b"DCTDecode" { return "jpg"; }
                        if name == b"JPXDecode" { return "jp2"; }
                        if name == b"FlateDecode" { return "png"; }
                    }
                }
            }
            _ => {}
        }
    }
    "bin"
}
