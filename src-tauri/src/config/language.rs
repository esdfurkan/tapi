use std::collections::HashMap;
use std::path::Path;
use std::fs;
use anyhow::Result;

#[derive(Clone, Debug, Default)]
pub struct Language {
    #[allow(dead_code)]
    pub strings: HashMap<String, String>,
}

impl Language {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut strings = HashMap::new();
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].to_string();
                continue;
            }

            if current_section == "strings" {
                if let Some((k, v)) = line.split_once('=') {
                    strings.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        
        Ok(Language { strings })
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> String {
        self.strings.get(key).cloned().unwrap_or_else(|| key.to_string())
    }
}
