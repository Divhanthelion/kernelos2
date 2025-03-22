use serde::{Serialize, Deserialize};
use web_sys::Storage;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    pub name: String,
    pub file_type: FileType,
    pub size: usize,
    pub created: u64,  // Timestamp
    pub modified: u64, // Timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileSystem {
    // Using a simplified approach where paths are keys
    files: HashMap<String, FileMetadata>,
}

impl FileSystem {
    pub fn new() -> Result<Self, String> {
        // Try to load existing file system from local storage
        if let Some(storage) = Self::get_storage() {
            if let Ok(Some(data)) = storage.get_item("wasm_desktop_fs") {
                if !data.is_empty() {
                    match serde_json::from_str::<FileSystem>(&data) {
                        Ok(fs) => return Ok(fs),
                        Err(_) => {
                            // If loading fails, create a new file system
                            log::warn!("Failed to load file system, creating new one");
                        }
                    }
                }
            }
        }

        // Create new file system with root directory
        let mut fs = FileSystem {
            files: HashMap::new(),
        };

        // Initialize with root directory
        let now = js_sys::Date::now() as u64;
        fs.files.insert("/".to_string(), FileMetadata {
            name: "/".to_string(),
            file_type: FileType::Directory,
            size: 0,
            created: now,
            modified: now,
        });

        // Create basic directory structure
        fs.create_directory("/home", true)?;
        fs.create_directory("/home/documents", true)?;
        fs.create_directory("/home/pictures", true)?;
        fs.create_directory("/applications", true)?;

        // Save the initial file system
        fs.save()?;

        Ok(fs)
    }

