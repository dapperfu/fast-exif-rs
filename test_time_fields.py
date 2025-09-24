#!/usr/bin/env python3
"""
Test that all time-related fields match exactly between exiftool and fast-exif-rs.
Time fields are critical and must be 1:1 matches with no errors.
"""

import os
import sys
import subprocess
import json
import random
from pathlib import Path
from collections import defaultdict

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

def extract_time_fields(data):
    """Extract all fields containing 'time' (case insensitive)"""
    time_fields = {}
    for key, value in data.items():
        if 'time' in key.lower():
            time_fields[key] = value
    return time_fields

def compare_time_fields(exif_times, fast_times):
    """Compare time fields between exiftool and fast-exif-rs"""
    comparison = {
        'exiftool_only': [],
        'fast_exif_only': [],
        'common_fields': [],
        'exact_matches': 0,
        'differences': []
    }
    
    exif_keys = set(exif_times.keys())
    fast_keys = set(fast_times.keys())
    
    comparison['exiftool_only'] = list(exif_keys - fast_keys)
    comparison['fast_exif_only'] = list(fast_keys - exif_keys)
    comparison['common_fields'] = list(exif_keys & fast_keys)
    
    # Compare values for common fields
    for field in comparison['common_fields']:
        exif_value = exif_times[field]
        fast_value = fast_times[field]
        
        if exif_value == fast_value:
            comparison['exact_matches'] += 1
        else:
            comparison['differences'].append({
                'field': field,
                'exiftool': exif_value,
                'fast_exif': fast_value
            })
    
    return comparison

def find_test_files(num_files=20):
    """Find test files from different years"""
    pictures_dir = Path("/keg/pictures")
    files_by_year = defaultdict(list)
    
    # Supported extensions
    extensions = {'.jpg', '.jpeg', '.cr2', '.nef', '.heic', '.heif', 
                  '.mov', '.mp4', '.3gp', '.tiff', '.tif', '.png', '.bmp'}
    
    print("Scanning files by year...")
    for year_dir in pictures_dir.iterdir():
        if year_dir.is_dir() and year_dir.name.isdigit():
            year = int(year_dir.name)
            if 2001 <= year <= 2025:
                for file_path in year_dir.rglob('*'):
                    if file_path.is_file() and file_path.suffix.lower() in extensions:
                        files_by_year[year].append(file_path)
    
    # Select files equally spaced between years
    years = sorted(files_by_year.keys())
    files_per_year = max(1, num_files // len(years))
    
    selected_files = []
    for year in years:
        year_files = files_by_year[year]
        if year_files:
            # Randomly sample files from this year
            sample_size = min(files_per_year, len(year_files))
            selected_files.extend(random.sample(year_files, sample_size))
    
    return selected_files[:num_files]

def main():
    print("=== TIME FIELDS VERIFICATION ===")
    print("Testing that all time-related fields match exactly between exiftool and fast-exif-rs")
    
    # Find test files
    test_files = find_test_files(20)
    print(f"Testing {len(test_files)} files...")
    
    results = []
    total_time_fields = 0
    exact_matches = 0
    differences = 0
    
    for i, file_path in enumerate(test_files, 1):
        print(f"\n[{i}/{len(test_files)}] Testing: {file_path.name}")
        
        # Run exiftool
        exif_data, exif_error = run_exiftool(file_path)
        
        # Run fast-exif-rs
        fast_data, fast_error = run_fast_exif_rs(file_path)
        
        if exif_data and fast_data:
            # Extract time fields
            exif_times = extract_time_fields(exif_data)
            fast_times = extract_time_fields(fast_data)
            
            print(f"  exiftool time fields: {len(exif_times)}")
            print(f"  fast-exif time fields: {len(fast_times)}")
            
            # Compare time fields
            comparison = compare_time_fields(exif_times, fast_times)
            
            print(f"  Common time fields: {len(comparison['common_fields'])}")
            print(f"  Exact matches: {comparison['exact_matches']}")
            print(f"  Differences: {len(comparison['differences'])}")
            
            if comparison['exiftool_only']:
                print(f"  exiftool only: {comparison['exiftool_only']}")
            if comparison['fast_exif_only']:
                print(f"  fast-exif only: {comparison['fast_exif_only']}")
            
            # Show differences
            if comparison['differences']:
                print("  TIME FIELD DIFFERENCES:")
                for diff in comparison['differences']:
                    print(f"    {diff['field']}:")
                    print(f"      exiftool: '{diff['exiftool']}'")
                    print(f"      fast-exif: '{diff['fast_exif']}'")
            
            # Accumulate statistics
            total_time_fields += len(comparison['common_fields'])
            exact_matches += comparison['exact_matches']
            differences += len(comparison['differences'])
            
            result = {
                'file_path': str(file_path),
                'file_name': file_path.name,
                'exif_times': exif_times,
                'fast_times': fast_times,
                'comparison': comparison
            }
            results.append(result)
            
        else:
            print(f"  exiftool: {'✓' if exif_data else '✗'}, fast-exif-rs: {'✓' if fast_data else '✗'}")
            if exif_error:
                print(f"    exiftool error: {exif_error}")
            if fast_error:
                print(f"    fast-exif error: {fast_error}")
    
    # Summary
    print("\n" + "="*60)
    print("TIME FIELDS VERIFICATION SUMMARY")
    print("="*60)
    
    if total_time_fields > 0:
        success_rate = (exact_matches / total_time_fields) * 100
        print(f"Total time fields compared: {total_time_fields}")
        print(f"Exact matches: {exact_matches}")
        print(f"Differences: {differences}")
        print(f"Success rate: {success_rate:.1f}%")
        
        if differences > 0:
            print(f"\n❌ CRITICAL: {differences} time field differences found!")
            print("Time fields must match exactly - no errors allowed.")
        else:
            print(f"\n✅ SUCCESS: All {exact_matches} time fields match exactly!")
    else:
        print("No time fields found to compare.")
    
    # Save detailed results
    output_file = "/projects/fast-exif-rs/time_fields_results.json"
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2, default=str)
    
    print(f"\nDetailed results saved to: {output_file}")
    
    # Show all time fields found
    print("\n" + "="*60)
    print("ALL TIME FIELDS FOUND")
    print("="*60)
    
    all_time_fields = set()
    for result in results:
        if 'exif_times' in result:
            all_time_fields.update(result['exif_times'].keys())
        if 'fast_times' in result:
            all_time_fields.update(result['fast_times'].keys())
    
    for field in sorted(all_time_fields):
        print(f"  {field}")

if __name__ == "__main__":
    main()
