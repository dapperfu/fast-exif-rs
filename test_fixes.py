#!/usr/bin/env python3
"""
Test the fixes on a small sample of files to verify improvements.
"""

import os
import sys
import subprocess
import json
import random
from pathlib import Path

# Add the current directory to Python path to import our module
sys.path.insert(0, '/projects/fast-exif-rs')

try:
    import fast_exif_reader
except ImportError:
    print("Error: fast-exif-rs module not found. Make sure it's built and installed.")
    sys.exit(1)

def run_exiftool(file_path):
    """Run exiftool -s on a file and return parsed output"""
    try:
        result = subprocess.run(
            ['exiftool', '-s', str(file_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode != 0:
            return None, f"exiftool error: {result.stderr}"
        
        # Parse exiftool output
        exif_data = {}
        for line in result.stdout.strip().split('\n'):
            if ':' in line:
                key, value = line.split(':', 1)
                exif_data[key.strip()] = value.strip()
        
        return exif_data, None
        
    except subprocess.TimeoutExpired:
        return None, "exiftool timeout"
    except Exception as e:
        return None, f"exiftool exception: {str(e)}"

def run_fast_exif_rs(file_path):
    """Run fast-exif-rs on a file and return parsed output"""
    try:
        reader = fast_exif_reader.FastExifReader()
        metadata = reader.read_file(str(file_path))
        
        # Convert to dict if it's a PyObject
        if hasattr(metadata, '__dict__'):
            return dict(metadata.__dict__), None
        elif isinstance(metadata, dict):
            return metadata, None
        else:
            return {}, "Unknown metadata format"
            
    except Exception as e:
        return None, f"fast-exif-rs exception: {str(e)}"

def test_specific_fields(exif_data, fast_data):
    """Test specific fields that we fixed"""
    fixes_tested = {
        'Orientation': False,
        'XResolution': False,
        'YResolution': False,
        'ResolutionUnit': False,
        'YCbCrPositioning': False,
        'Format': False,
        'ExifToolVersion': False
    }
    
    for field in fixes_tested.keys():
        if field in exif_data and field in fast_data:
            exif_value = exif_data[field]
            fast_value = fast_data[field]
            fixes_tested[field] = (exif_value == fast_value)
    
    return fixes_tested

def main():
    print("=== Testing EXIF Fixes ===")
    
    # Find a few test files
    pictures_dir = Path("/keg/pictures")
    test_files = []
    
    # Look for files in different years
    for year_dir in pictures_dir.iterdir():
        if year_dir.is_dir() and year_dir.name.isdigit():
            year = int(year_dir.name)
            if 2001 <= year <= 2025:
                for file_path in year_dir.rglob('*.jpg'):
                    if file_path.is_file():
                        test_files.append(file_path)
                        if len(test_files) >= 5:  # Test 5 files
                            break
                if len(test_files) >= 5:
                    break
    
    print(f"Testing {len(test_files)} files...")
    
    total_fixes = 0
    successful_fixes = 0
    
    for i, file_path in enumerate(test_files, 1):
        print(f"\n[{i}/{len(test_files)}] Testing: {file_path.name}")
        
        # Run exiftool
        exif_data, exif_error = run_exiftool(file_path)
        
        # Run fast-exif-rs
        fast_data, fast_error = run_fast_exif_rs(file_path)
        
        if exif_data and fast_data:
            fixes_tested = test_specific_fields(exif_data, fast_data)
            
            print("  Fixes tested:")
            for field, success in fixes_tested.items():
                status = "✓" if success else "✗"
                print(f"    {field}: {status}")
                total_fixes += 1
                if success:
                    successful_fixes += 1
            
            # Show some example values
            print("  Example values:")
            for field in ['Orientation', 'XResolution', 'YResolution', 'Format']:
                if field in exif_data and field in fast_data:
                    print(f"    {field}: exiftool='{exif_data[field]}', fast-exif='{fast_data[field]}'")
        else:
            print(f"  exiftool: {'✓' if exif_data else '✗'}, fast-exif-rs: {'✓' if fast_data else '✗'}")
    
    print(f"\n=== SUMMARY ===")
    print(f"Total fixes tested: {total_fixes}")
    print(f"Successful fixes: {successful_fixes}")
    print(f"Success rate: {successful_fixes/total_fixes*100:.1f}%" if total_fixes > 0 else "No fixes tested")

if __name__ == "__main__":
    main()
