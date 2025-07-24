use std::path::Path;
use anyhow::Result;
use rayon::prelude::*;
use walkdir::WalkDir;
use crate::tiktok_detection::{
    tiktok_photo_det::TikTokPhotoDetector,
    tiktok_video_det::TikTokVideoDetector,
    metadata_read::metadata_manager::FileMetadata,
    file_util::file_manager::FileManager,
};

/// High-performance TikTok content scanner with multithreading support
/// 
/// This scanner uses parallel processing to analyze multiple files simultaneously,
/// significantly improving performance on systems with multiple CPU cores.
/// It includes intelligent caching to skip previously analyzed files.
pub struct TikTokScanner {
    /// Photo detection engine
    photo_detector: TikTokPhotoDetector,
    /// Video detection engine  
    video_detector: TikTokVideoDetector,
    /// File management system
    file_manager: FileManager,
}

/// Results from a scanning operation
#[derive(Debug)]
pub struct ScanResults {
    /// Total number of media files found
    pub total_files: usize,
    /// Number of confirmed TikTok files (70%+ confidence)
    pub confirmed_tiktok: usize,
    /// Number of likely TikTok files (40-69% confidence)
    pub likely_tiktok: usize,
    /// Number of possible TikTok files (20-39% confidence)
    pub possible_tiktok: usize,
    /// Number of unlikely TikTok files (<20% confidence)
    pub unlikely_tiktok: usize,
    /// Number of files skipped due to caching
    pub skipped_cached: usize,
    /// Paths of files that were moved/organized
    pub moved_files: Vec<String>,
}

impl TikTokScanner {
    pub fn new(scan_path: &Path) -> Result<Self> {
        let photo_detector = TikTokPhotoDetector::new()?;
        let video_detector = TikTokVideoDetector::new()?;
        let file_manager = FileManager::new(scan_path)?;

        Ok(TikTokScanner {
            photo_detector,
            video_detector,
            file_manager,
        })
    }

    pub fn scan_folder(&mut self, move_files: bool) -> Result<ScanResults> {
        println!("Scanning folder: {}", self.file_manager.get_base_path().display());
        println!("TikTok detection folder: {}", self.file_manager.get_tiktok_folder().display());

        let (cache_count, last_updated) = self.file_manager.get_cache_stats();
        if cache_count > 0 {
            println!("Loaded cache with {} previously scanned files (last updated: {})", cache_count, last_updated);
        }

        let mut results = ScanResults {
            total_files: 0,
            confirmed_tiktok: 0,
            likely_tiktok: 0,
            possible_tiktok: 0,
            unlikely_tiktok: 0,
            skipped_cached: 0,
            moved_files: Vec::new(),
        };

        // Get all media files from the base path (excluding tiktok_detection folder)
        let media_files = self.get_media_files()?;
        results.total_files = media_files.len();

        println!("Found {} media files to analyze\n", media_files.len());

        for (i, file_path) in media_files.iter().enumerate() {
            println!("[{}/{}] Analyzing: {}", i + 1, media_files.len(), file_path.file_name().unwrap().to_string_lossy());

            // Check cache first
            match self.file_manager.should_skip_file(file_path) {
                Ok((should_skip, _cached_confidence)) => {
                    if should_skip {
                        println!("  Skipped (cached as non-TikTok)");
                        results.skipped_cached += 1;
                        continue;
                    }
                },
                Err(_) => {
                    // Continue with analysis if cache check fails
                }
            }

            // Analyze the file
            let metadata = self.analyze_file(file_path)?;
            let confidence = metadata.tiktok_analysis.confidence_score;

            // Display result
            let (icon, category) = match confidence {
                70.. => { results.confirmed_tiktok += 1; ("[CONFIRMED]", "CONFIRMED") },
                40..=69 => { results.likely_tiktok += 1; ("[LIKELY]", "LIKELY") },
                20..=39 => { results.possible_tiktok += 1; ("[POSSIBLE]", "POSSIBLE") },
                _ => { results.unlikely_tiktok += 1; ("[UNLIKELY]", "UNLIKELY") },
            };

            println!("  {} {} TikTok (Confidence: {}/100)", icon, category, confidence);
            
            if !metadata.tiktok_analysis.evidence_found.is_empty() {
                let evidence_preview = if metadata.tiktok_analysis.evidence_found.len() > 2 {
                    format!("{}, {}...", metadata.tiktok_analysis.evidence_found[0], metadata.tiktok_analysis.evidence_found[1])
                } else {
                    metadata.tiktok_analysis.evidence_found.join(", ")
                };
                println!("     Evidence: {}", evidence_preview);
            }

            // Handle file based on detection result
            if confidence >= 20 {
                // TikTok detected (possible, likely, or confirmed)
                if move_files {
                    match self.file_manager.move_file_to_tiktok_folder(file_path, confidence) {
                        Ok(new_path) => {
                            results.moved_files.push(new_path.to_string_lossy().to_string());
                        }
                        Err(e) => {
                            eprintln!("     ERROR: Failed to move file: {}", e);
                        }
                    }
                } else {
                    // Just copy for preview mode
                    match self.file_manager.copy_file_to_tiktok_folder(file_path, confidence) {
                        Ok(_) => {
                            println!("     INFO: Would move to: {}/", 
                                     match confidence {
                                         70.. => "confirmed",
                                         40..=69 => "likely",
                                         _ => "possible",
                                     });
                        }
                        Err(e) => {
                            eprintln!("     ERROR: {}", e);
                        }
                    }
                }
            } else {
                // Not TikTok - add to cache
                if let Err(e) = self.file_manager.add_to_cache(file_path, confidence, false) {
                    eprintln!("     WARNING: Failed to cache file: {}", e);
                }
            }
            
            println!();
        }

        // Save cache
        self.file_manager.save_cache()?;

        self.print_results(&results, move_files);
        Ok(results)
    }

