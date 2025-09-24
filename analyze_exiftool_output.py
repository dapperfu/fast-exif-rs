#!/usr/bin/env python3
"""
Analyze exactly what exiftool outputs vs what we output.
"""

import subprocess
import json
import sys
from pathlib import Path

# Add the python module to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

from fast_exif_reader import FastExifReader

def analyze_differences():
    """Analyze specific field differences to understand exact requirements."""
    test_file = "test_files/IMG_9345_.CR2"  # Canon RAW with many issues
    
    print(f"Analyzing exiftool vs fast-exif output for: {test_file}")
    
    # Get exiftool output
    result = subprocess.run(
        ["exiftool", "-json", test_file],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print("Failed to get exiftool output")
        return
        
    exiftool_data = json.loads(result.stdout)[0]
    
    # Get fast-exif output
    reader = FastExifReader()
    fast_exif_data = reader.read_file(test_file)
    
    # Focus on problematic fields
    problematic_fields = [
        "ExposureCompensation", "FlashpixVersion", "ExifVersion",
        "ApertureValue", "ShutterSpeedValue", "MaxApertureValue",
        "ExposureMode", "CustomRendered", "Sharpness", "MeteringMode"
    ]
    
    print(f"\nDETAILED FIELD ANALYSIS:")
    print(f"=" * 80)
    
    for field in problematic_fields:
        exiftool_val = exiftool_data.get(field)
        fast_exif_val = fast_exif_data.get(field)
        
        if exiftool_val is not None or fast_exif_val is not None:
            print(f"\n{field}:")
            print(f"  Exiftool:  '{exiftool_val}' (type: {type(exiftool_val).__name__})")
            print(f"  Fast-EXIF: '{fast_exif_val}' (type: {type(fast_exif_val).__name__})")
            
            if exiftool_val != fast_exif_val:
                print(f"  ❌ MISMATCH")
                
                # Try to understand the conversion needed
                if isinstance(fast_exif_val, str) and fast_exif_val.isdigit():
                    raw_val = int(fast_exif_val)
                    print(f"  Raw value: {raw_val}")
                    print(f"  Raw hex: {hex(raw_val)}")
                    
                    if field in ["FlashpixVersion", "ExifVersion"]:
                        # Version field analysis
                        hex_str = format(raw_val, '08X')
                        bytes_le = [raw_val & 0xFF, (raw_val >> 8) & 0xFF, (raw_val >> 16) & 0xFF, (raw_val >> 24) & 0xFF]
                        bytes_be = [(raw_val >> 24) & 0xFF, (raw_val >> 16) & 0xFF, (raw_val >> 8) & 0xFF, raw_val & 0xFF]
                        ascii_le = ''.join([chr(b) for b in bytes_le])
                        ascii_be = ''.join([chr(b) for b in bytes_be])
                        print(f"  ASCII (LE): '{ascii_le}'")
                        print(f"  ASCII (BE): '{ascii_be}'")
                        print(f"  Expected: '{exiftool_val}'")
                        
                        if ascii_be == exiftool_val:
                            print(f"  ✅ Solution: Use big-endian ASCII conversion")
                        elif ascii_le == exiftool_val:
                            print(f"  ✅ Solution: Use little-endian ASCII conversion")
                        else:
                            print(f"  ❓ Need different conversion")
                    
                    elif field == "ExposureCompensation":
                        # Try different conversion formulas
                        print(f"  Trying conversions:")
                        print(f"    raw_val / 100 = {raw_val / 100}")
                        print(f"    raw_val / 1000 = {raw_val / 1000}")
                        print(f"    (raw_val - 1000) / 100 = {(raw_val - 1000) / 100}")
                        print(f"  Expected: '{exiftool_val}'")
            else:
                print(f"  ✅ MATCH")

if __name__ == "__main__":
    analyze_differences()
