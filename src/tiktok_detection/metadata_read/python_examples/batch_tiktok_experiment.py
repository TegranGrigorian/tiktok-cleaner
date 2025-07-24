#!/usr/bin/env python3
"""
Batch TikTok Detection Experiment
Analyzes multiple folders of images to detect TikTok origins
"""

import os
import sys
import glob
from collections import defaultdict

# Import our existing detection functions
from image_tiktok_analyzer import analyze_image_for_tiktok_indicators
from metadata_extractor import extract_all_metadata, analyze_tiktok_evidence

def analyze_folder(folder_path, folder_name):
    """Analyze all images in a folder for TikTok indicators"""
    
    print(f"\n🔍 ANALYZING FOLDER: {folder_name}")
    print("=" * 80)
    
    # Supported image extensions
    image_extensions = ['*.jpg', '*.jpeg', '*.png', '*.webp', '*.gif', '*.bmp']
    
    # Find all image files
    image_files = []
    for ext in image_extensions:
        image_files.extend(glob.glob(os.path.join(folder_path, ext)))
        image_files.extend(glob.glob(os.path.join(folder_path, ext.upper())))
    
    if not image_files:
        print(f"❌ No image files found in {folder_name}")
        return []
    
    print(f"📁 Found {len(image_files)} image files")
    
    results = []
    tiktok_count = 0
    likely_count = 0
    possible_count = 0
    unlikely_count = 0
    
    for i, filepath in enumerate(sorted(image_files), 1):
        filename = os.path.basename(filepath)
        print(f"\n📸 [{i}/{len(image_files)}] Analyzing: {filename}")
        print("-" * 60)
        
        try:
            # Get metadata and analyze
            metadata = extract_all_metadata(filepath)
            analysis = analyze_tiktok_evidence(metadata)
            
            # Enhanced image-specific analysis
            image_indicators = analyze_image_specific_indicators(filepath, metadata, analysis)
            
            # Store results
            result = {
                'filename': filename,
                'filepath': filepath,
                'confidence_score': analysis['confidence_score'],
                'verdict': analysis['verdict'],
                'is_tiktok': analysis['is_tiktok'],
                'evidence_found': analysis['evidence_found'],
                'indicators': analysis['indicators'],
                'image_indicators': image_indicators
            }
            results.append(result)
            
            # Count by verdict
            if 'CONFIRMED' in analysis['verdict'] or 'LIKELY' in analysis['verdict']:
                if analysis['confidence_score'] >= 70:
                    tiktok_count += 1
                else:
                    likely_count += 1
            elif 'POSSIBLE' in analysis['verdict']:
                possible_count += 1
            else:
                unlikely_count += 1
            
            # Display quick result
            confidence_icon = "🔴" if analysis['confidence_score'] >= 70 else "🟡" if analysis['confidence_score'] >= 40 else "🔵" if analysis['confidence_score'] >= 20 else "⚪"
            print(f"{confidence_icon} {analysis['verdict']} ({analysis['confidence_score']}/100)")
            
            if analysis['evidence_found']:
                print(f"   Evidence: {', '.join(analysis['evidence_found'][:3])}{'...' if len(analysis['evidence_found']) > 3 else ''}")
            
        except Exception as e:
            print(f"❌ Error analyzing {filename}: {str(e)}")
            result = {
                'filename': filename,
                'filepath': filepath,
                'error': str(e)
            }
            results.append(result)
    
    # Summary for this folder
    print(f"\n📊 FOLDER SUMMARY: {folder_name}")
    print("-" * 40)
    print(f"🔴 High Confidence TikTok: {tiktok_count}")
    print(f"🟡 Likely TikTok: {likely_count}")
    print(f"🔵 Possible TikTok: {possible_count}")
    print(f"⚪ Unlikely TikTok: {unlikely_count}")
    
    return results

