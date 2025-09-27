#!/usr/bin/env python3
"""
Comprehensive format support test for fast-exif-rs
Tests all supported formats against real files from /keg/pictures/
"""

import os
import sys
import time
from pathlib import Path
from typing import Dict, List, Tuple
import json

# Add the project root to Python path
sys.path.insert(0, str(Path(__file__).parent))

try:
    import fast_exif_reader
except ImportError:
    print("Error: fast_exif_reader not found. Make sure the Rust module is built.")
    print("Try running: source venv/bin/activate && maturin develop")
    sys.exit(1)

class FormatSupportTester:
    def __init__(self):
        self.results = {
            "supported_formats": {},
            "unsupported_formats": {},
            "test_summary": {}
        }
        self.pictures_dir = Path("/keg/pictures")
        
    def get_test_files(self) -> Dict[str, List[str]]:
        """Get test files by format from /keg/pictures/"""
        formats = {
            "JPEG": [],
            "CR2": [],
            "HEIC": [],
            "MP4": [],
            "MOV": [],
            "DNG": [],
            "PNG": [],
            "GIF": [],
            "WEBP": [],
            "AVI": [],
            "WMV": [],
            "WEBM": [],
            "MKV": [],
            "ARW": [],
            "RAF": [],
            "SRW": [],
            "ORF": [],
            "PEF": [],
            "RW2": [],
        }
        
        if not self.pictures_dir.exists():
            print(f"Warning: {self.pictures_dir} not found")
            return formats
            
        # Find files by extension
        for format_name, extensions in [
            ("JPEG", [".jpg", ".jpeg"]),
            ("CR2", [".cr2"]),
            ("HEIC", [".heic", ".hif"]),
            ("MP4", [".mp4"]),
            ("MOV", [".mov"]),
            ("DNG", [".dng"]),
            ("PNG", [".png"]),
            ("GIF", [".gif"]),
            ("WEBP", [".webp"]),
            ("AVI", [".avi"]),
            ("WMV", [".wmv"]),
            ("WEBM", [".webm"]),
            ("MKV", [".mkv"]),
            ("ARW", [".arw"]),
            ("RAF", [".raf"]),
            ("SRW", [".srw"]),
            ("ORF", [".orf"]),
            ("PEF", [".pef"]),
            ("RW2", [".rw2"]),
        ]:
            for ext in extensions:
                files = list(self.pictures_dir.rglob(f"*{ext}"))
                formats[format_name].extend([str(f) for f in files[:5]])  # Limit to 5 files per format
                
        return formats
        
    def test_format_support(self, format_name: str, file_paths: List[str]) -> Dict:
        """Test support for a specific format"""
        if not file_paths:
            return {
                "status": "no_files",
                "message": f"No {format_name} files found",
                "files_tested": 0,
                "successful_reads": 0,
                "failed_reads": 0,
                "avg_fields": 0,
                "sample_fields": []
            }
            
        reader = fast_exif_reader.FastExifReader()
        successful_reads = 0
        failed_reads = 0
        total_fields = 0
        sample_fields = []
        
        print(f"\n📁 Testing {format_name} format ({len(file_paths)} files)")
        print("-" * 50)
        
        for i, file_path in enumerate(file_paths[:3]):  # Test first 3 files
            try:
                start_time = time.time()
                metadata = reader.read_file(file_path)
                end_time = time.time()
                
                field_count = len(metadata)
                total_fields += field_count
                successful_reads += 1
                
                if i == 0:  # Store sample fields from first file
                    sample_fields = list(metadata.items())[:5]
                
                print(f"  ✅ {os.path.basename(file_path)}: {field_count} fields ({end_time - start_time:.3f}s)")
                
            except Exception as e:
                failed_reads += 1
                print(f"  ❌ {os.path.basename(file_path)}: {e}")
                
        avg_fields = total_fields / successful_reads if successful_reads > 0 else 0
        
        return {
            "status": "tested",
            "message": f"Tested {len(file_paths)} files",
            "files_tested": len(file_paths),
            "successful_reads": successful_reads,
            "failed_reads": failed_reads,
            "avg_fields": avg_fields,
            "sample_fields": sample_fields
        }
        
    def run_comprehensive_test(self):
        """Run comprehensive format support test"""
        print("🔬 Comprehensive Format Support Test")
        print("=" * 60)
        print(f"Testing files from: {self.pictures_dir}")
        
        test_files = self.get_test_files()
        
        # Test each format
        for format_name, file_paths in test_files.items():
            result = self.test_format_support(format_name, file_paths)
            
            if result["successful_reads"] > 0:
                self.results["supported_formats"][format_name] = result
            else:
                self.results["unsupported_formats"][format_name] = result
                
        # Generate summary
        self.generate_summary()
        
    def generate_summary(self):
        """Generate test summary"""
        supported_count = len(self.results["supported_formats"])
        unsupported_count = len(self.results["unsupported_formats"])
        
        self.results["test_summary"] = {
            "total_formats_tested": supported_count + unsupported_count,
            "supported_formats": supported_count,
            "unsupported_formats": unsupported_count,
            "support_percentage": (supported_count / (supported_count + unsupported_count)) * 100 if (supported_count + unsupported_count) > 0 else 0
        }
        
        print("\n" + "=" * 60)
        print("📊 FORMAT SUPPORT SUMMARY")
        print("=" * 60)
        
        print(f"Total formats tested: {self.results['test_summary']['total_formats_tested']}")
        print(f"Supported formats: {self.results['test_summary']['supported_formats']}")
        print(f"Unsupported formats: {self.results['test_summary']['unsupported_formats']}")
        print(f"Support percentage: {self.results['test_summary']['support_percentage']:.1f}%")
        
        print(f"\n✅ Supported Formats:")
        for format_name, result in self.results["supported_formats"].items():
            print(f"  {format_name}: {result['successful_reads']}/{result['files_tested']} files, avg {result['avg_fields']:.1f} fields")
            
        if self.results["unsupported_formats"]:
            print(f"\n❌ Unsupported Formats:")
            for format_name, result in self.results["unsupported_formats"].items():
                print(f"  {format_name}: {result['message']}")
                
    def save_results(self, filename: str = "format_support_test_results.json"):
        """Save test results to JSON file"""
        results_path = Path(__file__).parent / filename
        
        # Add metadata
        self.results["metadata"] = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "python_version": sys.version,
            "fast_exif_rs_version": getattr(fast_exif_reader, '__version__', 'unknown'),
            "test_type": "Comprehensive Format Support",
            "test_directory": str(self.pictures_dir)
        }
        
        with open(results_path, 'w') as f:
            json.dump(self.results, f, indent=2)
            
        print(f"\n💾 Results saved to: {results_path}")
        
    def compare_with_exiftool(self):
        """Compare format support with exiftool"""
        print(f"\n🔍 Comparing with exiftool support...")
        
        # Get exiftool supported formats
        try:
            import subprocess
            result = subprocess.run(["exiftool", "-listf"], capture_output=True, text=True)
            if result.returncode == 0:
                exiftool_formats = result.stdout.split()
                
                print(f"exiftool supports {len(exiftool_formats)} formats")
                print(f"fast-exif-rs supports {len(self.results['supported_formats'])} formats")
                
                # Find formats supported by exiftool but not by fast-exif-rs
                missing_formats = []
                for format_name in exiftool_formats:
                    if format_name.upper() not in self.results["supported_formats"]:
                        missing_formats.append(format_name)
                        
                if missing_formats:
                    print(f"\n⚠️  Formats supported by exiftool but not by fast-exif-rs:")
                    for fmt in missing_formats[:20]:  # Show first 20
                        print(f"  {fmt}")
                    if len(missing_formats) > 20:
                        print(f"  ... and {len(missing_formats) - 20} more")
                        
        except Exception as e:
            print(f"Could not compare with exiftool: {e}")

def main():
    """Main test execution"""
    print("🧪 Fast-EXIF-RS Format Support Test")
    print("Testing comprehensive format support against real files")
    print()
    
    # Check if fast_exif_reader is available
    try:
        import fast_exif_reader
        print("✅ fast_exif_reader found")
    except ImportError:
        print("❌ fast_exif_reader not found. Please build the Rust module first.")
        return
        
    # Run test
    tester = FormatSupportTester()
    tester.run_comprehensive_test()
    tester.compare_with_exiftool()
    tester.save_results()

if __name__ == "__main__":
    main()
