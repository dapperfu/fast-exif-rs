#!/usr/bin/env python3
"""
Debug script to analyze HEIF EXIF data extraction in detail
"""

import sys
from pathlib import Path
from fast_exif_reader import FastExifReader

def debug_heif_exif(file_path: str):
    """Debug HEIF EXIF data extraction"""
    print(f"Debugging HEIF EXIF extraction: {file_path}")
    print("=" * 60)
    
    reader = FastExifReader()
    
    try:
        metadata = reader.read_file(file_path)
        
        print("Extracted metadata:")
        print("-" * 30)
        for key, value in sorted(metadata.items()):
            if 'expos' in key.lower() or key in ['ExposureTime', 'ExposureMode', 'ExposureBiasValue', 'FNumber', 'ISOSpeedRatings']:
                print(f"{key}: {value}")
        
        print("\n" + "=" * 60)
        print("Expected values from exiftool:")
        print("ExposureTime: 1/3200")
        print("ExposureMode: Auto")
        print("ExposureBiasValue: 0")
        
        print("\n" + "=" * 60)
        print("Analysis:")
        print(f"ExposureTime: Got {metadata.get('ExposureTime', 'NOT FOUND')}, Expected 1/3200")
        print(f"ExposureMode: Got {metadata.get('ExposureMode', 'NOT FOUND')}, Expected Auto (0)")
        print(f"ExposureBiasValue: Got {metadata.get('ExposureBiasValue', 'NOT FOUND')}, Expected 0")
        
        # Check if values are reasonable
        exposure_time = metadata.get('ExposureTime', '')
        if '/' in exposure_time:
            try:
                num, den = exposure_time.split('/')
                ratio = float(num) / float(den)
                print(f"ExposureTime ratio: {ratio:.6f}")
                if abs(ratio - (1/3200)) < 0.0001:
                    print("✓ ExposureTime is correct")
                else:
                    print("✗ ExposureTime is incorrect")
            except:
                print("✗ Could not parse ExposureTime")
        
        exposure_mode = metadata.get('ExposureMode', '')
        if exposure_mode == '0':
            print("✓ ExposureMode is correct (Auto)")
        elif exposure_mode.isdigit():
            print(f"✗ ExposureMode is incorrect: {exposure_mode} (should be 0 for Auto)")
        else:
            print(f"✗ ExposureMode format is incorrect: {exposure_mode}")
            
    except Exception as e:
        print(f"Error reading file: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python debug_heif_exif.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    debug_heif_exif(file_path)
