#!/usr/bin/env python3
"""
Detailed field comparison between fast-exif-rs and exiftool
"""

import sys
import os
import subprocess
import json
sys.path.insert(0, '/projects/fast-exif-rs/python')

from fast_exif_reader import FastExifReader

def run_exiftool(file_path):
    """Run exiftool and return parsed fields"""
    try:
        result = subprocess.run(['exiftool', '-s', file_path], 
                              capture_output=True, text=True, check=True)
        fields = {}
        for line in result.stdout.split('\n'):
            if ':' in line:
                key, value = line.split(':', 1)
                fields[key.strip()] = value.strip()
        return fields
    except subprocess.CalledProcessError as e:
        print(f"Error running exiftool: {e}")
        return {}
    except FileNotFoundError:
        print("exiftool not found")
        return {}

def compare_fields(file_path):
    """Compare fields between fast-exif-rs and exiftool"""
    if not os.path.exists(file_path):
        print(f"File not found: {file_path}")
        return
    
    print(f"\n{'='*80}")
    print(f"Detailed comparison: {os.path.basename(file_path)}")
    print(f"{'='*80}")
    
    # Get fast-exif-rs fields
    try:
        reader = FastExifReader()
        fast_exif_fields = reader.read_file(file_path)
    except Exception as e:
        print(f"fast-exif-rs error: {e}")
        fast_exif_fields = {}
    
    # Get exiftool fields
    exiftool_fields = run_exiftool(file_path)
    
    # Find common fields
    common_fields = set(fast_exif_fields.keys()) & set(exiftool_fields.keys())
    
    # Find fields only in fast-exif-rs
    only_fast_exif = set(fast_exif_fields.keys()) - set(exiftool_fields.keys())
    
    # Find fields only in exiftool
    only_exiftool = set(exiftool_fields.keys()) - set(fast_exif_fields.keys())
    
    print(f"Common fields: {len(common_fields)}")
    print(f"Only in fast-exif-rs: {len(only_fast_exif)}")
    print(f"Only in exiftool: {len(only_exiftool)}")
    
    # Show fields only in exiftool (what we're missing)
    if only_exiftool:
        print(f"\nFields only in exiftool ({len(only_exiftool)}):")
        for field in sorted(only_exiftool)[:20]:  # Show first 20
            print(f"  {field}: {exiftool_fields[field]}")
        if len(only_exiftool) > 20:
            print(f"  ... and {len(only_exiftool) - 20} more")
    
    # Show fields only in fast-exif-rs (what we have extra)
    if only_fast_exif:
        print(f"\nFields only in fast-exif-rs ({len(only_fast_exif)}):")
        for field in sorted(only_fast_exif)[:20]:  # Show first 20
            print(f"  {field}: {fast_exif_fields[field]}")
        if len(only_fast_exif) > 20:
            print(f"  ... and {len(only_fast_exif) - 20} more")
    
    # Show differences in common fields
    print(f"\nDifferences in common fields:")
    differences = []
    for field in sorted(common_fields):
        fast_val = fast_exif_fields[field]
        exif_val = exiftool_fields[field]
        if fast_val != exif_val:
            differences.append((field, fast_val, exif_val))
    
    if differences:
        for field, fast_val, exif_val in differences[:10]:  # Show first 10
            print(f"  {field}:")
            print(f"    fast-exif-rs: {fast_val}")
            print(f"    exiftool:     {exif_val}")
        if len(differences) > 10:
            print(f"  ... and {len(differences) - 10} more differences")
    else:
        print("  No differences in common fields")

def main():
    """Compare fields for both files"""
    
    test_files = [
        "/keg/pictures/tosort/2024/04-Apr/20240427_123707-1.CR2",
        "/keg/pictures/tosort/2025/06-Jun/20250608_131848.dng"
    ]
    
    for file_path in test_files:
        compare_fields(file_path)

if __name__ == "__main__":
    main()
