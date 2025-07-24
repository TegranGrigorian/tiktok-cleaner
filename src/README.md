# Source Code Documentation

This directory contains the main source code for the TikTok Cleaner application.

## Files Overview

### `main.rs`
The main entry point for the command-line application. Handles:
- Command-line argument parsing with clap
- Application initialization and configuration
- Coordination between scanning and test modes
- Error handling and user feedback

### `lib.rs`
Library module exports and public API definitions. Makes the core functionality available as a library for potential integration with other tools.

## Modules

### `tiktok_detection/`
Core detection and analysis module containing all TikTok identification algorithms, file management, and scanning logic. See `tiktok_detection/README.md` for detailed documentation.

### `bin/`
Additional binary utilities and standalone tools:
- `debug_image.rs`: Standalone image debugging utility for testing detection algorithms

## Usage Examples

### As a Library
```rust
use tiktok_cleaner::tiktok_detection::Scanner;

let scanner = Scanner::new("/path/to/scan", false)?;
let results = scanner.scan_folder_parallel()?;
```

### As a CLI Tool
```bash
cargo run --bin tiktok-cleaner -- --scan "/path/to/folder" --move
```

## Architecture Flow

1. **main.rs** parses CLI arguments and initializes the application
2. **Scanner** (from tiktok_detection) performs the actual file analysis
3. **Detection modules** analyze individual files for TikTok characteristics
4. **File utilities** handle caching, organization, and file operations
5. Results are presented to the user with confidence-based categorization