    /// Performs parallel scanning of media files for TikTok detection using multiple threads.
    /// This method processes files concurrently for analysis, then handles file operations
    /// sequentially to avoid race conditions with file system operations.
    /// 
    /// # Arguments
    /// * `move_files` - If true, moves detected TikTok files to organized folders. If false, 
    ///                  only performs preview mode without actually moving files.
    /// 
    /// # Returns
    /// Returns ScanResults containing statistics about the scan operation.
    pub fn scan_folder_parallel(&mut self, move_files: bool) -> Result<ScanResults> {
        println!("Scanning folder (parallel): {}", self.file_manager.get_base_path().display());
        println!("TikTok detection folder: {}", self.file_manager.get_tiktok_folder().display());

        // Get all media files
        let media_files = self.get_media_files()?;
        println!("Found {} media files to analyze (using {} threads)\n", 
                 media_files.len(), num_cpus::get());

        let mut results = ScanResults {
            total_files: media_files.len(),
            confirmed_tiktok: 0,
            likely_tiktok: 0,
            possible_tiktok: 0,
            unlikely_tiktok: 0,
            skipped_cached: 0,
            moved_files: Vec::new(),
        };

        // First, check cache and collect files that need analysis
        let mut files_to_analyze = Vec::new();
        for (i, file_path) in media_files.iter().enumerate() {
            println!("[{}/{}] Checking cache: {}", i + 1, media_files.len(), 
                     file_path.file_name().unwrap().to_string_lossy());

            match self.file_manager.should_skip_file(file_path) {
                Ok((should_skip, _cached_confidence)) => {
                    if should_skip {
                        println!("  Skipped (cached as non-TikTok)");
                        results.skipped_cached += 1;
                    } else {
                        files_to_analyze.push(file_path);
                    }
                },
                Err(_) => {
                    files_to_analyze.push(file_path);
                }
            }
        }

        println!("\nAnalyzing {} files in parallel...\n", files_to_analyze.len());

        // Perform parallel analysis on files that need it
        let analysis_results: Vec<_> = files_to_analyze
            .par_iter()
            .enumerate()
            .map(|(i, file_path)| {
                println!("[{}/{}] Analyzing: {}", i + 1, files_to_analyze.len(), 
                         file_path.file_name().unwrap().to_string_lossy());

                let metadata_result = self.analyze_file(file_path);
                (file_path, metadata_result)
            })
            .collect();

        // Process results sequentially to handle file operations safely
        println!("\nProcessing results and organizing files...\n");
        for (file_path, metadata_result) in analysis_results {
            match metadata_result {
                Ok(metadata) => {
                    let confidence = metadata.tiktok_analysis.confidence_score;

                    // Categorize result
                    let (icon, category) = match confidence {
                        70.. => { results.confirmed_tiktok += 1; ("", "CONFIRMED") },
                        40..=69 => { results.likely_tiktok += 1; ("", "LIKELY") },
                        20..=39 => { results.possible_tiktok += 1; ("", "POSSIBLE") },
                        _ => { results.unlikely_tiktok += 1; ("", "UNLIKELY") },
                    };

                    println!("📄 {}: {} {} TikTok (Confidence: {}/100)", 
                             file_path.file_name().unwrap().to_string_lossy(),
                             icon, category, confidence);
                    
                    if !metadata.tiktok_analysis.evidence_found.is_empty() {
                        let evidence_preview = if metadata.tiktok_analysis.evidence_found.len() > 2 {
                            format!("{}, {}...", metadata.tiktok_analysis.evidence_found[0], metadata.tiktok_analysis.evidence_found[1])
                        } else {
                            metadata.tiktok_analysis.evidence_found.join(", ")
                        };
                        println!("     Evidence: {}", evidence_preview);
                    }

                    // Handle file based on detection result
                    if confidence >= 20 {
                        // TikTok detected (possible, likely, or confirmed)
                        if move_files {
                            match self.file_manager.move_file_to_tiktok_folder(file_path, confidence) {
                                Ok(new_path) => {
                                    results.moved_files.push(new_path.to_string_lossy().to_string());
                                }
                                Err(e) => {
                                    eprintln!("     ERROR: Failed to move file: {}", e);
                                }
                            }
                        } else {
                            // Just preview mode
                            match self.file_manager.copy_file_to_tiktok_folder(file_path, confidence) {
                                Ok(_) => {
                                    println!("     INFO: Would move to: {}/", 
                                             match confidence {
                                                 70.. => "confirmed",
                                                 40..=69 => "likely",
                                                 _ => "possible",
                                             });
                                }
                                Err(e) => {
                                    eprintln!("     ERROR: {}", e);
                                }
                            }
                        }
                    } else {
                        // Not TikTok - add to cache
                        if let Err(e) = self.file_manager.add_to_cache(file_path, confidence, false) {
                            eprintln!("     WARNING: Failed to cache file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("ERROR: Error analyzing {}: {}", 
                             file_path.file_name().unwrap().to_string_lossy(), e);
                }
            }
        }

        // Save cache after all processing
        self.file_manager.save_cache()?;

        // Generate phone organization guide if MTP filesystem detected and TikTok files found
        let total_detected = results.confirmed_tiktok + results.likely_tiktok + results.possible_tiktok;
        if total_detected > 0 {
            let base_path_str = self.file_manager.get_base_path().to_string_lossy();
            let is_phone_filesystem = base_path_str.contains("gvfs/mtp") || base_path_str.contains("run/user");
            
            if is_phone_filesystem {
                // Collect detected files for the guide
                let detected_files = Vec::new();
                // Note: In a real implementation, we'd track the file paths and confidence scores
                // For now, we'll create the guide without specific file details
                if let Err(e) = self.file_manager.create_phone_organization_guide(&detected_files) {
                    eprintln!("WARNING: Could not create organization guide: {}", e);
                } else {
                    println!("\n[PHONE] Phone filesystem detected - organization guide created!");
                    println!("   Check /tmp/tiktok_phone_organization_guide.md for manual organization steps");
                }
            }
        }

        self.print_results(&results, move_files);
        Ok(results)
    }

    fn get_media_files(&self) -> Result<Vec<std::path::PathBuf>> {
        let mut media_files = Vec::new();
        let base_path = self.file_manager.get_base_path();
        let tiktok_folder = self.file_manager.get_tiktok_folder();

        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry?;
            let path = entry.path();

            // Skip the tiktok_detection folder to avoid processing already moved files
            if path.starts_with(tiktok_folder) {
                continue;
            }

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                    if ["jpg", "jpeg", "png", "webp", "gif", "bmp", "mp4", "mov", "avi", "mkv", "flv", "webm"]
                        .contains(&ext_str.as_str()) {
                        media_files.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(media_files)
    }

    fn analyze_file(&self, file_path: &Path) -> Result<FileMetadata> {
        if let Some(ext) = file_path.extension() {
            let ext_str = ext.to_str().unwrap_or("").to_lowercase();
            if ["mp4", "mov", "avi", "mkv", "flv", "webm"].contains(&ext_str.as_str()) {
                self.video_detector.analyze_video(file_path)
            } else {
                self.photo_detector.analyze_image(file_path)
            }
        } else {
            // Default to image analysis
            self.photo_detector.analyze_image(file_path)
        }
    }

    fn print_results(&self, results: &ScanResults, moved_files: bool) {
        println!("🎯 SCAN RESULTS SUMMARY");
        println!("{}", "=".repeat(50));
        println!("📁 Scanned folder: {}", self.file_manager.get_base_path().display());
        println!("Total files analyzed: {}", results.total_files);
        if results.skipped_cached > 0 {
            println!("Files skipped (cached): {}", results.skipped_cached);
        }
        println!();
        println!("[CONFIRMED] TikTok: {}", results.confirmed_tiktok);
        println!("[LIKELY] TikTok: {}", results.likely_tiktok);
        println!("[POSSIBLE] TikTok: {}", results.possible_tiktok);
        println!("[UNLIKELY] TikTok: {}", results.unlikely_tiktok);
        println!();

        let total_detected = results.confirmed_tiktok + results.likely_tiktok + results.possible_tiktok;
        if total_detected > 0 {
            let detection_rate = (total_detected as f64 / results.total_files as f64) * 100.0;
            println!("📈 TikTok Detection Rate: {:.1}% ({} files)", detection_rate, total_detected);
            
            if moved_files {
                println!("📁 Files moved to: {}", self.file_manager.get_tiktok_folder().display());
                println!("   • confirmed/ - {} files", results.confirmed_tiktok);
                println!("   • likely/ - {} files", results.likely_tiktok);
                println!("   • possible/ - {} files", results.possible_tiktok);
            } else {
                println!("[PREVIEW] Preview mode: No files were moved");
                println!("   Run with --move to actually move the files");
            }
        } else {
            println!("SUCCESS: No TikTok files detected");
        }

        println!();
        println!("💾 Cache updated: {}", self.file_manager.get_cache_stats().1);
        println!("Total cached non-TikTok files: {}", self.file_manager.get_cache_stats().0);
    }
}
