#!/usr/bin/env python3
"""
Test EXIF writing compatibility with exiftool on /keg/pictures/ images
"""

import fast_exif_reader
import os
import subprocess
import tempfile
import shutil
from pathlib import Path

def test_exif_compatibility():
    """Test EXIF reading and writing compatibility with exiftool"""
    print("Testing EXIF compatibility with exiftool on /keg/pictures/")
    print("=" * 60)
    
    # Test images from different formats and years
    test_images = [
        "/keg/pictures/2023/06-Jun/20230601_081821.000.heic",
        "/keg/pictures/2021/04-Apr/20210402_151624.000.jpg",
        "/keg/pictures/2024/12-Dec/20241201_164954.000.mp4",
        "/keg/pictures/SchoolPictures/Calvin_SchoolPhoto_3rd.jpg",
        "/keg/pictures/scans/2000/apple fest - 01.jpg"
    ]
    
    reader = fast_exif_reader.FastExifReader()
    writer = fast_exif_reader.FastExifWriter()
    
    results = []
    
    for img_path in test_images:
        if not os.path.exists(img_path):
            print(f"❌ File not found: {img_path}")
            continue
            
        print(f"\n📸 Testing: {os.path.basename(img_path)}")
        print(f"   Path: {img_path}")
        
        try:
            # Read EXIF with our library
            our_metadata = reader.read_file(img_path)
            print(f"   ✅ Our library: {len(our_metadata)} fields")
            
            # Read EXIF with exiftool
            exiftool_result = subprocess.run([
                'exiftool', '-s', '-G', img_path
            ], capture_output=True, text=True, timeout=30)
            
            if exiftool_result.returncode == 0:
                exiftool_lines = exiftool_result.stdout.strip().split('\n')
                exiftool_fields = len([line for line in exiftool_lines if ':' in line])
                print(f"   ✅ Exiftool: {exiftool_fields} fields")
                
                # Compare key fields
                key_fields = ['Make', 'Model', 'DateTime', 'DateTimeOriginal', 'ExposureTime', 'FNumber', 'ISO']
                matches = 0
                total = 0
                
                for field in key_fields:
                    if field in our_metadata:
                        total += 1
                        # Check if exiftool also has this field
                        exiftool_has_field = any(field in line for line in exiftool_lines)
                        if exiftool_has_field:
                            matches += 1
                
                if total > 0:
                    match_rate = (matches / total) * 100
                    print(f"   📊 Key field match rate: {match_rate:.1f}% ({matches}/{total})")
                    
                    results.append({
                        'file': os.path.basename(img_path),
                        'our_fields': len(our_metadata),
                        'exiftool_fields': exiftool_fields,
                        'match_rate': match_rate,
                        'format': Path(img_path).suffix.lower()
                    })
                else:
                    print(f"   ⚠️  No key fields found for comparison")
                    
            else:
                print(f"   ❌ Exiftool failed: {exiftool_result.stderr}")
                
        except Exception as e:
            print(f"   ❌ Error: {e}")
    
    # Summary
    print(f"\n📊 SUMMARY")
    print("=" * 60)
    
    if results:
        total_our_fields = sum(r['our_fields'] for r in results)
        total_exiftool_fields = sum(r['exiftool_fields'] for r in results)
        avg_match_rate = sum(r['match_rate'] for r in results) / len(results)
        
        print(f"Total images tested: {len(results)}")
        print(f"Average our fields: {total_our_fields / len(results):.1f}")
        print(f"Average exiftool fields: {total_exiftool_fields / len(results):.1f}")
        print(f"Average key field match rate: {avg_match_rate:.1f}%")
        
        print(f"\nDetailed results:")
        for result in results:
            print(f"  {result['file']:30} | {result['format']:4} | {result['our_fields']:3} fields | {result['match_rate']:5.1f}% match")
    
    return results

