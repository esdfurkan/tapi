use reqwest::{Client, multipart};
use std::path::Path;
use anyhow::Result;
use tokio::fs::{File};
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{BytesCodec, FramedRead};
use crate::utils::logger::log_debug;
use reqwest::header::{HeaderName, HeaderValue};
use std::str::FromStr;

#[derive(Clone)]
pub struct ApiEndpoints {
    pub storage: String,
    pub storage_headers: Option<String>,
    pub ocr: String,
    pub ocr_headers: Option<String>,
    pub translate: String,
    pub save_debug_json: bool,
}

impl Default for ApiEndpoints {
    fn default() -> Self {
        Self {
            storage: "https://api.toriitranslate.com/api/storage".to_string(),
            storage_headers: None,
            ocr: "https://api.toriitranslate.com/api/ocr".to_string(),
            ocr_headers: None,
            translate: "https://api.toriitranslate.com/api/upload".to_string(),
            save_debug_json: false,
        }
    }
}

pub struct ApiClient {
    client: Client,
    api_key: String,
    endpoints: ApiEndpoints,
}

impl ApiClient {
    pub fn new(api_key: String) -> Self {
        Self::new_with_endpoints(api_key, ApiEndpoints::default())
    }

    pub fn new_with_endpoints(api_key: String, endpoints: ApiEndpoints) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300)) 
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_key: api_key.trim().to_string(),
            endpoints,
        }
    }

    pub async fn translate_file(&self, file_path: &Path, model: &str, target_lang: &str, font: &str, text_align: &str, stroke_disabled: bool, inpaint_only: bool, min_font_size: u32) -> Result<Vec<u8>> {
        let mut retries = 0;
        let max_retries = 3;
        
        loop {
            log_debug(&format!("API: Opening file {:?}", file_path));
            let file = File::open(file_path).await?;
            let stream = FramedRead::new(file, BytesCodec::new());
            let file_body = reqwest::Body::wrap_stream(stream);
            
            let filename = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            log_debug(&format!("API: Preparing multipart for {}", filename));

            let form = multipart::Form::new()
                .part("file", multipart::Part::stream(file_body).file_name(filename));

            log_debug("API: Constructing request and headers...");
            
            let safe_header = |v: &str| -> String {
                v.chars().filter(|c| !c.is_control() && (*c as u32) < 127).collect()
            };

            let mut request_builder = self.client.post(&self.endpoints.translate)
                .multipart(form);

            log_debug("API: Adding headers...");
            request_builder = request_builder.header("Authorization", format!("Bearer {}", self.api_key));
            request_builder = request_builder.header("target_lang", safe_header(target_lang));
            request_builder = request_builder.header("translator", safe_header(model));
            request_builder = request_builder.header("font", safe_header(font));
            request_builder = request_builder.header("text_align", safe_header(text_align));
            request_builder = request_builder.header("stroke_disabled", stroke_disabled.to_string());
            request_builder = request_builder.header("inpaint_only", inpaint_only.to_string());
            request_builder = request_builder.header("min_font_size", min_font_size.to_string());

            log_debug("API: Sending request...");
            let result = request_builder.send().await;

            match result {
                Ok(response) => {
                    let status = response.status();
                    log_debug(&format!("API: Response status: {}", status));
                    
                    // Check success header
                    let success_header = response.headers()
                        .get("success")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("false");
                    
                    log_debug(&format!("API: Success header: {}", success_header));
                    
                    if success_header == "true" && status.is_success() {
                        let bytes = response.bytes().await?;
                        log_debug(&format!("API: Success! Bytes received: {}", bytes.len()));
                        return Ok(bytes.to_vec());
                    } else {
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        log_debug(&format!("API: Error response: {}", error_text));
                        
                        if retries >= max_retries {
                            return Err(anyhow::anyhow!("API Error ({}): {}", status, error_text));
                        }
                    }
                },
                Err(e) => {
                    log_debug(&format!("API: Network error: {}", e));
                    if retries >= max_retries {
                        return Err(anyhow::anyhow!("Network error after {} retries: {}", max_retries, e));
                    }
                }
            }
            
            retries += 1;
            log_debug(&format!("API: Retrying {}/{}", retries, max_retries));
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    fn add_custom_headers(request_builder: reqwest::RequestBuilder, headers_json: &Option<String>) -> reqwest::RequestBuilder {
        let mut builder = request_builder;
        if let Some(json_str) = headers_json {
            if !json_str.trim().is_empty() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(obj) = v.as_object() {
                        for (k, v) in obj {
                            if let Some(val_str) = v.as_str() {
                                if let (Ok(h_name), Ok(h_val)) = (HeaderName::from_str(k), HeaderValue::from_str(val_str)) {
                                    builder = builder.header(h_name, h_val);
                                }
                            }
                        }
                    }
                }
            }
        }
        builder
    }

    pub async fn call_storage(&self, storage_urls_arg: &str) -> Result<serde_json::Value> {
        let url = &self.endpoints.storage;
        log_debug(&format!("API: Calling Storage: {}", url));

        let mut request_builder = self.client.get(url);
        request_builder = request_builder.header("Authorization", format!("Bearer {}", self.api_key));
        
        request_builder = Self::add_custom_headers(request_builder, &self.endpoints.storage_headers);
        request_builder = request_builder.header("storage_urls", storage_urls_arg);

        let response = request_builder.send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        
        if self.endpoints.save_debug_json {
            if let Ok(mut file) =  File::create("debug_storage_response.json").await {
               let _ = file.write_all(&bytes).await;
            }
        }

        if !status.is_success() {
             return Err(anyhow::anyhow!("Storage API Error: {}", status));
        }

        let json: serde_json::Value = serde_json::from_slice(&bytes)?;
        Ok(json)
    }

    pub async fn call_ocr(&self, image_path: &Path) -> Result<serde_json::Value> {
         let url = &self.endpoints.ocr;
         log_debug(&format!("API: Calling OCR: {}", url));
         
         let file = File::open(image_path).await?;
         let stream = FramedRead::new(file, BytesCodec::new());
         let file_body = reqwest::Body::wrap_stream(stream);
         let filename = image_path.file_name().unwrap_or_default().to_string_lossy().to_string();

         let form = multipart::Form::new()
             .part("file", multipart::Part::stream(file_body).file_name(filename));

         let mut request_builder = self.client.post(url).multipart(form);
         request_builder = request_builder.header("Authorization", format!("Bearer {}", self.api_key));
         
         request_builder = Self::add_custom_headers(request_builder, &self.endpoints.ocr_headers);

         let response = request_builder.send().await?;
         let status = response.status();
         let bytes = response.bytes().await?;

         if self.endpoints.save_debug_json {
            if let Ok(mut file) =  File::create("debug_ocr_response.json").await {
               let _ = file.write_all(&bytes).await;
            }
         }
         
         if !status.is_success() {
             return Err(anyhow::anyhow!("OCR API Error: {}", status));
         }

         let json: serde_json::Value = serde_json::from_slice(&bytes)?;
         Ok(json)
    }
}
