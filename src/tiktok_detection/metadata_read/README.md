# Metadata Reading Module

This module handles extraction and analysis of metadata from image and video files to identify TikTok content characteristics.

## Files

### `metadata_manager.rs`
Core metadata extraction and analysis engine:

#### Key Functions
- **`read_image_metadata()`**: Extracts EXIF, AIGC, and format-specific metadata from images
- **`detect_aigc_metadata()`**: Specifically looks for AI-generated content markers that TikTok adds
- **`analyze_image_format()`**: Determines actual vs. declared image formats (WebP vs PNG detection)
- **`extract_creation_info()`**: Gets timestamp and creation metadata for caching

#### Supported Metadata Types
- **EXIF Data**: Standard image metadata including dimensions, creation time, camera info
- **AIGC Markers**: AI-generated content indicators specific to TikTok's processing
- **Format Analysis**: Detects format mismatches (e.g., WebP content in PNG container)
- **Filename Analysis**: Analyzes filename patterns for TikTok-specific hash patterns

#### Detection Strategies
1. **AIGC Detection**: Looks for specific metadata keys that indicate AI-generated content
2. **Format Verification**: Compares file headers with file extensions to detect disguised formats
3. **Dimension Analysis**: Identifies TikTok's standard resolutions and aspect ratios
4. **Timestamp Analysis**: Examines creation patterns typical of app-generated content

### `python_examples/`
Reference implementations and validation scripts:
- **`image_tiktok_analyzer.py`**: Python reference implementation for metadata analysis
- Used for algorithm validation and cross-verification of detection logic

## Usage Example

```rust
use crate::tiktok_detection::metadata_read::metadata_manager;

// Read metadata from an image file
let metadata = metadata_manager::read_image_metadata(&file_path)?;

// Check for AIGC indicators
if metadata_manager::detect_aigc_metadata(&metadata) {
    println!("AI-generated content detected");
}

// Analyze format consistency
let format_info = metadata_manager::analyze_image_format(&file_path)?;
if format_info.is_format_mismatch {
    println!("Format mismatch detected (possible TikTok content)");
}
```

## Detection Confidence

The metadata analysis contributes to the overall confidence score:
- **AIGC metadata present**: +30 points
- **Format mismatch (WebP as PNG)**: +25 points
- **TikTok-standard dimensions**: +20 points
- **Hash-pattern filename**: +15 points
- **Suspicious creation timestamp patterns**: +10 points

These scores are combined with other detection methods to produce the final confidence rating.
