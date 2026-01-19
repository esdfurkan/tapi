use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use std::path::{Path}; 
use tauri::command;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub children: Option<Vec<FileNode>>, // None = File, Some([]) = Empty Folder
    pub selected: bool,
    pub expanded: bool,
}

#[tauri::command]
pub fn open_folder(app: AppHandle, path: String) -> Result<(), String> {
    println!("Opening folder: {}", path);
    #[allow(deprecated)]
    app.shell().open(&path, None).map_err(|e| {
        let err = e.to_string();
        eprintln!("Failed to open folder: {}", err);
        err
    })
}

#[tauri::command]
pub fn open_translations_folder(app: AppHandle) -> Result<(), String> {
    let config_dir = app.path().app_config_dir().unwrap_or(std::path::PathBuf::from("."));
    let trans_dir = config_dir.join("translations");
    
    println!("Opening translations folder: {:?}", trans_dir);
    
    if !trans_dir.exists() {
        println!("Creating translations directory...");
        let _ = std::fs::create_dir_all(&trans_dir);
    }
    
    // Explicitly check existence after creation
    if !trans_dir.exists() {
        return Err("Failed to create translations directory".to_string());
    }

    #[allow(deprecated)]
    app.shell().open(trans_dir.to_string_lossy().to_string(), None).map_err(|e| {
        let err = e.to_string();
        eprintln!("Failed to open translations folder: {}", err);
        err
    })
}

#[command]
pub fn get_directory_structure(path: String) -> Result<FileNode, String> {
    let root_path = Path::new(&path);
    if !root_path.exists() {
        return Err("Path does not exist".to_string());
    }
    
    // We build the tree recursively with a depth limit to prevent hanging on large trees
    build_tree(root_path, 0, 3) // Max depth 3
}

#[command]
pub fn list_subdirectories(path: String) -> Result<Vec<String>, String> {
    let root = Path::new(&path);
    if !root.exists() {
        return Err("Path does not exist".to_string());
    }

    let mut folders = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    if let Ok(name) = entry.file_name().into_string() {
                       if !name.starts_with('.') { // Skip hidden folders
                           folders.push(name);
                       }
                    }
                }
            }
        }
    }
    // Sort alphabetically
    folders.sort();
    Ok(folders)
}

fn build_tree(path: &Path, current_depth: u32, max_depth: u32) -> Result<FileNode, String> {
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let path_str = path.to_string_lossy().to_string();
    
    if path.is_file() {
        return Ok(FileNode {
            name,
            path: path_str,
            children: None,
            selected: true,
            expanded: false,
        });
    }

    // If max depth reached, return as empty folder to stop recursion
    if current_depth > max_depth {
        return Ok(FileNode {
            name,
            path: path_str,
            children: Some(Vec::new()), 
            selected: true,
            expanded: false,
        });
    }

    // It's a directory
    let mut children = Vec::new();
    
    // Read dir
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            
            // Filter out hidden files or output folders
            if name.starts_with('.') || name == "translated" || name == ".f_history" || name.ends_with("_output") {
                continue;
            }
            
            // Only include images or folders or archives
            let is_image_or_archive = if p.is_file() {
                if let Some(ext) = p.extension() {
                    let e = ext.to_string_lossy().to_lowercase();
                    ["jpg", "jpeg", "png", "webp", "zip", "rar", "cbz", "cbr"].contains(&e.as_str())
                } else {
                    false
                }
            } else {
                false
            };
            
            if p.is_dir() || is_image_or_archive {
                 if let Ok(node) = build_tree(&p, current_depth + 1, max_depth) {
                    children.push(node);
                 }
            }
        }
    }
    
    // Sort directories first, then files
    children.sort_by(|a, b| {
         let a_is_dir = a.children.is_some();
         let b_is_dir = b.children.is_some();
         if a_is_dir == b_is_dir {
             a.name.cmp(&b.name)
         } else {
             if a_is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }
         }
    });

    Ok(FileNode {
        name,
        path: path_str,
        children: Some(children),
        selected: true,
        expanded: current_depth < 1, // Only expand top level
    })
}
