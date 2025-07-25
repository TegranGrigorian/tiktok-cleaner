use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use regex::Regex;
use image::GenericImageView;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokEvidence {
    pub is_tiktok: bool,
    pub confidence_score: u32,
    pub evidence_found: Vec<String>,
    pub indicators: HashMap<String, String>,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub filename: String,
    pub filepath: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub md5_hash: Option<String>,
    pub dimensions: Option<(u32, u32)>,
    pub aspect_ratio: Option<f64>,
    pub file_format: Option<String>,
    pub strings_found: Vec<String>,
    pub tiktok_analysis: TikTokEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageIndicators {
    pub dimensions: Option<String>,
    pub aspect_ratio: Option<f64>,
    pub file_format: Option<String>,
    pub file_size: String,
    pub filename_pattern: String,
    pub tiktok_characteristics: Vec<String>,
}

pub struct MetadataManager {
    tiktok_dimensions: Vec<(u32, u32)>,
    tiktok_video_id_regex: Regex,
    string_indicators: Vec<String>,
}

impl MetadataManager {
    pub fn new() -> Result<Self> {
        let tiktok_dimensions = vec![
            (576, 1024), (576, 1246), (576, 1280),
            (1080, 1920), (1080, 1800), (1080, 2340), (1080, 2400),
            (828, 1792), (750, 1334), (1125, 2436), (1242, 2688),
            (1284, 2778), (1170, 2532),
        ];

        let tiktok_video_id_regex = Regex::new(r"vid:v\d+g[fl]0000[a-f0-9]+")
            .context("Failed to compile TikTok video ID regex")?;

        let string_indicators = vec![
            "tiktok".to_string(),
            "douyin".to_string(), 
            "bytedance".to_string(),
            "musically".to_string(),
            "musical.ly".to_string(),
            "aigc_label_type".to_string(),
            "vid_md5".to_string(),
        ];

        Ok(MetadataManager {
            tiktok_dimensions,
            tiktok_video_id_regex,
            string_indicators,
        })
    }

    pub fn analyze_file(&self, filepath: &Path) -> Result<FileMetadata> {
        let filename = filepath.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let metadata = fs::metadata(filepath)
            .context("Failed to read file metadata")?;

        let size_bytes = metadata.len();
        let size_human = format_bytes(size_bytes);

        // Calculate MD5 hash
        let md5_hash = self.calculate_md5(filepath).ok();

        // Try to get image dimensions
        let (dimensions, file_format) = self.get_image_info(filepath);

        // Calculate aspect ratio
        let aspect_ratio = dimensions.map(|(w, h)| w as f64 / h as f64);

        // Search for strings in file
        let strings_found = self.search_strings_in_file(filepath);

        // Analyze for TikTok evidence
        let tiktok_analysis = self.analyze_tiktok_evidence(&filename, &dimensions, &aspect_ratio, &strings_found, &file_format);

        Ok(FileMetadata {
            filename,
            filepath: filepath.to_string_lossy().to_string(),
            size_bytes,
            size_human,
            md5_hash,
            dimensions,
            aspect_ratio,
            file_format,
            strings_found,
            tiktok_analysis,
        })
    }

    fn calculate_md5(&self, filepath: &Path) -> Result<String> {
        let data = fs::read(filepath)?;
        let digest = md5::compute(&data);
        Ok(format!("{:x}", digest))
    }

    fn get_image_info(&self, filepath: &Path) -> (Option<(u32, u32)>, Option<String>) {
        // First detect the actual file format by reading file header
        let actual_format = self.detect_file_format(filepath);
        
        // Try imagesize crate first (handles more formats reliably)
        if let Ok(size) = imagesize::size(filepath) {
            return (Some((size.width as u32, size.height as u32)), actual_format);
        }
        
        // Fallback to image crate
        if let Ok(img) = image::open(filepath) {
            let (width, height) = img.dimensions();
            let detected_format = format!("{:?}", img.color());
            (Some((width, height)), actual_format.or(Some(detected_format)))
        } else {
            // Last resort: extension-based detection
            let extension_format = filepath.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_uppercase());
            (None, actual_format.or(extension_format))
        }
    }

    fn detect_file_format(&self, filepath: &Path) -> Option<String> {
        use std::io::Read;
        
        if let Ok(mut file) = std::fs::File::open(filepath) {
            let mut header = [0u8; 16];
            if file.read(&mut header).is_ok() {
                // Check for WebP signature
                if header[0..4] == [0x52, 0x49, 0x46, 0x46] && header[8..12] == [0x57, 0x45, 0x42, 0x50] {
                    return Some("WebP".to_string());
                }
                // Check for PNG signature
                if header[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
                    return Some("PNG".to_string());
                }
                // Check for JPEG signature
                if header[0..2] == [0xFF, 0xD8] {
                    return Some("JPEG".to_string());
                }
            }
        }
        None
    }

    fn search_strings_in_file(&self, filepath: &Path) -> Vec<String> {
        if let Ok(data) = fs::read(filepath) {
            let max_search_bytes = std::cmp::min(data.len(), 1024 * 1024); // 1MB max
            let search_data = &data[..max_search_bytes];
            
            let mut found_strings = Vec::new();
            let mut current_string = String::new();
            
            for &byte in search_data {
                if (32..=126).contains(&byte) { // Printable ASCII
                    current_string.push(byte as char);
                } else {
                    if current_string.len() >= 4 {
                        let lower_string = current_string.to_lowercase();
                        if self.string_indicators.iter().any(|indicator| lower_string.contains(indicator)) {
                            found_strings.push(current_string.clone());
                        }
                    }
                    current_string.clear();
                }
            }
            
            // Check final string
            if current_string.len() >= 4 {
                let lower_string = current_string.to_lowercase();
                if self.string_indicators.iter().any(|indicator| lower_string.contains(indicator)) {
                    found_strings.push(current_string);
                }
            }
            
            found_strings
        } else {
            Vec::new()
        }
    }

    fn analyze_tiktok_evidence(
        &self,
        filename: &str,
        dimensions: &Option<(u32, u32)>,
        aspect_ratio: &Option<f64>,
        strings_found: &[String],
        file_format: &Option<String>,
    ) -> TikTokEvidence {
        let mut evidence = TikTokEvidence {
            is_tiktok: false,
            confidence_score: 0,
            evidence_found: Vec::new(),
            indicators: HashMap::new(),
            verdict: String::new(),
        };

        // Exclusion: If metadata contains camera photo indicators, set confidence to -1000
        let camera_keywords = ["Focal Length", "ISO", "Aperture"];
        let is_camera_photo = strings_found.iter().any(|s| {
            let lower = s.to_lowercase();
            camera_keywords.iter().any(|kw| lower.contains(kw))
        });
        if is_camera_photo {
            evidence.evidence_found.push("Camera photo metadata detected (focal length, ISO, or aperture)".to_string());
            evidence.indicators.insert("camera_photo".to_string(), "excluded".to_string());
            evidence.confidence_score = 0;
            evidence.verdict = "EXCLUDED: Camera photo detected".to_string();
            evidence.is_tiktok = false;
            return evidence;
        }

        // 1. Check for AIGC metadata in strings
        if strings_found.iter().any(|s| s.to_lowercase().contains("aigc_label_type")) {
            evidence.evidence_found.push("AIGC metadata found".to_string());
            evidence.indicators.insert("aigc_metadata".to_string(), "detected".to_string());
            evidence.confidence_score += 40;
        }

        // 2. Check for TikTok video IDs
        for string in strings_found {
            if self.tiktok_video_id_regex.is_match(string) {
                evidence.evidence_found.push("TikTok video ID found".to_string());
                evidence.indicators.insert("tiktok_video_id".to_string(), string.clone());
                evidence.confidence_score += 35;
                break;
            }
        }

        // 3. Check for vid_md5 (ByteDance content hash)
        if strings_found.iter().any(|s| s.to_lowercase().contains("vid_md5")) {
            evidence.evidence_found.push("ByteDance content hash found".to_string());
            evidence.indicators.insert("vid_md5".to_string(), "detected".to_string());
            evidence.confidence_score += 30;
        }

        // 4. Check video dimensions
        if let Some((width, height)) = dimensions {
            if self.tiktok_dimensions.contains(&(*width, *height)) {
                evidence.evidence_found.push(format!("TikTok-typical dimensions: {}x{}", width, height));
                evidence.indicators.insert("video_dimensions".to_string(), format!("{}x{}", width, height));
                evidence.confidence_score += 25;
            }

            // Check for 9:16 aspect ratio (TikTok standard)
            if let Some(ratio) = aspect_ratio {
                if (0.55..=0.58).contains(ratio) {
                    evidence.evidence_found.push("9:16 aspect ratio (TikTok standard)".to_string());
                    evidence.indicators.insert("aspect_ratio".to_string(), format!("{}:{}", width, height));
                    evidence.confidence_score += 15;
                }
            }

            // Check for portrait orientation
            if height > width {
                evidence.evidence_found.push("Portrait orientation".to_string());
                evidence.confidence_score += 5;
            }
        }

        // 5. Check for WebP format with PNG extension (TikTok app behavior)
        if filename.to_lowercase().ends_with(".png") {
            if let Some(format) = file_format {
                if format.to_lowercase().contains("webp") {
                    evidence.evidence_found.push("WebP format with PNG extension (TikTok app behavior)".to_string());
                    evidence.indicators.insert("format_mismatch".to_string(), "webp_as_png".to_string());
                    evidence.confidence_score += 15;
                }
            }
        }

        // 6. Check for hash-based filename (32 chars + extension)
        if filename.len() == 36 && filename.matches('.').count() == 1 {
            let name_part = filename.split('.').next().unwrap_or("");
            if name_part.len() == 32 && name_part.chars().all(|c| c.is_ascii_hexdigit()) {
                evidence.evidence_found.push("MD5-like hash filename (app-generated)".to_string());
                evidence.indicators.insert("filename_pattern".to_string(), "md5_hash".to_string());
                evidence.confidence_score += 10;
            }
        }

        // 7. Check string analysis for TikTok indicators
        if !strings_found.is_empty() {
            let tiktok_strings: Vec<&String> = strings_found.iter()
                .filter(|s| {
                    let lower = s.to_lowercase();
                    ["tiktok", "douyin", "bytedance", "musically"].iter()
                        .any(|indicator| lower.contains(indicator))
                })
                .collect();

            if !tiktok_strings.is_empty() {
                evidence.evidence_found.push("TikTok strings found in file".to_string());
                evidence.indicators.insert("string_indicators".to_string(), 
                    tiktok_strings.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
                evidence.confidence_score += 20;
            }
        }

        // Determine verdict based on confidence score
        if evidence.confidence_score >= 70 {
            evidence.is_tiktok = true;
            evidence.verdict = "CONFIRMED: File is from TikTok".to_string();
        } else if evidence.confidence_score >= 40 {
            evidence.is_tiktok = true;
            evidence.verdict = "LIKELY: Strong evidence suggests TikTok origin".to_string();
        } else if evidence.confidence_score >= 20 {
            evidence.verdict = "POSSIBLE: Some TikTok-like characteristics found".to_string();
        } else {
            evidence.verdict = "UNLIKELY: No significant TikTok evidence found".to_string();
        }

        evidence
    }

    pub fn analyze_folder(&self, folder_path: &Path) -> Result<Vec<FileMetadata>> {
        let mut results = Vec::new();
        
        if !folder_path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory"));
        }

        for entry in walkdir::WalkDir::new(folder_path) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                    if ["jpg", "jpeg", "png", "webp", "gif", "bmp", "mp4", "mov", "avi"]
                        .contains(&ext_str.as_str()) {
                        match self.analyze_file(path) {
                            Ok(metadata) => results.push(metadata),
                            Err(e) => eprintln!("Error analyzing {}: {}", path.display(), e),
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

impl Default for MetadataManager {
    fn default() -> Self {
        Self::new().expect("Failed to create MetadataManager")
    }
}