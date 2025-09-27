#!/usr/bin/env python3
"""
Comprehensive 1:1 compatibility test between fast-exif-rs and exiftool
for the top 10 file formats found in /keg/pictures/

Top 10 formats by file count:
1. JPG (226,112 files) - 90.1%
2. CR2 (19,535 files) - 7.8%
3. MP4 (3,470 files) - 1.4%
4. HEIC (3,275 files) - 1.3%
5. DNG (2,506 files) - 1.0%
6. JSON (2,050 files) - 0.8%
7. HIF (1,672 files) - 0.7%
8. MOV (158 files) - 0.1%
9. 3GP (129 files) - 0.1%
10. MKV (33 files) - 0.01%
"""

import os
import sys
import tempfile
import subprocess
import json
import time
from pathlib import Path
from typing import Dict, List, Tuple, Any, Set
import difflib

# Add the project root to Python path
sys.path.insert(0, str(Path(__file__).parent))

try:
    import fast_exif_reader
except ImportError:
    print("Error: fast_exif_reader not found. Make sure the Rust module is built.")
    print("Try running: source venv/bin/activate && maturin develop")
    sys.exit(1)

class ExifToolCompatibilityTester:
    def __init__(self):
        self.pictures_dir = Path("/keg/pictures")
        self.temp_dir = None
        self.results = {
            "compatibility": {},
            "summary": {},
            "issues": []
        }
        
        # Top 10 formats by file count
        self.top_formats = {
            "JPG": {"extensions": [".jpg", ".jpeg"], "priority": 1, "count": 226112},
            "CR2": {"extensions": [".cr2"], "priority": 2, "count": 19535},
            "MP4": {"extensions": [".mp4"], "priority": 3, "count": 3470},
            "HEIC": {"extensions": [".heic"], "priority": 4, "count": 3275},
            "DNG": {"extensions": [".dng"], "priority": 5, "count": 2506},
            "JSON": {"extensions": [".json"], "priority": 6, "count": 2050},
            "HIF": {"extensions": [".hif"], "priority": 7, "count": 1672},
            "MOV": {"extensions": [".mov"], "priority": 8, "count": 158},
            "3GP": {"extensions": [".3gp"], "priority": 9, "count": 129},
            "MKV": {"extensions": [".mkv"], "priority": 10, "count": 33}
        }
        
    def setup_temp_environment(self):
        """Create temporary directory for test files"""
        self.temp_dir = tempfile.mkdtemp(prefix="exiftool_compat_")
        print(f"Created temporary directory: {self.temp_dir}")
        
    def cleanup_temp_environment(self):
        """Clean up temporary files"""
        if self.temp_dir and os.path.exists(self.temp_dir):
            import shutil
            shutil.rmtree(self.temp_dir)
            print(f"Cleaned up temporary directory: {self.temp_dir}")
            
    def get_test_files(self, format_name: str, max_files: int = 5) -> List[str]:
        """Get test files for a specific format"""
        format_info = self.top_formats[format_name]
        test_files = []
        
        if not self.pictures_dir.exists():
            print(f"Warning: {self.pictures_dir} not found")
            return test_files
            
        for ext in format_info["extensions"]:
            files = list(self.pictures_dir.rglob(f"*{ext}"))
            test_files.extend([str(f) for f in files[:max_files]])
            
        return test_files
        
    def normalize_exiftool_output(self, output: str) -> Dict[str, str]:
        """Normalize exiftool output to match our format"""
        metadata = {}
        for line in output.strip().split('\n'):
            if ':' in line:
                key, value = line.split(':', 1)
                key = key.strip()
                value = value.strip()
                # Normalize common field names
                if key == "File Name":
                    key = "FileName"
                elif key == "Directory":
                    key = "Directory"
                elif key == "File Size":
                    key = "FileSize"
                elif key == "File Modification Date/Time":
                    key = "FileModifyDate"
                elif key == "File Access Date/Time":
                    key = "FileAccessDate"
                elif key == "File Inode Change Date/Time":
                    key = "FileInodeChangeDate"
                elif key == "File Permissions":
                    key = "FilePermissions"
                elif key == "File Type":
                    key = "FileType"
                elif key == "File Type Extension":
                    key = "FileTypeExtension"
                elif key == "MIME Type":
                    key = "MIMEType"
                metadata[key] = value
        return metadata
        
    def normalize_fast_exif_output(self, metadata: Dict[str, str]) -> Dict[str, str]:
        """Normalize fast-exif-rs output to match exiftool format"""
        normalized = {}
        for key, value in metadata.items():
            # Normalize field names to match exiftool
            if key == "Format":
                normalized["FileType"] = value
            elif key == "Make":
                normalized["Make"] = value
            elif key == "Model":
                normalized["Model"] = value
            elif key == "DateTime":
                normalized["DateTime"] = value
            elif key == "DateTimeOriginal":
                normalized["DateTimeOriginal"] = value
            elif key == "DateTimeDigitized":
                normalized["DateTimeDigitized"] = value
            elif key == "ExposureTime":
                normalized["ExposureTime"] = value
            elif key == "FNumber":
                normalized["FNumber"] = value
            elif key == "ISO":
                normalized["ISO"] = value
            elif key == "FocalLength":
                normalized["FocalLength"] = value
            elif key == "WhiteBalance":
                normalized["WhiteBalance"] = value
            elif key == "Flash":
                normalized["Flash"] = value
            elif key == "GPSLatitude":
                normalized["GPSLatitude"] = value
            elif key == "GPSLongitude":
                normalized["GPSLongitude"] = value
            else:
                normalized[key] = value
        return normalized
        
    def test_read_compatibility(self, format_name: str, test_files: List[str]) -> Dict[str, Any]:
        """Test read compatibility between fast-exif-rs and exiftool"""
        if not test_files:
            return {
                "status": "no_files",
                "compatibility": 0.0,
                "fast_exif_fields": 0,
                "exiftool_fields": 0,
                "common_fields": 0,
                "missing_fields": [],
                "extra_fields": [],
                "field_differences": []
            }
            
        reader = fast_exif_reader.FastExifReader()
        compatibility_results = []
        
        print(f"\n📖 Testing READ compatibility for {format_name}")
        print("-" * 50)
        
        for i, file_path in enumerate(test_files[:3]):  # Test first 3 files
            try:
                # Read with fast-exif-rs
                fast_metadata = reader.read_file(file_path)
                fast_normalized = self.normalize_fast_exif_output(fast_metadata)
                
                # Read with exiftool
                exiftool_cmd = ["exiftool", "-j", file_path]
                result = subprocess.run(exiftool_cmd, capture_output=True, text=True)
                
                if result.returncode == 0:
                    exiftool_data = json.loads(result.stdout)[0]
                    exiftool_normalized = self.normalize_exiftool_output(
                        '\n'.join([f"{k}: {v}" for k, v in exiftool_data.items()])
                    )
                else:
                    print(f"  ❌ exiftool failed for {os.path.basename(file_path)}")
                    continue
                    
                # Compare fields
                fast_fields = set(fast_normalized.keys())
                exiftool_fields = set(exiftool_normalized.keys())
                
                common_fields = fast_fields.intersection(exiftool_fields)
                missing_fields = exiftool_fields - fast_fields
                extra_fields = fast_fields - exiftool_fields
                
                # Calculate compatibility
                if len(exiftool_fields) > 0:
                    compatibility = len(common_fields) / len(exiftool_fields)
                else:
                    compatibility = 0.0
                    
                compatibility_results.append({
                    "file": os.path.basename(file_path),
                    "compatibility": compatibility,
                    "fast_fields": len(fast_fields),
                    "exiftool_fields": len(exiftool_fields),
                    "common_fields": len(common_fields),
                    "missing_fields": list(missing_fields),
                    "extra_fields": list(extra_fields)
                })
                
                print(f"  📄 {os.path.basename(file_path)}:")
                print(f"    Compatibility: {compatibility:.2%}")
                print(f"    fast-exif-rs: {len(fast_fields)} fields")
                print(f"    exiftool: {len(exiftool_fields)} fields")
                print(f"    Common: {len(common_fields)} fields")
                
                if missing_fields:
                    print(f"    Missing: {len(missing_fields)} fields")
                if extra_fields:
                    print(f"    Extra: {len(extra_fields)} fields")
                    
            except Exception as e:
                print(f"  ❌ Error testing {os.path.basename(file_path)}: {e}")
                continue
                
        # Calculate overall compatibility
        if compatibility_results:
            avg_compatibility = sum(r["compatibility"] for r in compatibility_results) / len(compatibility_results)
            total_fast_fields = sum(r["fast_fields"] for r in compatibility_results)
            total_exiftool_fields = sum(r["exiftool_fields"] for r in compatibility_results)
            total_common_fields = sum(r["common_fields"] for r in compatibility_results)
            
            # Collect all missing and extra fields
            all_missing = set()
            all_extra = set()
            for r in compatibility_results:
                all_missing.update(r["missing_fields"])
                all_extra.update(r["extra_fields"])
                
            return {
                "status": "tested",
                "compatibility": avg_compatibility,
                "fast_exif_fields": total_fast_fields,
                "exiftool_fields": total_exiftool_fields,
                "common_fields": total_common_fields,
                "missing_fields": list(all_missing),
                "extra_fields": list(all_extra),
                "file_results": compatibility_results
            }
        else:
            return {
                "status": "failed",
                "compatibility": 0.0,
                "fast_exif_fields": 0,
                "exiftool_fields": 0,
                "common_fields": 0,
                "missing_fields": [],
                "extra_fields": [],
                "file_results": []
            }
            
    def test_write_compatibility(self, format_name: str, test_files: List[str]) -> Dict[str, Any]:
        """Test write compatibility between fast-exif-rs and exiftool"""
        if not test_files:
            return {
                "status": "no_files",
                "write_success": 0.0,
                "read_back_success": 0.0,
                "field_preservation": 0.0
            }
            
        writer = fast_exif_reader.FastExifWriter()
        write_results = []
        
        print(f"\n✍️  Testing WRITE compatibility for {format_name}")
        print("-" * 50)
        
        # Test metadata to write
        test_metadata = {
            "Make": "Compatibility Test Camera",
            "Model": "Test Model v1.0",
            "DateTime": "2024:09:27 18:00:00",
            "DateTimeOriginal": "2024:09:27 18:00:00",
            "DateTimeDigitized": "2024:09:27 18:00:00",
            "ExposureTime": "1/125",
            "FNumber": "2.8",
            "ISO": "400",
            "FocalLength": "50mm",
            "WhiteBalance": "Auto",
            "Flash": "No Flash",
            "Software": "fast-exif-rs compatibility test"
        }
        
        for i, file_path in enumerate(test_files[:2]):  # Test first 2 files
            try:
                # Create output file paths
                fast_output = os.path.join(self.temp_dir, f"fast_{format_name.lower()}_{i}.jpg")
                exiftool_output = os.path.join(self.temp_dir, f"exiftool_{format_name.lower()}_{i}.jpg")
                
                # Test fast-exif-rs write
                try:
                    writer.write_exif(file_path, fast_output, test_metadata)
                    fast_write_success = True
                except Exception as e:
                    print(f"  ❌ fast-exif-rs write failed: {e}")
                    fast_write_success = False
                    
                # Test exiftool write
                try:
                    import shutil
                    shutil.copy2(file_path, exiftool_output)
                    
                    exiftool_cmd = ["exiftool", "-overwrite_original"]
                    for key, value in test_metadata.items():
                        exiftool_cmd.extend([f"-{key}={value}"])
                    exiftool_cmd.append(exiftool_output)
                    
                    result = subprocess.run(exiftool_cmd, capture_output=True, text=True)
                    exiftool_write_success = result.returncode == 0
                    
                except Exception as e:
                    print(f"  ❌ exiftool write failed: {e}")
                    exiftool_write_success = False
                    
                # Test read-back compatibility
                if fast_write_success and exiftool_write_success:
                    try:
                        # Read back with both tools
                        reader = fast_exif_reader.FastExifReader()
                        fast_readback = reader.read_file(fast_output)
                        
                        exiftool_cmd = ["exiftool", "-j", exiftool_output]
                        result = subprocess.run(exiftool_cmd, capture_output=True, text=True)
                        exiftool_readback = json.loads(result.stdout)[0] if result.returncode == 0 else {}
                        
                        # Compare key fields
                        key_fields = ["Make", "Model", "DateTime", "ExposureTime", "FNumber", "ISO"]
                        preserved_fields = 0
                        
                        for field in key_fields:
                            fast_value = fast_readback.get(field, "")
                            exiftool_value = exiftool_readback.get(field, "")
                            if fast_value == exiftool_value:
                                preserved_fields += 1
                                
                        field_preservation = preserved_fields / len(key_fields) if key_fields else 0.0
                        
                    except Exception as e:
                        print(f"  ❌ Read-back test failed: {e}")
                        field_preservation = 0.0
                else:
                    field_preservation = 0.0
                    
                write_results.append({
                    "file": os.path.basename(file_path),
                    "fast_write_success": fast_write_success,
                    "exiftool_write_success": exiftool_write_success,
                    "field_preservation": field_preservation
                })
                
                print(f"  📄 {os.path.basename(file_path)}:")
                print(f"    fast-exif-rs write: {'✅' if fast_write_success else '❌'}")
                print(f"    exiftool write: {'✅' if exiftool_write_success else '❌'}")
                print(f"    Field preservation: {field_preservation:.2%}")
                
            except Exception as e:
                print(f"  ❌ Error testing write for {os.path.basename(file_path)}: {e}")
                continue
                
        # Calculate overall write compatibility
        if write_results:
            fast_write_success = sum(1 for r in write_results if r["fast_write_success"]) / len(write_results)
            exiftool_write_success = sum(1 for r in write_results if r["exiftool_write_success"]) / len(write_results)
            avg_field_preservation = sum(r["field_preservation"] for r in write_results) / len(write_results)
            
            return {
                "status": "tested",
                "write_success": fast_write_success,
                "exiftool_write_success": exiftool_write_success,
                "field_preservation": avg_field_preservation,
                "file_results": write_results
            }
        else:
            return {
                "status": "failed",
                "write_success": 0.0,
                "exiftool_write_success": 0.0,
                "field_preservation": 0.0,
                "file_results": []
            }
            
    def run_comprehensive_compatibility_test(self):
        """Run comprehensive 1:1 compatibility test"""
        print("🔬 Comprehensive 1:1 Compatibility Test")
        print("fast-exif-rs vs exiftool for top 10 formats")
        print("=" * 60)
        
        self.setup_temp_environment()
        
        try:
            total_compatibility = 0.0
            tested_formats = 0
            
            for format_name, format_info in self.top_formats.items():
                print(f"\n🎯 Testing {format_name} (Priority {format_info['priority']}, {format_info['count']} files)")
                print("=" * 60)
                
                test_files = self.get_test_files(format_name)
                
                if not test_files:
                    print(f"  ⚠️  No {format_name} files found for testing")
                    self.results["compatibility"][format_name] = {
                        "status": "no_files",
                        "read_compatibility": 0.0,
                        "write_compatibility": 0.0,
                        "overall_compatibility": 0.0
                    }
                    continue
                    
                # Test read compatibility
                read_results = self.test_read_compatibility(format_name, test_files)
                
                # Test write compatibility
                write_results = self.test_write_compatibility(format_name, test_files)
                
                # Calculate overall compatibility
                overall_compatibility = (read_results["compatibility"] + write_results["field_preservation"]) / 2
                
                self.results["compatibility"][format_name] = {
                    "status": "tested",
                    "read_compatibility": read_results["compatibility"],
                    "write_compatibility": write_results["field_preservation"],
                    "overall_compatibility": overall_compatibility,
                    "read_details": read_results,
                    "write_details": write_results
                }
                
                total_compatibility += overall_compatibility
                tested_formats += 1
                
                print(f"\n📊 {format_name} Summary:")
                print(f"  Read compatibility: {read_results['compatibility']:.2%}")
                print(f"  Write compatibility: {write_results['field_preservation']:.2%}")
                print(f"  Overall compatibility: {overall_compatibility:.2%}")
                
            # Calculate overall summary
            if tested_formats > 0:
                avg_compatibility = total_compatibility / tested_formats
                self.results["summary"] = {
                    "total_formats_tested": tested_formats,
                    "average_compatibility": avg_compatibility,
                    "target_compatibility": 1.0,
                    "compatibility_gap": 1.0 - avg_compatibility
                }
                
                print(f"\n" + "=" * 60)
                print("📊 OVERALL COMPATIBILITY SUMMARY")
                print("=" * 60)
                print(f"Formats tested: {tested_formats}")
                print(f"Average compatibility: {avg_compatibility:.2%}")
                print(f"Target compatibility: 100.00%")
                print(f"Compatibility gap: {(1.0 - avg_compatibility):.2%}")
                
                if avg_compatibility >= 0.95:
                    print("🎉 Excellent compatibility! (≥95%)")
                elif avg_compatibility >= 0.90:
                    print("✅ Good compatibility! (≥90%)")
                elif avg_compatibility >= 0.80:
                    print("⚠️  Fair compatibility (≥80%)")
                else:
                    print("❌ Poor compatibility (<80%)")
                    
        finally:
            self.cleanup_temp_environment()
            
    def save_results(self, filename: str = "exiftool_compatibility_results.json"):
        """Save compatibility test results"""
        results_path = Path(__file__).parent / filename
        
        # Add metadata
        self.results["metadata"] = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "python_version": sys.version,
            "fast_exif_rs_version": getattr(fast_exif_reader, '__version__', 'unknown'),
            "test_type": "1:1 Compatibility Test",
            "target": "exiftool compatibility for top 10 formats"
        }
        
        with open(results_path, 'w') as f:
            json.dump(self.results, f, indent=2)
            
        print(f"\n💾 Results saved to: {results_path}")
        
    def generate_compatibility_report(self):
        """Generate detailed compatibility report"""
        print(f"\n📋 DETAILED COMPATIBILITY REPORT")
        print("=" * 60)
        
        for format_name, results in self.results["compatibility"].items():
            if results["status"] == "tested":
                print(f"\n{format_name}:")
                print(f"  Read: {results['read_compatibility']:.2%}")
                print(f"  Write: {results['write_compatibility']:.2%}")
                print(f"  Overall: {results['overall_compatibility']:.2%}")
                
                # Show missing fields
                read_details = results["read_details"]
                if read_details["missing_fields"]:
                    print(f"  Missing fields: {len(read_details['missing_fields'])}")
                    for field in read_details["missing_fields"][:5]:  # Show first 5
                        print(f"    - {field}")
                    if len(read_details["missing_fields"]) > 5:
                        print(f"    ... and {len(read_details['missing_fields']) - 5} more")

def main():
    """Main compatibility test execution"""
    print("🔬 Fast-EXIF-RS vs ExifTool 1:1 Compatibility Test")
    print("Testing top 10 file formats for complete compatibility")
    print()
    
    # Check if exiftool is available
    try:
        subprocess.run(["exiftool", "-ver"], check=True, capture_output=True)
        print("✅ exiftool found")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("❌ exiftool not found. Please install exiftool to run this test.")
        return
        
    # Check if fast_exif_reader is available
    try:
        import fast_exif_reader
        print("✅ fast_exif_reader found")
    except ImportError:
        print("❌ fast_exif_reader not found. Please build the Rust module first.")
        return
        
    # Run compatibility test
    tester = ExifToolCompatibilityTester()
    tester.run_comprehensive_compatibility_test()
    tester.generate_compatibility_report()
    tester.save_results()

if __name__ == "__main__":
    main()