def analyze_image_specific_indicators(filepath, metadata, analysis):
    """Enhanced image-specific analysis for TikTok detection"""
    
    indicators = {
        "dimensions": None,
        "aspect_ratio": None,
        "file_format": None,
        "file_size": None,
        "filename_pattern": None,
        "tiktok_characteristics": []
    }
    
    # Extract image dimensions and format info
    if "ffmpeg" in metadata and "streams" in metadata["ffmpeg"]:
        for stream in metadata["ffmpeg"]["streams"]:
            if stream.get("codec_type") == "video" or stream.get("codec_name") in ["webp", "mjpeg"]:
                width = stream.get("width")
                height = stream.get("height")
                if width and height:
                    indicators["dimensions"] = f"{width}x{height}"
                    aspect_ratio = width / height
                    indicators["aspect_ratio"] = f"{aspect_ratio:.3f}"
                    
                    # Check for TikTok-typical image characteristics
                    if height > width:  # Portrait orientation
                        indicators["tiktok_characteristics"].append("Portrait orientation")
                        
                    # Common TikTok screenshot dimensions
                    tiktok_dimensions = [
                        (1080, 1920), (1080, 1800), (1080, 2340), (1080, 2400),
                        (828, 1792), (750, 1334), (1125, 2436), (1242, 2688),
                        (1284, 2778), (1170, 2532)  # Common phone screen sizes
                    ]
                    
                    if (width, height) in tiktok_dimensions:
                        indicators["tiktok_characteristics"].append(f"Mobile screenshot size: {width}x{height}")
                        analysis["confidence_score"] += 25
                        analysis["evidence_found"].append(f"Mobile screenshot dimensions: {width}x{height}")
                    
                    # Check for 9:16 aspect ratio (TikTok standard)
                    if 0.55 <= aspect_ratio <= 0.58:  # Approximately 9:16
                        indicators["tiktok_characteristics"].append("9:16 aspect ratio (TikTok standard)")
    
    # File format analysis
    if "file_info" in metadata:
        file_info = metadata["file_info"]
        indicators["file_size"] = file_info.get("size_human", "Unknown")
        
        # Check file extension vs actual format
        filename = file_info.get("filename", "")
        if filename.endswith('.png') and "webp" in str(metadata.get("ffmpeg", {})).lower():
            indicators["tiktok_characteristics"].append("WebP format with PNG extension")
            analysis["confidence_score"] += 15
            analysis["evidence_found"].append("WebP format with PNG extension")
    
    # Check for suspicious filename patterns
    filename = os.path.basename(filepath)
    if len(filename) == 36 and filename.count('.') == 1:  # 32 chars + extension = MD5-like hash
        indicators["filename_pattern"] = "MD5-like hash"
        indicators["tiktok_characteristics"].append("Hash-based filename (app-generated)")
        analysis["confidence_score"] += 10
        analysis["evidence_found"].append("Hash-based filename pattern")
    elif filename.startswith(('IMG_', 'Screenshot_')):
        indicators["filename_pattern"] = "Standard mobile naming"
    else:
        indicators["filename_pattern"] = "Custom naming"
    
    return indicators

def generate_experiment_report(folder1_results, folder2_results, folder1_name, folder2_name):
    """Generate a comprehensive experiment report"""
    
    print(f"\n🎯 EXPERIMENT RESULTS SUMMARY")
    print("=" * 80)
    
    all_results = [
        (folder1_name, folder1_results),
        (folder2_name, folder2_results)
    ]
    
    total_tiktok = 0
    total_likely = 0
    total_possible = 0
    total_unlikely = 0
    
    for folder_name, results in all_results:
        tiktok_files = []
        likely_files = []
        possible_files = []
        unlikely_files = []
        
        for result in results:
            if 'error' in result:
                continue
                
            if result['confidence_score'] >= 70:
                tiktok_files.append(result)
                total_tiktok += 1
            elif result['confidence_score'] >= 40:
                likely_files.append(result)
                total_likely += 1
            elif result['confidence_score'] >= 20:
                possible_files.append(result)
                total_possible += 1
            else:
                unlikely_files.append(result)
                total_unlikely += 1
        
        print(f"\n📁 {folder_name.upper()}:")
        print(f"   🔴 High Confidence TikTok: {len(tiktok_files)}")
        if tiktok_files:
            for f in tiktok_files:
                print(f"      • {f['filename']} ({f['confidence_score']}/100)")
        
        print(f"   🟡 Likely TikTok: {len(likely_files)}")
        if likely_files:
            for f in likely_files:
                print(f"      • {f['filename']} ({f['confidence_score']}/100)")
        
        print(f"   🔵 Possible TikTok: {len(possible_files)}")
        print(f"   ⚪ Unlikely TikTok: {len(unlikely_files)}")
    
    print(f"\n🎯 OVERALL EXPERIMENT RESULTS:")
    print("-" * 40)
    print(f"🔴 Total High Confidence TikTok: {total_tiktok}")
    print(f"🟡 Total Likely TikTok: {total_likely}")
    print(f"🔵 Total Possible TikTok: {total_possible}")
    print(f"⚪ Total Unlikely TikTok: {total_unlikely}")
    
    total_files = total_tiktok + total_likely + total_possible + total_unlikely
    if total_files > 0:
        tiktok_percentage = ((total_tiktok + total_likely) / total_files) * 100
        print(f"\n📊 Detection Rate: {tiktok_percentage:.1f}% likely TikTok files")

def main():
    print("🧪 TIKTOK DETECTION EXPERIMENT")
    print("=" * 80)
    print("Testing our TikTok detection algorithm on two folders of images...")
    
    # Define folder paths
    base_path = "/home/tegran-grigorian/Downloads"
    folder1_path = os.path.join(base_path, "afolder")
    folder2_path = os.path.join(base_path, "afolder2")
    
    # Analyze both folders
    folder1_results = analyze_folder(folder1_path, "afolder")
    folder2_results = analyze_folder(folder2_path, "afolder2")
    
    # Generate comprehensive report
    generate_experiment_report(folder1_results, folder2_results, "afolder", "afolder2")
    
    print(f"\n✅ Experiment complete!")

if __name__ == "__main__":
    main()
