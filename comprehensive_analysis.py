#!/usr/bin/env python3
"""
Comprehensive analysis of differences between fast-exif-rs and exiftool
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

def analyze_differences(file_path):
    """Analyze differences for a specific file"""
    if not os.path.exists(file_path):
        print(f"File not found: {file_path}")
        return
    
    print(f"\n{'='*80}")
    print(f"COMPREHENSIVE ANALYSIS: {os.path.basename(file_path)}")
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
    
    # Find common fields with differences
    common_fields = set(fast_exif_fields.keys()) & set(exiftool_fields.keys())
    differences = []
    
    for field in sorted(common_fields):
        fast_val = fast_exif_fields[field]
        exif_val = exiftool_fields[field]
        if fast_val != exif_val:
            differences.append((field, fast_val, exif_val))
    
    print(f"Total common fields: {len(common_fields)}")
    print(f"Fields with differences: {len(differences)}")
    
    # Categorize differences
    rational_fields = []
    enum_fields = []
    computed_fields = []
    other_fields = []
    
    for field, fast_val, exif_val in differences:
        if '/' in fast_val and not '/' in exif_val:
            rational_fields.append((field, fast_val, exif_val))
        elif field in ['ColorSpace', 'Compression', 'CustomRendered', 'DriveMode', 'GPSAltitudeRef', 'GPSLatitudeRef', 'GPSLongitudeRef']:
            enum_fields.append((field, fast_val, exif_val))
        elif field in ['DOF', 'CircleOfConfusion', 'FocalLength35efl']:
            computed_fields.append((field, fast_val, exif_val))
        else:
            other_fields.append((field, fast_val, exif_val))
    
    print(f"\nRATIONAL VALUE DIFFERENCES ({len(rational_fields)}):")
    for field, fast_val, exif_val in rational_fields:
        print(f"  {field}:")
        print(f"    fast-exif-rs: {fast_val}")
        print(f"    exiftool:     {exif_val}")
    
    print(f"\nENUM VALUE DIFFERENCES ({len(enum_fields)}):")
    for field, fast_val, exif_val in enum_fields:
        print(f"  {field}:")
        print(f"    fast-exif-rs: {fast_val}")
        print(f"    exiftool:     {exif_val}")
    
    print(f"\nCOMPUTED FIELD DIFFERENCES ({len(computed_fields)}):")
    for field, fast_val, exif_val in computed_fields:
        print(f"  {field}:")
        print(f"    fast-exif-rs: {fast_val}")
        print(f"    exiftool:     {exif_val}")
    
    print(f"\nOTHER DIFFERENCES ({len(other_fields)}):")
    for field, fast_val, exif_val in other_fields:
        print(f"  {field}:")
        print(f"    fast-exif-rs: {fast_val}")
        print(f"    exiftool:     {exif_val}")
    
    # Show fields only in exiftool (what we're missing)
    only_exiftool = set(exiftool_fields.keys()) - set(fast_exif_fields.keys())
    if only_exiftool:
        print(f"\nFIELDS ONLY IN EXIFTOOL ({len(only_exiftool)}):")
        dng_specific = []
        camera_specific = []
        other_missing = []
        
        for field in sorted(only_exiftool):
            value = exiftool_fields[field]
            if field.startswith('DNG') or field in ['ActiveArea', 'AsShotNeutral', 'BlackLevel', 'CFALayout', 'CFAPattern', 'CameraCalibration', 'ColorMatrix']:
                dng_specific.append((field, value))
            elif field.startswith('AF') or field.startswith('AEB') or field.startswith('Canon') or field.startswith('Nikon'):
                camera_specific.append((field, value))
            else:
                other_missing.append((field, value))
        
        if dng_specific:
            print(f"  DNG-specific fields ({len(dng_specific)}):")
            for field, value in dng_specific[:10]:
                print(f"    {field}: {value}")
            if len(dng_specific) > 10:
                print(f"    ... and {len(dng_specific) - 10} more")
        
        if camera_specific:
            print(f"  Camera-specific fields ({len(camera_specific)}):")
            for field, value in camera_specific[:10]:
                print(f"    {field}: {value}")
            if len(camera_specific) > 10:
                print(f"    ... and {len(camera_specific) - 10} more")
        
        if other_missing:
            print(f"  Other missing fields ({len(other_missing)}):")
            for field, value in other_missing[:10]:
                print(f"    {field}: {value}")
            if len(other_missing) > 10:
                print(f"    ... and {len(other_missing) - 10} more")
    
    return {
        'rational_fields': rational_fields,
        'enum_fields': enum_fields,
        'computed_fields': computed_fields,
        'other_fields': other_fields,
        'missing_fields': list(only_exiftool)
    }

def main():
    """Analyze both files"""
    
    test_files = [
        "/keg/pictures/tosort/2024/04-Apr/20240427_123707-1.CR2",
        "/keg/pictures/tosort/2025/06-Jun/20250608_131848.dng"
    ]
    
    all_results = {}
    
    for file_path in test_files:
        result = analyze_differences(file_path)
        if result:
            all_results[os.path.basename(file_path)] = result
    
    # Summary
    print(f"\n{'='*80}")
    print("SUMMARY OF ISSUES TO FIX")
    print(f"{'='*80}")
    
    for filename, result in all_results.items():
        print(f"\n{filename}:")
        print(f"  Rational value issues: {len(result['rational_fields'])}")
        print(f"  Enum value issues: {len(result['enum_fields'])}")
        print(f"  Computed field issues: {len(result['computed_fields'])}")
        print(f"  Other issues: {len(result['other_fields'])}")
        print(f"  Missing fields: {len(result['missing_fields'])}")

if __name__ == "__main__":
    main()
