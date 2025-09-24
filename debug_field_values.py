#!/usr/bin/env python3
"""
Debug the actual field values to understand their format.
"""

import sys
from pathlib import Path

# Add the python module to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

from fast_exif_reader import FastExifReader

def debug_field_values():
    """Debug actual field values."""
    test_file = "test_files/IMG_9345_.CR2"
    
    print(f"Debugging field values: {test_file}")
    
    reader = FastExifReader()
    metadata = reader.read_file(test_file)
    
    # Check problematic fields
    problematic_fields = [
        "FlashpixVersion", "ExifVersion", "ExposureCompensation",
        "ShutterSpeedValue", "ApertureValue", "ExposureMode"
    ]
    
    print(f"\nProblematic field analysis:")
    for field in problematic_fields:
        if field in metadata:
            value = metadata[field]
            print(f"  {field}: '{value}' (type: {type(value).__name__}, len: {len(str(value))})")
            
            # Try to understand if it's parseable as integer
            try:
                int_val = int(value)
                print(f"    Can parse as int: {int_val}")
                print(f"    Hex: {hex(int_val)}")
            except ValueError:
                print(f"    Cannot parse as int")
        else:
            print(f"  {field}: NOT FOUND")
    
    print(f"\nAll metadata fields ({len(metadata)} total):")
    for key, value in sorted(metadata.items()):
        print(f"  {key}: {value}")

if __name__ == "__main__":
    debug_field_values()
