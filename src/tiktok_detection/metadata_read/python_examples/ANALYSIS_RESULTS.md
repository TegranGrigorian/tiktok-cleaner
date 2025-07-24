# TikTok MP4 Metadata Analysis Results

## Key Findings from Your MP4 Files

### Evidence of TikTok/ByteDance Origin

Both of your MP4 files contain **strong indicators** that they originated from TikTok/ByteDance:

#### 1. **AIGC (AI Generated Content) Metadata**
```json
"aigc_info": "{\"aigc_label_type\":0}"
```
- This is a **TikTok-specific metadata field**
- AIGC stands for "AI Generated Content"
- ByteDance uses this to track content generation methods
- `aigc_label_type: 0` typically means the content is not AI-generated

#### 2. **TikTok Video IDs**
- **Download.mp4**: `vid:v15044gf0000d183clnog65r11govn8g`
- **Download(1).mp4**: `vid:v24044gl0000d1ddm77og65p3h2g1jm0`

These follow TikTok's internal video ID format:
- `vid:` prefix is TikTok-specific
- The format `v[number]gf0000[hash]` is characteristic of TikTok's system
- These IDs can potentially be used to trace back to the original TikTok post

#### 3. **Video MD5 Hashes**
```json
"vid_md5": "6cfe10518db384ab6a42a6ad0e0444de"  // Download.mp4
"vid_md5": "5210b9619ac8b3f3c144774aea704e61"  // Download(1).mp4
```
- These are **TikTok's internal content hashes**
- Used by ByteDance for content deduplication and tracking
- Not standard MP4 metadata - specific to TikTok's processing pipeline

#### 4. **Technical Characteristics Typical of TikTok**
- **Vertical aspect ratios**: 
  - Download.mp4: 288:623 (approximately 9:19.5)
  - Download(1).mp4: 9:16 (standard TikTok format)
- **Encoder**: `Lavf58.76.100` (FFmpeg version commonly used by TikTok)
- **Container format**: ISO MP4 with specific brand markers

#### 5. **Video Dimensions**
- **576x1246** and **576x1024** - typical TikTok mobile video dimensions
- 30fps frame rate - standard for TikTok content

## Conclusion

**YES, these MP4 files can absolutely be traced back to TikTok** via their metadata. The presence of:
- TikTok-specific AIGC metadata
- Internal TikTok video IDs
- ByteDance content hashes
- Characteristic technical parameters

...provides clear forensic evidence of TikTok origin.

## Privacy Implications

This metadata reveals:
1. **Origin platform** (TikTok/ByteDance)
2. **Internal tracking IDs** that could potentially link to user accounts
3. **Content processing history** within TikTok's systems
4. **Technical fingerprinting** that persists even after download

## Recommendations

If you want to remove this identifying metadata:
1. Re-encode the video with `ffmpeg -i input.mp4 -c copy -map_metadata -1 output.mp4`
2. Use tools like `exiftool` to strip metadata
3. Convert through other formats that don't preserve custom metadata fields

The metadata extraction script provided will help you analyze any media files for similar tracking information.
