#!/usr/bin/env python3
"""
Comprehensive comparison between exiftool and fast-exif-cli outputs
"""

import re
from collections import defaultdict

def parse_exiftool_output(filename):
    """Parse exiftool output into a dictionary"""
    fields = {}
    with open(filename, 'r') as f:
        for line in f:
            line = line.strip()
            if ':' in line and not line.startswith('ExifTool Version'):
                parts = line.split(':', 1)
                if len(parts) == 2:
                    key = parts[0].strip()
                    value = parts[1].strip()
                    fields[key] = value
    return fields

def parse_fast_exif_output(filename):
    """Parse fast-exif-cli output into a dictionary"""
    fields = {}
    with open(filename, 'r') as f:
        for line in f:
            line = line.strip()
            if ':' in line and not line.startswith('========'):
                parts = line.split(':', 1)
                if len(parts) == 2:
                    key = parts[0].strip()
                    value = parts[1].strip()
                    fields[key] = value
    return fields

def normalize_field_name(name):
    """Normalize field names for comparison"""
    # Remove common prefixes/suffixes that might differ
    name = name.replace('Composite:', '')
    name = name.replace('EXIF:', '')
    name = name.replace('MakerNotes:', '')
    name = name.replace('File:', '')
    return name.strip()

def compare_outputs():
    """Compare exiftool and fast-exif-cli outputs"""
    exiftool_fields = parse_exiftool_output('exiftool_output.txt')
    fast_exif_fields = parse_fast_exif_output('fast_exif_output.txt')
    
    print("=== COMPREHENSIVE COMPARISON REPORT ===\n")
    
    # Fields only in exiftool
    exiftool_only = set(exiftool_fields.keys()) - set(fast_exif_fields.keys())
    print(f"FIELDS ONLY IN EXIFTOOL ({len(exiftool_only)}):")
    for field in sorted(exiftool_only):
        print(f"  {field}: {exiftool_fields[field]}")
    print()
    
    # Fields only in fast-exif-cli
    fast_exif_only = set(fast_exif_fields.keys()) - set(exiftool_fields.keys())
    print(f"FIELDS ONLY IN FAST-EXIF-CLI ({len(fast_exif_only)}):")
    for field in sorted(fast_exif_only):
        print(f"  {field}: {fast_exif_fields[field]}")
    print()
    
    # Fields with different values
    common_fields = set(exiftool_fields.keys()) & set(fast_exif_fields.keys())
    different_values = []
    
    for field in sorted(common_fields):
        exiftool_val = exiftool_fields[field]
        fast_exif_val = fast_exif_fields[field]
        
        if exiftool_val != fast_exif_val:
            different_values.append((field, exiftool_val, fast_exif_val))
    
    print(f"FIELDS WITH DIFFERENT VALUES ({len(different_values)}):")
    for field, exiftool_val, fast_exif_val in different_values:
        print(f"  {field}:")
        print(f"    ExifTool: {exiftool_val}")
        print(f"    Fast-Exif: {fast_exif_val}")
        print()
    
    # Summary statistics
    print("=== SUMMARY ===")
    print(f"ExifTool fields: {len(exiftool_fields)}")
    print(f"Fast-Exif fields: {len(fast_exif_fields)}")
    print(f"Common fields: {len(common_fields)}")
    print(f"ExifTool only: {len(exiftool_only)}")
    print(f"Fast-Exif only: {len(fast_exif_only)}")
    print(f"Different values: {len(different_values)}")
    
    # Calculate match percentage
    total_fields = len(exiftool_fields)
    matching_fields = len(common_fields) - len(different_values)
    match_percentage = (matching_fields / total_fields) * 100 if total_fields > 0 else 0
    print(f"Match percentage: {match_percentage:.1f}%")

if __name__ == "__main__":
    compare_outputs()
