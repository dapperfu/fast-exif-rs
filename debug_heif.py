#!/usr/bin/env python3
"""
Debug script to analyze HEIF file parsing
"""

import sys
from pathlib import Path
from fast_exif_reader import FastExifReader

def debug_heif_file(file_path: str):
    """Debug HEIF file parsing"""
    print(f"Debugging HEIF file: {file_path}")
    print("=" * 60)
    
    reader = FastExifReader()
    
    try:
        metadata = reader.read_file(file_path)
        
        print("Extracted metadata:")
        print("-" * 30)
        for key, value in sorted(metadata.items()):
            print(f"{key}: {value}")
        
        print("\n" + "=" * 60)
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
        print("Usage: python debug_heif.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    debug_heif_file(file_path)
