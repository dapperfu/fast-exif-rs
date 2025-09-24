#!/usr/bin/env python3
"""
Compare exiftool -s output with fast-exif-rs output on 100 randomly selected files.
Files are equally spaced between 2001-2025 years.
"""

import os
import sys
import subprocess
import json
import random
import tempfile
from pathlib import Path
from collections import defaultdict
import statistics

# Add the current directory to Python path to import our module
sys.path.insert(0, '/projects/fast-exif-rs')

try:
    import fast_exif_reader
except ImportError:
    print("Error: fast-exif-rs module not found. Make sure it's built and installed.")
    sys.exit(1)

def find_files_by_year():
    """Find files organized by year directories"""
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
    
    return files_by_year

def select_files(files_by_year, num_files=1000):
    """Select files equally spaced between 2001-2025"""
    years = sorted(files_by_year.keys())
    files_per_year = num_files // len(years)
    remainder = num_files % len(years)
    
    selected_files = []
    
    for i, year in enumerate(years):
        year_files = files_by_year[year]
        if not year_files:
            continue
            
        # Add remainder to first few years
        count = files_per_year + (1 if i < remainder else 0)
        
        # Randomly sample files from this year
        if len(year_files) >= count:
            selected_files.extend(random.sample(year_files, count))
        else:
            selected_files.extend(year_files)
    
    return selected_files[:num_files]

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

def compare_fields(exif_data, fast_data):
    """Compare fields between exiftool and fast-exif-rs"""
    comparison = {
        'exiftool_only': [],
        'fast_exif_only': [],
        'common_fields': [],
        'value_matches': 0,
        'value_differences': []
    }
    
    exif_keys = set(exif_data.keys())
    fast_keys = set(fast_data.keys())
    
    comparison['exiftool_only'] = list(exif_keys - fast_keys)
    comparison['fast_exif_only'] = list(fast_keys - exif_keys)
    comparison['common_fields'] = list(exif_keys & fast_keys)
    
    # Compare values for common fields
    for field in comparison['common_fields']:
        exif_value = exif_data[field]
        fast_value = fast_data[field]
        
        if exif_value == fast_value:
            comparison['value_matches'] += 1
        else:
            comparison['value_differences'].append({
                'field': field,
                'exiftool': exif_value,
                'fast_exif': fast_value
            })
    
    return comparison

def analyze_results(results):
    """Analyze comparison results"""
    total_files = len(results)
    successful_comparisons = [r for r in results if r['exif_data'] and r['fast_data']]
    
    if not successful_comparisons:
        return "No successful comparisons found"
    
    # Aggregate statistics
    stats = {
        'total_files': total_files,
        'successful_comparisons': len(successful_comparisons),
        'exiftool_failures': len([r for r in results if not r['exif_data']]),
        'fast_exif_failures': len([r for r in results if not r['fast_data']]),
        'avg_exiftool_fields': statistics.mean([len(r['exif_data']) for r in successful_comparisons]),
        'avg_fast_exif_fields': statistics.mean([len(r['fast_data']) for r in successful_comparisons]),
        'avg_common_fields': statistics.mean([len(r['comparison']['common_fields']) for r in successful_comparisons]),
        'avg_value_matches': statistics.mean([r['comparison']['value_matches'] for r in successful_comparisons]),
        'avg_value_differences': statistics.mean([len(r['comparison']['value_differences']) for r in successful_comparisons])
    }
    
    # Most common field differences
    field_diffs = defaultdict(int)
    for result in successful_comparisons:
        for diff in result['comparison']['value_differences']:
            field_diffs[diff['field']] += 1
    
    stats['most_common_differences'] = sorted(field_diffs.items(), key=lambda x: x[1], reverse=True)[:10]
    
    return stats

def main():
    print("=== EXIF Tool Comparison: exiftool vs fast-exif-rs ===")
    print("Selecting 1000 files equally spaced between 2001-2025...")
    
    # Find files by year
    files_by_year = find_files_by_year()
    print(f"Found files in years: {sorted(files_by_year.keys())}")
    
    # Select files
    selected_files = select_files(files_by_year, 1000)
    print(f"Selected {len(selected_files)} files for comparison")
    
    # Compare files
    results = []
    
    for i, file_path in enumerate(selected_files, 1):
        # Progress reporting every 50 files
        if i % 50 == 0 or i == 1:
            print(f"\n[{i}/{len(selected_files)}] Processing: {file_path.name}")
        
        # Run exiftool
        exif_data, exif_error = run_exiftool(file_path)
        
        # Run fast-exif-rs
        fast_data, fast_error = run_fast_exif_rs(file_path)
        
        # Compare if both succeeded
        comparison = None
        if exif_data and fast_data:
            comparison = compare_fields(exif_data, fast_data)
        
        try:
            file_size = file_path.stat().st_size
        except FileNotFoundError:
            file_size = 0
            
        result = {
            'file_path': str(file_path),
            'file_name': file_path.name,
            'file_size': file_size,
            'exif_data': exif_data,
            'exif_error': exif_error,
            'fast_data': fast_data,
            'fast_error': fast_error,
            'comparison': comparison
        }
        
        results.append(result)
        
        # Print brief summary every 50 files
        if i % 50 == 0 or i == 1:
            if exif_data and fast_data:
                common_fields = len(comparison['common_fields'])
                value_matches = comparison['value_matches']
                value_diffs = len(comparison['value_differences'])
                print(f"  Common fields: {common_fields}, Value matches: {value_matches}, Differences: {value_diffs}")
            else:
                print(f"  exiftool: {'✓' if exif_data else '✗'}, fast-exif-rs: {'✓' if fast_data else '✗'}")
    
    # Analyze results
    print("\n" + "="*60)
    print("ANALYSIS RESULTS")
    print("="*60)
    
    stats = analyze_results(results)
    
    if isinstance(stats, str):
        print(stats)
    else:
        print(f"Total files processed: {stats['total_files']}")
        print(f"Successful comparisons: {stats['successful_comparisons']}")
        print(f"exiftool failures: {stats['exiftool_failures']}")
        print(f"fast-exif-rs failures: {stats['fast_exif_failures']}")
        print(f"Average exiftool fields: {stats['avg_exiftool_fields']:.1f}")
        print(f"Average fast-exif-rs fields: {stats['avg_fast_exif_fields']:.1f}")
        print(f"Average common fields: {stats['avg_common_fields']:.1f}")
        print(f"Average value matches: {stats['avg_value_matches']:.1f}")
        print(f"Average value differences: {stats['avg_value_differences']:.1f}")
        
        print("\nMost common field differences:")
        for field, count in stats['most_common_differences']:
            print(f"  {field}: {count} differences")
    
    # Save detailed results
    output_file = "/projects/fast-exif-rs/comparison_results.json"
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2, default=str)
    
    print(f"\nDetailed results saved to: {output_file}")
    
    # Print some example differences
    print("\n" + "="*60)
    print("EXAMPLE VALUE DIFFERENCES")
    print("="*60)
    
    example_count = 0
    for result in results:
        if result['comparison'] and result['comparison']['value_differences']:
            print(f"\nFile: {result['file_name']}")
            for diff in result['comparison']['value_differences'][:3]:  # Show first 3 differences
                print(f"  Field: {diff['field']}")
                print(f"    exiftool: {diff['exiftool']}")
                print(f"    fast-exif: {diff['fast_exif']}")
                example_count += 1
                if example_count >= 10:  # Limit examples
                    break
        if example_count >= 10:
            break

if __name__ == "__main__":
    main()
