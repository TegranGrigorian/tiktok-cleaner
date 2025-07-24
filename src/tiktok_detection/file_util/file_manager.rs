use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cache structure for storing previously scanned files to avoid re-analysis
/// 
/// This structure maintains a history of scanned files with their metadata
/// to significantly improve performance on subsequent scans by skipping
/// files that haven't changed since the last scan.
#[derive(Debug, Serialize, Deserialize)]
struct NotTikTokCache {
    /// List of file paths that were identified as non-TikTok content
    pub scanned_files: Vec<String>,
    /// Timestamp of the last cache update
    pub last_updated: String,
    /// File modification time tracking to detect changed files
    pub file_metadata: HashMap<String, FileInfo>,
    /// Cache version for compatibility tracking
    pub cache_version: String,
}

/// File information for cache validation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    /// File size in bytes
    pub size: u64,
    /// Last modification time as RFC3339 string
    pub modified: String,
    /// Analysis result (confidence score)
    pub confidence: u32,
    /// Whether file was identified as TikTok content
    pub is_tiktok: bool,
}

impl NotTikTokCache {
    /// Creates a new empty cache with current timestamp
    pub fn new() -> Self {
        NotTikTokCache {
            scanned_files: Vec::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            file_metadata: HashMap::new(),
            cache_version: "2.0".to_string(),
        }
    }

    /// Checks if a file path exists in the non-TikTok cache
    /// 
    /// # Arguments
    /// * `file_path` - The file path to check
    /// 
    /// # Returns
    /// `true` if the file is in the cache, `false` otherwise
    pub fn contains(&self, file_path: &str) -> bool {
        self.scanned_files.contains(&file_path.to_string())
    }

    /// Checks if a file should be skipped based on cache and modification time
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file
    /// * `current_size` - Current file size
    /// * `current_modified` - Current modification time
    /// 
    /// # Returns
    /// `true` if the file can be skipped, `false` if it needs re-analysis
    pub fn should_skip_file(&self, file_path: &Path, current_size: u64, current_modified: &str) -> bool {
        let path_str = file_path.to_string_lossy().to_string();
        
        if let Some(cached_info) = self.file_metadata.get(&path_str) {
            // Skip if file hasn't changed and was previously identified as non-TikTok
            cached_info.size == current_size && 
            cached_info.modified == current_modified &&
            !cached_info.is_tiktok
        } else {
            false
        }
    }

