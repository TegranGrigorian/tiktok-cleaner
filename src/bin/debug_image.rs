use std::path::Path;
use tiktok_cleaner::tiktok_detection::metadata_read::metadata_manager::MetadataManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = MetadataManager::new()?;
    
    // Test a specific TikTok image
    let test_file = Path::new("src/tiktok_detection/metadata_read/python_examples/testsets/tiktok/06ce7f9478ac3fee135be300a06a372a.png");
    
    if test_file.exists() {
        println!("🔍 Debugging TikTok image: {}", test_file.display());
        
        let metadata = manager.analyze_file(test_file)?;
        
        println!("Filename: {}", metadata.filename);
        println!("Dimensions: {:?}", metadata.dimensions);
        println!("File format detected: {:?}", metadata.file_format);
        println!("File size: {}", metadata.size_human);
        println!("Confidence: {}", metadata.tiktok_analysis.confidence_score);
        println!("Evidence: {:?}", metadata.tiktok_analysis.evidence_found);
        
        // Try to see if we can detect WebP manually
        let header = std::fs::read(test_file).unwrap_or_default();
        if header.len() >= 12 {
            let webp_signature = &header[0..4] == b"RIFF" && &header[8..12] == b"WEBP";
            println!("Manual WebP detection: {}", webp_signature);
            if webp_signature {
                println!("File header: {:02X?}", &header[0..16]);
            }
        }
    } else {
        println!("Test file not found: {}", test_file.display());
    }
    
    Ok(())
}
