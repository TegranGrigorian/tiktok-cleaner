use std::path::Path;
use anyhow::Result;
use crate::tiktok_detection::metadata_read::metadata_manager::{MetadataManager, FileMetadata};

pub struct TikTokPhotoDetector {
    metadata_manager: MetadataManager,
}

impl TikTokPhotoDetector {
    pub fn new() -> Result<Self> {
        Ok(TikTokPhotoDetector {
            metadata_manager: MetadataManager::new()?,
        })
    }

    pub fn analyze_image(&self, filepath: &Path) -> Result<FileMetadata> {
        let mut metadata = self.metadata_manager.analyze_file(filepath)?;
        
        // Enhanced image-specific analysis
        self.enhance_image_analysis(&mut metadata);
        
        Ok(metadata)
    }

    fn enhance_image_analysis(&self, metadata: &mut FileMetadata) {
        let mut additional_score = 0;
        let mut additional_evidence = Vec::new();

        // Check for common TikTok screenshot dimensions
        if let Some((width, height)) = metadata.dimensions {
            // Common mobile screenshot sizes that often contain TikTok content
            let mobile_screenshot_sizes = [
                (1080, 1920), (1080, 1800), (1080, 2340), (1080, 2400),
                (828, 1792), (750, 1334), (1125, 2436), (1242, 2688),
                (1284, 2778), (1170, 2532)
            ];

            if mobile_screenshot_sizes.contains(&(width, height)) {
                additional_evidence.push(format!("Mobile screenshot dimensions: {}x{}", width, height));
                additional_score += 15;
            }

            // Check for perfect 9:16 aspect ratio (TikTok standard)
            if let Some(ratio) = metadata.aspect_ratio {
                if (0.5625 - 0.01..=0.5625 + 0.01).contains(&ratio) { // 9:16 = 0.5625
                    additional_evidence.push("Perfect 9:16 aspect ratio (TikTok standard)".to_string());
                    additional_score += 10;
                }
            }
        }

        // Check for TikTok-typical file characteristics
        if metadata.filename.len() == 36 && metadata.filename.matches('.').count() == 1 {
            if metadata.filename.ends_with(".png") {
                additional_evidence.push("32-character hash filename with PNG extension".to_string());
                additional_score += 8;
            }
        }

        // Check for typical TikTok file sizes (screenshots are usually 1-5MB)
        if metadata.size_bytes > 500_000 && metadata.size_bytes < 5_000_000 {
            additional_evidence.push("File size typical of mobile screenshot".to_string());
            additional_score += 5;
        }

        // Update the analysis with additional findings
        metadata.tiktok_analysis.confidence_score += additional_score;
        metadata.tiktok_analysis.evidence_found.extend(additional_evidence);

        // Re-evaluate verdict with enhanced analysis
        if metadata.tiktok_analysis.confidence_score >= 70 {
            metadata.tiktok_analysis.is_tiktok = true;
            metadata.tiktok_analysis.verdict = "CONFIRMED: File is from TikTok".to_string();
        } else if metadata.tiktok_analysis.confidence_score >= 40 {
            metadata.tiktok_analysis.is_tiktok = true;
            metadata.tiktok_analysis.verdict = "LIKELY: Strong evidence suggests TikTok origin".to_string();
        } else if metadata.tiktok_analysis.confidence_score >= 14 {
            metadata.tiktok_analysis.is_tiktok = true;
            metadata.tiktok_analysis.verdict = "POSSIBLE: Some TikTok-like characteristics found".to_string();
        }
    }

    pub fn analyze_folder(&self, folder_path: &Path) -> Result<Vec<FileMetadata>> {
        let results = self.metadata_manager.analyze_folder(folder_path)?;
        
        // Filter for images only
        let image_extensions = ["jpg", "jpeg", "png", "webp", "gif", "bmp"];
        let image_results: Vec<FileMetadata> = results.into_iter()
            .filter(|metadata| {
                if let Some(ext) = Path::new(&metadata.filename).extension() {
                    let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                    image_extensions.contains(&ext_str.as_str())
                } else {
                    false
                }
            })
            .collect();

        Ok(image_results)
    }

