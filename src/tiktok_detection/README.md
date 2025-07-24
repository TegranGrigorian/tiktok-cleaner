# TikTok Detection Module

This module contains the core algorithms and utilities for detecting TikTok content in image and video files.

## Module Structure

### Core Detection Files

#### `scanner.rs`
The main scanning engine that coordinates all detection activities:
- **Multithreaded processing** using rayon for parallel file analysis
- **Intelligent caching** to skip previously analyzed files
- **Progress tracking** and result aggregation
- **Phone filesystem compatibility** with MTP support
- **Confidence-based organization** of detected files

#### `tiktok_photo_det.rs`
Specialized algorithms for detecting TikTok images and screenshots:
- **AIGC metadata detection** - Identifies AI-generated content markers
- **Dimension analysis** - Recognizes TikTok's standard 1080x1920 resolution
- **Format analysis** - Detects WebP content disguised as PNG
- **Filename pattern matching** - Identifies MD5-like hash filenames

#### `tiktok_video_det.rs`
Video-specific detection algorithms:
- **ByteDance metadata detection** - Identifies TikTok's parent company markers
- **Content hash analysis** - Recognizes TikTok-specific encoding signatures
- **Dimension analysis** - Detects portrait video characteristics
- **Format verification** - Validates video container formats

#### `test_runner.rs`
Validation and testing framework:
- **Algorithm validation** on curated test datasets
- **Performance analysis** and benchmarking
- **Accuracy metrics** calculation (sensitivity, specificity)
- **Test result reporting** with detailed breakdowns

### Supporting Modules

#### `metadata_read/`
Metadata extraction and analysis utilities:
- **metadata_manager.rs**: Core metadata reading from EXIF, AIGC, and other sources
- **python_examples/**: Reference implementations for validation

#### `file_util/`
File management and caching systems:
- **file_manager.rs**: Enhanced caching with file metadata tracking
- **folder_manager.rs**: Confidence-based folder organization

## Detection Algorithm Overview

The detection system uses a multi-modal approach:

1. **Metadata Analysis**: Examines EXIF data, AIGC markers, and format-specific metadata
2. **Content Analysis**: Analyzes image dimensions, aspect ratios, and encoding characteristics
3. **Pattern Recognition**: Identifies filename patterns and hash-based naming schemes
4. **Confidence Scoring**: Combines multiple signals into a 0-100 confidence score

### Confidence Levels
- **70-100%**: Confirmed TikTok content (high confidence)
- **40-69%**: Likely TikTok content (medium confidence)
- **20-39%**: Possible TikTok content (low confidence)
- **0-19%**: Unlikely TikTok content (very low confidence)

## Performance Features

- **Parallel Processing**: Uses all available CPU cores for analysis
- **Smart Caching**: Avoids re-analyzing unchanged files
- **Memory Efficiency**: Streams file analysis without loading entire files
- **Phone Optimization**: Handles MTP filesystem limitations gracefully

## Usage Example

```rust
use crate::tiktok_detection::Scanner;

// Create scanner instance
let scanner = Scanner::new("/path/to/scan", true)?; // true = move files

// Run parallel scan
let results = scanner.scan_folder_parallel()?;

// Results contain confidence-categorized file counts
println!("Confirmed TikTok files: {}", results.confirmed_tiktok);
```
