#!/usr/bin/env python3
"""
Comprehensive EXIF testing script to validate all fixes against exiftool.

This script tests each file in test_files directory individually and compares
results with exiftool to identify remaining issues and ensure feature completeness.
"""

import sys
import os
import subprocess
import json
from pathlib import Path
from typing import Dict, List, Tuple, Optional

# Add the python module to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

try:
    from fast_exif_reader import FastExifReader
except ImportError as e:
    print(f"Error importing fast_exif_reader: {e}")
    print("Please build the Rust extension first with: cargo build --release")
    sys.exit(1)

class ExifTester:
    def __init__(self):
        self.reader = FastExifReader()
        self.test_files_dir = Path("test_files")
        self.results = {}
        
    def get_exiftool_output(self, file_path: str) -> Dict[str, str]:
        """Get exiftool output for a file."""
        try:
            # Run exiftool with JSON output
            result = subprocess.run(
                ["exiftool", "-json", file_path],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode == 0:
                data = json.loads(result.stdout)
                if data and len(data) > 0:
                    return data[0]  # Return first (and usually only) entry
            return {}
        except Exception as e:
            print(f"Error running exiftool on {file_path}: {e}")
            return {}
    
    def get_fast_exif_output(self, file_path: str) -> Dict[str, str]:
        """Get fast-exif-rs output for a file."""
        try:
            return self.reader.read_file(file_path)
        except Exception as e:
            print(f"Error reading {file_path} with fast-exif-rs: {e}")
            return {}
    
    def compare_fields(self, exiftool_data: Dict[str, str], fast_exif_data: Dict[str, str]) -> Dict[str, Dict]:
        """Compare fields between exiftool and fast-exif-rs."""
        comparison = {
            "matches": {},
            "differences": {},
            "missing_in_fast_exif": {},
            "extra_in_fast_exif": {}
        }
        
        # Fields to focus on (from correctness checklist)
        focus_fields = [
            "ExposureCompensation", "FlashpixVersion", "ExifVersion",
            "ApertureValue", "ShutterSpeedValue", "MaxApertureValue",
            "ExposureMode", "CustomRendered", "Sharpness", "MeteringMode",
            "CompressedBitsPerPixel", "FocalPlaneResolutionUnit",
            "DateTimeOriginal", "DateTime", "DateTimeDigitized",
            "Make", "Model", "ISO", "FNumber", "ExposureTime",
            "FocalLength", "Flash", "WhiteBalance", "Orientation"
        ]
        
        # Check focus fields first
        for field in focus_fields:
            exiftool_val = exiftool_data.get(field)
            fast_exif_val = fast_exif_data.get(field)
            
            if exiftool_val is not None and fast_exif_val is not None:
                if exiftool_val == fast_exif_val:
                    comparison["matches"][field] = exiftool_val
                else:
                    comparison["differences"][field] = {
                        "exiftool": exiftool_val,
                        "fast_exif": fast_exif_val
                    }
            elif exiftool_val is not None and fast_exif_val is None:
                comparison["missing_in_fast_exif"][field] = exiftool_val
            elif exiftool_val is None and fast_exif_val is not None:
                comparison["extra_in_fast_exif"][field] = fast_exif_val
        
        return comparison
    
    def test_file(self, file_path: Path) -> Dict:
        """Test a single file."""
        print(f"\n{'='*60}")
        print(f"Testing: {file_path.name}")
        print(f"{'='*60}")
        
        # Get outputs
        exiftool_data = self.get_exiftool_output(str(file_path))
        fast_exif_data = self.get_fast_exif_output(str(file_path))
        
        if not exiftool_data:
            print("❌ Failed to get exiftool output")
            return {"status": "exiftool_error"}
        
        if not fast_exif_data:
            print("❌ Failed to get fast-exif-rs output")
            return {"status": "fast_exif_error"}
        
        # Compare fields
        comparison = self.compare_fields(exiftool_data, fast_exif_data)
        
        # Print results
        print(f"📊 Field Analysis:")
        print(f"  Matches: {len(comparison['matches'])}")
        print(f"  Differences: {len(comparison['differences'])}")
        print(f"  Missing in fast-exif-rs: {len(comparison['missing_in_fast_exif'])}")
        print(f"  Extra in fast-exif-rs: {len(comparison['extra_in_fast_exif'])}")
        
        # Show differences
        if comparison["differences"]:
            print(f"\n🔍 Key Differences:")
            for field, values in comparison["differences"].items():
                print(f"  {field}:")
                print(f"    exiftool: {values['exiftool']}")
                print(f"    fast-exif: {values['fast_exif']}")
        
        # Show missing fields
        if comparison["missing_in_fast_exif"]:
            print(f"\n⚠️  Missing in fast-exif-rs:")
            for field, value in comparison["missing_in_fast_exif"].items():
                print(f"  {field}: {value}")
        
        # Calculate match rate
        total_focus_fields = len([f for f in comparison["matches"] if f in comparison["matches"]]) + len(comparison["differences"])
        if total_focus_fields > 0:
            match_rate = len(comparison["matches"]) / total_focus_fields * 100
            print(f"\n📈 Match Rate: {match_rate:.1f}%")
        else:
            match_rate = 0
            print(f"\n📈 Match Rate: N/A (no focus fields found)")
        
        return {
            "status": "success",
            "file": file_path.name,
            "match_rate": match_rate,
            "comparison": comparison,
            "exiftool_count": len(exiftool_data),
            "fast_exif_count": len(fast_exif_data)
        }
    
    def test_all_files(self) -> Dict:
        """Test all files in test_files directory."""
        print("🚀 Starting Comprehensive EXIF Testing")
        print("=" * 60)
        
        if not self.test_files_dir.exists():
            print(f"❌ Test files directory not found: {self.test_files_dir}")
            return {}
        
        # Get all test files
        test_files = list(self.test_files_dir.glob("*"))
        test_files = [f for f in test_files if f.is_file()]
        
        print(f"📁 Found {len(test_files)} test files")
        
        results = {}
        total_match_rate = 0
        successful_tests = 0
        
        for i, test_file in enumerate(test_files, 1):
            print(f"\n[{i}/{len(test_files)}] Processing {test_file.name}...")
            
            try:
                result = self.test_file(test_file)
                results[test_file.name] = result
                
                if result["status"] == "success":
                    successful_tests += 1
                    total_match_rate += result["match_rate"]
                    
            except Exception as e:
                print(f"❌ Error testing {test_file.name}: {e}")
                results[test_file.name] = {"status": "error", "error": str(e)}
        
        # Summary
        print(f"\n{'='*60}")
        print("📊 COMPREHENSIVE TEST SUMMARY")
        print(f"{'='*60}")
        print(f"Total files tested: {len(test_files)}")
        print(f"Successful tests: {successful_tests}")
        print(f"Failed tests: {len(test_files) - successful_tests}")
        
        if successful_tests > 0:
            avg_match_rate = total_match_rate / successful_tests
            print(f"Average match rate: {avg_match_rate:.1f}%")
        
        # Identify most problematic files
        print(f"\n🔍 Files with lowest match rates:")
        sorted_results = sorted(
            [(name, result) for name, result in results.items() if result.get("status") == "success"],
            key=lambda x: x[1]["match_rate"]
        )
        
        for name, result in sorted_results[:5]:
            print(f"  {name}: {result['match_rate']:.1f}%")
        
        # Identify common issues
        print(f"\n🔧 Common issues across files:")
        all_differences = {}
        for result in results.values():
            if result.get("status") == "success":
                for field, values in result["comparison"]["differences"].items():
                    if field not in all_differences:
                        all_differences[field] = 0
                    all_differences[field] += 1
        
        sorted_issues = sorted(all_differences.items(), key=lambda x: x[1], reverse=True)
        for field, count in sorted_issues[:10]:
            print(f"  {field}: {count} files")
        
        return {
            "summary": {
                "total_files": len(test_files),
                "successful_tests": successful_tests,
                "avg_match_rate": avg_match_rate if successful_tests > 0 else 0
            },
            "results": results,
            "common_issues": sorted_issues
        }

def main():
    """Run comprehensive EXIF testing."""
    tester = ExifTester()
    results = tester.test_all_files()
    
    # Save results to file
    with open("comprehensive_test_results.json", "w") as f:
        json.dump(results, f, indent=2)
    
    print(f"\n💾 Results saved to comprehensive_test_results.json")
    
    return 0 if results["summary"]["avg_match_rate"] > 80 else 1

if __name__ == "__main__":
    sys.exit(main())
