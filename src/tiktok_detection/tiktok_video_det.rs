use std::path::Path;
use anyhow::Result;
use crate::tiktok_detection::metadata_read::metadata_manager::{MetadataManager, FileMetadata};

pub struct TikTokVideoDetector {
    metadata_manager: MetadataManager,
}

impl TikTokVideoDetector {
    pub fn new() -> Result<Self> {
        Ok(TikTokVideoDetector {
            metadata_manager: MetadataManager::new()?,
        })
    }

    pub fn analyze_video(&self, filepath: &Path) -> Result<FileMetadata> {
        let mut metadata = self.metadata_manager.analyze_file(filepath)?;
        
        // Enhanced video-specific analysis
        self.enhance_video_analysis(&mut metadata);
        
        Ok(metadata)
    }

    fn enhance_video_analysis(&self, metadata: &mut FileMetadata) {
        let mut additional_score = 0;
        let mut additional_evidence = Vec::new();

        // Check for TikTok-specific video characteristics
        if let Some((width, height)) = metadata.dimensions {
            // TikTok's standard video dimensions
            let tiktok_video_dimensions = [
                (576, 1024), (576, 1246), (576, 1280),
                (720, 1280), (1080, 1920),
            ];

            if tiktok_video_dimensions.contains(&(width, height)) {
                additional_evidence.push(format!("TikTok standard video dimensions: {}x{}", width, height));
                additional_score += 30;
            }

            // Bonus for exact TikTok preferred dimensions
            if (width, height) == (576, 1024) || (width, height) == (1080, 1920) {
                additional_evidence.push("Exact TikTok preferred video dimensions".to_string());
                additional_score += 15;
            }

            if let Some(ratio) = metadata.aspect_ratio {
                // Relaxed: portrait if width < height, or aspect ratio < 0.8
                if width < height {
                    additional_evidence.push("Portrait orientation (width < height)".to_string());
                    additional_score += 10;
                }
                if (0.55..=0.58).contains(&ratio) {
                    additional_evidence.push("Vertical mobile video format (9:16)".to_string());
                    additional_score += 20;
                } else if ratio < 0.8 {
                    additional_evidence.push(format!("Portrait aspect ratio: {:.3}", ratio));
                    additional_score += 8;
                }
            }
        }

        // Check for TikTok-specific strings in metadata
        // Check for TikTok-specific video characteristics
        if let Some((width, height)) = metadata.dimensions {
            // TikTok's standard video dimensions
            let tiktok_video_dimensions = [
                (576, 1024), (576, 1246), (576, 1280),
                (720, 1280), (1080, 1920),
            ];

            if tiktok_video_dimensions.contains(&(width, height)) {
                additional_evidence.push(format!("TikTok standard video dimensions: {}x{}", width, height));
                additional_score += 30;
            }

            // Bonus for exact TikTok preferred dimensions
            if (width, height) == (576, 1024) || (width, height) == (1080, 1920) {
                additional_evidence.push("Exact TikTok preferred video dimensions".to_string());
                additional_score += 15;
            }

            if let Some(ratio) = metadata.aspect_ratio {
                // Relaxed: portrait if width < height, or aspect ratio < 0.8
                if width < height {
                    additional_evidence.push("Portrait orientation (width < height)".to_string());
                    additional_score += 10;
                }
                if (0.55..=0.58).contains(&ratio) {
                    additional_evidence.push("Vertical mobile video format (9:16)".to_string());
                    additional_score += 20;
                } else if ratio < 0.8 {
                    additional_evidence.push(format!("Portrait aspect ratio: {:.3}", ratio));
                    additional_score += 8;
                }
            }
        }
        let filename = &metadata.filename;
        // Check for TikTok-specific strings in metadata
        let tiktok_specific_strings = [
            "ByteDance",
            "Lavf58.76.100", // Common TikTok encoder
            "Lavf", // General FFmpeg encoder, TikTok often uses Lavf*
            "mp4v", // TikTok encoder
            "isom", // TikTok encoder
            "Douyin",
            "Musical.ly",
            "aigc_info",
            "vid_md5",
        ];
        for string in &metadata.strings_found {
            for tiktok_string in &tiktok_specific_strings {
                if string.to_lowercase().contains(&tiktok_string.to_lowercase()) {
                    additional_evidence.push(format!("TikTok-specific metadata: {}", tiktok_string));
                    additional_score += match tiktok_string.as_ref() {
                        "aigc_info" => 40,
                        "vid_md5" => 35,
                        "ByteDance" => 25,
                        "TikTok" => 25,
                        "Douyin" => 25,
                        "Lavf58.76.100" => 20,
                        "Lavf" => 10,
                        "mp4v" => 8,
                        "isom" => 8,
                        "Musical.ly" => 8,
                        _ => 10,
                    };
                    break;
                }
            }
        }
        if filename.to_lowercase().starts_with("download") && filename.to_lowercase().ends_with(".mp4") {
            additional_evidence.push("TikTok download naming pattern (Download*.mp4)".to_string());
            additional_score += 25; // Boosted from 10 to 25
        }

        // Check for reasonable file size (TikTok videos are typically 1-50MB)
        if metadata.size_bytes > 100_000 && metadata.size_bytes < 50_000_000 {
            additional_evidence.push("File size typical of TikTok video".to_string());
            additional_score += 5;
        }

        // Update the analysis with additional findings
        metadata.tiktok_analysis.confidence_score += additional_score;
        metadata.tiktok_analysis.evidence_found.extend(additional_evidence);

        // Re-evaluate verdict with enhanced analysis
        if metadata.tiktok_analysis.confidence_score >= 70 {
            metadata.tiktok_analysis.is_tiktok = true;
            metadata.tiktok_analysis.verdict = "CONFIRMED: Video is from TikTok".to_string();
        } else if metadata.tiktok_analysis.confidence_score >= 40 {
            metadata.tiktok_analysis.is_tiktok = true;
            metadata.tiktok_analysis.verdict = "LIKELY: Strong evidence suggests TikTok origin".to_string();
        } else if metadata.tiktok_analysis.confidence_score >= 14 {
            metadata.tiktok_analysis.is_tiktok = true;
            metadata.tiktok_analysis.verdict = "POSSIBLE: Some TikTok-like characteristics found".to_string();
        }
    }

