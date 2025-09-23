#!/usr/bin/env python3
"""
Analyze the invalid EXIF block to see what's in it
"""

import sys
from pathlib import Path

def analyze_invalid_exif_block(file_path: str):
    """Analyze the invalid EXIF block"""
    print(f"Analyzing Invalid EXIF Block: {file_path}")
    print("=" * 80)
    
    with open(file_path, 'rb') as f:
        data = f.read()
    
    # Find the invalid EXIF block (position 197)
    exif_pos = 197
    exif_start = exif_pos + 4 + 2  # Skip 'Exif' and padding
    
    print(f"Analyzing EXIF Block at position {exif_pos}:")
    print("-" * 40)
    
    if exif_start + 8 < len(data):
        tiff_header = data[exif_start:exif_start+8]
        print(f"TIFF header: {tiff_header.hex()}")
        
        # This should be invalid, but let's see what's there
        print("This block has invalid TIFF header, but let's examine the raw data...")
        
        # Look for any patterns that might be EXIF tags
        print("\nSearching for potential EXIF tag patterns:")
        print("-" * 40)
        
        # Look for ExposureMode tag (0xA402) in the raw data
        exposure_mode_pattern = b'\x02\xA4'  # Little-endian 0xA402
        exposure_mode_pos = data.find(exposure_mode_pattern, exif_pos)
        if exposure_mode_pos != -1:
            print(f"Found ExposureMode pattern at position {exposure_mode_pos}")
            # Show context around this position
            start = max(0, exposure_mode_pos - 10)
            end = min(len(data), exposure_mode_pos + 20)
            context = data[start:end]
            print(f"Context: {context.hex()}")
            
            # Try to interpret as EXIF tag
            if exposure_mode_pos + 12 <= len(data):
                tag_data = data[exposure_mode_pos:exposure_mode_pos+12]
                print(f"Tag data: {tag_data.hex()}")
                
                # Interpret as little-endian
                tag = int.from_bytes(tag_data[0:2], byteorder='little')
                tag_type = int.from_bytes(tag_data[2:4], byteorder='little')
                count = int.from_bytes(tag_data[4:8], byteorder='little')
                value_offset = int.from_bytes(tag_data[8:12], byteorder='little')
                
                print(f"Interpreted as:")
                print(f"  Tag: 0x{tag:04X}")
                print(f"  Type: {tag_type}")
                print(f"  Count: {count}")
                print(f"  Value Offset: {value_offset}")
                
                if tag_type == 3 and count == 1:  # SHORT type
                    print(f"  Value: {value_offset}")
        
        # Also look for other exposure-related tags
        exposure_tags = [
            (0x829A, b'\x9A\x82'),  # ExposureTime
            (0x8822, b'\x22\x88'),  # ExposureProgram  
            (0x9204, b'\x04\x92'),  # ExposureBiasValue
        ]
        
        for tag_id, pattern in exposure_tags:
            pos = data.find(pattern, exif_pos)
            if pos != -1:
                print(f"\nFound tag 0x{tag_id:04X} pattern at position {pos}")
                if pos + 12 <= len(data):
                    tag_data = data[pos:pos+12]
                    tag = int.from_bytes(tag_data[0:2], byteorder='little')
                    tag_type = int.from_bytes(tag_data[2:4], byteorder='little')
                    count = int.from_bytes(tag_data[4:8], byteorder='little')
                    value_offset = int.from_bytes(tag_data[8:12], byteorder='little')
                    
                    print(f"  Tag: 0x{tag:04X}, Type: {tag_type}, Count: {count}, Value: {value_offset}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python analyze_invalid_exif_block.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    analyze_invalid_exif_block(file_path)
