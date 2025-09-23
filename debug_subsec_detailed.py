#!/usr/bin/env python3
"""
Detailed debug script to analyze SubSecTime field data and compare with exiftool
"""

import sys
from pathlib import Path
from fast_exif_reader import FastExifReader

def debug_subsec_detailed(file_path: str):
    """Detailed debug SubSecTime fields to understand the data format"""
    print(f"Detailed SubSecTime Debug: {file_path}")
    print("=" * 80)
    
    reader = FastExifReader()
    
    try:
        # Read the file as bytes to analyze
        with open(file_path, 'rb') as f:
            file_data = f.read()
        
        print(f"File size: {len(file_data)} bytes")
        
        # Look for TIFF headers and analyze SubSecTime data
        print("\nSearching for TIFF headers with SubSecTime data:")
        print("-" * 50)
        
        tiff_positions = []
        for i in range(len(file_data) - 8):
            if file_data[i:i+2] == b'II' or file_data[i:i+2] == b'MM':
                tiff_positions.append(i)
        
        print(f"Found {len(tiff_positions)} TIFF headers")
        
        # Look for SubSecTime tags (0x9290, 0x9291, 0x9292) in each TIFF header
        subsec_tags = [0x9290, 0x9291, 0x9292]
        tag_names = {0x9290: "SubSecTime", 0x9291: "SubSecTimeOriginal", 0x9292: "SubSecTimeDigitized"}
        
        for i, pos in enumerate(tiff_positions[:10]):  # Check first 10 TIFF headers
            print(f"\nTIFF Header {i+1} at position {pos}:")
            
            # Check if this TIFF header contains SubSecTime tags
            tiff_data = file_data[pos:]
            if len(tiff_data) < 8:
                continue
                
            # Read TIFF header
            is_little_endian = tiff_data[0] == 0x49 and tiff_data[1] == 0x49
            ifd_offset = int.from_bytes(tiff_data[4:8], 'little' if is_little_endian else 'big')
            
            if ifd_offset >= len(tiff_data):
                continue
                
            # Read number of directory entries
            if ifd_offset + 2 > len(tiff_data):
                continue
                
            num_entries = int.from_bytes(tiff_data[ifd_offset:ifd_offset+2], 'little' if is_little_endian else 'big')
            
            if num_entries == 0 or num_entries > 100:
                continue
                
            print(f"  IFD offset: {ifd_offset}, Entries: {num_entries}")
            
            # Look for SubSecTime tags in this IFD
            for entry_idx in range(min(num_entries, 20)):  # Check first 20 entries
                entry_offset = ifd_offset + 2 + (entry_idx * 12)
                if entry_offset + 12 > len(tiff_data):
                    break
                    
                # Read tag
                tag = int.from_bytes(tiff_data[entry_offset:entry_offset+2], 'little' if is_little_endian else 'big')
                
                if tag in subsec_tags:
                    # Read format and count
                    format_type = int.from_bytes(tiff_data[entry_offset+2:entry_offset+4], 'little' if is_little_endian else 'big')
                    count = int.from_bytes(tiff_data[entry_offset+4:entry_offset+8], 'little' if is_little_endian else 'big')
                    value_offset = int.from_bytes(tiff_data[entry_offset+8:entry_offset+12], 'little' if is_little_endian else 'big')
                    
                    print(f"    Found {tag_names[tag]} (0x{tag:04X}):")
                    print(f"      Format: {format_type}, Count: {count}, Value offset: {value_offset}")
                    
                    # Read the actual value
                    if format_type == 2:  # ASCII string
                        if count <= 4:
                            # Value is stored directly in the offset field
                            value_bytes = tiff_data[entry_offset+8:entry_offset+12][:count]
                        else:
                            # Value is stored at the offset
                            value_offset_actual = pos + value_offset
                            if value_offset_actual + count <= len(file_data):
                                value_bytes = file_data[value_offset_actual:value_offset_actual+count]
                            else:
                                value_bytes = b""
                        
                        print(f"      Raw bytes: {list(value_bytes)}")
                        print(f"      As string: '{value_bytes.decode('ascii', errors='replace')}'")
                        print(f"      As hex: {value_bytes.hex()}")
        
        # Now test our current parsing
        print("\n" + "=" * 80)
        print("Current fast-exif-rs parsing result:")
        print("-" * 40)
        metadata = reader.read_file(file_path)
        
        subsec_fields = ['SubSecTime', 'SubSecTimeOriginal', 'SubSecTimeDigitized']
        for field in subsec_fields:
            if field in metadata:
                print(f"{field}: '{metadata[field]}'")
            else:
                print(f"{field}: Not found")
        
        print("\nExpected from exiftool:")
        print("SubSecTime: '92'")
        print("SubSecTimeOriginal: '92'")
        print("SubSecTimeDigitized: '92'")
        
        # Let's also check what our scoring system is doing
        print("\n" + "=" * 80)
        print("Analyzing all EXIF data blocks found by our parser:")
        print("-" * 50)
        
        # This is a bit of a hack - we'll manually call the EXIF finding functions
        # to see what data we're actually finding
        try:
            # Look for all EXIF data using our current method
            all_exif_data = []
            
            # Strategy 1: Look for EXIF data in item data boxes
            pos = 0
            while pos + 8 < len(file_data):
                size = int.from_bytes(file_data[pos:pos+4], 'big')
                if size == 0 or size > len(file_data):
                    break
                atom_type = file_data[pos + 4:pos + 8]
                if atom_type == b"idat":
                    # Found item data box - look for EXIF data
                    exif_start = pos + 8 + 4  # Skip version/flags
                    for i in range(exif_start, min(exif_start + 100, pos + size)):
                        if file_data[i:i+4] == b"Exif":
                            tiff_start = i + 4
                            if tiff_start + 8 < len(file_data):
                                if (file_data[tiff_start] == 0x49 and file_data[tiff_start + 1] == 0x49) or \
                                   (file_data[tiff_start] == 0x4D and file_data[tiff_start + 1] == 0x4D):
                                    all_exif_data.append((tiff_start, file_data[tiff_start:]))
                pos += size
            
            print(f"Found {len(all_exif_data)} EXIF data blocks")
            
            for i, (offset, exif_data) in enumerate(all_exif_data):
                print(f"\nEXIF Block {i+1} at offset {offset}:")
                # Try to parse this EXIF data and look for SubSecTime
                try:
                    temp_metadata = {}
                    reader.parse_tiff_exif(exif_data, temp_metadata)
                    
                    subsec_found = False
                    for field in subsec_fields:
                        if field in temp_metadata:
                            print(f"  {field}: '{temp_metadata[field]}'")
                            subsec_found = True
                    
                    if not subsec_found:
                        print("  No SubSecTime fields found")
                    
                    # Show some other fields to identify which EXIF block this is
                    if 'Make' in temp_metadata:
                        print(f"  Make: '{temp_metadata['Make']}'")
                    if 'Model' in temp_metadata:
                        print(f"  Model: '{temp_metadata['Model']}'")
                    if 'DateTimeOriginal' in temp_metadata:
                        print(f"  DateTimeOriginal: '{temp_metadata['DateTimeOriginal']}'")
                        
                except Exception as e:
                    print(f"  Error parsing EXIF block {i+1}: {e}")
                    
        except Exception as e:
            print(f"Error analyzing EXIF blocks: {e}")
        
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python debug_subsec_detailed.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    debug_subsec_detailed(file_path)
