#!/usr/bin/env python3
"""
Debug script to trace EXIF block selection in HEIF files
"""

import sys
from pathlib import Path

def trace_exif_selection(file_path: str):
    """Trace which EXIF block is being selected"""
    print(f"Tracing EXIF block selection: {file_path}")
    print("=" * 60)
    
    with open(file_path, 'rb') as f:
        data = f.read()
    
    # Find all EXIF blocks
    exif_positions = []
    pos = 0
    while True:
        pos = data.find(b'Exif', pos)
        if pos == -1:
            break
        exif_positions.append(pos)
        pos += 1
    
    print(f"Found {len(exif_positions)} EXIF blocks at positions: {exif_positions}")
    
    for i, exif_pos in enumerate(exif_positions):
        print(f"\n--- EXIF Block {i+1} at position {exif_pos} ---")
        
        # Extract EXIF data
        exif_start = exif_pos + 4
        if exif_start + 2 < len(data):
            exif_start += 2  # Skip padding
        
        # Check TIFF header
        if exif_start + 8 < len(data):
            tiff_header = data[exif_start:exif_start+8]
            print(f"TIFF header: {tiff_header.hex()}")
            
            # Check byte order
            if tiff_header[:2] == b'II':
                print("Byte order: Little-endian")
                is_little_endian = True
            elif tiff_header[:2] == b'MM':
                print("Byte order: Big-endian")
                is_little_endian = False
            else:
                print("Byte order: Unknown")
                continue
            
            # Check TIFF version
            version = int.from_bytes(tiff_header[2:4], byteorder='little' if is_little_endian else 'big')
            print(f"TIFF version: {version}")
            
            if version != 42:
                print("❌ Invalid TIFF version - should be rejected")
                continue
            else:
                print("✅ Valid TIFF version")
            
            # Read IFD offset
            ifd_offset = int.from_bytes(tiff_header[4:8], byteorder='little' if is_little_endian else 'big')
            print(f"IFD offset: {ifd_offset}")
            
            # Try to read IFD entries
            ifd_pos = exif_start + ifd_offset
            if ifd_pos + 2 < len(data):
                entry_count = int.from_bytes(data[ifd_pos:ifd_pos+2], byteorder='little' if is_little_endian else 'big')
                print(f"IFD entry count: {entry_count}")
                
                # Look for specific exposure-related tags
                for j in range(min(entry_count, 20)):
                    entry_pos = ifd_pos + 2 + (j * 12)
                    if entry_pos + 12 <= len(data):
                        entry = data[entry_pos:entry_pos+12]
                        tag = int.from_bytes(entry[0:2], byteorder='little' if is_little_endian else 'big')
                        tag_type = int.from_bytes(entry[2:4], byteorder='little' if is_little_endian else 'big')
                        count = int.from_bytes(entry[4:8], byteorder='little' if is_little_endian else 'big')
                        value_offset = int.from_bytes(entry[8:12], byteorder='little' if is_little_endian else 'big')
                        
                        if tag in [0x829A, 0xA402, 0x8822, 0x9204]:  # ExposureTime, ExposureMode, ExposureProgram, ExposureBiasValue
                            tag_name = {
                                0x829A: "ExposureTime",
                                0xA402: "ExposureMode", 
                                0x8822: "ExposureProgram",
                                0x9204: "ExposureBiasValue"
                            }[tag]
                            
                            print(f"  {tag_name}: type={tag_type}, count={count}, offset={value_offset}")
                            
                            # Try to read the actual value
                            if tag_type == 5:  # Rational
                                value_pos = exif_start + value_offset
                                if value_pos + 8 <= len(data):
                                    numerator = int.from_bytes(data[value_pos:value_pos+4], byteorder='little' if is_little_endian else 'big')
                                    denominator = int.from_bytes(data[value_pos+4:value_pos+8], byteorder='little' if is_little_endian else 'big')
                                    print(f"    Value: {numerator}/{denominator}")
                            elif tag_type == 3:  # Short
                                if count == 1 and value_offset < 65536:
                                    print(f"    Value: {value_offset}")
                                else:
                                    value_pos = exif_start + value_offset
                                    if value_pos + 2 <= len(data):
                                        value = int.from_bytes(data[value_pos:value_pos+2], byteorder='little' if is_little_endian else 'big')
                                        print(f"    Value: {value}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python trace_exif_selection.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    trace_exif_selection(file_path)
