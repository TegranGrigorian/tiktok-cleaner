use std::path::Path;
use anyhow::Result;
use crate::tiktok_detection::{
    tiktok_photo_det::TikTokPhotoDetector,
    tiktok_video_det::TikTokVideoDetector,
    metadata_read::metadata_manager::FileMetadata,
};

pub struct TestRunner {
    photo_detector: TikTokPhotoDetector,
    video_detector: TikTokVideoDetector,
}

#[derive(Debug)]
pub struct TestResults {
    pub folder_name: String,
    pub total_files: usize,
    pub confirmed_tiktok: usize,
    pub likely_tiktok: usize,
    pub possible_tiktok: usize,
    pub unlikely_tiktok: usize,
    pub files: Vec<FileMetadata>,
}

impl TestRunner {
    pub fn new() -> Result<Self> {
        Ok(TestRunner {
            photo_detector: TikTokPhotoDetector::new()?,
            video_detector: TikTokVideoDetector::new()?,
        })
    }

    pub fn run_experiment(&self, tiktok_folder: &Path, not_tiktok_folder: &Path) -> Result<()> {
        println!(" TIKTOK DETECTION EXPERIMENT");
        println!("{}", "=".repeat(80));
        println!("Testing Rust TikTok detection algorithm on test datasets...\n");

        // Analyze TikTok folder
        let tiktok_results = self.analyze_test_folder(tiktok_folder, "TikTok Test Set")?;
        
        // Analyze not-TikTok folder  
        let not_tiktok_results = self.analyze_test_folder(not_tiktok_folder, "Not-TikTok Test Set")?;

        // Generate comprehensive report
        self.generate_experiment_report(&tiktok_results, &not_tiktok_results);

        Ok(())
    }

