#!/usr/bin/env python3
"""
Advanced 1:1 Compatibility Test: fast-exif-rs vs PyExifTool

This script performs a sophisticated comparison that handles:
- Group prefixes (File:, EXIF:, MakerNotes:, etc.)
- Field name normalization
- Value format differences
- Computed fields comparison

Author: Claude Sonnet 4 (claude-3-5-sonnet-20241022)
Generated via Cursor IDE (cursor.sh) with AI assistance
"""

import os
import sys
import json
import time
import traceback
from typing import Dict, List, Set, Tuple, Any
from pathlib import Path
import re

# Add project root to path
sys.path.insert(0, '/projects/fast-exif-rs')

try:
    import fast_exif_reader
    FAST_EXIF_AVAILABLE = True
except ImportError as e:
    print(f"❌ fast-exif-reader not available: {e}")
    FAST_EXIF_AVAILABLE = False

try:
    from exiftool import ExifTool
    EXIFTOOL_AVAILABLE = True
except ImportError as e:
    print(f"❌ PyExifTool not available: {e}")
    EXIFTOOL_AVAILABLE = False

class AdvancedExifToolCompatibilityTester:
    """Advanced compatibility tester with sophisticated field matching"""
    
    def __init__(self):
        self.results = {
            'total_files': 0,
            'successful_comparisons': 0,
            'failed_comparisons': 0,
            'field_analysis': {},
            'format_support': {},
            'performance_comparison': {},
            'detailed_results': []
        }
        
        # Test files
        self.test_files = [
            '/keg/pictures/2025/09-Sep/20250921_120522.130.jpg',
        ]
        
        # Filter to only existing files
        self.test_files = [f for f in self.test_files if os.path.exists(f)]
    
    def normalize_field_name(self, field_name: str) -> str:
        """Normalize field name by removing group prefixes and standardizing"""
        # Remove group prefixes (File:, EXIF:, MakerNotes:, etc.)
        base_name = field_name.split(':', 1)[-1] if ':' in field_name else field_name
        
        # Standardize common field names
        name_mappings = {
            'ModifyDate': 'DateTime',
            'CreateDate': 'DateTimeOriginal', 
            'DateTimeCreated': 'DateTimeDigitized',
            'ISO': 'ISOSpeedRatings',
            'ImageWidth': 'ImageWidth',
            'ImageHeight': 'ImageHeight',
            'FocalLength': 'FocalLength',
            'FNumber': 'FNumber',
            'ExposureTime': 'ExposureTime',
            'ShutterSpeed': 'ShutterSpeed',
            'Aperture': 'FNumber',
            'Make': 'Make',
            'Model': 'Model',
            'LensModel': 'LensModel',
            'WhiteBalance': 'WhiteBalance',
            'Flash': 'Flash',
            'MeteringMode': 'MeteringMode',
            'ExposureProgram': 'ExposureProgram',
            'ExposureMode': 'ExposureMode',
            'Contrast': 'Contrast',
            'Saturation': 'Saturation',
            'Sharpness': 'Sharpness',
            'ColorSpace': 'ColorSpace',
            'ComponentsConfiguration': 'ComponentsConfiguration',
            'ExifByteOrder': 'ExifByteOrder',
            'MIMEType': 'MIMEType',
            'FileName': 'FileName',
        }
        
        return name_mappings.get(base_name, base_name)
    
    def normalize_value(self, value: str, field_name: str) -> str:
        """Normalize field values for comparison"""
        if not isinstance(value, str):
            value = str(value)
        
        # Remove trailing zeros from decimal numbers
        if re.match(r'^\d+\.\d+$', value):
            value = value.rstrip('0').rstrip('.')
        
        # Normalize boolean values
        if value.lower() in ['true', '1', 'yes']:
            return '1'
        elif value.lower() in ['false', '0', 'no']:
            return '0'
        
        # Normalize date formats
        if 'date' in field_name.lower() or 'time' in field_name.lower():
            # Remove timezone info for comparison
            value = re.sub(r'[+-]\d{2}:\d{2}$', '', value)
        
        return value.strip()
    
    def run_advanced_test(self) -> Dict[str, Any]:
        """Run advanced compatibility test"""
        print("🔍 ADVANCED EXIFTOOL COMPATIBILITY TEST")
        print("=" * 60)
        
        if not FAST_EXIF_AVAILABLE or not EXIFTOOL_AVAILABLE:
            print("❌ Required dependencies not available")
            return self.results
        
        print(f"📁 Testing {len(self.test_files)} files")
        print(f"🔧 fast-exif-rs version: {fast_exif_reader.__version__}")
        print()
        
        # Initialize readers
        try:
            fast_reader = fast_exif_reader.FastExifReader()
            print("✅ fast-exif-rs reader initialized")
        except Exception as e:
            print(f"❌ Failed to initialize fast-exif-rs reader: {e}")
            return self.results
        
        # Test each file
        for i, test_file in enumerate(self.test_files, 1):
            print(f"\n📸 TESTING FILE {i}/{len(self.test_files)}: {os.path.basename(test_file)}")
            print("-" * 50)
            
            try:
                file_result = self._test_single_file_advanced(fast_reader, test_file)
                self.results['detailed_results'].append(file_result)
                self.results['total_files'] += 1
                
                if file_result['success']:
                    self.results['successful_comparisons'] += 1
                else:
                    self.results['failed_comparisons'] += 1
                    
            except Exception as e:
                print(f"❌ Error testing file {test_file}: {e}")
                traceback.print_exc()
                self.results['failed_comparisons'] += 1
        
        # Generate analysis
        self._generate_advanced_analysis()
        self._print_advanced_summary()
        
        return self.results
    
    def _test_single_file_advanced(self, fast_reader: Any, file_path: str) -> Dict[str, Any]:
        """Test a single file with advanced field matching"""
        result = {
            'file_path': file_path,
            'file_name': os.path.basename(file_path),
            'success': False,
            'field_analysis': {},
            'performance': {},
            'errors': []
        }
        
        try:
            # Get fast-exif-rs data
            print("  🔄 Testing fast-exif-rs...")
            start_time = time.time()
            fast_metadata = fast_reader.read_file(file_path)
            fast_time = time.time() - start_time
            
            result['performance']['fast_exif_time'] = fast_time
            print(f"    ✅ fast-exif-rs: {len(fast_metadata)} fields in {fast_time:.3f}s")
            
        except Exception as e:
            error_msg = f"fast-exif-rs error: {e}"
            result['errors'].append(error_msg)
            print(f"    ❌ {error_msg}")
            return result
        
        try:
            # Get PyExifTool data
            print("  🔄 Testing PyExifTool...")
            start_time = time.time()
            
            with ExifTool() as et:
                exiftool_result = et.execute_json(file_path)
                exiftool_metadata = exiftool_result[0] if exiftool_result else {}
            
            exiftool_time = time.time() - start_time
            
            result['performance']['exiftool_time'] = exiftool_time
            print(f"    ✅ PyExifTool: {len(exiftool_metadata)} fields in {exiftool_time:.3f}s")
            
        except Exception as e:
            error_msg = f"PyExifTool error: {e}"
            result['errors'].append(error_msg)
            print(f"    ❌ {error_msg}")
            return result
        
        # Advanced field comparison
        self._advanced_field_comparison(fast_metadata, exiftool_metadata, result)
        
        # Performance comparison
        if fast_time > 0 and exiftool_time > 0:
            speedup = exiftool_time / fast_time
            result['performance']['speedup'] = speedup
            print(f"    ⚡ Speedup: {speedup:.2f}x faster")
        
        result['success'] = True
        return result
    
    def _advanced_field_comparison(self, fast_metadata: Dict, exiftool_metadata: Dict, result: Dict):
        """Advanced field comparison with normalization"""
        
        # Normalize field names and values
        fast_normalized = {}
        for key, value in fast_metadata.items():
            norm_key = self.normalize_field_name(key)
            norm_value = self.normalize_value(value, norm_key)
            fast_normalized[norm_key] = norm_value
        
        exiftool_normalized = {}
        for key, value in exiftool_metadata.items():
            norm_key = self.normalize_field_name(key)
            norm_value = self.normalize_value(value, norm_key)
            exiftool_normalized[norm_key] = norm_value
        
        # Field overlap analysis
        fast_fields = set(fast_normalized.keys())
        exiftool_fields = set(exiftool_normalized.keys())
        
        common_fields = fast_fields.intersection(exiftool_fields)
        fast_only = fast_fields - exiftool_fields
        exiftool_only = exiftool_fields - fast_fields
        
        print(f"    📊 Advanced Field Analysis:")
        print(f"      • Common normalized fields: {len(common_fields)}")
        print(f"      • fast-exif-rs only: {len(fast_only)}")
        print(f"      • PyExifTool only: {len(exiftool_only)}")
        
        # Check value differences for common fields
        value_differences = []
        exact_matches = 0
        
        for field in common_fields:
            fast_value = fast_normalized[field]
            exiftool_value = exiftool_normalized[field]
            
            if fast_value == exiftool_value:
                exact_matches += 1
            else:
                value_differences.append({
                    'field': field,
                    'fast_exif_value': fast_value,
                    'exiftool_value': exiftool_value
                })
        
        match_percentage = (exact_matches / len(common_fields)) * 100 if common_fields else 0
        
        print(f"    🎯 Value Matching:")
        print(f"      • Exact matches: {exact_matches}/{len(common_fields)} ({match_percentage:.1f}%)")
        print(f"      • Value differences: {len(value_differences)}")
        
        # Store analysis results
        result['field_analysis'] = {
            'fast_fields': len(fast_normalized),
            'exiftool_fields': len(exiftool_normalized),
            'common_fields': len(common_fields),
            'fast_only': len(fast_only),
            'exiftool_only': len(exiftool_only),
            'exact_matches': exact_matches,
            'value_differences': len(value_differences),
            'match_percentage': match_percentage,
            'value_differences_details': value_differences[:10]  # Store first 10 differences
        }
        
        # Show sample differences
        if value_differences:
            print(f"    ⚠️  Sample value differences:")
            for diff in value_differences[:5]:
                print(f"      • {diff['field']}: '{diff['fast_exif_value']}' vs '{diff['exiftool_value']}'")
        
        # Show unique fields
        if fast_only:
            print(f"    🔍 fast-exif-rs unique fields (sample): {list(fast_only)[:5]}")
        if exiftool_only:
            print(f"    🔍 PyExifTool unique fields (sample): {list(exiftool_only)[:5]}")
    
    def _generate_advanced_analysis(self):
        """Generate advanced analysis"""
        if not self.results['detailed_results']:
            return
        
        # Aggregate field analysis
        total_fast_fields = 0
        total_exiftool_fields = 0
        total_common_fields = 0
        total_exact_matches = 0
        total_value_differences = 0
        
        for result in self.results['detailed_results']:
            if result['success'] and 'field_analysis' in result:
                fa = result['field_analysis']
                total_fast_fields += fa['fast_fields']
                total_exiftool_fields += fa['exiftool_fields']
                total_common_fields += fa['common_fields']
                total_exact_matches += fa['exact_matches']
                total_value_differences += fa['value_differences']
        
        files_count = len([r for r in self.results['detailed_results'] if r['success']])
        
        self.results['field_analysis'] = {
            'avg_fast_fields': total_fast_fields / files_count if files_count > 0 else 0,
            'avg_exiftool_fields': total_exiftool_fields / files_count if files_count > 0 else 0,
            'avg_common_fields': total_common_fields / files_count if files_count > 0 else 0,
            'total_exact_matches': total_exact_matches,
            'total_value_differences': total_value_differences,
            'overall_match_percentage': (total_exact_matches / total_common_fields) * 100 if total_common_fields > 0 else 0
        }
        
        # Performance analysis
        total_speedup = 0
        speedup_count = 0
        for result in self.results['detailed_results']:
            if result['success'] and 'speedup' in result['performance']:
                total_speedup += result['performance']['speedup']
                speedup_count += 1
        
        self.results['performance_comparison'] = {
            'average_speedup': total_speedup / speedup_count if speedup_count > 0 else 0,
            'files_tested': speedup_count
        }
    
    def _print_advanced_summary(self):
        """Print advanced summary"""
        print("\n" + "=" * 60)
        print("📊 ADVANCED COMPATIBILITY SUMMARY")
        print("=" * 60)
        
        print(f"📁 Total files tested: {self.results['total_files']}")
        print(f"✅ Successful comparisons: {self.results['successful_comparisons']}")
        print(f"❌ Failed comparisons: {self.results['failed_comparisons']}")
        
        if self.results['successful_comparisons'] > 0:
            success_rate = self.results['successful_comparisons'] / self.results['total_files'] * 100
            print(f"📈 Success rate: {success_rate:.1f}%")
        
        # Field analysis
        if 'field_analysis' in self.results:
            fa = self.results['field_analysis']
            print(f"\n🔍 ADVANCED FIELD ANALYSIS:")
            print(f"  • Average fast-exif-rs fields: {fa['avg_fast_fields']:.0f}")
            print(f"  • Average PyExifTool fields: {fa['avg_exiftool_fields']:.0f}")
            print(f"  • Average common fields: {fa['avg_common_fields']:.0f}")
            print(f"  • Exact value matches: {fa['total_exact_matches']}")
            print(f"  • Value differences: {fa['total_value_differences']}")
            print(f"  • Overall match percentage: {fa['overall_match_percentage']:.1f}%")
        
        # Performance
        if 'performance_comparison' in self.results:
            pc = self.results['performance_comparison']
            print(f"\n⚡ PERFORMANCE COMPARISON:")
            print(f"  • Average speedup: {pc['average_speedup']:.2f}x faster")
            print(f"  • Files with performance data: {pc['files_tested']}")
        
        # Overall assessment
        print(f"\n🎯 OVERALL ASSESSMENT:")
        if self.results['successful_comparisons'] == self.results['total_files'] and self.results['total_files'] > 0:
            print("  ✅ EXCELLENT: All files processed successfully")
        elif self.results['successful_comparisons'] > 0:
            print("  ⚠️  PARTIAL: Some files processed successfully")
        else:
            print("  ❌ FAILED: No files processed successfully")
        
        if 'field_analysis' in self.results:
            match_pct = self.results['field_analysis']['overall_match_percentage']
            if match_pct > 90:
                print("  🎯 COMPATIBILITY: Excellent field value matching")
            elif match_pct > 70:
                print("  ✅ COMPATIBILITY: Good field value matching")
            elif match_pct > 50:
                print("  ⚠️  COMPATIBILITY: Moderate field value matching")
            else:
                print("  ❌ COMPATIBILITY: Poor field value matching")

def main():
    """Main function"""
    print("🚀 Starting Advanced ExifTool Compatibility Test")
    print("=" * 60)
    
    if not FAST_EXIF_AVAILABLE or not EXIFTOOL_AVAILABLE:
        print("❌ Required dependencies not available")
        return 1
    
    tester = AdvancedExifToolCompatibilityTester()
    results = tester.run_advanced_test()
    
    # Save results
    results_file = '/projects/fast-exif-rs/advanced_exiftool_compatibility_results.json'
    try:
        with open(results_file, 'w') as f:
            json.dump(results, f, indent=2, default=str)
        print(f"\n💾 Results saved to: {results_file}")
    except Exception as e:
        print(f"⚠️  Could not save results: {e}")
    
    return 0 if results['successful_comparisons'] > 0 else 1

if __name__ == "__main__":
    sys.exit(main())
