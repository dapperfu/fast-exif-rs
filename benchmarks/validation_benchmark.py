#!/usr/bin/env python3
"""
Validation Benchmark - Shows accuracy issues with Fast EXIF Reader

This benchmark clearly demonstrates that speed without accuracy is useless.
"""

import os
import sys
import time
import subprocess
from typing import Dict, List, Tuple

# Add the parent directory to path to import our module
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from fast_exif_reader import FastExifReader
    FAST_EXIF_AVAILABLE = True
except ImportError:
    print("Warning: fast_exif_reader not available")
    FAST_EXIF_AVAILABLE = False

try:
    from PIL import Image
    from PIL.ExifTags import TAGS
    PIL_AVAILABLE = True
except ImportError:
    print("Warning: PIL/Pillow not available")
    PIL_AVAILABLE = False


def get_exiftool_metadata(file_path: str) -> Dict[str, str]:
    """Get metadata using ExifTool"""
    try:
        result = subprocess.run(
            ['exiftool', '-s3', '-fast', file_path],
            capture_output=True,
            text=True,
            timeout=10,
            check=True
        )
        
        metadata = {}
        for line in result.stdout.split('\n'):
            if ':' in line:
                key, value = line.split(':', 1)
                metadata[key.strip()] = value.strip()
        
        return metadata
    except Exception as e:
        print(f"ExifTool error for {file_path}: {e}")
        return {}


def get_pil_metadata(file_path: str) -> Dict[str, str]:
    """Get metadata using PIL"""
    if not PIL_AVAILABLE:
        return {}
    
    try:
        with Image.open(file_path) as img:
            exif_data = img._getexif()
            if exif_data:
                metadata = {}
                for tag_id, value in exif_data.items():
                    tag = TAGS.get(tag_id, tag_id)
                    metadata[str(tag)] = str(value)
                return metadata
            return {}
    except Exception as e:
        print(f"PIL error for {file_path}: {e}")
        return {}


def get_fast_exif_metadata(file_path: str) -> Dict[str, str]:
    """Get metadata using Fast EXIF Reader"""
    if not FAST_EXIF_AVAILABLE:
        return {}
    
    try:
        reader = FastExifReader()
        return reader.read_file(file_path)
    except Exception as e:
        print(f"Fast EXIF error for {file_path}: {e}")
        return {}


def compare_metadata(exiftool_data: Dict[str, str], pil_data: Dict[str, str], 
                     fast_exif_data: Dict[str, str], file_name: str) -> Dict[str, any]:
    """Compare metadata from different methods"""
    
    print(f"\n{'='*60}")
    print(f"VALIDATION RESULTS FOR: {file_name}")
    print(f"{'='*60}")
    
    print(f"ExifTool fields:     {len(exiftool_data):>4}")
    print(f"PIL fields:          {len(pil_data):>4}")
    print(f"Fast EXIF fields:    {len(fast_exif_data):>4}")
    
    # Show sample fields from each method
    print(f"\nSample ExifTool fields:")
    for i, (key, value) in enumerate(list(exiftool_data.items())[:5]):
        print(f"  {key}: {value}")
    
    print(f"\nSample PIL fields:")
    for i, (key, value) in enumerate(list(pil_data.items())[:5]):
        print(f"  {key}: {value}")
    
    print(f"\nSample Fast EXIF fields:")
    for i, (key, value) in enumerate(list(fast_exif_data.items())[:5]):
        print(f"  {key}: {value}")
    
    # Calculate accuracy
    if len(exiftool_data) > 0:
        fast_exif_accuracy = len(fast_exif_data) / len(exiftool_data) * 100
        pil_accuracy = len(pil_data) / len(exiftool_data) * 100
    else:
        fast_exif_accuracy = 0
        pil_accuracy = 0
    
    print(f"\nACCURACY COMPARISON:")
    print(f"Fast EXIF Reader: {fast_exif_accuracy:.1f}% ({len(fast_exif_data)}/{len(exiftool_data)})")
    print(f"PIL/Pillow:       {pil_accuracy:.1f}% ({len(pil_data)}/{len(exiftool_data)})")
    
    return {
        'file': file_name,
        'exiftool_fields': len(exiftool_data),
        'pil_fields': len(pil_data),
        'fast_exif_fields': len(fast_exif_data),
        'fast_exif_accuracy': fast_exif_accuracy,
        'pil_accuracy': pil_accuracy
    }