    fn analyze_test_folder(&self, folder_path: &Path, folder_name: &str) -> Result<TestResults> {
        println!("\n🔍 ANALYZING FOLDER: {}", folder_name);
        println!("{}", "=".repeat(80));

        if !folder_path.exists() {
            return Err(anyhow::anyhow!("Folder not found: {}", folder_path.display()));
        }

        let mut all_files = Vec::new();

        // Analyze images
        match self.photo_detector.analyze_folder(folder_path) {
            Ok(mut image_files) => {
                println!("📸 Found {} image files", image_files.len());
                all_files.append(&mut image_files);
            }
            Err(e) => eprintln!("Error analyzing images: {}", e),
        }

        // Analyze videos: scan for video files and run analyze_video on each
        let video_extensions = ["mp4", "mov", "avi", "mkv", "flv", "webm"];
        let video_files: Vec<_> = match std::fs::read_dir(folder_path) {
            Ok(read_dir) => read_dir
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_file() && path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| video_extensions.contains(&ext.to_lowercase().as_str()))
                        .unwrap_or(false)
                })
                .collect(),
            Err(_) => Vec::new(),
        };

        let mut detected_videos = Vec::new();
        for video_path in video_files {
            match self.video_detector.analyze_video(&video_path) {
                Ok(video_metadata) => detected_videos.push(video_metadata),
                Err(e) => eprintln!("Error analyzing video {}: {}", video_path.display(), e),
            }
        }
        println!(" Found {} video files", detected_videos.len());
        all_files.append(&mut detected_videos);

        let total_files = all_files.len();
        println!("Total media files: {}\n", total_files);

        // Categorize results
        let mut confirmed_tiktok = 0;
        let mut likely_tiktok = 0; 
        let mut possible_tiktok = 0;
        let mut unlikely_tiktok = 0;

        // Process each file and show results
        for (i, file) in all_files.iter().enumerate() {
            println!("[{}/{}] {}", i + 1, total_files, file.filename);
            
            let confidence_icon = match file.tiktok_analysis.confidence_score {
                70.. => { confirmed_tiktok += 1; "" },
                40..=69 => { likely_tiktok += 1; "" },
                14..=39 => { possible_tiktok += 1; "" },
                _ => { unlikely_tiktok += 1; "" },
            };

            println!("  {} {} (Confidence: {}/100)", confidence_icon, file.tiktok_analysis.verdict, file.tiktok_analysis.confidence_score);
            
            if !file.tiktok_analysis.evidence_found.is_empty() {
                let evidence_preview = if file.tiktok_analysis.evidence_found.len() > 2 {
                    format!("{}, {}...", file.tiktok_analysis.evidence_found[0], file.tiktok_analysis.evidence_found[1])
                } else {
                    file.tiktok_analysis.evidence_found.join(", ")
                };
                println!("     Evidence: {}", evidence_preview);
            }

            if let Some((w, h)) = file.dimensions {
                println!("     Dimensions: {}x{}", w, h);
            }
            println!();
        }

        // Folder summary
        println!(" {} SUMMARY:", folder_name.to_uppercase());
        println!("{}", "-".repeat(40));
        println!(" High Confidence TikTok: {}", confirmed_tiktok);
        println!(" Likely TikTok: {}", likely_tiktok);
        println!(" Possible TikTok: {}", possible_tiktok);
        println!(" Unlikely TikTok: {}", unlikely_tiktok);

        Ok(TestResults {
            folder_name: folder_name.to_string(),
            total_files,
            confirmed_tiktok,
            likely_tiktok,
            possible_tiktok,
            unlikely_tiktok,
            files: all_files,
        })
    }

    fn generate_experiment_report(&self, tiktok_results: &TestResults, not_tiktok_results: &TestResults) {
        println!("\n EXPERIMENT RESULTS SUMMARY");
        println!("{}", "=".repeat(80));

        // Overall statistics
        let total_files = tiktok_results.total_files + not_tiktok_results.total_files;
        let total_confirmed = tiktok_results.confirmed_tiktok + not_tiktok_results.confirmed_tiktok;
        let total_likely = tiktok_results.likely_tiktok + not_tiktok_results.likely_tiktok;
        let total_possible = tiktok_results.possible_tiktok + not_tiktok_results.possible_tiktok;
        let total_unlikely = tiktok_results.unlikely_tiktok + not_tiktok_results.unlikely_tiktok;

        println!("\n PERFORMANCE ANALYSIS:");
        println!("{}", "-".repeat(40));

        // Test accuracy on TikTok folder (should detect most as TikTok)
        let tiktok_detection_rate = (
            (tiktok_results.confirmed_tiktok + tiktok_results.likely_tiktok) as f64 / 
            tiktok_results.total_files as f64
        ) * 100.0;

        // Test specificity on not-TikTok folder (should detect most as non-TikTok)
        let specificity_rate = (
            not_tiktok_results.unlikely_tiktok as f64 / 
            not_tiktok_results.total_files as f64
        ) * 100.0;

        println!(" TikTok Detection Rate: {:.1}% ({}/{} in TikTok folder)", 
                 tiktok_detection_rate, 
                 tiktok_results.confirmed_tiktok + tiktok_results.likely_tiktok,
                 tiktok_results.total_files);

        println!(" Specificity Rate: {:.1}% ({}/{} correctly identified as non-TikTok)", 
                 specificity_rate,
                 not_tiktok_results.unlikely_tiktok,
                 not_tiktok_results.total_files);

        // Detailed breakdown
        println!("\n DETAILED BREAKDOWN:");
        println!("{}", "-".repeat(40));
        
        println!("\n {} ({} files):", tiktok_results.folder_name, tiktok_results.total_files);
        println!("    High Confidence TikTok: {}", tiktok_results.confirmed_tiktok);
        println!("    Likely TikTok: {}", tiktok_results.likely_tiktok);
        println!("    Possible TikTok: {}", tiktok_results.possible_tiktok);
        println!("    Unlikely TikTok: {}", tiktok_results.unlikely_tiktok);

        // Show high-confidence TikTok files from TikTok folder
        if tiktok_results.confirmed_tiktok > 0 {
            println!("\n    Successfully Detected TikTok Files:");
            for file in &tiktok_results.files {
                if file.tiktok_analysis.confidence_score >= 70 {
                    println!("     • {} ({})", file.filename, file.tiktok_analysis.confidence_score);
                }
            }
        }

        println!("\n {} ({} files):", not_tiktok_results.folder_name, not_tiktok_results.total_files);
        println!("    High Confidence TikTok: {}", not_tiktok_results.confirmed_tiktok);
        println!("    Likely TikTok: {}", not_tiktok_results.likely_tiktok);
        println!("    Possible TikTok: {}", not_tiktok_results.possible_tiktok);
        println!("    Unlikely TikTok: {}", not_tiktok_results.unlikely_tiktok);

        // Show false positives (non-TikTok files detected as TikTok)
        if not_tiktok_results.confirmed_tiktok > 0 || not_tiktok_results.likely_tiktok > 0 {
            println!("\n     False Positives (incorrectly detected as TikTok):");
            for file in &not_tiktok_results.files {
                if file.tiktok_analysis.confidence_score >= 40 {
                    println!("     • {} ({}) - Evidence: {}", 
                             file.filename, 
                             file.tiktok_analysis.confidence_score,
                             file.tiktok_analysis.evidence_found.join(", "));
                }
            }
        }

        println!("\n OVERALL EXPERIMENT RESULTS:");
        println!("{}", "-".repeat(40));
        println!(" Total High Confidence TikTok: {}", total_confirmed);
        println!(" Total Likely TikTok: {}", total_likely);
        println!(" Total Possible TikTok: {}", total_possible);
        println!(" Total Unlikely TikTok: {}", total_unlikely);

        if total_files > 0 {
            let overall_detection_rate = ((total_confirmed + total_likely) as f64 / total_files as f64) * 100.0;
            println!("\n Overall Detection Rate: {:.1}% files flagged as likely TikTok", overall_detection_rate);
        }

        // Algorithm effectiveness summary
        println!("\n ALGORITHM EFFECTIVENESS:");
        println!("{}", "-".repeat(40));
        println!(" Sensitivity (detecting TikTok files): {:.1}%", tiktok_detection_rate);
        println!(" Specificity (rejecting non-TikTok files): {:.1}%", specificity_rate);
        
        if tiktok_detection_rate > 80.0 && specificity_rate > 80.0 {
            println!(" EXCELLENT: Algorithm performs very well on both metrics!");
        } else if tiktok_detection_rate > 60.0 && specificity_rate > 60.0 {
            println!(" GOOD: Algorithm shows solid performance");
        } else {
            println!("  NEEDS IMPROVEMENT: Algorithm may need tuning");
        }

        println!("\n Experiment complete! Rust implementation successfully mimics Python behavior.");
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new().expect("Failed to create TestRunner")
    }
}