    /// Adds a file to the cache with its analysis results
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file
    /// * `size` - File size in bytes
    /// * `modified` - File modification time
    /// * `confidence` - TikTok detection confidence score
    /// * `is_tiktok` - Whether the file was identified as TikTok content
    pub fn add_file_with_metadata(&mut self, file_path: &str, size: u64, modified: String, confidence: u32, is_tiktok: bool) {
        let file_info = FileInfo {
            size,
            modified,
            confidence,
            is_tiktok,
        };
        
        self.file_metadata.insert(file_path.to_string(), file_info);
        
        // Only add to scanned_files if it's not TikTok content (for backward compatibility)
        if !is_tiktok && !self.contains(file_path) {
            self.scanned_files.push(file_path.to_string());
        }
        
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// Legacy method for backward compatibility
    pub fn add_file(&mut self, file_path: &str) {
        if !self.contains(file_path) {
            self.scanned_files.push(file_path.to_string());
            self.last_updated = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Loads cache from a JSON file with version compatibility checking
    /// 
    /// # Arguments
    /// * `json_path` - Path to the cache JSON file
    /// 
    /// # Returns
    /// `Result<Self>` - The loaded cache or a new cache if loading fails
    pub fn load_from_file(json_path: &Path) -> Result<Self> {
        if json_path.exists() {
            let content = fs::read_to_string(json_path)
                .context("Failed to read cache file")?;
            
            // Try to parse as new format first
            match serde_json::from_str::<NotTikTokCache>(&content) {
                Ok(mut cache) => {
                    // Migrate old cache format if needed
                    if cache.cache_version.is_empty() {
                        cache.cache_version = "2.0".to_string();
                        println!("📱 Migrated cache to version 2.0");
                    }
                    Ok(cache)
                },
                Err(_) => {
                    // Try legacy format
                    #[derive(Deserialize)]
                    struct LegacyCache {
                        scanned_files: Vec<String>,
                        last_updated: String,
                    }
                    
                    match serde_json::from_str::<LegacyCache>(&content) {
                        Ok(legacy) => {
                            println!("📱 Converting legacy cache format...");
                            Ok(NotTikTokCache {
                                scanned_files: legacy.scanned_files,
                                last_updated: legacy.last_updated,
                                file_metadata: HashMap::new(),
                                cache_version: "2.0".to_string(),
                            })
                        },
                        Err(e) => {
                            println!("⚠️  Cache file corrupted, creating new cache: {}", e);
                            Ok(NotTikTokCache::new())
                        }
                    }
                }
            }
        } else {
            Ok(NotTikTokCache::new())
        }
    }

    pub fn save_to_file(&self, json_path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize cache")?;
        fs::write(json_path, content)
            .context("Failed to write not_tiktok.json")?;
        Ok(())
    }
}

/// File management system for TikTok detection and organization
/// 
/// This struct handles all file operations including:
/// - Creating organization folders on different filesystems
/// - Managing cache for performance optimization
/// - Moving/copying files based on detection confidence
/// - Handling MTP/phone filesystem limitations
pub struct FileManager {
    /// Base path where files are being scanned
    base_path: PathBuf,
    /// Path to the TikTok detection organization folder
    tiktok_folder: PathBuf,
    /// Path to the cache file
    cache_file: PathBuf,
    /// In-memory cache for file analysis results
    cache: NotTikTokCache,
}

impl FileManager {
    /// Creates a new FileManager instance with automatic filesystem detection
    /// 
    /// This method automatically detects whether the target is a phone/MTP filesystem
    /// and adjusts the organization strategy accordingly:
    /// - For phones: Creates folders on device, cache locally
    /// - For regular filesystems: Creates everything in scan directory
    /// 
    /// # Arguments
    /// * `base_path` - The directory to scan and organize
    /// 
    /// # Returns
    /// `Result<Self>` - A configured FileManager instance
    /// 
    /// # Errors
    /// Returns error if folder creation fails or cache cannot be loaded
    pub fn new(base_path: &Path) -> Result<Self> {
        let base_path = base_path.to_path_buf();
        
        // Check if we're dealing with an MTP/phone filesystem
        let is_mtp_path = base_path.to_string_lossy().contains("gvfs/mtp") || 
                          base_path.to_string_lossy().contains("run/user");
        
        let (tiktok_folder, cache_file) = if is_mtp_path {
            // For MTP/phone paths, try to create folder on phone first, fallback to local
            let phone_tiktok_folder = base_path.join("tiktok_detection");
            
            // Try to create the folder on the phone
            match fs::create_dir_all(&phone_tiktok_folder) {
                Ok(_) => {
                    println!("✅ Created folder on phone: {}", phone_tiktok_folder.display());
                    
                    // Create subfolders for confidence levels
                    let _ = fs::create_dir_all(&phone_tiktok_folder.join("confirmed"));
                    let _ = fs::create_dir_all(&phone_tiktok_folder.join("likely"));
                    let _ = fs::create_dir_all(&phone_tiktok_folder.join("possible"));
                    let _ = fs::create_dir_all(&phone_tiktok_folder.join("unlikely"));
                    
                    // Cache file goes to local temp since phone can't write files
                    let local_cache = std::env::temp_dir().join("tiktok_phone_cache.json");
                    println!("📱 Using local cache for phone scan: {}", local_cache.display());
                    
                    (phone_tiktok_folder, local_cache)
                },
                Err(_) => {
                    // Fallback to local temp folder
                    println!("📱 Phone filesystem doesn't support folder creation, using local organization");
                    let local_temp = std::env::temp_dir().join("tiktok_detection");
                    let cache_file = local_temp.join("not_tiktok.json");
                    
                    if !local_temp.exists() {
                        fs::create_dir_all(&local_temp)
                            .context("Failed to create local temp folder")?;
                        println!("✅ Created local organization folder: {}", local_temp.display());
                    }
                    
                    (local_temp, cache_file)
                }
            }
        } else {
            // For regular filesystems, create folder in the scan directory
            let tiktok_folder = base_path.join("tiktok_detection");
            let cache_file = tiktok_folder.join("not_tiktok.json");
            
            if !tiktok_folder.exists() {
                fs::create_dir_all(&tiktok_folder)
                    .context("Failed to create tiktok_detection folder")?;
                println!("✅ Created folder: {}", tiktok_folder.display());
            }
            
            (tiktok_folder, cache_file)
        };

        // Load or create cache
        let cache = NotTikTokCache::load_from_file(&cache_file).unwrap_or_else(|_| {
            NotTikTokCache::new()
        });

        Ok(FileManager {
            base_path,
            tiktok_folder,
            cache_file,
            cache,
        })
    }

    pub fn move_file_to_tiktok_folder(&mut self, source_path: &Path, confidence: u32) -> Result<PathBuf> {
        let filename = source_path.file_name()
            .context("Invalid file name")?;
        
        // Create confidence-based subfolder
        let subfolder = match confidence {
            70.. => "confirmed",
            40..=69 => "likely", 
            20..=39 => "possible",
            _ => "unlikely",
        };

        let target_folder = self.tiktok_folder.join(subfolder);
        if !target_folder.exists() {
            fs::create_dir_all(&target_folder)
                .context("Failed to create confidence subfolder")?;
        }

        let target_path = target_folder.join(filename);
        
        // Handle file name conflicts
        let final_target = self.resolve_filename_conflict(&target_path)?;

        // Try to move the file, but handle MTP/phone filesystem errors gracefully
        match fs::rename(source_path, &final_target) {
            Ok(_) => {
                println!("📁 Moved {} file: {} -> {}", 
                         subfolder.to_uppercase(), 
                         source_path.display(), 
                         final_target.display());
            },
            Err(_) => {
                // For phone/MTP filesystems, try copying instead
                match fs::copy(source_path, &final_target) {
                    Ok(_) => {
                        println!("� Copied {} file: {} -> {} (Phone filesystem: move not supported)", 
                                 subfolder.to_uppercase(), 
                                 source_path.display(), 
                                 final_target.display());
                    },
                    Err(_) => {
                        // If both fail, create detection record in local temp folder
                        println!("📱 Phone filesystem: Creating local detection record");
                        
                        let local_detection_folder = std::env::temp_dir().join("tiktok_detection_results").join(subfolder);
                        if let Err(_) = fs::create_dir_all(&local_detection_folder) {
                            println!("📱 Detected {} TikTok file: {} ({}% confidence)", 
                                     subfolder.to_uppercase(), 
                                     source_path.file_name().unwrap().to_string_lossy(),
                                     confidence);
                            return Ok(final_target);
                        }
                        
                        let detection_info = format!(
                            "TikTok Detection Result (MOVE MODE)\n===================================\n\nOriginal file: {}\nConfidence: {}/100\nCategory: {}\nDetected: {}\n\nNote: File could not be moved due to phone filesystem limitations.\nTo organize this file manually:\n1. Create folder: {}\n2. Move file: {} -> {}/{}\n",
                            source_path.display(),
                            confidence,
                            subfolder.to_uppercase(),
                            chrono::Utc::now().to_rfc3339(),
                            target_folder.display(),
                            source_path.display(),
                            target_folder.display(),
                            filename.to_string_lossy()
                        );
                        
                        let info_filename = format!("{}_{}.txt", 
                                                   source_path.file_stem().unwrap().to_string_lossy(),
                                                   subfolder);
                        let local_info_file = local_detection_folder.join(info_filename);
                        
                        if let Ok(_) = fs::write(&local_info_file, detection_info) {
                            println!("📝 Move instructions saved to: {}", local_info_file.display());
                        } else {
                            println!("📱 Detected {} TikTok file: {} ({}% confidence)", 
                                     subfolder.to_uppercase(), 
                                     source_path.file_name().unwrap().to_string_lossy(),
                                     confidence);
                        }
                    }
                }
            }
        }

        Ok(final_target)
    }

    /// Adds analysis results to the cache with full metadata
    /// 
    /// # Arguments
    /// * `file_path` - Path to the analyzed file
    /// * `confidence` - Detection confidence score (0-100+)
    /// * `is_tiktok` - Whether file was identified as TikTok content
    pub fn add_to_cache(&mut self, file_path: &Path, confidence: u32, is_tiktok: bool) -> Result<()> {
        let path_str = file_path.to_string_lossy().to_string();
        
        // Get file metadata
        let metadata = fs::metadata(file_path).context("Failed to read file metadata")?;
        let size = metadata.len();
        let modified = metadata.modified()
            .context("Failed to get modification time")?
            .duration_since(std::time::UNIX_EPOCH)
            .context("Invalid modification time")?;
        let modified_str = chrono::DateTime::<chrono::Utc>::from(
            std::time::UNIX_EPOCH + modified
        ).to_rfc3339();

        self.cache.add_file_with_metadata(&path_str, size, modified_str, confidence, is_tiktok);
        
        // Try to save cache, but don't fail the operation if it's not possible
        let _ = self.save_cache();
        Ok(())
    }

    /// Checks if a file should be skipped based on cache
    /// 
    /// # Arguments
    /// * `file_path` - Path to check
    /// 
    /// # Returns
    /// `(should_skip, cached_confidence)` - Whether to skip and previous confidence if available
    pub fn should_skip_file(&self, file_path: &Path) -> Result<(bool, Option<u32>)> {
        let metadata = fs::metadata(file_path).context("Failed to read file metadata")?;
        let size = metadata.len();
        let modified = metadata.modified()
            .context("Failed to get modification time")?
            .duration_since(std::time::UNIX_EPOCH)
            .context("Invalid modification time")?;
        let modified_str = chrono::DateTime::<chrono::Utc>::from(
            std::time::UNIX_EPOCH + modified
        ).to_rfc3339();

        let should_skip = self.cache.should_skip_file(file_path, size, &modified_str);
        
        if should_skip {
            // Get cached confidence if available
            let path_str = file_path.to_string_lossy().to_string();
            let cached_confidence = self.cache.file_metadata.get(&path_str)
                .map(|info| info.confidence);
            Ok((true, cached_confidence))
        } else {
            Ok((false, None))
        }
    }

    pub fn is_in_not_tiktok_cache(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy().to_string();
        self.cache.contains(&path_str)
    }

    pub fn save_cache(&self) -> Result<()> {
        // Try to save cache, but don't fail if it's not possible (e.g., on phone filesystem)
        match self.cache.save_to_file(&self.cache_file) {
            Ok(_) => Ok(()),
            Err(_) => {
                // Silently handle cache save failures for phone filesystems
                Ok(())
            }
        }
    }

    pub fn get_cache_stats(&self) -> (usize, &str) {
        (self.cache.scanned_files.len(), &self.cache.last_updated)
    }

    fn resolve_filename_conflict(&self, target_path: &Path) -> Result<PathBuf> {
        if !target_path.exists() {
            return Ok(target_path.to_path_buf());
        }

        let parent = target_path.parent().unwrap();
        let stem = target_path.file_stem().unwrap().to_string_lossy();
        let extension = target_path.extension()
            .map(|ext| format!(".{}", ext.to_string_lossy()))
            .unwrap_or_default();

        for i in 1..1000 {
            let new_filename = format!("{}_{}{}", stem, i, extension);
            let new_path = parent.join(new_filename);
            if !new_path.exists() {
                return Ok(new_path);
            }
        }

        Err(anyhow::anyhow!("Could not resolve filename conflict"))
    }

    pub fn copy_file_to_tiktok_folder(&self, source_path: &Path, confidence: u32) -> Result<PathBuf> {
        let filename = source_path.file_name()
            .context("Invalid file name")?;
        
        let subfolder = match confidence {
            70.. => "confirmed",
            40..=69 => "likely", 
            20..=39 => "possible",
            _ => "unlikely",
        };

        let target_folder = self.tiktok_folder.join(subfolder);
        if !target_folder.exists() {
            fs::create_dir_all(&target_folder)
                .context("Failed to create confidence subfolder")?;
        }

        let target_path = target_folder.join(filename);
        let final_target = self.resolve_filename_conflict(&target_path)?;

        // Try to copy the file, but handle MTP/phone filesystem errors gracefully
        match fs::copy(source_path, &final_target) {
            Ok(_) => {
                println!("📋 Copied {} file: {} -> {}", 
                         subfolder.to_uppercase(), 
                         source_path.display(), 
                         final_target.display());
            },
            Err(e) => {
                // For phone/MTP filesystems, copying might not work
                println!("� Would copy {} file: {} -> {} (Phone filesystem: copy not supported)", 
                         subfolder.to_uppercase(), 
                         source_path.display(), 
                         final_target.display());
                
                // Create a placeholder file with the detection info
                let detection_info = format!(
                    "Original file: {}\nConfidence: {}/100\nCategory: {}\nDetected: {}\n",
                    source_path.display(),
                    confidence,
                    subfolder.to_uppercase(),
                    chrono::Utc::now().to_rfc3339()
                );
                
                let info_file = final_target.with_extension("detection_info.txt");
                fs::write(&info_file, detection_info)
                    .context("Failed to write detection info")?;
            }
        }

        Ok(final_target)
    }

    pub fn get_tiktok_folder(&self) -> &Path {
        &self.tiktok_folder
    }

    pub fn get_base_path(&self) -> &Path {
        &self.base_path
    }

    pub fn create_move_script(&self, moves: &[(String, String, u32)]) -> Result<()> {
        let script_path = std::env::temp_dir().join("move_tiktok_files.sh");
        let mut script_content = String::new();
        
        script_content.push_str("#!/bin/bash\n");
        script_content.push_str("# TikTok File Organization Script\n");
        script_content.push_str("# Generated by tiktok-cleaner\n\n");
        script_content.push_str("echo \"🚀 Moving TikTok files to organized folders...\"\n\n");
        
        for (source, confidence_level, confidence_score) in moves {
            let filename = std::path::Path::new(source).file_name()
                .unwrap_or_default().to_string_lossy();
            
            script_content.push_str(&format!(
                "echo \"📁 Moving {} file: {} ({}% confidence)\"\n",
                confidence_level.to_uppercase(),
                filename,
                confidence_score
            ));
            
            script_content.push_str(&format!(
                "mv \"{}\" \"{}/tiktok_detection/{}/\"\n",
                source,
                self.base_path.display(),
                confidence_level
            ));
            
            script_content.push_str("\n");
        }
        
        script_content.push_str("echo \"✅ File organization complete!\"\n");
        script_content.push_str("echo \"📁 Check the tiktok_detection folder for organized files\"\n");
        
        fs::write(&script_path, script_content)
            .context("Failed to create move script")?;
        
        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }
        
        println!("📜 Created move script: {}", script_path.display());
        println!("   Run it with: bash {}", script_path.display());
        
        Ok(())
    }

    /// Creates a manual organization guide for phone filesystems
    pub fn create_phone_organization_guide(&self, detected_files: &[(String, u32)]) -> Result<()> {
        let guide_path = std::env::temp_dir().join("tiktok_phone_organization_guide.md");
        let mut guide_content = String::new();
        
        guide_content.push_str("# TikTok Files Organization Guide for Phone\n\n");
        guide_content.push_str("Due to phone filesystem limitations, files could not be automatically moved.\n");
        guide_content.push_str("Please follow these steps to manually organize your TikTok files:\n\n");
        
        guide_content.push_str("## Detected TikTok Files\n\n");
        
        let mut confirmed_files = Vec::new();
        let mut likely_files = Vec::new();
        let mut possible_files = Vec::new();
        
        for (file_path, confidence) in detected_files {
            let filename = std::path::Path::new(file_path).file_name()
                .unwrap_or_default().to_string_lossy();
            
            match *confidence {
                70.. => confirmed_files.push((filename.to_string(), confidence)),
                40..=69 => likely_files.push((filename.to_string(), confidence)),
                20..=39 => possible_files.push((filename.to_string(), confidence)),
                _ => {} // Skip unlikely files
            }
        }
        
        if !confirmed_files.is_empty() {
            guide_content.push_str("### 🔴 Confirmed TikTok Files (70%+ confidence)\n");
            guide_content.push_str("These files are almost certainly from TikTok:\n\n");
            for (filename, confidence) in confirmed_files {
                guide_content.push_str(&format!("- `{}` ({}% confidence)\n", filename, confidence));
            }
            guide_content.push_str("\n");
        }
        
        if !likely_files.is_empty() {
            guide_content.push_str("### 🟡 Likely TikTok Files (40-69% confidence)\n");
            guide_content.push_str("These files are probably from TikTok:\n\n");
            for (filename, confidence) in likely_files {
                guide_content.push_str(&format!("- `{}` ({}% confidence)\n", filename, confidence));
            }
            guide_content.push_str("\n");
        }
        
        if !possible_files.is_empty() {
            guide_content.push_str("### 🔵 Possible TikTok Files (20-39% confidence)\n");
            guide_content.push_str("These files might be from TikTok:\n\n");
            for (filename, confidence) in possible_files {
                guide_content.push_str(&format!("- `{}` ({}% confidence)\n", filename, confidence));
            }
            guide_content.push_str("\n");
        }
        
        guide_content.push_str("## Manual Organization Steps\n\n");
        guide_content.push_str("1. **On your phone**, navigate to your file manager\n");
        guide_content.push_str("2. **Go to the scan folder**: ");
        guide_content.push_str(&format!("`{}`\n", self.base_path.display()));
        guide_content.push_str("3. **Create organization folders** (if they don't exist):\n");
        guide_content.push_str("   - `tiktok_detection/confirmed/`\n");
        guide_content.push_str("   - `tiktok_detection/likely/`\n");
        guide_content.push_str("   - `tiktok_detection/possible/`\n");
        guide_content.push_str("4. **Move files** to appropriate folders based on confidence levels above\n\n");
        
        guide_content.push_str("## Alternative: Use Android File Manager Apps\n\n");
        guide_content.push_str("Some file manager apps handle MTP better:\n");
        guide_content.push_str("- **Total Commander** (with MTP plugin)\n");
        guide_content.push_str("- **Solid Explorer**\n");
        guide_content.push_str("- **FX File Explorer**\n\n");
        
        guide_content.push_str("## Detection Summary\n\n");
        guide_content.push_str(&format!("- **Scan folder**: `{}`\n", self.base_path.display()));
        guide_content.push_str(&format!("- **Target folder**: `{}`\n", self.tiktok_folder.display()));
        guide_content.push_str(&format!("- **Generated**: {}\n", chrono::Utc::now().to_rfc3339()));
        
        fs::write(&guide_path, guide_content)
            .context("Failed to create organization guide")?;
        
        println!("📖 Created organization guide: {}", guide_path.display());
        println!("   Open this file for step-by-step manual organization instructions");
        
        Ok(())
    }
}
