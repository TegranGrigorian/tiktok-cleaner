#!/usr/bin/env python3
"""
TikTok Detector - Simple version that just detects if a file is from TikTok
Usage: python tiktok_detector.py <file_path>
"""

import sys
import os

# Import the main metadata extractor functions
from metadata_extractor import extract_all_metadata, analyze_tiktok_evidence

def main():
    if len(sys.argv) != 2:
        print("Usage: python tiktok_detector.py <file_path>")
        print("Example: python tiktok_detector.py Download.mp4")
        sys.exit(1)
    
    filepath = sys.argv[1]
    
    # Convert to absolute path
    if not os.path.isabs(filepath):
        filepath = os.path.abspath(filepath)
    
    if not os.path.exists(filepath):
        print(f"❌ ERROR: File not found: {filepath}")
        sys.exit(1)
    
    print(f"🔍 Analyzing: {os.path.basename(filepath)}")
    print("-" * 50)
    
    # Extract metadata (suppress output)
    try:
        metadata = extract_all_metadata(filepath)
        analysis = analyze_tiktok_evidence(metadata)
        
        # Display results
        print(f"🎯 RESULT: {analysis['verdict']}")
        print(f"📊 Confidence: {analysis['confidence_score']}/100")
        
        if analysis["evidence_found"]:
            print(f"\n🔎 Evidence ({len(analysis['evidence_found'])} items):")
            for i, evidence in enumerate(analysis["evidence_found"], 1):
                print(f"   {i}. {evidence}")
        
        if analysis["indicators"]:
            print(f"\n📋 Key Identifiers:")
            for key, value in analysis["indicators"].items():
                if key == "tiktok_video_id":
                    print(f"   • TikTok Video ID: {value}")
                elif key == "vid_md5":
                    print(f"   • Content Hash: {value}")
                elif key == "aigc_info":
                    print(f"   • AIGC Metadata: {value}")
                elif key == "video_dimensions":
                    print(f"   • Dimensions: {value}")
        
        # Exit code based on result
        if analysis["is_tiktok"]:
            print(f"\n✅ CONFIRMED: This file originated from TikTok")
            sys.exit(0)
        else:
            print(f"\n❌ NOT DETECTED: No strong TikTok evidence found")
            sys.exit(1)
            
    except Exception as e:
        print(f"❌ ERROR: Failed to analyze file - {str(e)}")
        sys.exit(2)

if __name__ == "__main__":
    main()
