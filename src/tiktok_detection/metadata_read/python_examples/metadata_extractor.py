#!/usr/bin/env python3
"""
Comprehensive Metadata Extractor
Extracts and outputs all available metadata from media files in JSON format.
Particularly useful for investigating TikTok MP4 files for source identification.
"""

import json
import os
import sys
import hashlib
from datetime import datetime
from pathlib import Path

# Try to import optional libraries for enhanced metadata extraction
try:
    from PIL import Image
    from PIL.ExifTags import TAGS
    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False

try:
    import mutagen
    from mutagen.id3 import ID3NoHeaderError
    MUTAGEN_AVAILABLE = True
except ImportError:
    MUTAGEN_AVAILABLE = False

try:
    import ffmpeg
    FFMPEG_AVAILABLE = True
except ImportError:
    FFMPEG_AVAILABLE = False

try:
    import exifread
    EXIFREAD_AVAILABLE = True
except ImportError:
    EXIFREAD_AVAILABLE = False


def get_file_basic_info(filepath):
    """Extract basic file system information"""
    stat = os.stat(filepath)
    return {
        "filename": os.path.basename(filepath),
        "filepath": str(filepath),
        "size_bytes": stat.st_size,
        "size_human": format_bytes(stat.st_size),
        "created": datetime.fromtimestamp(stat.st_ctime).isoformat(),
        "modified": datetime.fromtimestamp(stat.st_mtime).isoformat(),
        "accessed": datetime.fromtimestamp(stat.st_atime).isoformat(),
        "permissions": oct(stat.st_mode)[-3:],
        "md5_hash": get_file_hash(filepath, 'md5'),
        "sha256_hash": get_file_hash(filepath, 'sha256')
    }


def format_bytes(bytes_value):
    """Convert bytes to human readable format"""
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if bytes_value < 1024.0:
            return f"{bytes_value:.2f} {unit}"
        bytes_value /= 1024.0
    return f"{bytes_value:.2f} PB"


