# TikTok Cleaner 🚀

A high-performance Rust application for detecting and organizing TikTok files from phone storage using advanced metadata analysis and intelligent caching.

## ✨ Features

- **🚀 Multithreaded Processing**: Parallel file analysis using all available CPU cores
- **📱 Phone Filesystem Support**: Works seamlessly with MTP/Android phone storage
- **🧠 Intelligent Caching**: Avoids re-analyzing unchanged files for fast subsequent scans
- **📊 Confidence Scoring**: Categorizes files as confirmed (70%+), likely (40-69%), possible (20-39%), or unlikely (<20%) TikTok
- **📁 Smart Organization**: Automatically organizes detected files into confidence-based folders
- **🔍 Advanced Detection**: Multiple detection methods including AIGC metadata, video IDs, dimensions, and format analysis

## 🎯 Detection Algorithm

The algorithm analyzes multiple characteristics to identify TikTok content:

### Image Detection
- **AIGC Metadata**: Detects TikTok's AI-generated content markers
- **Dimensions**: Recognizes TikTok's standard 1080x1920 resolution and 9:16 aspect ratio
- **Format Analysis**: Identifies WebP content disguised as PNG (TikTok app behavior)
- **Filename Patterns**: Recognizes MD5-like hash filenames from app generation

### Video Detection
- **ByteDance Metadata**: Detects TikTok's parent company identifiers
- **Content Hashes**: Recognizes TikTok-specific video encoding signatures
- **Dimension Analysis**: Identifies portrait video characteristics

## 🚀 Usage

### Quick Start

```bash
# Preview scan (doesn't move files)
cargo run --bin tiktok-cleaner -- --scan "/path/to/phone/DCIM"

# Actually organize detected TikTok files
cargo run --bin tiktok-cleaner -- --scan "/path/to/phone/DCIM" --move

# Run built-in test experiment
cargo run --bin tiktok-cleaner -- --test
```

### Android Phone via MTP

```bash
# Find your phone's MTP path (usually in /run/user/1000/gvfs/)
ls /run/user/1000/gvfs/

# Scan phone storage
cargo run --bin tiktok-cleaner -- --scan "/run/user/1000/gvfs/mtp:host=SAMSUNG_*/Internal storage/DCIM" --move
```

### Local Folder

```bash
# Scan local Pictures folder
cargo run --bin tiktok-cleaner -- --scan "/home/user/Pictures" --move
```

## 📊 Performance

- **Multithreaded**: Uses all available CPU cores for parallel analysis
- **Cached**: Skips previously analyzed files that haven't changed
- **Memory Efficient**: Streams file analysis without loading entire files into memory
- **Phone Optimized**: Handles MTP filesystem limitations gracefully

## 📁 Output Organization

Detected TikTok files are organized into confidence-based folders:

```
your_scan_folder/
├── tiktok_detection/
│   ├── confirmed/     # 70%+ confidence (definitely TikTok)
│   ├── likely/        # 40-69% confidence (probably TikTok)
│   └── possible/      # 20-39% confidence (might be TikTok)
└── .tiktok_cache.json # Intelligent caching file
```

## 🧪 Test Results

The algorithm has been validated on curated test datasets:

- **Sensitivity**: 27.3% detection rate on actual TikTok files
- **Specificity**: 100% - correctly rejects all non-TikTok content
- **Accuracy**: High precision with minimal false positives

## 🛠️ Installation

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone and build**:
   ```bash
   git clone <repository-url>
   cd tiktok-cleaner
   cargo build --release
   ```

3. **Run**:
   ```bash
   cargo run --bin tiktok-cleaner -- --help
   ```

## 📋 Requirements

- **Rust 2021 Edition** or later
- **Dependencies**: All managed by Cargo
  - `image` - Image processing and format detection
  - `walkdir` - Recursive directory traversal
  - `serde` - JSON serialization for caching
  - `clap` - Command-line argument parsing
  - `rayon` - Parallel processing
  - `num_cpus` - CPU core detection
  - `anyhow` - Error handling
  - `chrono` - Timestamp management
  - `md5` - Hash verification

## 🔧 Architecture

```
tiktok-cleaner/
├── src/
│   ├── main.rs                    # CLI interface and main entry point
│   ├── lib.rs                     # Library exports
│   └── tiktok_detection/
│       ├── mod.rs                 # Module declarations
│       ├── scanner.rs             # Main scanning engine with multithreading
│       ├── metadata_read/         # Metadata extraction and analysis
│       ├── tiktok_photo_det.rs    # Image-specific TikTok detection
│       ├── tiktok_video_det.rs    # Video-specific TikTok detection
│       ├── test_runner.rs         # Test experiment runner
│       └── file_util/             # File management and caching
│           ├── file_manager.rs    # Enhanced caching and file operations
│           └── folder_manager.rs  # Folder organization utilities
```

## 📈 Changelog

### v0.1.0 - Enhanced Performance Release
- ✅ **Multithreaded Processing**: Parallel file analysis using rayon
- ✅ **Enhanced Caching**: File metadata tracking with change detection
- ✅ **Comprehensive Documentation**: Detailed code documentation throughout
- ✅ **Phone Compatibility**: Robust MTP filesystem support
- ✅ **Performance Optimization**: Thread count optimization based on CPU cores

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🎯 Use Cases

- **Phone Cleanup**: Organize TikTok screenshots cluttering your phone storage
- **Digital Organization**: Separate TikTok content from personal photos
- **Content Analysis**: Analyze large media collections for TikTok content
- **Batch Processing**: Handle thousands of files efficiently with multithreading
- **Research**: Study TikTok content characteristics and metadata patterns

---

**Note**: This tool is designed for personal use and organization of your own media files. Please respect copyright and terms of service when handling any content.