    pub fn generate_summary(&self, results: &[FileMetadata]) -> String {
        let mut summary = String::new();
        
        let total_files = results.len();
        let confirmed_tiktok = results.iter().filter(|r| r.tiktok_analysis.confidence_score >= 70).count();
        let likely_tiktok = results.iter().filter(|r| r.tiktok_analysis.confidence_score >= 40 && r.tiktok_analysis.confidence_score < 70).count();
        let possible_tiktok = results.iter().filter(|r| r.tiktok_analysis.confidence_score >= 14 && r.tiktok_analysis.confidence_score < 40).count();
        let unlikely_tiktok = results.iter().filter(|r| r.tiktok_analysis.confidence_score < 14).count();

        summary.push_str(&format!("📊 TikTok Photo Detection Summary\n"));
        summary.push_str(&format!("================================\n\n"));
        summary.push_str(&format!("Total files analyzed: {}\n\n", total_files));
        summary.push_str(&format!("🔴 Confirmed TikTok: {} files\n", confirmed_tiktok));
        summary.push_str(&format!("🟡 Likely TikTok: {} files\n", likely_tiktok));
        summary.push_str(&format!("🔵 Possible TikTok: {} files\n", possible_tiktok));
        summary.push_str(&format!("⚪ Unlikely TikTok: {} files\n\n", unlikely_tiktok));

        if total_files > 0 {
            let detection_rate = ((confirmed_tiktok + likely_tiktok) as f64 / total_files as f64) * 100.0;
            summary.push_str(&format!("Detection rate: {:.1}% likely TikTok files\n\n", detection_rate));
        }

        // Show details for confirmed TikTok files
        if confirmed_tiktok > 0 {
            summary.push_str("🔴 Confirmed TikTok Files:\n");
            for result in results.iter().filter(|r| r.tiktok_analysis.confidence_score >= 70) {
                summary.push_str(&format!("  • {} ({})\n", result.filename, result.tiktok_analysis.verdict));
                summary.push_str(&format!("    Confidence: {}/100\n", result.tiktok_analysis.confidence_score));
                if let Some((w, h)) = result.dimensions {
                    summary.push_str(&format!("    Dimensions: {}x{}\n", w, h));
                }
                summary.push_str(&format!("    Evidence: {}\n\n", result.tiktok_analysis.evidence_found.join(", ")));
            }
        }

        summary
    }

    pub fn print_detailed_analysis(&self, metadata: &FileMetadata) {
        println!("🔍 Analyzing: {}", metadata.filename);
        println!("{}", "=".repeat(60));
        
        println!("🎯 RESULT: {}", metadata.tiktok_analysis.verdict);
        println!("📊 Confidence: {}/100", metadata.tiktok_analysis.confidence_score);
        
        if !metadata.tiktok_analysis.evidence_found.is_empty() {
            println!("\n🔎 Evidence ({} items):", metadata.tiktok_analysis.evidence_found.len());
            for (i, evidence) in metadata.tiktok_analysis.evidence_found.iter().enumerate() {
                println!("   {}. {}", i + 1, evidence);
            }
        }

        if !metadata.tiktok_analysis.indicators.is_empty() {
            println!("\n📋 Key Identifiers:");
            for (key, value) in &metadata.tiktok_analysis.indicators {
                println!("   • {}: {}", key, value);
            }
        }

        println!("\n📱 Technical Details:");
        if let Some((w, h)) = metadata.dimensions {
            println!("   • Dimensions: {}x{}", w, h);
        }
        if let Some(ratio) = metadata.aspect_ratio {
            println!("   • Aspect Ratio: {:.3}", ratio);
        }
        println!("   • File Size: {}", metadata.size_human);
        if let Some(format) = &metadata.file_format {
            println!("   • Format: {}", format);
        }

        println!();
    }
}

impl Default for TikTokPhotoDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create TikTokPhotoDetector")
    }
}