#!/usr/bin/env python3
"""
Working EXIF Benchmark

A simplified benchmark that works with any image files,
even those without EXIF data.
"""

import os
import sys
import time
import subprocess
import multiprocessing as mp
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path
import statistics
from typing import List, Tuple

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
    PIL_AVAILABLE = True
except ImportError:
    print("Warning: PIL/Pillow not available")
    PIL_AVAILABLE = False


def get_image_files(directory: str, max_files: int = None) -> List[str]:
    """Get list of image files from directory"""
    image_extensions = {'.jpg', '.jpeg', '.cr2', '.nef', '.heic', '.heif', '.tiff', '.tif', '.png'}
    files = []
    
    print(f"Scanning {directory}...")
    
    for root, dirs, filenames in os.walk(directory):
        for filename in filenames:
            if Path(filename).suffix.lower() in image_extensions:
                files.append(os.path.join(root, filename))
                if max_files and len(files) >= max_files:
                    break
        if max_files and len(files) >= max_files:
            break
    
    print(f"Found {len(files)} image files")
    return files


def test_exiftool(file_path: str) -> Tuple[bool, float, int]:
    """Test ExifTool on a single file"""
    start_time = time.time()
    try:
        result = subprocess.run(
            ['exiftool', '-s3', '-fast', file_path],
            capture_output=True,
            text=True,
            timeout=10,
            check=True
        )
        end_time = time.time()
        # Count non-empty lines as metadata fields
        field_count = len([line for line in result.stdout.split('\n') if line.strip()])
        return True, end_time - start_time, field_count
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, FileNotFoundError):
        end_time = time.time()
        return False, end_time - start_time, 0


def test_pil(file_path: str) -> Tuple[bool, float, int]:
    """Test PIL on a single file"""
    if not PIL_AVAILABLE:
        return False, 0.0, 0
    
    start_time = time.time()
    try:
        with Image.open(file_path) as img:
            exif_data = img._getexif()
            field_count = len(exif_data) if exif_data else 0
        end_time = time.time()
        return True, end_time - start_time, field_count
    except Exception:
        end_time = time.time()
        return False, end_time - start_time, 0


def test_fast_exif(file_path: str) -> Tuple[bool, float, int]:
    """Test our Rust implementation on a single file"""
    if not FAST_EXIF_AVAILABLE:
        return False, 0.0, 0
    
    start_time = time.time()
    try:
        reader = FastExifReader()
        metadata = reader.read_file(file_path)
        field_count = len(metadata)
        end_time = time.time()
        return True, end_time - start_time, field_count
    except Exception:
        end_time = time.time()
        return False, end_time - start_time, 0


def benchmark_method(method_name: str, test_func, files: List[str], num_processes: int = None) -> dict:
    """Benchmark a single method"""
    if num_processes is None:
        num_processes = min(mp.cpu_count(), len(files))
    
    print(f"\nBenchmarking {method_name} with {num_processes} processes...")
    
    times = []
    success_count = 0
    error_count = 0
    total_fields = 0
    
    with ProcessPoolExecutor(max_workers=num_processes) as executor:
        future_to_file = {executor.submit(test_func, f): f for f in files}
        
        for future in as_completed(future_to_file):
            success, elapsed_time, field_count = future.result()
            times.append(elapsed_time)
            if success:
                success_count += 1
                total_fields += field_count
            else:
                error_count += 1
    
    if times:
        return {
            'method': method_name,
            'total_time': sum(times),
            'avg_time': statistics.mean(times),
            'median_time': statistics.median(times),
            'min_time': min(times),
            'max_time': max(times),
            'success_count': success_count,
            'error_count': error_count,
            'success_rate': success_count / len(files) * 100,
            'files_per_second': len(files) / sum(times),
            'total_fields': total_fields,
            'avg_fields_per_file': total_fields / success_count if success_count > 0 else 0
        }
    else:
        return {
            'method': method_name,
            'total_time': 0,
            'avg_time': 0,
            'median_time': 0,
            'min_time': 0,
            'max_time': 0,
            'success_count': 0,
            'error_count': len(files),
            'success_rate': 0,
            'files_per_second': 0,
            'total_fields': 0,
            'avg_fields_per_file': 0
        }


def main():
    directory = "/keg/pictures/incoming/2025/09-Sep/"
    file_counts = [10, 100]
    
    if not os.path.exists(directory):
        print(f"Directory not found: {directory}")
        print("Please check the path and try again.")
        return 1
    
    # Check if ExifTool is available
    try:
        subprocess.run(['exiftool', '-ver'], capture_output=True, check=True)
        print("✓ ExifTool available")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("✗ ExifTool not found. Install from: https://exiftool.org/")
        return 1
    
    # Get all image files
    all_files = get_image_files(directory)
    
    if not all_files:
        print(f"No image files found in {directory}")
        return 1
    
    # Show file format distribution
    format_counts = {}
    for file_path in all_files:
        ext = Path(file_path).suffix.lower()
        format_counts[ext] = format_counts.get(ext, 0) + 1
    
    print("\nFile format distribution:")
    for ext, count in sorted(format_counts.items()):
        print(f"  {ext}: {count} files")
    
    # Run benchmarks for different file counts
    for file_count in file_counts:
        print(f"\n{'='*60}")
        print(f"BENCHMARKING {file_count} FILES")
        print(f"{'='*60}")
        
        test_files = all_files[:file_count]
        
        # Define methods to test
        methods = [
            ("ExifTool", test_exiftool),
        ]
        
        if PIL_AVAILABLE:
            methods.append(("PIL/Pillow", test_pil))
        
        if FAST_EXIF_AVAILABLE:
            methods.append(("Fast EXIF Reader", test_fast_exif))
        
        # Run benchmarks
        results = []
        for method_name, test_func in methods:
            result = benchmark_method(method_name, test_func, test_files)
            results.append(result)
        
        # Print results
        print(f"\nResults for {file_count} files:")
        print("-" * 80)
        
        # Sort by files per second (descending)
        results.sort(key=lambda x: x['files_per_second'], reverse=True)
        
        for i, result in enumerate(results):
            print(f"{i+1}. {result['method']:20} | "
                  f"Total: {result['total_time']:.2f}s | "
                  f"Avg: {result['avg_time']:.3f}s | "
                  f"Files/s: {result['files_per_second']:.1f} | "
                  f"Success: {result['success_rate']:.1f}% | "
                  f"Fields: {result['total_fields']}")
        
        # Calculate speedup
        if len(results) > 1:
            fastest_time = results[0]['total_time']
            print(f"\nSpeedup comparison (relative to fastest):")
            for result in results:
                speedup = fastest_time / result['total_time'] if result['total_time'] > 0 else 0
                print(f"  {result['method']:20}: {speedup:.2f}x")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

