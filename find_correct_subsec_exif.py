#!/usr/bin/env python3
"""
Find EXIF data blocks that contain the correct SubSecTime value (92)
"""

import sys
from pathlib import Path

def find_correct_subsec_exif(file_path: str):
    """Find EXIF data blocks containing SubSecTime value 92"""
    print(f"Searching for EXIF blocks with SubSecTime=92: {file_path}")
    print("=" * 60)
    
    try:
        with open(file_path, 'rb') as f:
            file_data = f.read()
        
        print(f"File size: {len(file_data)} bytes")
        
        # Look for EXIF patterns
        exif_positions = []
        for i in range(len(file_data) - 4):
            if file_data[i:i+4] == b'Exif':
                exif_positions.append(i)
        
        print(f"Found {len(exif_positions)} EXIF patterns")
        
        # For each EXIF pattern, look for SubSecTime tags and their values
        for i, exif_pos in enumerate(exif_positions):
            print(f"\nEXIF {i+1} at position {exif_pos}:")
            
            # Look for TIFF header after EXIF
            tiff_start = exif_pos + 4
            if tiff_start + 8 >= len(file_data):
                continue
                
            # Check if this is a valid TIFF header
            if file_data[tiff_start:tiff_start+2] not in [b'II', b'MM']:
                print("  Not a valid TIFF header")
                continue
                
            is_little_endian = file_data[tiff_start] == 0x49 and file_data[tiff_start + 1] == 0x49
            
            # Read IFD offset
            ifd_offset = int.from_bytes(file_data[tiff_start+4:tiff_start+8], 'little' if is_little_endian else 'big')
            
            if ifd_offset == 0 or ifd_offset >= len(file_data) - tiff_start:
                print("  Invalid IFD offset")
                continue
                
            # Read number of directory entries
            ifd_pos = tiff_start + ifd_offset
            if ifd_pos + 2 >= len(file_data):
                continue
                
            num_entries = int.from_bytes(file_data[ifd_pos:ifd_pos+2], 'little' if is_little_endian else 'big')
            
            if num_entries == 0 or num_entries > 100:
                print(f"  Invalid number of entries: {num_entries}")
                continue
                
            print(f"  IFD offset: {ifd_offset}, Entries: {num_entries}")
            
            # Look for SubSecTime tags in this IFD
            subsec_tags = [0x9290, 0x9291, 0x9292]
            tag_names = {0x9290: "SubSecTime", 0x9291: "SubSecTimeOriginal", 0x9292: "SubSecTimeDigitized"}
            
            found_subsec = False
            for entry_idx in range(min(num_entries, 50)):  # Check first 50 entries
                entry_offset = ifd_pos + 2 + (entry_idx * 12)
                if entry_offset + 12 > len(file_data):
                    break
                    
                # Read tag
                tag = int.from_bytes(file_data[entry_offset:entry_offset+2], 'little' if is_little_endian else 'big')
                
                if tag in subsec_tags:
                    # Read format and count
                    format_type = int.from_bytes(file_data[entry_offset+2:entry_offset+4], 'little' if is_little_endian else 'big')
                    count = int.from_bytes(file_data[entry_offset+4:entry_offset+8], 'little' if is_little_endian else 'big')
                    value_offset = int.from_bytes(file_data[entry_offset+8:entry_offset+12], 'little' if is_little_endian else 'big')
                    
                    print(f"    Found {tag_names[tag]} (0x{tag:04X}):")
                    print(f"      Format: {format_type}, Count: {count}, Value offset: {value_offset}")
                    
                    # Read the actual value
                    if format_type == 2:  # ASCII string
                        if count <= 4:
                            # Value is stored directly in the offset field
                            value_bytes = file_data[entry_offset+8:entry_offset+12][:count]
                        else:
                            # Value is stored at the offset
                            value_offset_actual = tiff_start + value_offset
                            if value_offset_actual + count <= len(file_data):
                                value_bytes = file_data[value_offset_actual:value_offset_actual+count]
                            else:
                                value_bytes = b""
                        
                        print(f"      Raw bytes: {list(value_bytes)}")
                        try:
                            value_str = value_bytes.decode('ascii', errors='replace')
                            print(f"      As string: '{value_str}'")
                            
                            # Check if this is the correct value
                            if value_str == "92":
                                print(f"      *** CORRECT VALUE FOUND! ***")
                                found_subsec = True
                        except:
                            print(f"      As string: (decode error)")
            
            if not found_subsec:
                print("  No SubSecTime fields with value '92' found")
        
        # Also look for the string "92" near EXIF patterns
        print(f"\n" + "=" * 60)
        print("Looking for '92' near EXIF patterns:")
        print("-" * 40)
        
        for i, exif_pos in enumerate(exif_positions):
            print(f"\nEXIF {i+1} at position {exif_pos}:")
            
            # Search for '92' in a window around this EXIF
            search_start = max(0, exif_pos - 1000)
            search_end = min(len(file_data), exif_pos + 10000)
            search_data = file_data[search_start:search_end]
            
            positions_92 = []
            for j in range(len(search_data) - 1):
                if search_data[j:j+2] == b'92':
                    positions_92.append(search_start + j)
            
            print(f"  Found {len(positions_92)} occurrences of '92' in window")
            for pos in positions_92[:5]:  # Show first 5
                print(f"    '92' at position {pos}")
                # Show context
                ctx_start = max(0, pos - 20)
                ctx_end = min(len(file_data), pos + 20)
                context = file_data[ctx_start:ctx_end]
                print(f"    Context: {context.hex()}")
        
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python find_correct_subsec_exif.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    find_correct_subsec_exif(file_path)
