#!/usr/bin/env python3
"""
Debug script to examine both EXIF blocks in HEIF files
"""

import sys
from pathlib import Path

def examine_exif_blocks(file_path: str):
    """Examine both EXIF blocks to find the correct one"""
    print(f"Examining EXIF blocks: {file_path}")
    print("=" * 60)
    
    with open(file_path, 'rb') as f:
        data = f.read()
    
    # Find EXIF blocks
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
        
        # Extract EXIF data (skip 'Exif' marker and padding)
        exif_start = exif_pos + 4
        # Skip padding bytes (usually 2 bytes)
        if exif_start + 2 < len(data):
            exif_start += 2
        
        # Look for TIFF header
        tiff_start = exif_start
        if tiff_start + 8 < len(data):
            tiff_header = data[tiff_start:tiff_start+8]
            print(f"TIFF header: {tiff_header.hex()}")
            
            # Check byte order
            if tiff_header[:2] == b'II':
                print("Byte order: Little-endian")
            elif tiff_header[:2] == b'MM':
                print("Byte order: Big-endian")
            else:
                print("Byte order: Unknown")
            
            # Read TIFF version
            version = int.from_bytes(tiff_header[2:4], byteorder='little' if tiff_header[:2] == b'II' else 'big')
            print(f"TIFF version: {version}")
            
            # Read IFD offset
            ifd_offset = int.from_bytes(tiff_header[4:8], byteorder='little' if tiff_header[:2] == b'II' else 'big')
            print(f"IFD offset: {ifd_offset}")
            
            # Try to read some EXIF tags
            ifd_pos = tiff_start + ifd_offset
            if ifd_pos + 2 < len(data):
                entry_count = int.from_bytes(data[ifd_pos:ifd_pos+2], byteorder='little' if tiff_header[:2] == b'II' else 'big')
                print(f"IFD entry count: {entry_count}")
                
                # Read first few entries
                for j in range(min(5, entry_count)):
                    entry_pos = ifd_pos + 2 + (j * 12)
                    if entry_pos + 12 <= len(data):
                        entry = data[entry_pos:entry_pos+12]
                        tag = int.from_bytes(entry[0:2], byteorder='little' if tiff_header[:2] == b'II' else 'big')
                        tag_type = int.from_bytes(entry[2:4], byteorder='little' if tiff_header[:2] == b'II' else 'big')
                        count = int.from_bytes(entry[4:8], byteorder='little' if tiff_header[:2] == b'II' else 'big')
                        value_offset = int.from_bytes(entry[8:12], byteorder='little' if tiff_header[:2] == b'II' else 'big')
                        
                        tag_name = {
                            0x829A: "ExposureTime",
                            0xA402: "ExposureMode", 
                            0x8822: "ExposureProgram",
                            0x9204: "ExposureBiasValue"
                        }.get(tag, f"Tag_{tag:04X}")
                        
                        print(f"  Entry {j}: {tag_name} (type={tag_type}, count={count}, offset={value_offset})")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python examine_exif_blocks.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    examine_exif_blocks(file_path)
