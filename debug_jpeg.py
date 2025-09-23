#!/usr/bin/env python3
"""
Debug JPEG structure to understand why EXIF parsing fails
"""

import sys
import os

# Add the parent directory to path to import our module
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

def analyze_jpeg(file_path):
    """Analyze JPEG file structure"""
    print(f"Analyzing: {file_path}")
    
    with open(file_path, 'rb') as f:
        data = f.read()
    
    print(f"File size: {len(data)} bytes")
    
    # Check JPEG header
    if data[:2] != b'\xff\xd8':
        print("ERROR: Not a valid JPEG file")
        return
    
    print("✓ Valid JPEG header")
    
    # Look for APP1 segments (EXIF)
    pos = 2
    app1_count = 0
    
    while pos < len(data) - 1:
        if data[pos] == 0xff:
            marker = data[pos + 1]
            
            if marker == 0xe1:  # APP1 segment
                app1_count += 1
                print(f"\nFound APP1 segment at position {pos}")
                
                # Read segment length
                if pos + 3 < len(data):
                    length = (data[pos + 2] << 8) | data[pos + 3]
                    print(f"  Segment length: {length}")
                    
                    # Check if it's EXIF
                    if pos + 6 < len(data):
                        identifier = data[pos + 4:pos + 6]
                        print(f"  Identifier: {identifier}")
                        
                        if identifier == b'Exif':
                            print("  ✓ EXIF segment found!")
                            
                            # Show EXIF data start
                            exif_start = pos + 6
                            print(f"  EXIF data starts at: {exif_start}")
                            
                            # Check TIFF header
                            if exif_start + 8 < len(data):
                                tiff_header = data[exif_start:exif_start + 8]
                                print(f"  TIFF header: {tiff_header.hex()}")
                                
                                if tiff_header[:2] in [b'II', b'MM']:
                                    print("  ✓ Valid TIFF header")
                                else:
                                    print("  ✗ Invalid TIFF header")
                        else:
                            print(f"  Not EXIF segment: {identifier}")
                
                pos += length + 2
            elif marker == 0xd9:  # End of image
                print(f"\nFound end marker at position {pos}")
                break
            else:
                pos += 2
        else:
            pos += 1
    
    print(f"\nTotal APP1 segments found: {app1_count}")

if __name__ == "__main__":
    test_file = "/keg/pictures/incoming/2025/09-Sep/20250907_132644.050.jpg"
    analyze_jpeg(test_file)