def benchmark_speed_and_accuracy(files: List[str]) -> Dict[str, any]:
    """Benchmark both speed and accuracy"""
    
    print(f"\n{'='*80}")
    print(f"SPEED AND ACCURACY BENCHMARK")
    print(f"{'='*80}")
    print(f"Testing {len(files)} files")
    
    # Speed benchmarks
    methods = [
        ("ExifTool", get_exiftool_metadata),
        ("PIL/Pillow", get_pil_metadata),
        ("Fast EXIF Reader", get_fast_exif_metadata)
    ]
    
    speed_results = {}
    
    for method_name, method_func in methods:
        print(f"\nBenchmarking {method_name}...")
        
        start_time = time.time()
        success_count = 0
        total_fields = 0
        
        for file_path in files:
            try:
                metadata = method_func(file_path)
                if metadata:
                    success_count += 1
                    total_fields += len(metadata)
            except Exception:
                pass
        
        end_time = time.time()
        total_time = end_time - start_time
        
        speed_results[method_name] = {
            'total_time': total_time,
            'files_per_second': len(files) / total_time if total_time > 0 else 0,
            'success_count': success_count,
            'total_fields': total_fields,
            'avg_fields_per_file': total_fields / success_count if success_count > 0 else 0
        }
        
        print(f"  Total time: {total_time:.2f}s")
        print(f"  Files/sec: {len(files) / total_time:.1f}")
        print(f"  Success: {success_count}/{len(files)}")
        print(f"  Total fields: {total_fields}")
    
    # Accuracy validation (sample files)
    print(f"\n{'='*80}")
    print(f"ACCURACY VALIDATION")
    print(f"{'='*80}")
    
    validation_results = []
    sample_files = files[:3]  # Test first 3 files
    
    for file_path in sample_files:
        file_name = os.path.basename(file_path)
        
        exiftool_data = get_exiftool_metadata(file_path)
        pil_data = get_pil_metadata(file_path)
        fast_exif_data = get_fast_exif_metadata(file_path)
        
        result = compare_metadata(exiftool_data, pil_data, fast_exif_data, file_name)
        validation_results.append(result)
    
    # Summary
    print(f"\n{'='*80}")
    print(f"FINAL SUMMARY")
    print(f"{'='*80}")
    
    print(f"\nSPEED RANKINGS:")
    sorted_speed = sorted(speed_results.items(), key=lambda x: x[1]['files_per_second'], reverse=True)
    for i, (method, data) in enumerate(sorted_speed):
        print(f"{i+1}. {method:20} | {data['files_per_second']:6.1f} files/s | {data['total_fields']:5} fields")
    
    print(f"\nACCURACY SUMMARY:")
    avg_fast_exif_accuracy = sum(r['fast_exif_accuracy'] for r in validation_results) / len(validation_results)
    avg_pil_accuracy = sum(r['pil_accuracy'] for r in validation_results) / len(validation_results)
    
    print(f"Fast EXIF Reader: {avg_fast_exif_accuracy:.1f}% average accuracy")
    print(f"PIL/Pillow:       {avg_pil_accuracy:.1f}% average accuracy")
    
    print(f"\nCONCLUSION:")
    if avg_fast_exif_accuracy < 10:
        print("❌ Fast EXIF Reader is FAST but INACCURATE - not suitable for production use")
    elif avg_fast_exif_accuracy < 50:
        print("⚠️  Fast EXIF Reader is FAST but PARTIALLY ACCURATE - needs improvement")
    else:
        print("✅ Fast EXIF Reader is FAST and ACCURATE - suitable for production use")
    
    return {
        'speed_results': speed_results,
        'validation_results': validation_results,
        'avg_fast_exif_accuracy': avg_fast_exif_accuracy,
        'avg_pil_accuracy': avg_pil_accuracy
    }


def main():
    directory = "/keg/pictures/incoming/2025/09-Sep/"
    
    if not os.path.exists(directory):
        print(f"Directory not found: {directory}")
        return 1
    
    # Get image files
    image_extensions = {'.jpg', '.jpeg', '.cr2', '.nef', '.heic', '.heif', '.tiff', '.tif', '.png'}
    files = []
    
    print(f"Scanning {directory}...")
    for root, dirs, filenames in os.walk(directory):
        for filename in filenames:
            if filename.lower().endswith(tuple(image_extensions)):
                files.append(os.path.join(root, filename))
                if len(files) >= 10:  # Limit to 10 files for testing
                    break
        if len(files) >= 10:
            break
    
    if not files:
        print(f"No image files found in {directory}")
        return 1
    
    print(f"Found {len(files)} image files")
    
    # Run benchmark
    results = benchmark_speed_and_accuracy(files)
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

