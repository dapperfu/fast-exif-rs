#!/usr/bin/env python3
"""
Script to analyze and fix the most common differences between exiftool and fast-exif-rs.
Focus on minimizing differences by improving field parsing and formatting.
"""

import json
from collections import defaultdict, Counter

def analyze_differences():
    """Analyze the comparison results to identify patterns"""
    with open('/projects/fast-exif-rs/comparison_results.json', 'r') as f:
        results = json.load(f)
    
    # Count field differences
    field_diffs = defaultdict(list)
    orientation_values = defaultdict(int)
    resolution_values = defaultdict(int)
    
    for result in results:
        if result['comparison']:
            for diff in result['comparison']['value_differences']:
                field = diff['field']
                exif_value = diff['exiftool']
                fast_value = diff['fast_exif']
                
                field_diffs[field].append({
                    'exif': exif_value,
                    'fast': fast_value,
                    'file': result['file_name']
                })
                
                # Special analysis for specific fields
                if field == 'Orientation':
                    orientation_values[f"{fast_value} -> {exif_value}"] += 1
                elif field in ['XResolution', 'YResolution']:
                    resolution_values[f"{fast_value} -> {exif_value}"] += 1
    
    return field_diffs, orientation_values, resolution_values

def main():
    print("=== Analyzing EXIF Differences ===")
    
    field_diffs, orientation_values, resolution_values = analyze_differences()
    
    print("\n=== ORIENTATION VALUES ===")
    for mapping, count in sorted(orientation_values.items(), key=lambda x: x[1], reverse=True):
        print(f"{mapping}: {count} files")
    
    print("\n=== RESOLUTION VALUES ===")
    for mapping, count in sorted(resolution_values.items(), key=lambda x: x[1], reverse=True)[:10]:
        print(f"{mapping}: {count} files")
    
    print("\n=== FIELD DIFFERENCE PATTERNS ===")
    for field, diffs in sorted(field_diffs.items(), key=lambda x: len(x[1]), reverse=True)[:10]:
        print(f"\n{field} ({len(diffs)} differences):")
        
        # Count unique patterns
        patterns = Counter()
        for diff in diffs[:20]:  # Sample first 20
            pattern = f"{diff['fast']} -> {diff['exif']}"
            patterns[pattern] += 1
        
        for pattern, count in patterns.most_common(5):
            print(f"  {pattern}: {count} times")
    
    # Generate recommendations
    print("\n=== RECOMMENDATIONS ===")
    
    if 'Orientation' in field_diffs:
        print("1. ORIENTATION: Convert numeric values to human-readable format")
        print("   - 1 -> 'Horizontal (normal)'")
        print("   - 3 -> 'Rotate 180'")
        print("   - 6 -> 'Rotate 90 CW'")
        print("   - 8 -> 'Rotate 90 CCW'")
    
    if 'XResolution' in field_diffs or 'YResolution' in field_diffs:
        print("2. RESOLUTION: Fix rational value parsing")
        print("   - Current: Returning raw TIFF values")
        print("   - Should: Parse as rational and return numerator")
        print("   - Most common: 72 (standard DPI)")
    
    if 'Format' in field_diffs:
        print("3. FORMAT: Standardize format field")
        print("   - Current: 'JPEG'")
        print("   - Should: 'image/jpeg' (MIME type)")
    
    if 'ExifToolVersion' in field_diffs:
        print("4. EXIFTOOLVERSION: Add version field")
        print("   - Should: 'fast-exif-cli 0.4.8'")
    
    print("\n=== NEXT STEPS ===")
    print("1. Fix TIFF rational value parsing in parsers/tiff.rs")
    print("2. Add orientation value mapping")
    print("3. Standardize format field naming")
    print("4. Add missing computed fields")

if __name__ == "__main__":
    main()
