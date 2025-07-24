# File Utilities Module

This module provides file management, caching, and organization utilities for the TikTok detection system.

## Files

### `file_manager.rs`
Enhanced file management with intelligent caching and phone filesystem support:

#### Core Features
- **Intelligent Caching**: Tracks file metadata (size, modification time) to avoid re-analyzing unchanged files
- **Phone Compatibility**: Handles MTP filesystem limitations with fallback strategies
- **Cache Management**: JSON-based cache with file integrity verification
- **Organization**: Moves or copies files to confidence-based folders

#### Key Functions
- **`new()`**: Initializes file manager with cache loading
- **`add_to_cache()`**: Adds file analysis results to cache with metadata
- **`is_file_cached()`**: Checks if file needs re-analysis based on metadata
- **`move_file_to_tiktok_folder()`**: Organizes files into confidence-based folders
- **`create_phone_organization_guide()`**: Creates manual organization guide for MTP limitations
- **`get_cache_stats()`**: Returns cache statistics for performance monitoring

#### Caching Strategy
The cache stores:
```json
{
  "file_path": {
    "size": 1234567,
    "modified": "2025-01-15T10:30:00Z",
    "confidence": 75,
    "is_tiktok": true
  }
}
```

#### Phone Filesystem Handling
- **MTP Detection**: Automatically detects MTP-mounted phone storage
- **Fallback Mode**: Creates local detection records when direct file moves fail
- **Organization Guide**: Generates manual instructions for phone-based organization

### `folder_manager.rs`
Folder organization and structure management:

#### Functions
- **`create_tiktok_folders()`**: Creates confidence-based folder structure
- **`get_confidence_folder()`**: Maps confidence scores to appropriate folders
- **`ensure_folder_exists()`**: Creates folders as needed with error handling

#### Folder Structure
```
scan_directory/
├── tiktok_detection/
│   ├── confirmed/     # 70%+ confidence
│   ├── likely/        # 40-69% confidence
│   └── possible/      # 20-39% confidence
└── .tiktok_cache.json # Cache file
```

## Usage Examples

### Basic File Management
```rust
use crate::tiktok_detection::file_util::file_manager::FileManager;

let mut file_manager = FileManager::new("/scan/path")?;

// Check if file needs analysis
if !file_manager.is_file_cached(&file_path)? {
    // Analyze file...
    let confidence = analyze_file(&file_path)?;
    
    // Add to cache
    file_manager.add_to_cache(&file_path, confidence, confidence >= 20)?;
    
    // Organize if TikTok content
    if confidence >= 20 {
        file_manager.move_file_to_tiktok_folder(&file_path, confidence)?;
    }
}
```

### Phone-Specific Handling
```rust
// The file manager automatically detects phone filesystems
let result = file_manager.move_file_to_tiktok_folder(&phone_file, confidence);

match result {
    Ok(path) => println!("File moved to: {}", path.display()),
    Err(_) => {
        // Fallback: create organization guide
        file_manager.create_phone_organization_guide(&detected_files)?;
        println!("Manual organization guide created");
    }
}
```

## Performance Features

- **Cache Efficiency**: Only re-analyzes files that have changed
- **Batch Operations**: Optimized for processing thousands of files
- **Memory Management**: Minimal memory footprint for large file sets
- **Error Recovery**: Graceful handling of filesystem limitations
