/*!
# TikTok Cleaner - TikTok Detection and Organization Tool

A high-performance Rust application for detecting and organizing TikTok files from phone storage.
Uses advanced metadata analysis, image characteristics, and filename patterns to identify 
TikTok content with confidence-based categorization.

## Features

- **🚀 Multithreaded Processing**: Parallel file analysis for optimal performance
- **📱 Phone Filesystem Support**: Works with MTP/Android phone storage 
- **🧠 Intelligent Caching**: Avoids re-analyzing unchanged files
- **📊 Confidence Scoring**: Categorizes files as confirmed, likely, possible, or unlikely TikTok
- **📁 Smart Organization**: Automatically organizes detected files into confidence-based folders
- **🔍 Multiple Detection Methods**: AIGC metadata, video IDs, dimensions, format analysis

## Usage

### Scan a folder (preview mode)
```bash
cargo run --bin tiktok-cleaner -- --scan "/path/to/phone/DCIM"
```

### Actually move/organize files  
```bash
cargo run --bin tiktok-cleaner -- --scan "/path/to/phone/DCIM" --move
```

### Run test experiment
```bash
cargo run --bin tiktok-cleaner -- --test
```

## Examples

### Android Phone via MTP
```bash
cargo run --bin tiktok-cleaner -- --scan "/run/user/1000/gvfs/mtp:host=SAMSUNG_SAMSUNG_Android_R5CW90Y5HRF/Internal storage/DCIM" --move
```

### Local folder
```bash  
cargo run --bin tiktok-cleaner -- --scan "/home/user/Pictures" --move
```
*/

use std::path::Path;
use clap::{Arg, Command};
use tiktok_cleaner::tiktok_detection::{test_runner::TestRunner, scanner::TikTokScanner};

/// Main entry point for the TikTok Detection and Organization Tool
///
/// Handles command-line argument parsing and dispatches to appropriate functionality:
/// - `--scan`: Scan a folder for TikTok files with optional `--move` flag
/// - `--test`: Run built-in test experiment on sample files
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("TikTok Cleaner")
        .version("1.0")
        .about("🚀 TikTok Detection and Organization Tool")
        .arg(
            Arg::new("scan")
                .long("scan")
                .value_name("FOLDER")
                .help("Scan a folder for TikTok files (e.g., phone DCIM folder)")
                .conflicts_with("test")
        )
        .arg(
            Arg::new("move")
                .long("move")
                .action(clap::ArgAction::SetTrue)
                .help("Actually move files (default is preview mode)")
                .requires("scan")
        )
        .arg(
            Arg::new("test")
                .long("test")
                .action(clap::ArgAction::SetTrue)
                .help("Run test experiment on built-in test sets")
                .conflicts_with("scan")
        )
        .get_matches();

    if matches.get_flag("test") {
        // Run test experiment
        run_test_experiment()?;
    } else if let Some(scan_path) = matches.get_one::<String>("scan") {
        // Run scanner on specified folder
        let move_files = matches.get_flag("move");
        run_scanner(scan_path, move_files)?;
    } else {
        // Show help if no arguments provided
        eprintln!("🚀 TikTok Detection and Organization Tool\n");
        eprintln!("Usage examples:");
        eprintln!("  # Preview scan of phone folder:");
        eprintln!("  cargo run -- --scan \"/run/user/1000/gvfs/mtp:host=SAMSUNG_SAMSUNG_Android_R5CW90Y5HRF/Internal storage/DCIM\"");
        eprintln!("");
        eprintln!("  # Actually move files:");
        eprintln!("  cargo run -- --scan \"/path/to/phone/DCIM\" --move");
        eprintln!("");
        eprintln!("  # Run test experiment:");
        eprintln!("  cargo run -- --test");
        eprintln!("");
        eprintln!("For full help: cargo run -- --help");
    }

    Ok(())
}

/// Runs the TikTok scanner on a specified folder path
///
/// This function performs comprehensive TikTok detection using multithreaded analysis.
/// It validates the input path, initializes the scanner with enhanced caching, and
/// processes files in parallel for optimal performance.
///
/// # Arguments
/// * `scan_path` - Path to the folder to scan (e.g., phone DCIM folder)
/// * `move_files` - If true, actually moves detected files. If false, preview mode only.
///
/// # Examples
/// ```
/// // Preview scan without moving files
/// run_scanner("/path/to/phone/DCIM", false)?;
/// 
/// // Scan and organize detected TikTok files
/// run_scanner("/path/to/phone/DCIM", true)?;
/// ```
///
/// # Phone Filesystem Support
/// This function is designed to work with MTP-mounted Android phone storage:
/// ```
/// run_scanner("/run/user/1000/gvfs/mtp:host=SAMSUNG_*/Internal storage/DCIM", true)?;
/// ```
fn run_scanner(scan_path: &str, move_files: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(scan_path);
    
    if !path.exists() {
        eprintln!("❌ Scan folder not found: {}", path.display());
        eprintln!("   Please ensure the path is correct and accessible");
        return Ok(());
    }

    if !path.is_dir() {
        eprintln!("❌ Path is not a directory: {}", path.display());
        return Ok(());
    }

    println!("🚀 TikTok Detection and Organization Tool");
    if move_files {
        println!("🔧 Mode: MOVE FILES");
    } else {
        println!("👁️  Mode: PREVIEW (use --move to actually move files)");
    }
    println!();

    let mut scanner = TikTokScanner::new(path)?;
    let results = scanner.scan_folder_parallel(move_files)?;

    if move_files && (results.confirmed_tiktok + results.likely_tiktok + results.possible_tiktok) > 0 {
        println!("\n✅ TikTok files have been organized into confidence-based folders!");
        println!("📁 Check the 'tiktok_detection' folder in your scan directory");
    }

    Ok(())
}

/// Runs the built-in test experiment to validate TikTok detection accuracy
///
/// This function executes comprehensive testing on curated datasets of TikTok and
/// non-TikTok files to measure algorithm performance. It provides detailed metrics
/// including sensitivity, specificity, and confidence score distribution.
///
/// The test validates:
/// - Detection of confirmed TikTok files (with AIGC metadata, proper dimensions)  
/// - Rejection of non-TikTok content (photos, videos from other sources)
/// - Confidence scoring accuracy across different file types
/// - Algorithm consistency and reliability
///
/// # Test Datasets
/// - **TikTok Test Set**: Contains actual TikTok screenshots and videos with known metadata
/// - **Not-TikTok Test Set**: Contains regular photos and videos from other sources
///
/// # Returns
/// Comprehensive performance metrics including detection rates and specificity scores.
fn run_test_experiment() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TikTok Detection Algorithm - Test Experiment");
    println!("Running built-in test on sample files...\n");

    // Define paths to test sets
    let tiktok_folder = Path::new("src/tiktok_detection/metadata_read/python_examples/testsets/tiktok");
    let not_tiktok_folder = Path::new("src/tiktok_detection/metadata_read/python_examples/testsets/not_tiktok");

    // Check if test folders exist
    if !tiktok_folder.exists() {
        eprintln!("❌ TikTok test folder not found: {}", tiktok_folder.display());
        eprintln!("   Please ensure the testsets are available");
        return Ok(());
    }

    if !not_tiktok_folder.exists() {
        eprintln!("❌ Not-TikTok test folder not found: {}", not_tiktok_folder.display());
        eprintln!("   Please ensure the testsets are available");
        return Ok(());
    }

    // Create test runner and run experiment
    let test_runner = TestRunner::new()?;
    test_runner.run_experiment(tiktok_folder, not_tiktok_folder)?;

    Ok(())
}
