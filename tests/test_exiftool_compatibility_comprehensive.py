#!/usr/bin/env python3
"""
Comprehensive exiftool compatibility tests for /keg/pictures/ images
"""

import fast_exif_reader
import os
import subprocess
import tempfile
import json
from pathlib import Path
from collections import defaultdict

def test_exiftool_compatibility():
    """Test comprehensive compatibility with exiftool"""
    print("🔍 COMPREHENSIVE EXIFTOOL COMPATIBILITY TEST")
    print("=" * 60)
    
    # Initialize components
    reader = fast_exif_reader.FastExifReader()
    writer = fast_exif_reader.FastExifWriter()
    copier = fast_exif_reader.FastExifCopier()
    
    # Find test images by format
    format_images = defaultdict(list)
    
    print("📊 Scanning /keg/pictures/ for test images...")
    
    for root, dirs, files in os.walk("/keg/pictures"):
        for file in files:
            if file.lower().endswith(('.jpg', '.jpeg', '.heic', '.heif', '.cr2', '.nef', '.orf', '.dng')):
                img_path = os.path.join(root, file)
                ext = Path(file).suffix.lower()
                
                try:
                    metadata = reader.read_file(img_path)
                    if len(metadata) > 5:  # Has some EXIF data
                        format_images[ext].append((img_path, len(metadata)))
                except:
                    pass
                
                # Limit per format for performance
                if len(format_images[ext]) >= 3:
                    break
        if all(len(images) >= 3 for images in format_images.values()):
            break
    
    print(f"Found test images:")
    for ext, images in format_images.items():
        print(f"  {ext}: {len(images)} images")
    
    if not format_images:
        print("❌ No test images found")
        return
    
    # Test each format
    results = {}
    
    for ext, images in format_images.items():
        print(f"\n📸 Testing {ext.upper()} format")
        print("-" * 30)
        
        format_results = {
            'total_tested': 0,
            'read_success': 0,
            'exiftool_compatible': 0,
            'key_fields_match': 0,
            'write_success': 0,
            'errors': []
        }
        
        for img_path, field_count in images:
            format_results['total_tested'] += 1
            print(f"  Testing: {os.path.basename(img_path)}")
            
            try:
                # Test reading
                our_metadata = reader.read_file(img_path)
                format_results['read_success'] += 1
                print(f"    ✅ Read: {len(our_metadata)} fields")
                
                # Test exiftool compatibility
                exiftool_result = subprocess.run([
                    'exiftool', '-s', '-G', '-json', img_path
                ], capture_output=True, text=True, timeout=30)
                
                if exiftool_result.returncode == 0:
                    format_results['exiftool_compatible'] += 1
                    
                    try:
                        exiftool_data = json.loads(exiftool_result.stdout)[0]
                        exiftool_fields = len(exiftool_data)
                        print(f"    ✅ Exiftool: {exiftool_fields} fields")
                        
                        # Compare key fields
                        key_fields = ['Make', 'Model', 'DateTime', 'DateTimeOriginal', 'ExposureTime', 'FNumber', 'ISO']
                        matches = 0
                        total = 0
                        
                        for field in key_fields:
                            if field in our_metadata:
                                total += 1
                                if field in exiftool_data:
                                    matches += 1
                        
                        if total > 0:
                            match_rate = (matches / total) * 100
                            format_results['key_fields_match'] += match_rate
                            print(f"    📊 Key fields: {match_rate:.1f}% match ({matches}/{total})")
                        
                    except json.JSONDecodeError:
                        print(f"    ⚠️  Exiftool JSON parse error")
                else:
                    print(f"    ❌ Exiftool failed: {exiftool_result.stderr[:100]}")
                
                # Test writing (only for JPEG for now)
                if ext in ['.jpg', '.jpeg']:
                    test_metadata = {
                        "Make": "Test Camera",
                        "Model": "Test Model", 
                        "DateTime": "2024:01:01 12:00:00"
                    }
                    
                    with tempfile.NamedTemporaryFile(suffix='.jpg', delete=False) as tmp_file:
                        output_path = tmp_file.name
                    
                    try:
                        writer.write_exif(img_path, output_path, test_metadata)
                        format_results['write_success'] += 1
                        print(f"    ✅ Write: Success")
                        
                        # Verify written data
                        written_metadata = reader.read_file(output_path)
                        written_fields = sum(1 for field in test_metadata if field in written_metadata)
                        print(f"    📊 Written fields: {written_fields}/{len(test_metadata)}")
                        
                    except Exception as e:
                        print(f"    ❌ Write failed: {e}")
                        format_results['errors'].append(f"Write error: {e}")
                    finally:
                        if os.path.exists(output_path):
                            os.unlink(output_path)
                
            except Exception as e:
                print(f"    ❌ Error: {e}")
                format_results['errors'].append(f"Read error: {e}")
        
        # Calculate averages
        if format_results['total_tested'] > 0:
            format_results['read_success_rate'] = (format_results['read_success'] / format_results['total_tested']) * 100
            format_results['exiftool_compatibility_rate'] = (format_results['exiftool_compatible'] / format_results['total_tested']) * 100
            format_results['avg_key_fields_match'] = format_results['key_fields_match'] / max(1, format_results['total_tested'])
            format_results['write_success_rate'] = (format_results['write_success'] / format_results['total_tested']) * 100
        
        results[ext] = format_results
        
        print(f"  📊 {ext.upper()} Summary:")
        print(f"    Read success: {format_results['read_success_rate']:.1f}%")
        print(f"    Exiftool compatible: {format_results['exiftool_compatibility_rate']:.1f}%")
        print(f"    Avg key fields match: {format_results['avg_key_fields_match']:.1f}%")
        print(f"    Write success: {format_results['write_success_rate']:.1f}%")
    
    # Overall summary
    print(f"\n🎯 OVERALL COMPATIBILITY SUMMARY")
    print("=" * 60)
    
    total_tested = sum(r['total_tested'] for r in results.values())
    total_read_success = sum(r['read_success'] for r in results.values())
    total_exiftool_compatible = sum(r['exiftool_compatible'] for r in results.values())
    total_write_success = sum(r['write_success'] for r in results.values())
    
    if total_tested > 0:
        overall_read_rate = (total_read_success / total_tested) * 100
        overall_exiftool_rate = (total_exiftool_compatible / total_tested) * 100
        overall_write_rate = (total_write_success / total_tested) * 100
        
        print(f"Total images tested: {total_tested}")
        print(f"Overall read success rate: {overall_read_rate:.1f}%")
        print(f"Overall exiftool compatibility: {overall_exiftool_rate:.1f}%")
        print(f"Overall write success rate: {overall_write_rate:.1f}%")
        
        # Format-specific breakdown
        print(f"\nFormat-specific results:")
        for ext, result in results.items():
            print(f"  {ext.upper():4}: Read {result['read_success_rate']:5.1f}% | "
                  f"Exiftool {result['exiftool_compatibility_rate']:5.1f}% | "
                  f"Write {result['write_success_rate']:5.1f}%")
        
        # Compatibility assessment
        print(f"\n🔍 COMPATIBILITY ASSESSMENT")
        print("-" * 30)
        
        if overall_read_rate >= 90:
            print("✅ EXIF reading: Excellent compatibility")
        elif overall_read_rate >= 75:
            print("✅ EXIF reading: Good compatibility")
        elif overall_read_rate >= 50:
            print("⚠️  EXIF reading: Moderate compatibility")
        else:
            print("❌ EXIF reading: Poor compatibility")
        
        if overall_exiftool_rate >= 90:
            print("✅ Exiftool compatibility: Excellent")
        elif overall_exiftool_rate >= 75:
            print("✅ Exiftool compatibility: Good")
        elif overall_exiftool_rate >= 50:
            print("⚠️  Exiftool compatibility: Moderate")
        else:
            print("❌ Exiftool compatibility: Poor")
        
        if overall_write_rate >= 90:
            print("✅ EXIF writing: Excellent compatibility")
        elif overall_write_rate >= 75:
            print("✅ EXIF writing: Good compatibility")
        elif overall_write_rate >= 50:
            print("⚠️  EXIF writing: Moderate compatibility")
        else:
            print("❌ EXIF writing: Poor compatibility")
        
        # Recommendations
        print(f"\n💡 RECOMMENDATIONS")
        print("-" * 20)
        
        if overall_read_rate < 90:
            print("• Improve EXIF reading for better format coverage")
        
        if overall_exiftool_rate < 90:
            print("• Enhance field mapping to match exiftool output")
        
        if overall_write_rate < 90:
            print("• Implement better EXIF writing for all formats")
        
        print("• Focus on HEIC format (most common in collection)")
        print("• Add support for RAW formats (CR2, NEF, ORF, DNG)")
        print("• Optimize for large-scale processing (243,815+ images)")
    
    return results

if __name__ == "__main__":
    test_exiftool_compatibility()