    pub fn generate_summary(&self, results: &[FileMetadata]) -> String {
        let mut summary = String::new();
        
        let total_files = results.len();
        let confirmed_tiktok = results.iter().filter(|r| r.tiktok_analysis.confidence_score >= 70).count();
        let likely_tiktok = results.iter().filter(|r| r.tiktok_analysis.confidence_score >= 40 && r.tiktok_analysis.confidence_score < 70).count();
        let possible_tiktok = results.iter().filter(|r| r.tiktok_analysis.confidence_score >= 14 && r.tiktok_analysis.confidence_score < 40).count();
        let unlikely_tiktok = results.iter().filter(|r| r.tiktok_analysis.confidence_score < 14).count();

        summary.push_str(&format!("📊 TikTok Video Detection Summary\n"));
        summary.push_str(&format!("================================\n\n"));
        summary.push_str(&format!("Total videos analyzed: {}\n\n", total_files));
        summary.push_str(&format!("🔴 Confirmed TikTok: {} videos\n", confirmed_tiktok));
        summary.push_str(&format!("🟡 Likely TikTok: {} videos\n", likely_tiktok));
        summary.push_str(&format!("🔵 Possible TikTok: {} videos\n", possible_tiktok));
        summary.push_str(&format!("⚪ Unlikely TikTok: {} videos\n\n", unlikely_tiktok));

        if total_files > 0 {
            let detection_rate = ((confirmed_tiktok + likely_tiktok) as f64 / total_files as f64) * 100.0;
            summary.push_str(&format!("Detection rate: {:.1}% likely TikTok videos\n\n", detection_rate));
        }

        // Show details for confirmed TikTok videos
        if confirmed_tiktok > 0 {
            summary.push_str("🔴 Confirmed TikTok Videos:\n");
            for result in results.iter().filter(|r| r.tiktok_analysis.confidence_score >= 70) {
                summary.push_str(&format!("  • {} ({})\n", result.filename, result.tiktok_analysis.verdict));
                summary.push_str(&format!("    Confidence: {}/100\n", result.tiktok_analysis.confidence_score));
                if let Some((w, h)) = result.dimensions {
                    summary.push_str(&format!("    Dimensions: {}x{}\n", w, h));
                }
                summary.push_str(&format!("    Size: {}\n", result.size_human));
                summary.push_str(&format!("    Evidence: {}\n\n", result.tiktok_analysis.evidence_found.join(", ")));
            }
        }

        summary
    }

    pub fn print_detailed_analysis(&self, metadata: &FileMetadata) {
        println!("🎥 Analyzing Video: {}", metadata.filename);
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
            if let Some(ratio) = metadata.aspect_ratio {
                println!("   • Aspect Ratio: {:.3}", ratio);
            }
        }
        println!("   • File Size: {}", metadata.size_human);
        if let Some(format) = &metadata.file_format {
            println!("   • Format: {}", format);
        }

        if !metadata.strings_found.is_empty() && metadata.strings_found.len() <= 10 {
            println!("   • TikTok-related strings found: {}", metadata.strings_found.len());
            for (i, string) in metadata.strings_found.iter().take(5).enumerate() {
                println!("     {}. {}", i + 1, string);
            }
            if metadata.strings_found.len() > 5 {
                println!("     ... and {} more", metadata.strings_found.len() - 5);
            }
        }

        println!();
    }

    pub fn check_video_integrity(&self, filepath: &Path) -> Result<bool> {
        // Basic check to see if the video file can be opened
        // This would typically use ffmpeg or similar library
        // For now, just check if file exists and has reasonable size
        let metadata = std::fs::metadata(filepath)?;
        Ok(metadata.len() > 1000 && metadata.len() < 100_000_000) // 1KB to 100MB
    }
}

impl Default for TikTokVideoDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create TikTokVideoDetector")
    }
}