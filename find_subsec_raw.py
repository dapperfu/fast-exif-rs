#!/usr/bin/env python3
"""
Simple debug script to find SubSecTime data in raw file
"""

import sys
from pathlib import Path

def find_subsec_in_raw(file_path: str):
    """Find SubSecTime data in raw file bytes"""
    print(f"Searching for SubSecTime data in raw file: {file_path}")
    print("=" * 60)
    
    try:
        with open(file_path, 'rb') as f:
            file_data = f.read()
        
        print(f"File size: {len(file_data)} bytes")
        
        # Look for the string "92" in the file (the correct SubSecTime value)
        print("\nSearching for '92' in file:")
        positions_92 = []
        for i in range(len(file_data) - 1):
            if file_data[i:i+2] == b'92':
                positions_92.append(i)
                print(f"  Found '92' at position {i}")
        
        print(f"Total '92' occurrences: {len(positions_92)}")
        
        # Look for the string "17" in the file (the incorrect value we're getting)
        print("\nSearching for '17' in file:")
        positions_17 = []
        for i in range(len(file_data) - 1):
            if file_data[i:i+2] == b'17':
                positions_17.append(i)
                print(f"  Found '17' at position {i}")
        
        print(f"Total '17' occurrences: {len(positions_17)}")
        
        # Look for SubSecTime tag patterns (0x9290, 0x9291, 0x9292)
        print("\nSearching for SubSecTime tag patterns:")
        subsec_tags = [b'\x92\x90', b'\x92\x91', b'\x92\x92']
        tag_names = {b'\x92\x90': "SubSecTime", b'\x92\x91': "SubSecTimeOriginal", b'\x92\x92': "SubSecTimeDigitized"}
        
        for tag_bytes in subsec_tags:
            positions = []
            for i in range(len(file_data) - 1):
                if file_data[i:i+2] == tag_bytes:
                    positions.append(i)
            
            print(f"  {tag_names[tag_bytes]} (0x{tag_bytes.hex()}): {len(positions)} occurrences")
            for pos in positions[:5]:  # Show first 5 occurrences
                print(f"    At position {pos}")
                # Show context around this position
                start = max(0, pos - 20)
                end = min(len(file_data), pos + 20)
                context = file_data[start:end]
                print(f"    Context: {context.hex()}")
                
                # Try to extract the value (assuming it's 4 bytes after the tag)
                if pos + 6 < len(file_data):
                    value_bytes = file_data[pos+4:pos+6]
                    print(f"    Potential value bytes: {list(value_bytes)}")
                    try:
                        value_str = value_bytes.decode('ascii', errors='replace')
                        print(f"    As string: '{value_str}'")
                    except:
                        print(f"    As string: (decode error)")
        
        # Look for EXIF patterns
        print("\nSearching for EXIF patterns:")
        exif_positions = []
        for i in range(len(file_data) - 4):
            if file_data[i:i+4] == b'Exif':
                exif_positions.append(i)
        
        print(f"Found {len(exif_positions)} EXIF patterns")
        for i, pos in enumerate(exif_positions[:3]):  # Show first 3
            print(f"  EXIF {i+1} at position {pos}")
            # Show context
            start = max(0, pos - 10)
            end = min(len(file_data), pos + 50)
            context = file_data[start:end]
            print(f"    Context: {context.hex()}")
        
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python find_subsec_raw.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    find_subsec_in_raw(file_path)
