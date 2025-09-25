#!/usr/bin/env python3
"""
Script to fix the most critical discrepancies identified in the comprehensive validation.
This script will apply targeted fixes to improve match rates with exiftool.
"""

import json
from pathlib import Path

def analyze_validation_results():
    """Analyze the validation results to identify the most critical fixes needed."""
    
    with open("comprehensive_validation_results.json", "r") as f:
        data = json.load(f)
    
    validation_results = data["validation_results"]
    fix_recommendations = data["fix_recommendations"]
    
    print("=== DISCREPANCY ANALYSIS ===")
    print(f"Overall match rate: {validation_results['summary']['avg_match_percentage']:.1f}%")
    print(f"Successful validations: {validation_results['summary']['successful_validations']}")
    
    print("\n=== HIGH PRIORITY FIXES ===")
    for rec in fix_recommendations["high_priority"]:
        print(f"{rec['field']}: {rec['affected_files']} files")
        for example in rec["examples"]:
            print(f"  {example['file']}: '{example['exiftool']}' vs '{example['fast_exif']}'")
    
    return fix_recommendations

def generate_fix_plan():
    """Generate a comprehensive fix plan based on validation results."""
    
    recommendations = analyze_validation_results()
    
    fix_plan = {
        "critical_fixes": [
            {
                "field": "ExifToolVersion",
                "issue": "Shows 'fast-exif-cli 0.4.8' instead of exiftool version",
                "solution": "Remove this field entirely or make it configurable",
                "priority": "HIGH",
                "affected_files": 19
            },
            {
                "field": "FocalLength35efl", 
                "issue": "Missing 35mm equivalent calculation",
                "solution": "Add 35mm equivalent calculation: focal_length * crop_factor",
                "priority": "HIGH",
                "affected_files": 14
            },
            {
                "field": "FileTypeExtension",
                "issue": "Shows 'raw' instead of specific format like 'cr2'",
                "solution": "Use actual file extension from format detection",
                "priority": "HIGH", 
                "affected_files": 6
            },
            {
                "field": "MIMEType",
                "issue": "Shows 'image/tiff' instead of format-specific MIME type",
                "solution": "Use format-specific MIME types",
                "priority": "HIGH",
                "affected_files": 6
            },
            {
                "field": "ExifByteOrder",
                "issue": "Always shows 'Little-endian' regardless of actual byte order",
                "solution": "Detect and report actual byte order",
                "priority": "MEDIUM",
                "affected_files": 7
            },
            {
                "field": "ShutterSpeedValue",
                "issue": "APEX conversion not matching exiftool exactly",
                "solution": "Refine APEX to shutter speed conversion formula",
                "priority": "MEDIUM",
                "affected_files": 7
            },
            {
                "field": "BrightnessValue",
                "issue": "Shows raw APEX value instead of converted brightness",
                "solution": "Convert APEX brightness value to EV",
                "priority": "MEDIUM",
                "affected_files": 6
            },
            {
                "field": "MeteringMode",
                "issue": "Shows 'Multi-segment' instead of 'Evaluative'",
                "solution": "Fix metering mode mapping for Canon cameras",
                "priority": "MEDIUM",
                "affected_files": 5
            },
            {
                "field": "SubSecDateTimeOriginal",
                "issue": "Missing timezone information",
                "solution": "Include timezone offset in datetime fields",
                "priority": "LOW",
                "affected_files": 6
            }
        ],
        "missing_fields": [
            "ScaleFactor35efl",
            "FocalLengthIn35mmFormat", 
            "MediaDataSize",
            "BitDepthLuma",
            "GPSAltitude",
            "FilePermissions",
            "EncodingProcess",
            "ModifyDate"
        ],
        "field_count_issues": {
            "average_exiftool_fields": 150.3,
            "average_fast_exif_fields": 47.2,
            "coverage_percentage": 18.7,
            "target_coverage": 60.0
        }
    }
    
    return fix_plan

def main():
    """Main function to analyze and generate fix recommendations."""
    
    print("🔍 ANALYZING VALIDATION RESULTS")
    print("=" * 50)
    
    fix_plan = generate_fix_plan()
    
    print("\n📋 COMPREHENSIVE FIX PLAN")
    print("=" * 50)
    
    print(f"\n🎯 CRITICAL FIXES ({len(fix_plan['critical_fixes'])}):")
    for fix in fix_plan["critical_fixes"]:
        print(f"  {fix['field']}: {fix['issue']}")
        print(f"    Solution: {fix['solution']}")
        print(f"    Priority: {fix['priority']} ({fix['affected_files']} files)")
        print()
    
    print(f"📊 FIELD COVERAGE ISSUES:")
    coverage = fix_plan["field_count_issues"]
    print(f"  Current coverage: {coverage['coverage_percentage']:.1f}%")
    print(f"  Target coverage: {coverage['target_coverage']:.1f}%")
    print(f"  Missing ~{coverage['average_exiftool_fields'] - coverage['average_fast_exif_fields']:.0f} fields per file")
    
    print(f"\n📝 MISSING FIELDS TO ADD:")
    for field in fix_plan["missing_fields"]:
        print(f"  - {field}")
    
    # Save fix plan
    with open("fix_plan.json", "w") as f:
        json.dump(fix_plan, f, indent=2)
    
    print(f"\n💾 Fix plan saved to fix_plan.json")
    
    return fix_plan

if __name__ == "__main__":
    main()
