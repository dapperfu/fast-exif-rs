#!/usr/bin/env python3
"""
Comprehensive validation script to test every file through exiftool and fast-exif-rs.
Validates both field counts and accuracy of field values, then identifies and fixes discrepancies.

This script:
1. Runs all test files through both exiftool (via Python subprocess) and fast-exif-rs
2. Compares field counts and values between the two tools
3. Identifies discrepancies in field counts and values
4. Provides detailed analysis of issues to fix
5. Generates comprehensive reports for fixing discrepancies
"""

import sys
import os
import subprocess
import json
import time
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Any
from collections import defaultdict
import statistics

# Add the python module to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

try:
    from fast_exif_reader import FastExifReader
except ImportError as e:
    print(f"Error importing fast_exif_reader: {e}")
    print("Please build the Rust extension first with: cargo build --release")
    sys.exit(1)

class ComprehensiveValidator:
    def __init__(self):
        self.reader = FastExifReader()
        self.test_files_dir = Path("test_files")
        self.results = {}
        self.field_analysis = defaultdict(list)
        self.discrepancy_summary = defaultdict(int)
        
    def get_exiftool_output(self, file_path: str) -> Tuple[Dict[str, str], Optional[str]]:
        """Get exiftool output for a file using Python subprocess."""
        try:
            # Run exiftool with JSON output for structured data
            result = subprocess.run(
                ["exiftool", "-json", "-s", file_path],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode == 0:
                data = json.loads(result.stdout)
                if data and len(data) > 0:
                    return data[0], None  # Return first (and usually only) entry
            return {}, f"exiftool returned code {result.returncode}: {result.stderr}"
        except subprocess.TimeoutExpired:
            return {}, "exiftool timeout"
        except json.JSONDecodeError as e:
            return {}, f"exiftool JSON decode error: {e}"
        except Exception as e:
            return {}, f"exiftool exception: {e}"
    
    def get_fast_exif_output(self, file_path: str) -> Tuple[Dict[str, str], Optional[str]]:
        """Get fast-exif-rs output for a file."""
        try:
            metadata = self.reader.read_file(file_path)
            # Ensure we have a proper dict
            if isinstance(metadata, dict):
                return metadata, None
            else:
                return dict(metadata), None
        except Exception as e:
            return {}, f"fast-exif-rs exception: {e}"
    
    def normalize_value(self, value: Any) -> str:
        """Normalize values for comparison."""
        if value is None:
            return ""
        return str(value).strip()
    
    def compare_field_values(self, exiftool_data: Dict[str, str], fast_exif_data: Dict[str, str]) -> Dict[str, Any]:
        """Compare field values between exiftool and fast-exif-rs."""
        comparison = {
            "field_counts": {
                "exiftool_total": len(exiftool_data),
                "fast_exif_total": len(fast_exif_data),
                "common_fields": 0,
                "exiftool_only": 0,
                "fast_exif_only": 0
            },
            "value_comparison": {
                "exact_matches": {},
                "differences": {},
                "missing_in_fast_exif": {},
                "extra_in_fast_exif": {}
            },
            "field_accuracy": {
                "total_common_fields": 0,
                "exact_matches": 0,
                "differences": 0,
                "match_percentage": 0.0
            }
        }
        
        # Get field sets
        exiftool_fields = set(exiftool_data.keys())
        fast_exif_fields = set(fast_exif_data.keys())
        common_fields = exiftool_fields & fast_exif_fields
        
        # Update field counts
        comparison["field_counts"]["common_fields"] = len(common_fields)
        comparison["field_counts"]["exiftool_only"] = len(exiftool_fields - fast_exif_fields)
        comparison["field_counts"]["fast_exif_only"] = len(fast_exif_fields - exiftool_fields)
        
        # Compare values for common fields
        for field in sorted(common_fields):
            exiftool_val = self.normalize_value(exiftool_data[field])
            fast_exif_val = self.normalize_value(fast_exif_data[field])
            
            if exiftool_val == fast_exif_val:
                comparison["value_comparison"]["exact_matches"][field] = exiftool_val
            else:
                comparison["value_comparison"]["differences"][field] = {
                    "exiftool": exiftool_val,
                    "fast_exif": fast_exif_val
                }
        
        # Handle missing/extra fields
        for field in exiftool_fields - fast_exif_fields:
            comparison["value_comparison"]["missing_in_fast_exif"][field] = exiftool_data[field]
        
        for field in fast_exif_fields - exiftool_fields:
            comparison["value_comparison"]["extra_in_fast_exif"][field] = fast_exif_data[field]
        
        # Calculate accuracy metrics
        total_common = len(common_fields)
        exact_matches = len(comparison["value_comparison"]["exact_matches"])
        differences = len(comparison["value_comparison"]["differences"])
        
        comparison["field_accuracy"]["total_common_fields"] = total_common
        comparison["field_accuracy"]["exact_matches"] = exact_matches
        comparison["field_accuracy"]["differences"] = differences
        comparison["field_accuracy"]["match_percentage"] = (exact_matches / total_common * 100) if total_common > 0 else 0.0
        
        return comparison
    
    def categorize_discrepancies(self, differences: Dict[str, Dict]) -> Dict[str, List]:
        """Categorize discrepancies by type for easier analysis."""
        categories = {
            "version_fields": [],
            "apex_conversions": [],
            "rational_values": [],
            "enum_mappings": [],
            "string_formatting": [],
            "numeric_precision": [],
            "unit_conversions": [],
            "other": []
        }
        
        for field, values in differences.items():
            exif_val = values["exiftool"]
            fast_val = values["fast_exif"]
            
            # Categorize based on field name and value patterns
            if field in ["ExifVersion", "FlashpixVersion", "InteroperabilityVersion"]:
                categories["version_fields"].append((field, exif_val, fast_val))
            elif field in ["ShutterSpeedValue", "ApertureValue", "MaxApertureValue", "ExposureCompensation"]:
                categories["apex_conversions"].append((field, exif_val, fast_val))
            elif "/" in str(fast_val) and "/" not in str(exif_val):
                categories["rational_values"].append((field, exif_val, fast_val))
            elif field in ["ExposureMode", "Flash", "WhiteBalance", "MeteringMode", "CustomRendered"]:
                categories["enum_mappings"].append((field, exif_val, fast_val))
            elif field in ["Make", "Model", "Software"] and isinstance(exif_val, str) and isinstance(fast_val, str):
                categories["string_formatting"].append((field, exif_val, fast_val))
            elif field in ["FocalPlaneResolutionUnit", "ResolutionUnit"]:
                categories["unit_conversions"].append((field, exif_val, fast_val))
            elif any(char.isdigit() for char in str(exif_val)) and any(char.isdigit() for char in str(fast_val)):
                categories["numeric_precision"].append((field, exif_val, fast_val))
            else:
                categories["other"].append((field, exif_val, fast_val))
        
        return categories
    
    def validate_file(self, file_path: Path) -> Dict[str, Any]:
        """Validate a single file through both tools."""
        print(f"\n{'='*80}")
        print(f"VALIDATING: {file_path.name}")
        print(f"{'='*80}")
        
        # Get outputs from both tools
        exiftool_data, exif_error = self.get_exiftool_output(str(file_path))
        fast_exif_data, fast_error = self.get_fast_exif_output(str(file_path))
        
        if exif_error:
            print(f"❌ exiftool error: {exif_error}")
            return {"status": "exiftool_error", "error": exif_error}
        
        if fast_error:
            print(f"❌ fast-exif-rs error: {fast_error}")
            return {"status": "fast_exif_error", "error": fast_error}
        
        # Compare the outputs
        comparison = self.compare_field_values(exiftool_data, fast_exif_data)
        
        # Print validation results
        print(f"📊 FIELD COUNT ANALYSIS:")
        counts = comparison["field_counts"]
        print(f"  exiftool fields: {counts['exiftool_total']}")
        print(f"  fast-exif-rs fields: {counts['fast_exif_total']}")
        print(f"  common fields: {counts['common_fields']}")
        print(f"  exiftool only: {counts['exiftool_only']}")
        print(f"  fast-exif-rs only: {counts['fast_exif_only']}")
        
        accuracy = comparison["field_accuracy"]
        print(f"\n📈 VALUE ACCURACY:")
        print(f"  Total common fields: {accuracy['total_common_fields']}")
        print(f"  Exact matches: {accuracy['exact_matches']}")
        print(f"  Differences: {accuracy['differences']}")
        print(f"  Match percentage: {accuracy['match_percentage']:.1f}%")
        
        # Show top differences
        differences = comparison["value_comparison"]["differences"]
        if differences:
            print(f"\n🔍 TOP DIFFERENCES:")
            for i, (field, values) in enumerate(list(differences.items())[:5]):
                print(f"  {field}:")
                print(f"    exiftool: '{values['exiftool']}'")
                print(f"    fast-exif: '{values['fast_exif']}'")
            if len(differences) > 5:
                print(f"  ... and {len(differences) - 5} more differences")
        
        # Show missing fields
        missing = comparison["value_comparison"]["missing_in_fast_exif"]
        if missing:
            print(f"\n⚠️  MISSING IN FAST-EXIF-RS ({len(missing)}):")
            for field in list(missing.keys())[:5]:
                print(f"  {field}: {missing[field]}")
            if len(missing) > 5:
                print(f"  ... and {len(missing) - 5} more missing fields")
        
        # Categorize discrepancies for analysis
        categorized = self.categorize_discrepancies(differences)
        
        return {
            "status": "success",
            "file": file_path.name,
            "file_path": str(file_path),
            "comparison": comparison,
            "categorized_discrepancies": categorized,
            "exiftool_data": exiftool_data,
            "fast_exif_data": fast_exif_data
        }
    
    def validate_all_files(self) -> Dict[str, Any]:
        """Validate all files in the test_files directory."""
        print("🚀 STARTING COMPREHENSIVE VALIDATION")
        print("=" * 80)
        
        if not self.test_files_dir.exists():
            print(f"❌ Test files directory not found: {self.test_files_dir}")
            return {}
        
        # Get all test files
        test_files = list(self.test_files_dir.glob("*"))
        test_files = [f for f in test_files if f.is_file()]
        
        print(f"📁 Found {len(test_files)} test files")
        
        results = {}
        successful_validations = 0
        total_match_percentage = 0.0
        
        for i, test_file in enumerate(test_files, 1):
            print(f"\n[{i}/{len(test_files)}] Processing {test_file.name}...")
            
            try:
                result = self.validate_file(test_file)
                results[test_file.name] = result
                
                if result["status"] == "success":
                    successful_validations += 1
                    total_match_percentage += result["comparison"]["field_accuracy"]["match_percentage"]
                    
                    # Track field-level analysis
                    for field, values in result["comparison"]["value_comparison"]["differences"].items():
                        self.field_analysis[field].append({
                            "file": test_file.name,
                            "exiftool": values["exiftool"],
                            "fast_exif": values["fast_exif"]
                        })
                        self.discrepancy_summary[field] += 1
                        
            except Exception as e:
                print(f"❌ Error validating {test_file.name}: {e}")
                results[test_file.name] = {"status": "error", "error": str(e)}
        
        # Generate comprehensive summary
        self.generate_summary(results, successful_validations, total_match_percentage)
        
        return {
            "summary": {
                "total_files": len(test_files),
                "successful_validations": successful_validations,
                "avg_match_percentage": total_match_percentage / successful_validations if successful_validations > 0 else 0
            },
            "results": results,
            "field_analysis": dict(self.field_analysis),
            "discrepancy_summary": dict(self.discrepancy_summary)
        }
    
    def generate_summary(self, results: Dict, successful_validations: int, total_match_percentage: float):
        """Generate comprehensive summary of validation results."""
        print(f"\n{'='*80}")
        print("📊 COMPREHENSIVE VALIDATION SUMMARY")
        print(f"{'='*80}")
        
        print(f"Total files processed: {len(results)}")
        print(f"Successful validations: {successful_validations}")
        print(f"Failed validations: {len(results) - successful_validations}")
        
        if successful_validations > 0:
            avg_match_percentage = total_match_percentage / successful_validations
            print(f"Average match percentage: {avg_match_percentage:.1f}%")
        
        # Files with lowest match rates
        print(f"\n🔍 FILES WITH LOWEST MATCH RATES:")
        sorted_results = sorted(
            [(name, result) for name, result in results.items() if result.get("status") == "success"],
            key=lambda x: x[1]["comparison"]["field_accuracy"]["match_percentage"]
        )
        
        for name, result in sorted_results[:5]:
            match_rate = result["comparison"]["field_accuracy"]["match_percentage"]
            print(f"  {name}: {match_rate:.1f}%")
        
        # Most problematic fields
        print(f"\n🔧 MOST PROBLEMATIC FIELDS:")
        sorted_discrepancies = sorted(self.discrepancy_summary.items(), key=lambda x: x[1], reverse=True)
        for field, count in sorted_discrepancies[:10]:
            print(f"  {field}: {count} files with discrepancies")
        
        # Field count analysis
        print(f"\n📈 FIELD COUNT ANALYSIS:")
        field_counts = []
        for result in results.values():
            if result.get("status") == "success":
                counts = result["comparison"]["field_counts"]
                field_counts.append({
                    "exiftool": counts["exiftool_total"],
                    "fast_exif": counts["fast_exif_total"],
                    "common": counts["common_fields"]
                })
        
        if field_counts:
            avg_exiftool = statistics.mean([fc["exiftool"] for fc in field_counts])
            avg_fast_exif = statistics.mean([fc["fast_exif"] for fc in field_counts])
            avg_common = statistics.mean([fc["common"] for fc in field_counts])
            
            print(f"  Average exiftool fields: {avg_exiftool:.1f}")
            print(f"  Average fast-exif-rs fields: {avg_fast_exif:.1f}")
            print(f"  Average common fields: {avg_common:.1f}")
            print(f"  Field coverage: {(avg_common/avg_exiftool*100):.1f}%")
    
    def generate_fix_recommendations(self, results: Dict) -> Dict[str, Any]:
        """Generate specific recommendations for fixing discrepancies."""
        recommendations = {
            "high_priority": [],
            "medium_priority": [],
            "low_priority": [],
            "missing_fields": [],
            "extra_fields": []
        }
        
        # Analyze discrepancies by frequency and impact
        field_impact = defaultdict(int)
        field_examples = defaultdict(list)
        
        for result in results.values():
            if result.get("status") == "success":
                for field, values in result["comparison"]["value_comparison"]["differences"].items():
                    field_impact[field] += 1
                    field_examples[field].append({
                        "file": result["file"],
                        "exiftool": values["exiftool"],
                        "fast_exif": values["fast_exif"]
                    })
        
        # Categorize by priority
        for field, count in field_impact.items():
            examples = field_examples[field][:3]  # First 3 examples
            
            if count >= 10:  # High priority - affects many files
                recommendations["high_priority"].append({
                    "field": field,
                    "affected_files": count,
                    "examples": examples
                })
            elif count >= 5:  # Medium priority
                recommendations["medium_priority"].append({
                    "field": field,
                    "affected_files": count,
                    "examples": examples
                })
            else:  # Low priority
                recommendations["low_priority"].append({
                    "field": field,
                    "affected_files": count,
                    "examples": examples
                })
        
        return recommendations

def main():
    """Run comprehensive validation."""
    validator = ComprehensiveValidator()
    
    # Run validation
    start_time = time.time()
    results = validator.validate_all_files()
    end_time = time.time()
    
    print(f"\n⏱️  Validation completed in {end_time - start_time:.2f} seconds")
    
    # Generate fix recommendations
    recommendations = validator.generate_fix_recommendations(results["results"])
    
    # Save detailed results
    output_data = {
        "validation_results": results,
        "fix_recommendations": recommendations,
        "timestamp": time.time(),
        "validation_duration": end_time - start_time
    }
    
    with open("comprehensive_validation_results.json", "w") as f:
        json.dump(output_data, f, indent=2, default=str)
    
    print(f"\n💾 Detailed results saved to comprehensive_validation_results.json")
    
    # Print fix recommendations
    print(f"\n{'='*80}")
    print("🔧 FIX RECOMMENDATIONS")
    print(f"{'='*80}")
    
    print(f"\nHIGH PRIORITY FIXES ({len(recommendations['high_priority'])}):")
    for rec in recommendations["high_priority"]:
        print(f"  {rec['field']}: {rec['affected_files']} files")
        for example in rec["examples"]:
            print(f"    {example['file']}: '{example['exiftool']}' vs '{example['fast_exif']}'")
    
    print(f"\nMEDIUM PRIORITY FIXES ({len(recommendations['medium_priority'])}):")
    for rec in recommendations["medium_priority"]:
        print(f"  {rec['field']}: {rec['affected_files']} files")
    
    print(f"\nLOW PRIORITY FIXES ({len(recommendations['low_priority'])}):")
    for rec in recommendations["low_priority"]:
        print(f"  {rec['field']}: {rec['affected_files']} files")
    
    # Return success/failure based on overall match rate
    avg_match_rate = results["summary"]["avg_match_percentage"]
    success_threshold = 90.0  # Target 90%+ match rate
    
    if avg_match_rate >= success_threshold:
        print(f"\n✅ VALIDATION SUCCESSFUL: {avg_match_rate:.1f}% match rate (target: {success_threshold}%)")
        return 0
    else:
        print(f"\n❌ VALIDATION NEEDS IMPROVEMENT: {avg_match_rate:.1f}% match rate (target: {success_threshold}%)")
        return 1

if __name__ == "__main__":
    sys.exit(main())
