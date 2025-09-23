#!/usr/bin/env python3
"""
Detailed JPEG analysis to understand non-standard EXIF segments
"""

import sys
import os

def analyze_jpeg_detailed(file_path):
    """Detailed analysis of JPEG structure"""
    print(f"Detailed analysis of: {file_path}")
    
    with open(file_path, 'rb') as f:
        data = f.read()
    
    print(f"File size: {len(data)} bytes")
    
    # Check JPEG header
    if data[:2] != b'\xff\xd8':
        print("ERROR: Not a valid JPEG file")
        return
    
    print("✓ Valid JPEG header")
    
    # Look for all segments
    pos = 2
    segment_count = 0
    
    while pos < len(data) - 1:
        if data[pos] == 0xff:
            marker = data[pos + 1]
            
            # Skip padding bytes
            if marker == 0xff:
                pos += 1
                continue
            
            segment_count += 1
            
            # Get segment name
            marker_names = {
                0xe0: "APP0 (JFIF)",
                0xe1: "APP1 (EXIF/Other)",
                0xe2: "APP2 (FlashPix)",
                0xe3: "APP3 (Other)",
                0xe4: "APP4 (Other)",
                0xe5: "APP5 (Other)",
                0xe6: "APP6 (Other)",
                0xe7: "APP7 (Other)",
                0xe8: "APP8 (Other)",
                0xe9: "APP9 (Other)",
                0xea: "APP10 (Other)",
                0xeb: "APP11 (Other)",
                0xec: "APP12 (Other)",
                0xed: "APP13 (Other)",
                0xee: "APP14 (Other)",
                0xef: "APP15 (Other)",
                0xfe: "COM (Comment)",
                0xd9: "EOI (End of Image)"
            }
            
            marker_name = marker_names.get(marker, f"Unknown (0xff{marker:02x})")
            
            if marker == 0xd9:  # End of image
                print(f"  Found end marker at position {pos}")
                break
            
            # Read segment length
            if pos + 3 < len(data):
                length = (data[pos + 2] << 8) | data[pos + 3]
                print(f"\nSegment {segment_count}: {marker_name} at position {pos}")
                print(f"  Length: {length}")
                
                # Show segment content for APP segments
                if 0xe0 <= marker <= 0xef:  # APP0-APP15
                    if pos + 4 + min(50, length - 2) < len(data):
                        content_start = pos + 4
                        content_end = content_start + min(50, length - 2)
                        content = data[content_start:content_end]
                        
                        print(f"  Content (first 50 bytes): {content.hex()}")
                        print(f"  Content (as string): {content.decode('utf-8', errors='ignore')}")
                        
                        # Look for EXIF signature
                        if b'Exif' in content:
                            exif_pos = content.find(b'Exif')
                            print(f"  ✓ Found 'Exif' at offset {exif_pos}")
                            
                            # Check TIFF header
                            tiff_start = content_start + exif_pos + 4
                            if tiff_start + 8 < len(data):
                                tiff_header = data[tiff_start:tiff_start + 8]
                                print(f"  TIFF header: {tiff_header.hex()}")
                                
                                if tiff_header[:2] in [b'II', b'MM']:
                                    print("  ✓ Valid TIFF header")
                                else:
                                    print("  ✗ Invalid TIFF header")
                
                pos += length + 2
            else:
                pos += 2
        else:
            pos += 1
    
    print(f"\nTotal segments found: {segment_count}")

if __name__ == "__main__":
    test_file = "/keg/pictures/incoming/2025/09-Sep/20250907_132644.050.jpg"
    analyze_jpeg_detailed(test_file)

