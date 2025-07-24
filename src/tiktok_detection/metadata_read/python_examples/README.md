# TikTok Detection Scripts

## Overview

These scripts can analyze media files and detect if they originated from TikTok by examining their metadata for TikTok-specific identifiers.

## Files

### 1. `metadata_extractor.py` - Full Metadata Analysis
**Complete metadata extraction with TikTok detection**

```bash
python metadata_extractor.py filename.mp4
```

**Features:**
- Extracts ALL metadata from media files
- Shows comprehensive TikTok analysis at the top
- Outputs full JSON metadata
- Perfect for forensic analysis

### 2. `tiktok_detector.py` - Simple TikTok Detection
**Quick TikTok detection with clean output**

```bash
python tiktok_detector.py filename.mp4
```

**Features:**
- Clean, easy-to-read output
- Shows confidence score (0-100+)
- Lists key evidence found
- Exit codes: 0=TikTok detected, 1=No TikTok, 2=Error

## TikTok Detection Criteria

The scripts look for these TikTok-specific indicators:

### 🔴 **Definitive Evidence (High Confidence)**
- **AIGC Metadata**: `{"aigc_label_type":0}` - TikTok's AI content labeling
- **Video IDs**: `vid:v[digits]gf0000[hash]` - TikTok's internal video tracking
- **Content Hashes**: `vid_md5` field - ByteDance content fingerprinting

### 🟡 **Supporting Evidence (Medium Confidence)**
- **Dimensions**: 576x1024, 576x1246 (TikTok standard sizes)
- **Aspect Ratio**: 9:16 vertical mobile format
- **Encoder**: `Lavf58.76.100` (common in TikTok pipeline)

### 🟢 **Additional Indicators (Low Confidence)**
- **String Analysis**: Direct mentions of "tiktok", "douyin", "bytedance"

## Confidence Scoring

- **70+ points**: CONFIRMED TikTok file
- **40-69 points**: LIKELY TikTok file  
- **20-39 points**: POSSIBLE TikTok characteristics
- **0-19 points**: UNLIKELY to be TikTok

## Example Results

### ✅ TikTok File Detected
```
🎯 RESULT: CONFIRMED: File is from TikTok
📊 Confidence: 335/100
🔎 Evidence (12 items):
   1. AIGC metadata found
   2. TikTok video ID found
   3. ByteDance content hash found
   4. TikTok-typical dimensions: 576x1246
📋 Key Identifiers:
   • TikTok Video ID: vid:v15044gf0000d183clnog65r11govn8g
   • Content Hash: 6cfe10518db384ab6a42a6ad0e0444de
   • AIGC Metadata: {"aigc_label_type":0}
```

### ❌ Non-TikTok File
```
🎯 RESULT: UNLIKELY: No significant TikTok evidence found
📊 Confidence: 0/100
❌ NOT DETECTED: No strong TikTok evidence found
```

## Privacy Implications

**TikTok embeds persistent tracking data in downloaded videos:**
- Internal video IDs that can trace back to original posts
- Content fingerprints for deduplication
- AI content labeling metadata
- Technical processing signatures

**To remove this metadata:**
```bash
ffmpeg -i tiktok_video.mp4 -c copy -map_metadata -1 clean_video.mp4
```

## Installation

**Basic usage** (works with built-in libraries):
```bash
python tiktok_detector.py video.mp4
```

**Enhanced analysis** (install optional dependencies):
```bash
pip install -r requirements.txt
python metadata_extractor.py video.mp4
```

## Use Cases

- **Digital Forensics**: Determine video source platforms
- **Content Verification**: Verify if content originated from TikTok
- **Privacy Analysis**: Understand what tracking data persists in downloads
- **Research**: Study platform-specific metadata patterns
- **Security**: Identify potentially manipulated or re-encoded content