    fn get_storage() -> Option<Storage> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok())
            .flatten()
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(storage) = Self::get_storage() {
            let serialized = serde_json::to_string(self)
                .map_err(|e| format!("Failed to serialize file system: {}", e))?;

            storage
                .set_item("wasm_desktop_fs", &serialized)
                .map_err(|e| format!("Failed to save file system: {:?}", e))?;

            Ok(())
        } else {
            Err("Local storage not available".to_string())
        }
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<FileMetadata>, String> {
        // Normalize path
        let path = Self::normalize_path(path);
        
        // Check if path exists and is a directory
        if let Some(metadata) = self.files.get(&path) {
            if !matches!(metadata.file_type, FileType::Directory) {
                return Err(format!("{} is not a directory", path));
            }
        } else {
            return Err(format!("Directory {} does not exist", path));
        }

        // List all files in this directory
        let mut files = Vec::new();
        let path_prefix = if path.ends_with('/') { path.clone() } else { format!("{}/", path) };
        
        for (file_path, metadata) in &self.files {
            if file_path != &path && file_path.starts_with(&path_prefix) {
                // Only include direct children, not descendants
                let remaining = &file_path[path_prefix.len()..];
                if !remaining.contains('/') {
                    files.push(metadata.clone());
                }
            }
        }

        Ok(files)
    }

    pub fn create_directory(&mut self, path: &str, create_parents: bool) -> Result<(), String> {
        let path = Self::normalize_path(path);
        
        // Check if the directory already exists
        if self.files.contains_key(&path) {
            return Err(format!("{} already exists", path));
        }

        if create_parents {
            // Ensure parent directories exist
            let parent_path = Path::new(&path).parent()
                .ok_or_else(|| "Invalid path".to_string())?
                .to_string_lossy()
                .to_string();
            
            if !parent_path.is_empty() && parent_path != "/" && !self.files.contains_key(&parent_path) {
                self.create_directory(&parent_path, true)?;
            }
        } else {
            // Check if parent directory exists
            let parent_path = Path::new(&path).parent()
                .ok_or_else(|| "Invalid path".to_string())?
                .to_string_lossy()
                .to_string();
            
            if !parent_path.is_empty() && parent_path != "/" && !self.files.contains_key(&parent_path) {
                return Err(format!("Parent directory {} does not exist", parent_path));
            }
        }

        // Create directory
        let now = js_sys::Date::now() as u64;
        let name = Path::new(&path).file_name()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_string_lossy()
            .to_string();
        
        self.files.insert(path, FileMetadata {
            name,
            file_type: FileType::Directory,
            size: 0,
            created: now,
            modified: now,
        });

        self.save()?;
        Ok(())
    }

    pub fn write_file(&mut self, path: &str, contents: &str) -> Result<(), String> {
        let path = Self::normalize_path(path);
        
        // Make sure parent directory exists
        let parent_path = Path::new(&path).parent()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_string_lossy()
            .to_string();
        
        if !parent_path.is_empty() && parent_path != "/" && !self.files.contains_key(&parent_path) {
            return Err(format!("Parent directory {} does not exist", parent_path));
        }

        // Get filename
        let name = Path::new(&path).file_name()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_string_lossy()
            .to_string();

        // Create or update file metadata
        let now = js_sys::Date::now() as u64;
        let created = if let Some(existing) = self.files.get(&path) {
            existing.created
        } else {
            now
        };

        self.files.insert(path.clone(), FileMetadata {
            name,
            file_type: FileType::File,
            size: contents.len(),
            created,
            modified: now,
        });

        // Store file contents separately
        if let Some(storage) = Self::get_storage() {
            let content_key = format!("wasm_desktop_file:{}", path);
            storage
                .set_item(&content_key, contents)
                .map_err(|e| format!("Failed to write file: {:?}", e))?;
        } else {
            return Err("Local storage not available".to_string());
        }

        self.save()?;
        Ok(())
    }

    pub fn read_file(&self, path: &str) -> Result<String, String> {
        let path = Self::normalize_path(path);
        
        // Check if file exists
        if let Some(metadata) = self.files.get(&path) {
            if !matches!(metadata.file_type, FileType::File) {
                return Err(format!("{} is not a file", path));
            }
        } else {
            return Err(format!("File {} does not exist", path));
        }

        // Retrieve file contents
        if let Some(storage) = Self::get_storage() {
            let content_key = format!("wasm_desktop_file:{}", path);
            if let Ok(Some(contents)) = storage.get_item(&content_key) {
                Ok(contents)
            } else {
                Err(format!("Failed to read file {}", path))
            }
        } else {
            Err("Local storage not available".to_string())
        }
    }

    pub fn delete(&mut self, path: &str, recursive: bool) -> Result<(), String> {
        let path = Self::normalize_path(path);
        
        // Check if path exists
        if !self.files.contains_key(&path) {
            return Err(format!("{} does not exist", path));
        }

        let is_directory = matches!(self.files.get(&path).unwrap().file_type, FileType::Directory);
        
        if is_directory {
            // Check for children
            let children = self.list_directory(&path)?;
            if !children.is_empty() && !recursive {
                return Err(format!("Directory {} is not empty", path));
            }

            if recursive {
                // Delete all children recursively
                let path_prefix = if path.ends_with('/') { path.clone() } else { format!("{}/", path) };
                
                // Collect paths to delete first to avoid borrowing issues
                let paths_to_delete: Vec<String> = self.files.keys()
                    .filter(|k| **k != path && k.starts_with(&path_prefix))
                    .cloned()
                    .collect();
                
                // Delete files first
                if let Some(storage) = Self::get_storage() {
                    for file_path in &paths_to_delete {
                        if matches!(self.files.get(file_path).unwrap().file_type, FileType::File) {
                            let content_key = format!("wasm_desktop_file:{}", file_path);
                            let _ = storage.remove_item(&content_key);
                        }
                    }
                }
                
                // Then remove all entries
                for file_path in paths_to_delete {
                    self.files.remove(&file_path);
                }
            }
        } else {
            // Delete file content
            if let Some(storage) = Self::get_storage() {
                let content_key = format!("wasm_desktop_file:{}", path);
                let _ = storage.remove_item(&content_key);
            }
        }

        // Remove the entry itself
        self.files.remove(&path);
        
        self.save()?;
        Ok(())
    }

    // Helper method to normalize paths
    fn normalize_path(path: &str) -> String {
        let path = path.trim();
        if path.is_empty() {
            return "/".to_string();
        }
        
        let path_obj = PathBuf::from(path);
        let normalized = path_obj.to_string_lossy().to_string();
        
        if normalized == "." {
            "/".to_string()
        } else {
            normalized
        }
    }
} 