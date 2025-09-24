#!/usr/bin/env python3
"""
Debug script to check version field data types and values.
"""

import sys
from pathlib import Path

# Add the python module to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

from fast_exif_reader import FastExifReader

def debug_version_fields():
    """Debug version field parsing."""
    print("Debugging version field parsing...")
    
    # Test with sample file
    test_file = "test_files/20130418_101628-1_.jpg"
    
    try:
        reader = FastExifReader()
        metadata = reader.read_file(test_file)
        
        print(f"\nVersion fields found:")
        for field in ["FlashpixVersion", "ExifVersion"]:
            if field in metadata:
                value = metadata[field]
                print(f"  {field}: {value} (type: {type(value).__name__})")
                
                # Try to convert to hex if it's a number
                try:
                    if isinstance(value, str) and value.isdigit():
                        num = int(value)
                        hex_str = format(num, '08X')
                        print(f"    -> As hex: {hex_str}")
                        
                        # Convert to version format (4 hex chars)
                        if len(hex_str) >= 8:
                            version_str = hex_str[-8:]  # Take last 8 chars
                            # Convert to 4 hex chars (2 bytes each)
                            byte1 = version_str[0:2]
                            byte2 = version_str[2:4] 
                            byte3 = version_str[4:6]
                            byte4 = version_str[6:8]
                            formatted_version = f"{byte1}{byte2}{byte3}{byte4}"
                            print(f"    -> Formatted version: {formatted_version}")
                except Exception as e:
                    print(f"    -> Error converting: {e}")
            else:
                print(f"  {field}: (not present)")
        
        print("\nAll fields:")
        for key, value in sorted(metadata.items()):
            if "Version" in key or "ExposureMode" in key or "CustomRendered" in key:
                print(f"  {key}: {value}")
        
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    debug_version_fields()
