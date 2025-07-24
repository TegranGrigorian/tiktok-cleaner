#!/usr/bin/env python3
"""
TikTok Image Detection Analysis
Specialized script for analyzing image files for TikTok origin indicators
"""

import os
import sys
from datetime import datetime

# Import our existing detection functions
from metadata_extractor import extract_all_metadata, analyze_tiktok_evidence

def analyze_image_for_tiktok_indicators(filepath):
    """Enhanced analysis specifically for TikTok image indicators"""
    
    print(f"🔍 Analyzing: {os.path.basename(filepath)}")
    print("=" * 60)
    
    # Get basic metadata
    metadata = extract_all_metadata(filepath)
    
    # Standard TikTok analysis
    tiktok_analysis = analyze_tiktok_evidence(metadata)
    
    # Additional image-specific indicators
    image_indicators = {
        "dimensions": None,
        "aspect_ratio": None,
        "file_format": None,
        "file_size": None,
        "typical_tiktok_characteristics": []
    }
    
    # Extract image dimensions and format info
    if "ffmpeg" in metadata and "streams" in metadata["ffmpeg"]:
        for stream in metadata["ffmpeg"]["streams"]:
            if stream.get("codec_type") == "video" or stream.get("codec_name") in ["webp", "mjpeg"]:
                width = stream.get("width")
                height = stream.get("height")
                if width and height:
                    image_indicators["dimensions"] = f"{width}x{height}"
                    aspect_ratio = width / height
                    image_indicators["aspect_ratio"] = f"{aspect_ratio:.3f}"
                    
                    # Check for TikTok-typical image characteristics
                    if height > width:  # Portrait orientation
                        image_indicators["typical_tiktok_characteristics"].append("Portrait orientation")
                        
                    # Common TikTok screenshot dimensions
                    tiktok_dimensions = [
                        (1080, 1920), (1080, 1800), (1080, 2340), (1080, 2400),
                        (828, 1792), (750, 1334), (1125, 2436), (1242, 2688),
                        (1284, 2778), (1170, 2532)  # Common phone screen sizes
                    ]
                    
                    if (width, height) in tiktok_dimensions:
                        image_indicators["typical_tiktok_characteristics"].append(f"Common mobile screenshot size: {width}x{height}")
                        tiktok_analysis["confidence_score"] += 25
                        tiktok_analysis["evidence_found"].append(f"Mobile screenshot dimensions: {width}x{height}")
                    
                    # Check for 9:16 aspect ratio (TikTok standard)
                    if 0.55 <= aspect_ratio <= 0.58:  # Approximately 9:16
                        image_indicators["typical_tiktok_characteristics"].append("9:16 aspect ratio (TikTok standard)")
    
    # File format analysis
    if "file_info" in metadata:
        file_info = metadata["file_info"]
        image_indicators["file_size"] = file_info.get("size_human", "Unknown")
        
        # Check file extension vs actual format
        filename = file_info.get("filename", "")
        if filename.endswith('.png') and "webp" in str(metadata.get("ffmpeg", {})).lower():
            image_indicators["typical_tiktok_characteristics"].append("WebP format with PNG extension (common in TikTok screenshots)")
            tiktok_analysis["confidence_score"] += 15
            tiktok_analysis["evidence_found"].append("WebP format with PNG extension")
    
    # Check for suspicious filename patterns
    filename = os.path.basename(filepath)
    if len(filename) == 36 and filename.count('.') == 1:  # 32 chars + extension = MD5-like hash
        image_indicators["typical_tiktok_characteristics"].append("MD5-like hash filename (common in app downloads)")
        tiktok_analysis["confidence_score"] += 10
        tiktok_analysis["evidence_found"].append("Hash-based filename pattern")
    
    # Re-evaluate verdict with image-specific evidence
    if tiktok_analysis["confidence_score"] >= 50:
        tiktok_analysis["is_tiktok"] = True
        tiktok_analysis["verdict"] = "LIKELY: Strong evidence suggests TikTok origin"
    elif tiktok_analysis["confidence_score"] >= 25:
        tiktok_analysis["verdict"] = "POSSIBLE: Some TikTok-like characteristics found"
    
    # Display results
    print(f"🎯 TIKTOK DETECTION RESULT: {tiktok_analysis['verdict']}")
    print(f"📊 Confidence Score: {tiktok_analysis['confidence_score']}/100")
    
    if tiktok_analysis["evidence_found"]:
        print(f"\n🔍 Evidence Found ({len(tiktok_analysis['evidence_found'])} items):")
        for i, evidence in enumerate(tiktok_analysis["evidence_found"], 1):
            print(f"   {i}. {evidence}")
    
    if image_indicators["typical_tiktok_characteristics"]:
        print(f"\n📱 Image Characteristics:")
        for char in image_indicators["typical_tiktok_characteristics"]:
            print(f"   • {char}")
    
    print(f"\n📋 Technical Details:")
    print(f"   • Dimensions: {image_indicators['dimensions']}")
    print(f"   • Aspect Ratio: {image_indicators['aspect_ratio']}")
    print(f"   • File Size: {image_indicators['file_size']}")
    
    if "ffmpeg" in metadata and "format" in metadata["ffmpeg"]:
        format_info = metadata["ffmpeg"]["format"]
        print(f"   • Format: {format_info.get('format_long_name', 'Unknown')}")
    
    return tiktok_analysis

def main():
    if len(sys.argv) < 2:
        print("Usage: python image_tiktok_analyzer.py <image1> [image2] ...")
        print("Example: python image_tiktok_analyzer.py image.png")
        sys.exit(1)
    
    for filepath in sys.argv[1:]:
        if not os.path.exists(filepath):
            print(f"❌ File not found: {filepath}")
            continue
            
        analyze_image_for_tiktok_indicators(filepath)
        print("\n" + "="*60 + "\n")

if __name__ == "__main__":
    main()
