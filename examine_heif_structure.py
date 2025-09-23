#!/usr/bin/env python3
"""
Debug script to examine raw EXIF data from HEIF files
"""

import sys
from pathlib import Path

def examine_heif_structure(file_path: str):
    """Examine HEIF file structure to understand EXIF data location"""
    print(f"Examining HEIF structure: {file_path}")
    print("=" * 60)
    
    with open(file_path, 'rb') as f:
        data = f.read()
    
    print(f"File size: {len(data)} bytes")
    
    # Look for EXIF signatures
    exif_signatures = [b'Exif', b'EXIF', b'II*\x00', b'MM\x00*']
    
    for sig in exif_signatures:
        pos = 0
        count = 0
        while True:
            pos = data.find(sig, pos)
            if pos == -1:
                break
            print(f"Found '{sig.decode('ascii', errors='ignore')}' at position {pos}")
            count += 1
            pos += 1
        if count > 0:
            print(f"Total occurrences of '{sig.decode('ascii', errors='ignore')}': {count}")
    
    # Look for HEIF atoms
    print("\nHEIF atoms found:")
    pos = 0
    atom_count = 0
    while pos + 8 < len(data):
        try:
            # Read atom size (big-endian)
            size = (data[pos] << 24) | (data[pos + 1] << 16) | (data[pos + 2] << 8) | data[pos + 3]
            if size == 0 or size > len(data):
                break
            
            atom_type = data[pos + 4:pos + 8]
            print(f"Atom at {pos}: type='{atom_type.decode('ascii', errors='ignore')}', size={size}")
            
            # Look for EXIF data in this atom
            if atom_type in [b'meta', b'idat', b'exif']:
                atom_data = data[pos + 8:pos + size]
                exif_pos = atom_data.find(b'Exif')
                if exif_pos != -1:
                    print(f"  -> EXIF data found at offset {exif_pos} within atom")
                    # Show some context around EXIF
                    start = max(0, exif_pos - 10)
                    end = min(len(atom_data), exif_pos + 50)
                    context = atom_data[start:end]
                    print(f"  -> Context: {context.hex()}")
            
            pos += size
            atom_count += 1
            if atom_count > 20:  # Limit output
                print("... (truncated)")
                break
        except:
            break
    
    print(f"\nTotal atoms found: {atom_count}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python examine_heif_structure.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    examine_heif_structure(file_path)