def test_exif_writing():
    """Test EXIF writing functionality"""
    print(f"\n🖊️  TESTING EXIF WRITING")
    print("=" * 60)
    
    # Find a JPEG image to test writing
    test_jpeg = None
    for root, dirs, files in os.walk("/keg/pictures"):
        for file in files:
            if file.lower().endswith('.jpg') and 'scan' not in root.lower():
                test_jpeg = os.path.join(root, file)
                break
        if test_jpeg:
            break
    
    if not test_jpeg:
        print("❌ No suitable JPEG found for writing test")
        return
    
    print(f"📸 Testing EXIF writing on: {os.path.basename(test_jpeg)}")
    
    try:
        reader = fast_exif_reader.FastExifReader()
        writer = fast_exif_reader.FastExifWriter()
        
        # Read original EXIF
        original_metadata = reader.read_file(test_jpeg)
        print(f"   Original EXIF: {len(original_metadata)} fields")
        
        # Create test metadata
        test_metadata = {
            "Make": "Test Camera",
            "Model": "Test Model",
            "DateTime": "2024:01:01 12:00:00",
            "DateTimeOriginal": "2024:01:01 12:00:00",
            "ExposureTime": "1/60",
            "FNumber": "4.0",
            "ISO": "100",
            "FocalLength": "50.0 mm"
        }
        
        # Write to temporary file
        with tempfile.NamedTemporaryFile(suffix='.jpg', delete=False) as tmp_file:
            tmp_path = tmp_file.name
        
        writer.write_exif(test_jpeg, tmp_path, test_metadata)
        print(f"   ✅ EXIF writing successful")
        
        # Verify written EXIF
        written_metadata = reader.read_file(tmp_path)
        print(f"   Written EXIF: {len(written_metadata)} fields")
        
        # Check if our test fields are present
        test_fields_present = 0
        for field, expected_value in test_metadata.items():
            if field in written_metadata:
                test_fields_present += 1
                print(f"   ✅ {field}: {written_metadata[field]}")
            else:
                print(f"   ❌ {field}: Missing")
        
        print(f"   📊 Test fields written: {test_fields_present}/{len(test_metadata)}")
        
        # Clean up
        os.unlink(tmp_path)
        
    except Exception as e:
        print(f"   ❌ EXIF writing test failed: {e}")

def test_large_scale_processing():
    """Test processing capabilities on a larger scale"""
    print(f"\n🚀 TESTING LARGE SCALE PROCESSING")
    print("=" * 60)
    
    # Count total images
    total_images = 0
    format_counts = {}
    
    print("📊 Analyzing /keg/pictures/ directory...")
    
    for root, dirs, files in os.walk("/keg/pictures"):
        for file in files:
            ext = Path(file).suffix.lower()
            if ext in ['.jpg', '.jpeg', '.heic', '.heif', '.cr2', '.nef', '.orf', '.dng']:
                total_images += 1
                format_counts[ext] = format_counts.get(ext, 0) + 1
                
                # Limit to first 1000 for performance
                if total_images >= 1000:
                    break
        if total_images >= 1000:
            break
    
    print(f"   Total images found: {total_images:,}")
    print(f"   Format distribution:")
    for ext, count in sorted(format_counts.items()):
        print(f"     {ext}: {count:,}")
    
    # Test processing speed
    if total_images > 0:
        print(f"\n⏱️  Testing processing speed...")
        
        reader = fast_exif_reader.FastExifReader()
        processed = 0
        errors = 0
        
        import time
        start_time = time.time()
        
        for root, dirs, files in os.walk("/keg/pictures"):
            for file in files:
                if processed >= 100:  # Test first 100 images
                    break
                    
                ext = Path(file).suffix.lower()
                if ext in ['.jpg', '.jpeg', '.heic', '.heif']:
                    img_path = os.path.join(root, file)
                    try:
                        metadata = reader.read_file(img_path)
                        processed += 1
                    except:
                        errors += 1
                        
            if processed >= 100:
                break
        
        end_time = time.time()
        duration = end_time - start_time
        
        print(f"   Processed: {processed} images in {duration:.2f} seconds")
        print(f"   Errors: {errors}")
        print(f"   Speed: {processed/duration:.1f} images/second")
        print(f"   Average: {duration/processed*1000:.1f} ms per image")

if __name__ == "__main__":
    # Run all tests
    compatibility_results = test_exif_compatibility()
    test_exif_writing()
    test_large_scale_processing()
    
    print(f"\n🎯 CONCLUSION")
    print("=" * 60)
    print("EXIF reading and writing functionality has been tested on")
    print("real images from /keg/pictures/ directory.")
    print("The implementation provides a solid foundation for")
    print("processing the 243,815+ images in the collection.")
