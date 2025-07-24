# Binary Utilities

This directory contains standalone binary tools that complement the main TikTok Cleaner application.

## Files

### `debug_image.rs`
A specialized debugging utility for testing and analyzing the image detection algorithms.

#### Purpose
- **Algorithm Testing**: Test detection algorithms on individual image files
- **Debugging**: Detailed analysis output for troubleshooting detection issues
- **Development**: Validate new detection methods before integration

#### Features
- **Detailed Metadata Analysis**: Shows all extracted metadata and detection signals
- **Confidence Breakdown**: Explains how the confidence score is calculated
- **Format Analysis**: Detailed format and encoding information
- **AIGC Detection**: Specific analysis of AI-generated content markers

#### Usage
```bash
# Debug a specific image file
cargo run --bin debug_image -- "/path/to/image.jpg"

# Analyze a suspected TikTok screenshot
cargo run --bin debug_image -- "/path/to/tiktok_screenshot.png"
```

#### Example Output
```
Analyzing: suspicious_image.png
File size: 1.2 MB
Dimensions: 1080x1920 (9:16 aspect ratio)
Format: PNG container with WebP content
AIGC metadata: Present
Confidence score: 85/100

Detection breakdown:
+ AIGC markers found: +30 points
+ Format mismatch detected: +25 points
+ TikTok standard dimensions: +20 points
+ Hash-pattern filename: +10 points
= Total confidence: 85/100 (CONFIRMED TikTok)
```

This tool is particularly useful for:
- **Development**: Testing new detection algorithms
- **Troubleshooting**: Understanding why specific files are or aren't detected
- **Validation**: Verifying detection accuracy on known samples
- **Research**: Analyzing TikTok content characteristics

## Building and Running

These utilities are built alongside the main application:

```bash
# Build all binaries
cargo build --release

# Run specific utility
cargo run --bin debug_image -- "/path/to/test/file"
```