def get_file_hash(filepath, algorithm='md5'):
    """Calculate file hash"""
    hash_obj = hashlib.new(algorithm)
    with open(filepath, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_obj.update(chunk)
    return hash_obj.hexdigest()


def extract_exif_pil(filepath):
    """Extract EXIF data using PIL"""
    if not PIL_AVAILABLE:
        return None
    
    try:
        with Image.open(filepath) as image:
            exifdata = image.getexif()
            if not exifdata:
                return None
            
            exif_dict = {}
            for tag_id in exifdata:
                tag = TAGS.get(tag_id, tag_id)
                data = exifdata.get(tag_id)
                if isinstance(data, bytes):
                    try:
                        data = data.decode('utf-8')
                    except UnicodeDecodeError:
                        data = str(data)
                exif_dict[tag] = data
            return exif_dict
    except Exception as e:
        return {"error": str(e)}


def extract_exif_exifread(filepath):
    """Extract EXIF data using exifread library"""
    if not EXIFREAD_AVAILABLE:
        return None
    
    try:
        with open(filepath, 'rb') as f:
            tags = exifread.process_file(f)
            if not tags:
                return None
            
            exif_dict = {}
            for tag in tags.keys():
                if tag not in ['JPEGThumbnail', 'TIFFThumbnail', 'Filename', 'EXIF MakerNote']:
                    value = str(tags[tag])
                    exif_dict[tag] = value
            return exif_dict
    except Exception as e:
        return {"error": str(e)}


def extract_mutagen_metadata(filepath):
    """Extract metadata using mutagen"""
    if not MUTAGEN_AVAILABLE:
        return None
    
    try:
        audiofile = mutagen.File(filepath)
        if audiofile is None:
            return None
        
        metadata = {}
        for key, value in audiofile.items():
            if isinstance(value, list):
                value = [str(v) for v in value]
            else:
                value = str(value)
            metadata[key] = value
        
        # Add format info
        if hasattr(audiofile, 'info'):
            info = audiofile.info
            metadata['_format_info'] = {
                'bitrate': getattr(info, 'bitrate', None),
                'length': getattr(info, 'length', None),
                'sample_rate': getattr(info, 'sample_rate', None),
                'channels': getattr(info, 'channels', None),
                'format': str(type(info).__name__)
            }
        
        return metadata
    except Exception as e:
        return {"error": str(e)}


def extract_ffmpeg_metadata(filepath):
    """Extract metadata using ffmpeg"""
    if not FFMPEG_AVAILABLE:
        return None
    
    try:
        probe = ffmpeg.probe(filepath)
        return {
            'format': probe.get('format', {}),
            'streams': probe.get('streams', [])
        }
    except Exception as e:
        return {"error": str(e)}


def extract_video_metadata_ffprobe(filepath):
    """Extract video metadata using ffprobe command line"""
    try:
        import subprocess
        import json
        
        # Try to use ffprobe from the yt-to-mp3 directory first
        ffprobe_paths = [
            '/home/tegran-grigorian/Downloads/yt-to-mp3-4-linux-x86/bin/linux/ffprobe',
            'ffprobe'  # fallback to system ffprobe
        ]
        
        for ffprobe_path in ffprobe_paths:
            try:
                cmd = [
                    ffprobe_path,
                    '-v', 'quiet',
                    '-print_format', 'json',
                    '-show_format',
                    '-show_streams',
                    '-show_chapters',
                    '-show_programs',
                    str(filepath)
                ]
                
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
                if result.returncode == 0:
                    return json.loads(result.stdout)
                    
            except (subprocess.TimeoutExpired, FileNotFoundError):
                continue
                
        return {"error": "ffprobe not available"}
    except Exception as e:
        return {"error": str(e)}


def extract_hex_header(filepath, bytes_to_read=512):
    """Extract file header in hex format to identify file signatures"""
    try:
        with open(filepath, 'rb') as f:
            header_bytes = f.read(bytes_to_read)
            return {
                'hex': header_bytes.hex(),
                'ascii': ''.join(chr(b) if 32 <= b <= 126 else '.' for b in header_bytes),
                'magic_number': header_bytes[:16].hex()
            }
    except Exception as e:
        return {"error": str(e)}


def search_for_strings(filepath, min_length=4, max_search_bytes=1024*1024):
    """Search for readable strings in the file that might indicate source"""
    try:
        strings_found = []
        tiktok_indicators = []
        
        with open(filepath, 'rb') as f:
            data = f.read(max_search_bytes)
            
        current_string = ""
        for byte in data:
            if 32 <= byte <= 126:  # Printable ASCII
                current_string += chr(byte)
            else:
                if len(current_string) >= min_length:
                    strings_found.append(current_string)
                    # Look for TikTok-related strings
                    lower_string = current_string.lower()
                    if any(indicator in lower_string for indicator in 
                          ['tiktok', 'douyin', 'bytedance', 'musically', 'musical.ly']):
                        tiktok_indicators.append(current_string)
                current_string = ""
        
        # Don't forget the last string
        if len(current_string) >= min_length:
            strings_found.append(current_string)
            lower_string = current_string.lower()
            if any(indicator in lower_string for indicator in 
                  ['tiktok', 'douyin', 'bytedance', 'musically', 'musical.ly']):
                tiktok_indicators.append(current_string)
        
        return {
            'total_strings_found': len(strings_found),
            'strings': strings_found[:50],  # Limit to first 50 strings
            'tiktok_indicators': tiktok_indicators,
            'search_bytes': min(len(data), max_search_bytes)
        }
    except Exception as e:
        return {"error": str(e)}


def analyze_tiktok_evidence(metadata):
    """Analyze metadata for TikTok-specific evidence and return detection results"""
    evidence = {
        "is_tiktok": False,
        "confidence_score": 0,
        "evidence_found": [],
        "indicators": {},
        "verdict": ""
    }
    
    # Check for definitive TikTok markers
    tiktok_markers = []
    
    # 1. Check for AIGC metadata (TikTok-specific)
    if "mutagen" in metadata:
        mutagen_data = metadata["mutagen"]
        for key, value in mutagen_data.items():
            if isinstance(value, list):
                value_str = " ".join(str(v) for v in value)
            else:
                value_str = str(value)
            
            if "aigc_label_type" in value_str.lower():
                tiktok_markers.append("AIGC metadata found")
                evidence["indicators"]["aigc_metadata"] = value_str
                evidence["confidence_score"] += 40
    
    # Also check ffmpeg/ffprobe data for AIGC
    for section in ["ffmpeg", "ffprobe"]:
        if section in metadata and "format" in metadata[section]:
            tags = metadata[section]["format"].get("tags", {})
            if "aigc_info" in tags:
                tiktok_markers.append("AIGC info in format tags")
                evidence["indicators"]["aigc_info"] = tags["aigc_info"]
                evidence["confidence_score"] += 40
    
    # 2. Check for TikTok video IDs (vid: prefix)
    for section in ["mutagen", "ffmpeg", "ffprobe"]:
        if section in metadata:
            data = metadata[section]
            if section == "mutagen":
                for key, value in data.items():
                    value_str = str(value)
                    if "vid:" in value_str and ("gf0000" in value_str or "gl0000" in value_str):
                        tiktok_markers.append("TikTok video ID found")
                        evidence["indicators"]["tiktok_video_id"] = value_str
                        evidence["confidence_score"] += 35
            elif "format" in data and "tags" in data["format"]:
                tags = data["format"]["tags"]
                if "comment" in tags and "vid:" in tags["comment"]:
                    tiktok_markers.append("TikTok video ID in comment tag")
                    evidence["indicators"]["tiktok_video_id"] = tags["comment"]
                    evidence["confidence_score"] += 35
    
    # 3. Check for vid_md5 (ByteDance content hash)
    for section in ["ffmpeg", "ffprobe"]:
        if section in metadata and "format" in metadata[section]:
            tags = metadata[section]["format"].get("tags", {})
            if "vid_md5" in tags:
                tiktok_markers.append("ByteDance content hash found")
                evidence["indicators"]["vid_md5"] = tags["vid_md5"]
                evidence["confidence_score"] += 30
    
    # 4. Check video dimensions (typical TikTok sizes)
    for section in ["ffmpeg", "ffprobe"]:
        if section in metadata and "streams" in metadata[section]:
            for stream in metadata[section]["streams"]:
                if stream.get("codec_type") == "video":
                    width = stream.get("width")
                    height = stream.get("height")
                    if width and height:
                        # TikTok common dimensions: 576x1024, 576x1246, etc.
                        if width == 576 and height in [1024, 1246, 1280]:
                            tiktok_markers.append(f"TikTok-typical dimensions: {width}x{height}")
                            evidence["indicators"]["video_dimensions"] = f"{width}x{height}"
                            evidence["confidence_score"] += 15
                        # 9:16 aspect ratio check
                        aspect_ratio = width / height if height > 0 else 0
                        if 0.55 <= aspect_ratio <= 0.58:  # Approximately 9:16
                            tiktok_markers.append("Vertical mobile video format")
                            evidence["indicators"]["aspect_ratio"] = f"{width}:{height}"
                            evidence["confidence_score"] += 10
    
    # 5. Check for specific encoder signatures
    for section in ["ffmpeg", "ffprobe"]:
        if section in metadata and "format" in metadata[section]:
            tags = metadata[section]["format"].get("tags", {})
            if "encoder" in tags and "Lavf58.76.100" in tags["encoder"]:
                tiktok_markers.append("TikTok-associated encoder version")
                evidence["indicators"]["encoder"] = tags["encoder"]
                evidence["confidence_score"] += 10
    
    # 6. Check string analysis for TikTok indicators
    if "string_analysis" in metadata and metadata["string_analysis"].get("tiktok_indicators"):
        indicators = metadata["string_analysis"]["tiktok_indicators"]
        if indicators:
            tiktok_markers.append("TikTok strings found in file")
            evidence["indicators"]["string_indicators"] = indicators
            evidence["confidence_score"] += 20
    
    # Set evidence found
    evidence["evidence_found"] = tiktok_markers
    
    # Determine verdict based on confidence score
    if evidence["confidence_score"] >= 70:
        evidence["is_tiktok"] = True
        evidence["verdict"] = "CONFIRMED: File is from TikTok"
    elif evidence["confidence_score"] >= 40:
        evidence["is_tiktok"] = True
        evidence["verdict"] = "LIKELY: Strong evidence suggests TikTok origin"
    elif evidence["confidence_score"] >= 20:
        evidence["verdict"] = "POSSIBLE: Some TikTok-like characteristics found"
    else:
        evidence["verdict"] = "UNLIKELY: No significant TikTok evidence found"
    
    return evidence


def extract_all_metadata(filepath):
    """Extract all available metadata from a file"""
    if not os.path.exists(filepath):
        return {"error": f"File not found: {filepath}"}
    
    metadata = {
        "extraction_timestamp": datetime.now().isoformat(),
        "extractor_version": "1.1",
        "file_info": get_file_basic_info(filepath),
        "hex_header": extract_hex_header(filepath),
        "string_analysis": search_for_strings(filepath),
    }
    
    # Try different metadata extraction methods
    exif_pil = extract_exif_pil(filepath)
    if exif_pil:
        metadata["exif_pil"] = exif_pil
    
    exif_exifread = extract_exif_exifread(filepath)
    if exif_exifread:
        metadata["exif_exifread"] = exif_exifread
    
    mutagen_data = extract_mutagen_metadata(filepath)
    if mutagen_data:
        metadata["mutagen"] = mutagen_data
    
    ffmpeg_data = extract_ffmpeg_metadata(filepath)
    if ffmpeg_data:
        metadata["ffmpeg"] = ffmpeg_data
    
    ffprobe_data = extract_video_metadata_ffprobe(filepath)
    if ffprobe_data:
        metadata["ffprobe"] = ffprobe_data
    
    # Library availability info
    metadata["libraries_available"] = {
        "PIL": PIL_AVAILABLE,
        "mutagen": MUTAGEN_AVAILABLE,
        "ffmpeg": FFMPEG_AVAILABLE,
        "exifread": EXIFREAD_AVAILABLE
    }
    
    # Add TikTok detection analysis
    metadata["tiktok_analysis"] = analyze_tiktok_evidence(metadata)
    
    return metadata


def main():
    if len(sys.argv) != 2:
        print("Usage: python metadata_extractor.py <file_path>")
        print("Example: python metadata_extractor.py Download.mp4")
        sys.exit(1)
    
    filepath = sys.argv[1]
    
    # Convert to absolute path
    if not os.path.isabs(filepath):
        filepath = os.path.abspath(filepath)
    
    print(f"Analyzing file: {filepath}")
    print("=" * 50)
    
    metadata = extract_all_metadata(filepath)
    
    # Show TikTok analysis first
    if "tiktok_analysis" in metadata:
        analysis = metadata["tiktok_analysis"]
        print(f"🎯 TIKTOK DETECTION RESULT: {analysis['verdict']}")
        print(f"📊 Confidence Score: {analysis['confidence_score']}/100")
        
        if analysis["evidence_found"]:
            print(f"🔍 Evidence Found:")
            for i, evidence in enumerate(analysis["evidence_found"], 1):
                print(f"   {i}. {evidence}")
        
        if analysis["indicators"]:
            print(f"📝 Key Indicators:")
            for key, value in analysis["indicators"].items():
                print(f"   • {key}: {value}")
        
        print("\n" + "=" * 50)
        print("FULL METADATA:")
    
    # Output full metadata as formatted JSON
    print(json.dumps(metadata, indent=2, default=str))


if __name__ == "__main__":
    main()