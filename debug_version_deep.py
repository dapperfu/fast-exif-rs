#!/usr/bin/env python3
"""
Deep debugging for version field issues.
"""

import sys
from pathlib import Path

# Add the python module to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

from fast_exif_reader import FastExifReader

def debug_version_deep():
    """Deep debug version field processing."""
    test_file = "test_files/20130418_101628-1_.jpg"
    
    print(f"Deep debugging version fields: {test_file}")
    
    reader = FastExifReader()
    metadata = reader.read_file(test_file)
    
    # Check all fields that contain "Version"
    print(f"\nAll version-related fields:")
    for key, value in sorted(metadata.items()):
        if "version" in key.lower() or "Version" in key:
            print(f"  {key}: {value}")
    
    # Analyze the raw values
    for field in ["FlashpixVersion", "ExifVersion"]:
        if field in metadata:
            value = metadata[field]
            print(f"\n{field} analysis:")
            print(f"  Raw value: {value}")
            
            if isinstance(value, str) and value.isdigit():
                num = int(value)
                print(f"  As integer: {num}")
                print(f"  As hex: {format(num, '08X')}")
                
                # Extract bytes in different orders
                byte1 = num & 0xFF
                byte2 = (num >> 8) & 0xFF
                byte3 = (num >> 16) & 0xFF
                byte4 = (num >> 24) & 0xFF
                
                print(f"  Little-endian bytes: {byte1}, {byte2}, {byte3}, {byte4}")
                print(f"  Little-endian ASCII: '{chr(byte1)}{chr(byte2)}{chr(byte3)}{chr(byte4)}'")
                
                # Try big-endian interpretation
                byte1_be = (num >> 24) & 0xFF
                byte2_be = (num >> 16) & 0xFF
                byte3_be = (num >> 8) & 0xFF
                byte4_be = num & 0xFF
                
                print(f"  Big-endian bytes: {byte1_be}, {byte2_be}, {byte3_be}, {byte4_be}")
                print(f"  Big-endian ASCII: '{chr(byte1_be)}{chr(byte2_be)}{chr(byte3_be)}{chr(byte4_be)}'")
                
                # Try different interpretations
                print(f"  As 4 separate bytes: {[chr(b) for b in [byte1, byte2, byte3, byte4]]}")
                print(f"  As 4 separate bytes (BE): {[chr(b) for b in [byte1_be, byte2_be, byte3_be, byte4_be]]}")

if __name__ == "__main__":
    debug_version_deep()
