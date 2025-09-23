#!/usr/bin/env python3
"""
Script to examine the exact EXIF data being extracted by our tool
"""

import sys
from pathlib import Path

def examine_extracted_exif_data(file_path: str):
    """Examine the exact EXIF data being extracted"""
    print(f"Examining Extracted EXIF Data: {file_path}")
    print("=" * 80)
    
    with open(file_path, 'rb') as f:
        data = f.read()
    
    # Find the valid EXIF block (position 2052)
    exif_pos = 2052
    exif_start = exif_pos + 4 + 2  # Skip 'Exif' and padding
    
    print(f"EXIF Block at position {exif_pos}:")
    print("-" * 40)
    
    if exif_start + 8 < len(data):
        tiff_header = data[exif_start:exif_start+8]
        print(f"TIFF header: {tiff_header.hex()}")
        
        # Check byte order
        if tiff_header[:2] == b'II':
            print("Byte order: Little-endian")
            is_little_endian = True
        else:
            print("Byte order: Big-endian")
            is_little_endian = False
        
        # Read IFD offset
        ifd_offset = int.from_bytes(tiff_header[4:8], byteorder='little' if is_little_endian else 'big')
        print(f"IFD offset: {ifd_offset}")
        
        # Read IFD entries
        ifd_pos = exif_start + ifd_offset
        if ifd_pos + 2 < len(data):
            entry_count = int.from_bytes(data[ifd_pos:ifd_pos+2], byteorder='little' if is_little_endian else 'big')
            print(f"IFD entry count: {entry_count}")
            
            print("\nLooking for ExposureMode tag (0xA402):")
            print("-" * 40)
            
            found_exposure_mode = False
            for j in range(entry_count):
                entry_pos = ifd_pos + 2 + (j * 12)
                if entry_pos + 12 <= len(data):
                    entry = data[entry_pos:entry_pos+12]
                    tag = int.from_bytes(entry[0:2], byteorder='little' if is_little_endian else 'big')
                    tag_type = int.from_bytes(entry[2:4], byteorder='little' if is_little_endian else 'big')
                    count = int.from_bytes(entry[4:8], byteorder='little' if is_little_endian else 'big')
                    value_offset = int.from_bytes(entry[8:12], byteorder='little' if is_little_endian else 'big')
                    
                    if tag == 0xA402:  # ExposureMode
                        found_exposure_mode = True
                        print(f"Found ExposureMode tag!")
                        print(f"  Tag: 0x{tag:04X}")
                        print(f"  Type: {tag_type}")
                        print(f"  Count: {count}")
                        print(f"  Value Offset: {value_offset}")
                        
                        # Try to read the actual value
                        if tag_type == 3:  # SHORT
                            if count == 1 and value_offset < 65536:
                                print(f"  Value: {value_offset}")
                            else:
                                value_pos = exif_start + value_offset
                                if value_pos + 2 <= len(data):
                                    value = int.from_bytes(data[value_pos:value_pos+2], byteorder='little' if is_little_endian else 'big')
                                    print(f"  Value: {value}")
                        else:
                            print(f"  Unexpected type: {tag_type}")
            
            if not found_exposure_mode:
                print("ExposureMode tag (0xA402) NOT FOUND in this EXIF block!")
                print("\nThis explains why we're getting the wrong value - we're reading from a different source!")
                
                # Let's check if there are other EXIF blocks
                print("\nSearching for other EXIF blocks...")
                print("-" * 40)
                
                # Look for ExposureMode pattern in the entire file
                exposure_mode_pattern = b'\x02\xA4'  # Little-endian 0xA402
                pos = 0
                while True:
                    pos = data.find(exposure_mode_pattern, pos)
                    if pos == -1:
                        break
                    
                    print(f"Found ExposureMode pattern at position {pos}")
                    
                    # Check if this is in a different EXIF block
                    if pos != exif_pos:
                        print(f"  This is in a different EXIF block!")
                        
                        # Try to find the EXIF block this belongs to
                        exif_block_start = pos
                        while exif_block_start > 0:
                            if data[exif_block_start-4:exif_block_start] == b'Exif':
                                break
                            exif_block_start -= 1
                        
                        if exif_block_start > 0:
                            print(f"  EXIF block starts at position {exif_block_start}")
                            
                            # Check the TIFF header
                            tiff_start = exif_block_start + 4 + 2
                            if tiff_start + 8 < len(data):
                                tiff_header = data[tiff_start:tiff_start+8]
                                print(f"  TIFF header: {tiff_header.hex()}")
                                
                                if tiff_header[:2] == b'II':
                                    version = int.from_bytes(tiff_header[2:4], byteorder='little')
                                    print(f"  TIFF version: {version}")
                                    if version == 42:
                                        print(f"  ✅ This is a valid EXIF block!")
                                    else:
                                        print(f"  ❌ This is an invalid EXIF block!")
                    
                    pos += 1

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python examine_extracted_exif_data.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    examine_extracted_exif_data(file_path)
