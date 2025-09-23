#!/usr/bin/env python3
"""
Script to trace what our Rust code is actually reading
"""

import sys
from pathlib import Path
from fast_exif_reader import FastExifReader

def trace_rust_exif_reading(file_path: str):
    """Trace what our Rust code is actually reading"""
    print(f"Tracing Rust EXIF Reading: {file_path}")
    print("=" * 80)
    
    reader = FastExifReader()
    
    # Get all metadata
    try:
        metadata = reader.read_file(file_path)
        
        print("1. ALL METADATA EXTRACTED:")
        print("-" * 40)
        for k, v in sorted(metadata.items()):
            print(f"  {k}: {v}")
        
        print("\n2. EXPOSURE-RELATED FIELDS:")
        print("-" * 40)
        exposure_fields = {k: v for k, v in metadata.items() if 'expos' in k.lower() or k in ['ExposureTime', 'ExposureMode', 'ExposureBiasValue', 'FNumber', 'ISOSpeedRatings', 'ExposureProgram']}
        for k, v in sorted(exposure_fields.items()):
            print(f"  {k}: {v}")
        
        print("\n3. COMPARISON WITH EXIFTOOL:")
        print("-" * 40)
        print("  Expected from exiftool:")
        print("    ExposureTime: 1/3200")
        print("    ExposureMode: Auto (0)")
        print("    ExposureBiasValue: 0")
        print("    ExposureProgram: Shutter speed priority AE (4)")
        
        print("\n  Our tool results:")
        for k, v in sorted(exposure_fields.items()):
            print(f"    {k}: {v}")
        
        # Check if we have any unexpected fields
        print("\n4. UNEXPECTED FIELDS:")
        print("-" * 40)
        unexpected_fields = []
        for k, v in metadata.items():
            if 'expos' in k.lower() and k not in ['ExposureTime', 'ExposureMode', 'ExposureBiasValue', 'ExposureProgram']:
                unexpected_fields.append((k, v))
        
        if unexpected_fields:
            for k, v in unexpected_fields:
                print(f"  {k}: {v}")
        else:
            print("  No unexpected exposure-related fields found")
        
        # Check for duplicate or conflicting values
        print("\n5. POTENTIAL CONFLICTS:")
        print("-" * 40)
        if 'ExposureMode' in metadata:
            exposure_mode = metadata['ExposureMode']
            if exposure_mode != '0' and exposure_mode != 'Auto':
                print(f"  ExposureMode value '{exposure_mode}' is unexpected")
                print(f"  Expected: '0' or 'Auto'")
        
        if 'ExposureBiasValue' in metadata:
            exposure_bias = metadata['ExposureBiasValue']
            if exposure_bias != '0' and exposure_bias != '0/6':
                print(f"  ExposureBiasValue value '{exposure_bias}' is unexpected")
                print(f"  Expected: '0' or '0/6'")
        
    except Exception as e:
        print(f"Error reading file: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python trace_rust_exif_reading.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    trace_rust_exif_reading(file_path)
