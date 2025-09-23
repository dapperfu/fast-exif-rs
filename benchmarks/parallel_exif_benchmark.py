#!/usr/bin/env python3
"""
Parallel EXIF Extraction Benchmark

Compares performance of:
1. ExifTool (command line)
2. Pure Python methods (PIL/Pillow, exifread)
3. Fast EXIF Reader (Rust implementation)

Tests with different file counts: 10, 100, 1000 files
Uses parallel processing for fair comparison
"""

import os
import sys
import time
import subprocess
import multiprocessing as mp
from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor, as_completed
from pathlib import Path
import statistics
from typing import List, Dict, Any, Tuple
import json
import argparse

# Add the parent directory to path to import our module
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from fast_exif_reader import FastExifReader
except ImportError:
    print("Warning: fast_exif_reader not available. Install with: pip install -e .")
    FastExifReader = None

try:
    from PIL import Image
    from PIL.ExifTags import TAGS
except ImportError:
    print("Warning: PIL/Pillow not available. Install with: pip install Pillow")
    Image = None
    TAGS = None

try:
    import exifread
except ImportError:
    print("Warning: exifread not available. Install with: pip install exifread")
    exifread = None


class BenchmarkResults:
    """Container for benchmark results"""
    def __init__(self):
        self.methods = {}
        self.file_counts = []
        self.total_files = 0
        self.supported_formats = set()
        
    def add_result(self, method: str, file_count: int, times: List[float], 
                   success_count: int, error_count: int, avg_memory_mb: float = 0):
        if method not in self.methods:
            self.methods[method] = {}
        
        self.methods[method][file_count] = {
            'times': times,
            'avg_time': statistics.mean(times),
            'median_time': statistics.median(times),
            'min_time': min(times),
            'max_time': max(times),
            'std_dev': statistics.stdev(times) if len(times) > 1 else 0,
            'success_count': success_count,
            'error_count': error_count,
            'success_rate': success_count / (success_count + error_count) * 100,
            'avg_memory_mb': avg_memory_mb,
            'files_per_second': file_count / statistics.mean(times) if times else 0
        }
    
    def print_summary(self):
        print("\n" + "="*80)
        print("BENCHMARK SUMMARY")
        print("="*80)
        
        for file_count in sorted(self.file_counts):
            print(f"\nFile Count: {file_count}")
            print("-" * 40)
            
            # Sort methods by average time
            method_times = []
            for method, data in self.methods.items():
                if file_count in data:
                    method_times.append((method, data[file_count]['avg_time']))
            
            method_times.sort(key=lambda x: x[1])
            
            for i, (method, avg_time) in enumerate(method_times):
                data = self.methods[method][file_count]
                speedup = method_times[0][1] / avg_time if avg_time > 0 else 0
                
                print(f"{i+1}. {method:20} | "
                      f"Avg: {avg_time:.3f}s | "
                      f"Files/s: {data['files_per_second']:.1f} | "
                      f"Success: {data['success_rate']:.1f}% | "
                      f"Speedup: {speedup:.2f}x")
    
    def save_json(self, filename: str):
        """Save results to JSON file"""
        with open(filename, 'w') as f:
            json.dump({
                'methods': self.methods,
                'file_counts': self.file_counts,
                'total_files': self.total_files,
                'supported_formats': list(self.supported_formats)
            }, f, indent=2)


def get_image_files(directory: str, max_files: int = None) -> List[str]:
    """Get list of image files from directory"""
    image_extensions = {'.jpg', '.jpeg', '.cr2', '.nef', '.heic', '.heif', '.tiff', '.tif'}
    files = []
    
    for root, dirs, filenames in os.walk(directory):
        for filename in filenames:
            if Path(filename).suffix.lower() in image_extensions:
                files.append(os.path.join(root, filename))
                if max_files and len(files) >= max_files:
                    break
        if max_files and len(files) >= max_files:
            break
    
    return files[:max_files] if max_files else files


