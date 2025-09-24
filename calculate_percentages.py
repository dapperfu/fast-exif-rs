#!/usr/bin/env python3
"""
Calculate percentage success rates from the comparison results.
"""

import json

def main():
    # Load the comparison results
    with open('/projects/fast-exif-rs/comparison_results.json', 'r') as f:
        results = json.load(f)
    
    successful_comparisons = [r for r in results if r['exif_data'] and r['fast_data']]
    
    if not successful_comparisons:
        print("No successful comparisons found")
        return
    
    # Calculate statistics
    total_files = len(results)
    successful_files = len(successful_comparisons)
    
    # Calculate averages
    avg_exiftool_fields = sum(len(r['exif_data']) for r in successful_comparisons) / len(successful_comparisons)
    avg_fast_exif_fields = sum(len(r['fast_data']) for r in successful_comparisons) / len(successful_comparisons)
    avg_common_fields = sum(len(r['comparison']['common_fields']) for r in successful_comparisons) / len(successful_comparisons)
    avg_value_matches = sum(r['comparison']['value_matches'] for r in successful_comparisons) / len(successful_comparisons)
    avg_value_differences = sum(len(r['comparison']['value_differences']) for r in successful_comparisons) / len(successful_comparisons)
    
    # Calculate percentages
    field_match_percentage = (avg_value_matches / avg_common_fields) * 100 if avg_common_fields > 0 else 0
    field_coverage_percentage = (avg_common_fields / avg_exiftool_fields) * 100 if avg_exiftool_fields > 0 else 0
    
    print("=== DETAILED STATISTICS ===")
    print(f"Total files processed: {total_files}")
    print(f"Successful comparisons: {successful_files}")
    print(f"Overall success rate: {(successful_files/total_files)*100:.1f}%")
    print()
    print("=== FIELD STATISTICS (Raw Numbers) ===")
    print(f"Average exiftool fields: {avg_exiftool_fields:.1f}")
    print(f"Average fast-exif-rs fields: {avg_fast_exif_fields:.1f}")
    print(f"Average common fields: {avg_common_fields:.1f}")
    print(f"Average value matches: {avg_value_matches:.1f}")
    print(f"Average value differences: {avg_value_differences:.1f}")
    print()
    print("=== PERCENTAGE STATISTICS ===")
    print(f"Field coverage: {field_coverage_percentage:.1f}% (common fields / exiftool fields)")
    print(f"Value match rate: {field_match_percentage:.1f}% (exact matches / common fields)")
    print(f"Value difference rate: {(avg_value_differences/avg_common_fields)*100:.1f}% (differences / common fields)")
    
    # Show distribution of success rates
    print("\n=== SUCCESS RATE DISTRIBUTION ===")
    success_rates = []
    for r in successful_comparisons:
        if r['comparison']['common_fields']:
            rate = (r['comparison']['value_matches'] / len(r['comparison']['common_fields'])) * 100
            success_rates.append(rate)
    
    if success_rates:
        success_rates.sort()
        print(f"Min success rate: {min(success_rates):.1f}%")
        print(f"Max success rate: {max(success_rates):.1f}%")
        print(f"Median success rate: {success_rates[len(success_rates)//2]:.1f}%")
        
        # Count files by success rate ranges
        excellent = sum(1 for r in success_rates if r >= 90)
        good = sum(1 for r in success_rates if 80 <= r < 90)
        fair = sum(1 for r in success_rates if 70 <= r < 80)
        poor = sum(1 for r in success_rates if r < 70)
        
        print(f"\nFiles with 90%+ value matches: {excellent} ({excellent/len(success_rates)*100:.1f}%)")
        print(f"Files with 80-89% value matches: {good} ({good/len(success_rates)*100:.1f}%)")
        print(f"Files with 70-79% value matches: {fair} ({fair/len(success_rates)*100:.1f}%)")
        print(f"Files with <70% value matches: {poor} ({poor/len(success_rates)*100:.1f}%)")

if __name__ == "__main__":
    main()
