#!/usr/bin/env python3
"""
Analyze the most common value differences to identify patterns and fix them.
"""

import json
from collections import defaultdict, Counter

def main():
    # Load the comparison results
    with open('/projects/fast-exif-rs/comparison_results.json', 'r') as f:
        results = json.load(f)
    
    successful_comparisons = [r for r in results if r['exif_data'] and r['fast_data']]
    
    # Count field differences
    field_diffs = defaultdict(list)
    
    for result in successful_comparisons:
        for diff in result['comparison']['value_differences']:
            field = diff['field']
            exif_value = diff['exiftool']
            fast_value = diff['fast_exif']
            
            field_diffs[field].append({
                'exif': exif_value,
                'fast': fast_value,
                'file': result['file_name']
            })
    
    print("=== TOP 20 FIELD DIFFERENCES ===")
    print("Field Name | Count | Most Common Pattern")
    print("-" * 80)
    
    # Sort by frequency
    sorted_fields = sorted(field_diffs.items(), key=lambda x: len(x[1]), reverse=True)
    
    for field, diffs in sorted_fields[:20]:
        count = len(diffs)
        
        # Find most common pattern
        patterns = Counter()
        for diff in diffs[:100]:  # Sample first 100
            pattern = f"{diff['fast']} -> {diff['exif']}"
            patterns[pattern] += 1
        
        most_common = patterns.most_common(1)[0] if patterns else ("N/A", 0)
        pattern_text = most_common[0][:50] + "..." if len(most_common[0]) > 50 else most_common[0]
        
        print(f"{field:<20} | {count:>5} | {pattern_text}")
    
    print("\n=== DETAILED ANALYSIS OF TOP 10 FIELDS ===")
    
    for field, diffs in sorted_fields[:10]:
        print(f"\n{field} ({len(diffs)} differences):")
        
        # Count unique patterns
        patterns = Counter()
        for diff in diffs:
            pattern = f"{diff['fast']} -> {diff['exif']}"
            patterns[pattern] += 1
        
        # Show top 5 patterns
        for pattern, count in patterns.most_common(5):
            percentage = (count / len(diffs)) * 100
            print(f"  {pattern}: {count} times ({percentage:.1f}%)")
    
    # Analyze specific problematic fields
    print("\n=== SPECIFIC FIELD ANALYSIS ===")
    
    # ExposureCompensation analysis
    if 'ExposureCompensation' in field_diffs:
        print("\nExposureCompensation patterns:")
        patterns = Counter()
        for diff in field_diffs['ExposureCompensation']:
            patterns[f"{diff['fast']} -> {diff['exif']}"] += 1
        
        for pattern, count in patterns.most_common(10):
            print(f"  {pattern}: {count} times")
    
    # ShutterSpeedValue analysis
    if 'ShutterSpeedValue' in field_diffs:
        print("\nShutterSpeedValue patterns:")
        patterns = Counter()
        for diff in field_diffs['ShutterSpeedValue']:
            patterns[f"{diff['fast']} -> {diff['exif']}"] += 1
        
        for pattern, count in patterns.most_common(10):
            print(f"  {pattern}: {count} times")
    
    # FlashpixVersion analysis
    if 'FlashpixVersion' in field_diffs:
        print("\nFlashpixVersion patterns:")
        patterns = Counter()
        for diff in field_diffs['FlashpixVersion']:
            patterns[f"{diff['fast']} -> {diff['exif']}"] += 1
        
        for pattern, count in patterns.most_common(10):
            print(f"  {pattern}: {count} times")

if __name__ == "__main__":
    main()