def benchmark_exiftool(files: List[str], num_processes: int = None) -> Tuple[List[float], int, int]:
    """Benchmark ExifTool using parallel subprocess calls"""
    if num_processes is None:
        num_processes = min(mp.cpu_count(), len(files))
    
    def extract_exif_exiftool(file_path: str) -> Tuple[bool, float]:
        """Extract EXIF using ExifTool"""
        start_time = time.time()
        try:
            result = subprocess.run(
                ['exiftool', '-s3', '-fast', file_path],
                capture_output=True,
                text=True,
                timeout=30,
                check=True
            )
            end_time = time.time()
            return True, end_time - start_time
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired, FileNotFoundError):
            end_time = time.time()
            return False, end_time - start_time
    
    times = []
    success_count = 0
    error_count = 0
    
    with ProcessPoolExecutor(max_workers=num_processes) as executor:
        future_to_file = {executor.submit(extract_exif_exiftool, f): f for f in files}
        
        for future in as_completed(future_to_file):
            success, elapsed_time = future.result()
            times.append(elapsed_time)
            if success:
                success_count += 1
            else:
                error_count += 1
    
    return times, success_count, error_count


def benchmark_pil(files: List[str], num_processes: int = None) -> Tuple[List[float], int, int]:
    """Benchmark PIL/Pillow EXIF extraction"""
    if Image is None:
        return [], 0, len(files)
    
    if num_processes is None:
        num_processes = min(mp.cpu_count(), len(files))
    
    def extract_exif_pil(file_path: str) -> Tuple[bool, float]:
        """Extract EXIF using PIL"""
        start_time = time.time()
        try:
            with Image.open(file_path) as img:
                exif_data = img._getexif()
                if exif_data:
                    # Convert to readable format
                    exif_dict = {TAGS[k]: v for k, v in exif_data.items() if k in TAGS}
            end_time = time.time()
            return True, end_time - start_time
        except Exception:
            end_time = time.time()
            return False, end_time - start_time
    
    times = []
    success_count = 0
    error_count = 0
    
    with ProcessPoolExecutor(max_workers=num_processes) as executor:
        future_to_file = {executor.submit(extract_exif_pil, f): f for f in files}
        
        for future in as_completed(future_to_file):
            success, elapsed_time = future.result()
            times.append(elapsed_time)
            if success:
                success_count += 1
            else:
                error_count += 1
    
    return times, success_count, error_count


def benchmark_exifread(files: List[str], num_processes: int = None) -> Tuple[List[float], int, int]:
    """Benchmark exifread library"""
    if exifread is None:
        return [], 0, len(files)
    
    if num_processes is None:
        num_processes = min(mp.cpu_count(), len(files))
    
    def extract_exif_exifread(file_path: str) -> Tuple[bool, float]:
        """Extract EXIF using exifread"""
        start_time = time.time()
        try:
            with open(file_path, 'rb') as f:
                tags = exifread.process_file(f)
            end_time = time.time()
            return True, end_time - start_time
        except Exception:
            end_time = time.time()
            return False, end_time - start_time
    
    times = []
    success_count = 0
    error_count = 0
    
    with ProcessPoolExecutor(max_workers=num_processes) as executor:
        future_to_file = {executor.submit(extract_exif_exifread, f): f for f in files}
        
        for future in as_completed(future_to_file):
            success, elapsed_time = future.result()
            times.append(elapsed_time)
            if success:
                success_count += 1
            else:
                error_count += 1
    
    return times, success_count, error_count


def benchmark_fast_exif_reader(files: List[str], num_processes: int = None) -> Tuple[List[float], int, int]:
    """Benchmark our Rust implementation"""
    if FastExifReader is None:
        return [], 0, len(files)
    
    if num_processes is None:
        num_processes = min(mp.cpu_count(), len(files))
    
    def extract_exif_fast(file_path: str) -> Tuple[bool, float]:
        """Extract EXIF using our Rust implementation"""
        start_time = time.time()
        try:
            reader = FastExifReader()
            metadata = reader.read_file(file_path)
            end_time = time.time()
            return True, end_time - start_time
        except Exception:
            end_time = time.time()
            return False, end_time - start_time
    
    times = []
    success_count = 0
    error_count = 0
    
    with ProcessPoolExecutor(max_workers=num_processes) as executor:
        future_to_file = {executor.submit(extract_exif_fast, f): f for f in files}
        
        for future in as_completed(future_to_file):
            success, elapsed_time = future.result()
            times.append(elapsed_time)
            if success:
                success_count += 1
            else:
                error_count += 1
    
    return times, success_count, error_count


