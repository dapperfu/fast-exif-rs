#!/usr/bin/env python3
"""
Comprehensive debug script to analyze HEIF file parsing
"""

import sys
from pathlib import Path
from fast_exif_reader import FastExifReader

def debug_heif_comprehensive(file_path: str):
    """Comprehensive debug HEIF file parsing"""
    print(f"Comprehensive HEIF Debug: {file_path}")
    print("=" * 80)
    
    reader = FastExifReader()
    
    try:
        # Read the file as bytes to analyze
        with open(file_path, 'rb') as f:
            file_data = f.read()
        
        print(f"File size: {len(file_data)} bytes")
        # Note: detect_format is not exposed to Python, so we'll skip this
        
        # Look for EXIF patterns in the raw file
        print("\nSearching for EXIF patterns in raw file:")
        exif_positions = []
        for i in range(len(file_data) - 4):
            if file_data[i:i+4] == b'Exif':
                exif_positions.append(i)
                print(f"  Found 'Exif' at position {i}")
        
        print(f"Total EXIF patterns found: {len(exif_positions)}")
        
        # Look for TIFF headers directly (not just after EXIF)
        print("\nSearching for TIFF headers directly:")
        tiff_positions = []
        for i in range(len(file_data) - 8):
            if file_data[i:i+2] == b'II' or file_data[i:i+2] == b'MM':
                tiff_positions.append(i)
                print(f"  Found TIFF header at position {i}: {file_data[i:i+2].decode()}")
        
        print(f"Total TIFF headers found: {len(tiff_positions)}")
        
        # Test each TIFF block individually
        print("\nTesting each TIFF block individually:")
        for i, pos in enumerate(tiff_positions):
            print(f"\n  TIFF Block {i+1} (starting at position {pos}):")
            try:
                metadata = {}
                reader.parse_tiff_exif(file_data[pos:], metadata)
                
                # Show key fields
                key_fields = ['DateTime', 'DateTimeOriginal', 'CreateDate', 'Make', 'Model', 'LensModel']
                for field in key_fields:
                    if field in metadata:
                        print(f"    {field}: {metadata[field]}")
                
                # Count total fields
                print(f"    Total fields: {len(metadata)}")
                
            except Exception as e:
                print(f"    Error parsing TIFF block {i+1}: {e}")
        
        # Now test the full parsing
        print("\n" + "=" * 80)
        print("Full parsing result:")
        metadata = reader.read_file(file_path)
        
        print("Extracted metadata:")
        print("-" * 30)
        for key, value in sorted(metadata.items()):
            print(f"{key}: {value}")
        
        print("\n" + "=" * 80)
        print("Analysis:")
        
        # Check for timestamp fields
        timestamp_fields = ['DateTime', 'DateTimeOriginal', 'CreateDate', 'SubSecCreateDate']
        found_timestamps = []
        for field in timestamp_fields:
            if field in metadata:
                found_timestamps.append(f"{field}: {metadata[field]}")
        
        if found_timestamps:
            print("✓ Timestamps found:")
            for ts in found_timestamps:
                print(f"  {ts}")
        else:
            print("✗ No timestamps found")
        
        # Check for camera info
        camera_fields = ['Make', 'Model', 'LensID']
        found_camera = []
        for field in camera_fields:
            if field in metadata:
                found_camera.append(f"{field}: {metadata[field]}")
        
        if found_camera:
            print("✓ Camera info found:")
            for info in found_camera:
                print(f"  {info}")
        else:
            print("✗ No camera info found")
        
        # Check for HEIF-specific fields
        heif_fields = ['Brand', 'HandlerType', 'Format']
        found_heif = []
        for field in heif_fields:
            if field in metadata:
                found_heif.append(f"{field}: {metadata[field]}")
        
        if found_heif:
            print("✓ HEIF info found:")
            for info in found_heif:
                print(f"  {info}")
        else:
            print("✗ No HEIF info found")
            
    except Exception as e:
        print(f"Error reading file: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python debug_heif_comprehensive.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    debug_heif_comprehensive(file_path)
