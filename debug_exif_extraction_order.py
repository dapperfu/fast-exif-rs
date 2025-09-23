#!/usr/bin/env python3
"""
Debug script to trace which EXIF extraction function is being used
"""

import sys
from pathlib import Path

def debug_exif_extraction_order(file_path: str):
    """Debug which EXIF extraction function is being used"""
    print(f"Debugging EXIF extraction order: {file_path}")
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
    
    # Check which atoms contain EXIF data
    print("\nChecking HEIF atoms for EXIF data:")
    pos = 0
    atom_count = 0
    while pos + 8 < len(data):
        try:
            # Read atom size (big-endian)
            size = (data[pos] << 24) | (data[pos + 1] << 16) | (data[pos + 2] << 8) | data[pos + 3]
            if size == 0 or size > len(data):
                break
            
            atom_type = data[pos + 4:pos + 8]
            
            # Check if this atom contains any of our EXIF blocks
            atom_start = pos
            atom_end = pos + size
            contained_exif = []
            for exif_pos in exif_positions:
                if atom_start < exif_pos < atom_end:
                    contained_exif.append(exif_pos)
            
            if contained_exif:
                print(f"Atom at {pos}: type='{atom_type.decode('ascii', errors='ignore')}', size={size}")
                print(f"  -> Contains EXIF blocks at positions: {contained_exif}")
                
                # Check which EXIF block would be found first by our functions
                if atom_type == b'idat':
                    print(f"  -> This would be found by find_exif_in_item_data_boxes")
                    # Extract EXIF data from this atom
                    atom_data = data[pos + 8:pos + size]
                    exif_pos = atom_data.find(b'Exif')
                    if exif_pos != -1:
                        exif_start = exif_pos + 4
                        if exif_start + 2 < len(atom_data):
                            exif_start += 2  # Skip padding
                        if exif_start + 8 < len(atom_data):
                            tiff_header = atom_data[exif_start:exif_start+8]
                            print(f"  -> EXIF TIFF header: {tiff_header.hex()}")
                            if tiff_header[:2] == b'II':
                                version = int.from_bytes(tiff_header[2:4], byteorder='little')
                                print(f"  -> TIFF version: {version}")
                                if version == 42:
                                    print(f"  -> ✅ Valid EXIF block")
                                else:
                                    print(f"  -> ❌ Invalid EXIF block")
                
                elif atom_type == b'meta':
                    print(f"  -> This would be found by find_exif_in_meta_structure")
                    # Extract EXIF data from this atom
                    atom_data = data[pos + 8:pos + size]
                    exif_pos = atom_data.find(b'Exif')
                    if exif_pos != -1:
                        exif_start = exif_pos + 4
                        if exif_start + 2 < len(atom_data):
                            exif_start += 2  # Skip padding
                        if exif_start + 8 < len(atom_data):
                            tiff_header = atom_data[exif_start:exif_start+8]
                            print(f"  -> EXIF TIFF header: {tiff_header.hex()}")
                            if tiff_header[:2] == b'II':
                                version = int.from_bytes(tiff_header[2:4], byteorder='little')
                                print(f"  -> TIFF version: {version}")
                                if version == 42:
                                    print(f"  -> ✅ Valid EXIF block")
                                else:
                                    print(f"  -> ❌ Invalid EXIF block")
            
            pos += size
            atom_count += 1
            if atom_count > 10:  # Limit output
                break
        except:
            break

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python debug_exif_extraction_order.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    debug_exif_extraction_order(file_path)
