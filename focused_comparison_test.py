#!/usr/bin/env python3
"""
Focused comparison test to understand specific differences with exiftool.
"""

import sys
import subprocess
import json
from pathlib import Path

# Add the python module to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

from fast_exif_reader import FastExifReader

def get_exiftool_data(file_path):
    """Get exiftool data for comparison."""
    try:
        result = subprocess.run(
            ["exiftool", "-json", file_path],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode == 0:
            data = json.loads(result.stdout)
            if data and len(data) > 0:
                return data[0]
    except Exception as e:
        print(f"Error getting exiftool data: {e}")
    return {}

def compare_specific_fields(file_path):
    """Compare specific problematic fields."""
    print(f"\n{'='*60}")
    print(f"Focused comparison: {file_path}")
    print(f"{'='*60}")
    
    # Get data from both sources
    reader = FastExifReader()
    try:
        fast_exif_data = reader.read_file(file_path)
    except Exception as e:
        print(f"Fast-EXIF error: {e}")
        return
        
    exiftool_data = get_exiftool_data(file_path)
    if not exiftool_data:
        print("No exiftool data available")
        return
    
    # Focus on the most problematic fields
    focus_fields = [
        "ExposureCompensation", "FlashpixVersion", "ExifVersion",
        "ApertureValue", "ShutterSpeedValue", "MaxApertureValue", 
        "ExposureMode", "CustomRendered", "Sharpness", "MeteringMode",
        "ISO", "FNumber", "Make", "Model"
    ]
    
    print(f"{'Field':<20} {'Exiftool':<25} {'Fast-EXIF':<25} {'Match'}")
    print("-" * 80)
    
    matches = 0
    total = 0
    
    for field in focus_fields:
        exiftool_val = exiftool_data.get(field, "---")
        fast_exif_val = fast_exif_data.get(field, "---")
        
        if exiftool_val != "---" or fast_exif_val != "---":
            total += 1
            match = "✓" if exiftool_val == fast_exif_val else "✗"
            if exiftool_val == fast_exif_val:
                matches += 1
                
            # Truncate long values for display
            exiftool_display = str(exiftool_val)[:24] if len(str(exiftool_val)) > 24 else str(exiftool_val)
            fast_exif_display = str(fast_exif_val)[:24] if len(str(fast_exif_val)) > 24 else str(fast_exif_val)
            
            print(f"{field:<20} {exiftool_display:<25} {fast_exif_display:<25} {match}")
    
    if total > 0:
        match_rate = matches / total * 100
        print(f"\nMatch rate: {matches}/{total} ({match_rate:.1f}%)")
    else:
        print("\nNo comparable fields found")

def main():
    """Run focused comparison on key files."""
    # Test a few representative files
    test_files = [
        "test_files/20130418_101628-1_.jpg",  # JPEG with version issues
        "test_files/IMG_9345_.CR2",           # Canon RAW with APEX issues  
        "test_files/20241224_161213_.heic",   # HEIC with exposure issues
        "test_files/R0011229_20190622202257_.JPG"  # Ricoh with metering issues
    ]
    
    print("🔍 FOCUSED COMPARISON TEST")
    print("Analyzing specific field differences...")
    
    for test_file in test_files:
        if Path(test_file).exists():
            compare_specific_fields(test_file)
        else:
            print(f"\nFile not found: {test_file}")

if __name__ == "__main__":
    main()
