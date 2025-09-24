#!/usr/bin/env python3
"""
Compare field counts between fast-exif-rs and exiftool
"""

import sys
import os
import subprocess
import json
sys.path.insert(0, '/projects/fast-exif-rs/python')

from fast_exif_reader import FastExifReader

def run_exiftool(file_path):
    """Run exiftool and return field count"""
    try:
        result = subprocess.run(['exiftool', '-s', file_path], 
                              capture_output=True, text=True, check=True)
        lines = [line.strip() for line in result.stdout.split('\n') if line.strip()]
        return len(lines), lines
    except subprocess.CalledProcessError as e:
        print(f"Error running exiftool: {e}")
        return 0, []
    except FileNotFoundError:
        print("exiftool not found")
        return 0, []

def test_file(file_path):
    """Test both fast-exif-rs and exiftool on a file"""
    if not os.path.exists(file_path):
        print(f"File not found: {file_path}")
        return
    
    print(f"\n{'='*60}")
    print(f"Testing: {os.path.basename(file_path)}")
    print(f"{'='*60}")
    
    # Test fast-exif-rs
    try:
        reader = FastExifReader()
        metadata = reader.read_file(file_path)
        fast_exif_count = len(metadata)
        print(f"fast-exif-rs fields: {fast_exif_count}")
    except Exception as e:
        print(f"fast-exif-rs error: {e}")
        fast_exif_count = 0
        metadata = {}
    
    # Test exiftool
    exiftool_count, exiftool_lines = run_exiftool(file_path)
    print(f"exiftool fields: {exiftool_count}")
    
    # Compare counts
    difference = exiftool_count - fast_exif_count
    percentage = (fast_exif_count / exiftool_count * 100) if exiftool_count > 0 else 0
    
    print(f"\nComparison:")
    print(f"  fast-exif-rs: {fast_exif_count} fields")
    print(f"  exiftool:     {exiftool_count} fields")
    print(f"  Difference:   {difference} fields")
    print(f"  Coverage:     {percentage:.1f}%")
    
    # Show some key fields from both
    print(f"\nKey fields from fast-exif-rs:")
    key_fields = ['Format', 'Make', 'Model', 'DateTimeOriginal', 'SubSecTimeOriginal', 'SubSecDateTimeOriginal']
    for field in key_fields:
        if field in metadata:
            print(f"  {field}: {metadata[field]}")
    
    print(f"\nKey fields from exiftool:")
    for line in exiftool_lines:
        if any(field in line for field in key_fields):
            print(f"  {line}")
    
    return {
        'file': os.path.basename(file_path),
        'fast_exif_count': fast_exif_count,
        'exiftool_count': exiftool_count,
        'difference': difference,
        'coverage_percentage': percentage
    }

def main():
    """Compare field counts for CR2 and DNG files"""
    
    test_files = [
        "/keg/pictures/tosort/2024/04-Apr/20240427_123707-1.CR2",
        "/keg/pictures/tosort/2025/06-Jun/20250608_131848.dng"
    ]
    
    results = []
    
    for file_path in test_files:
        result = test_file(file_path)
        if result:
            results.append(result)
    
    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    
    for result in results:
        print(f"{result['file']:30} | {result['fast_exif_count']:3} | {result['exiftool_count']:3} | {result['difference']:+3} | {result['coverage_percentage']:5.1f}%")
    
    print(f"{'='*60}")
    print("Legend: fast-exif-rs | exiftool | difference | coverage")
    print(f"{'='*60}")

if __name__ == "__main__":
    main()