def run_benchmark(directory: str, file_counts: List[int], num_runs: int = 3):
    """Run comprehensive benchmark"""
    print(f"Scanning directory: {directory}")
    all_files = get_image_files(directory)
    
    if not all_files:
        print(f"No image files found in {directory}")
        return
    
    print(f"Found {len(all_files)} image files")
    
    # Show file format distribution
    format_counts = {}
    for file_path in all_files:
        ext = Path(file_path).suffix.lower()
        format_counts[ext] = format_counts.get(ext, 0) + 1
    
    print("File format distribution:")
    for ext, count in sorted(format_counts.items()):
        print(f"  {ext}: {count} files")
    
    results = BenchmarkResults()
    results.total_files = len(all_files)
    results.supported_formats = set(format_counts.keys())
    
    for file_count in file_counts:
        print(f"\n{'='*60}")
        print(f"BENCHMARKING {file_count} FILES")
        print(f"{'='*60}")
        
        # Use first N files for this test
        test_files = all_files[:file_count]
        results.file_counts.append(file_count)
        
        # Run each method multiple times
        methods = [
            ("ExifTool", benchmark_exiftool),
            ("PIL/Pillow", benchmark_pil),
            ("exifread", benchmark_exifread),
            ("Fast EXIF Reader", benchmark_fast_exif_reader),
        ]
        
        for method_name, benchmark_func in methods:
            print(f"\nTesting {method_name}...")
            
            all_times = []
            all_success = 0
            all_errors = 0
            
            for run in range(num_runs):
                print(f"  Run {run + 1}/{num_runs}...", end=" ", flush=True)
                
                times, success_count, error_count = benchmark_func(test_files)
                
                all_times.extend(times)
                all_success += success_count
                all_errors += error_count
                
                avg_time = statistics.mean(times) if times else 0
                print(f"Avg: {avg_time:.3f}s, Success: {success_count}/{len(test_files)}")
            
            # Calculate overall statistics
            if all_times:
                results.add_result(method_name, file_count, all_times, all_success, all_errors)
            else:
                print(f"  No successful runs for {method_name}")
    
    # Print summary
    results.print_summary()
    
    # Save results
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    results_file = f"benchmark_results_{timestamp}.json"
    results.save_json(results_file)
    print(f"\nResults saved to: {results_file}")


def main():
    parser = argparse.ArgumentParser(description="Parallel EXIF extraction benchmark")
    parser.add_argument("directory", help="Directory containing image files")
    parser.add_argument("--file-counts", nargs="+", type=int, default=[10, 100, 1000],
                       help="File counts to test (default: 10 100 1000)")
    parser.add_argument("--runs", type=int, default=3,
                       help="Number of runs per test (default: 3)")
    parser.add_argument("--max-files", type=int,
                       help="Maximum files to scan from directory")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.directory):
        print(f"Directory not found: {args.directory}")
        return 1
    
    # Check if ExifTool is available
    try:
        subprocess.run(['exiftool', '-ver'], capture_output=True, check=True)
        print("✓ ExifTool available")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("✗ ExifTool not found. Install from: https://exiftool.org/")
    
    # Check Python libraries
    if Image:
        print("✓ PIL/Pillow available")
    else:
        print("✗ PIL/Pillow not available")
    
    if exifread:
        print("✓ exifread available")
    else:
        print("✗ exifread not available")
    
    if FastExifReader:
        print("✓ Fast EXIF Reader available")
    else:
        print("✗ Fast EXIF Reader not available")
    
    print(f"\nStarting benchmark with file counts: {args.file_counts}")
    print(f"Number of runs per test: {args.runs}")
    
    run_benchmark(args.directory, args.file_counts, args.runs)
    return 0


if __name__ == "__main__":
    sys.exit(main())

