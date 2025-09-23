#!/usr/bin/env python3
"""
Comprehensive benchmark comparing Python vs Rust multiprocessing implementations.

This script benchmarks:
1. Python multiprocessing (ProcessPoolExecutor)
2. Rust multiprocessing (Rayon parallel processing)
3. Single-threaded Python
4. Single-threaded Rust

Tests various scenarios:
- Different file counts (10, 50, 100, 500, 1000)
- Different thread counts (1, 2, 4, 8, 16)
- Memory usage tracking
- Detailed performance metrics
"""

import os
import sys
import time
import psutil
import multiprocessing as mp
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path
import statistics
import argparse
import json
from typing import List, Dict, Any, Tuple
from datetime import datetime
import gc

# Add the parent directory to path to import our module
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from fast_exif_reader import (
        FastExifReader, 
        MultiprocessingExifReader,
        process_files_parallel,
        process_directory_parallel
    )
    from fast_exif_reader.multiprocessing import (
        extract_exif_batch,
        MultiprocessingExifReader as PythonMultiprocessingExifReader
    )
    RUST_AVAILABLE = True
except ImportError as e:
    print(f"Warning: Rust implementation not available: {e}")
    RUST_AVAILABLE = False

try:
    from PIL import Image
    from PIL.ExifTags import TAGS
    PIL_AVAILABLE = True
except ImportError:
    print("Warning: PIL/Pillow not available")
    PIL_AVAILABLE = False


class BenchmarkResults:
    """Container for comprehensive benchmark results"""
    
    def __init__(self):
        self.results = {}
        self.system_info = self._get_system_info()
        self.timestamp = datetime.now().isoformat()
    
    def _get_system_info(self) -> Dict[str, Any]:
        """Get system information for context"""
        return {
            'cpu_count': mp.cpu_count(),
            'memory_total_gb': psutil.virtual_memory().total / (1024**3),
            'python_version': sys.version,
            'platform': sys.platform
        }
    
    def add_result(self, method: str, file_count: int, thread_count: int, 
                   times: List[float], memory_usage_mb: List[float],
                   success_count: int, error_count: int) -> None:
        """Add benchmark result"""
        if method not in self.results:
            self.results[method] = {}
        
        key = f"{file_count}_files_{thread_count}_threads"
        
        if times:
            self.results[method][key] = {
                'file_count': file_count,
                'thread_count': thread_count,
                'times': times,
                'total_time': sum(times),
                'avg_time': statistics.mean(times),
                'median_time': statistics.median(times),
                'min_time': min(times),
                'max_time': max(times),
                'std_dev': statistics.stdev(times) if len(times) > 1 else 0,
                'memory_usage_mb': memory_usage_mb,
                'avg_memory_mb': statistics.mean(memory_usage_mb) if memory_usage_mb else 0,
                'max_memory_mb': max(memory_usage_mb) if memory_usage_mb else 0,
                'success_count': success_count,
                'error_count': error_count,
                'success_rate': success_count / (success_count + error_count) * 100,
                'files_per_second': file_count / statistics.mean(times),
                'throughput_mb_per_sec': self._calculate_throughput(file_count, times)
            }
    
    def _calculate_throughput(self, file_count: int, times: List[float]) -> float:
        """Calculate throughput in MB/s (estimated)"""
        # Estimate average file size as 5MB for throughput calculation
        avg_file_size_mb = 5.0
        total_data_mb = file_count * avg_file_size_mb
        avg_time = statistics.mean(times)
        return total_data_mb / avg_time if avg_time > 0 else 0.0
    
    def print_summary(self) -> None:
        """Print comprehensive summary"""
        print("\n" + "="*100)
        print("RUST vs PYTHON MULTIPROCESSING BENCHMARK SUMMARY")
        print("="*100)
        print(f"Timestamp: {self.timestamp}")
        print(f"System: {self.system_info['cpu_count']} CPUs, {self.system_info['memory_total_gb']:.1f}GB RAM")
        print(f"Python: {self.system_info['python_version'].split()[0]}")
        
        # Group results by file count
        file_counts = set()
        thread_counts = set()
        for method_results in self.results.values():
            for result in method_results.values():
                file_counts.add(result['file_count'])
                thread_counts.add(result['thread_count'])
        
        file_counts = sorted(file_counts)
        thread_counts = sorted(thread_counts)
        
        for file_count in file_counts:
            print(f"\n{'='*80}")
            print(f"RESULTS FOR {file_count} FILES")
            print(f"{'='*80}")
            
            # Find best thread count for each method
            method_best = {}
            for method, results in self.results.items():
                best_result = None
                best_throughput = 0
                
                for key, result in results.items():
                    if result['file_count'] == file_count:
                        if result['files_per_second'] > best_throughput:
                            best_throughput = result['files_per_second']
                            best_result = result
                
                if best_result:
                    method_best[method] = best_result
            
            # Sort by files per second
            sorted_methods = sorted(method_best.items(), 
                                  key=lambda x: x[1]['files_per_second'], 
                                  reverse=True)
            
            print(f"{'Method':<25} {'Threads':<8} {'Total(s)':<8} {'Files/s':<8} {'MB/s':<8} {'Memory(MB)':<10} {'Success%':<8}")
            print("-" * 100)
            
            fastest_time = sorted_methods[0][1]['total_time'] if sorted_methods else 0
            
            for method, result in sorted_methods:
                speedup = fastest_time / result['total_time'] if result['total_time'] > 0 else 0
                print(f"{method:<25} {result['thread_count']:<8} {result['total_time']:<8.2f} "
                      f"{result['files_per_second']:<8.1f} {result['throughput_mb_per_sec']:<8.1f} "
                      f"{result['avg_memory_mb']:<10.1f} {result['success_rate']:<8.1f}")
            
            # Show speedup analysis
            if len(sorted_methods) >= 2:
                print(f"\nSpeedup Analysis:")
                fastest_method = sorted_methods[0][0]
                fastest_result = sorted_methods[0][1]
                
                for method, result in sorted_methods[1:]:
                    speedup = fastest_result['files_per_second'] / result['files_per_second']
                    print(f"  {fastest_method} is {speedup:.2f}x faster than {method}")
    
    def save_json(self, filename: str) -> None:
        """Save results to JSON file"""
        output = {
            'timestamp': self.timestamp,
            'system_info': self.system_info,
            'results': self.results
        }
        
        with open(filename, 'w') as f:
            json.dump(output, f, indent=2)
        
        print(f"\nResults saved to: {filename}")


def get_image_files(directory: str, max_files: int = None) -> List[str]:
    """Get list of image files from directory"""
    image_extensions = {'.jpg', '.jpeg', '.cr2', '.nef', '.heic', '.heif', '.tiff', '.tif', '.png', '.bmp'}
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


def monitor_memory_usage() -> List[float]:
    """Monitor memory usage during execution"""
    process = psutil.Process()
    memory_samples = []
    
    def sample_memory():
        memory_samples.append(process.memory_info().rss / (1024 * 1024))  # MB
    
    return memory_samples


def benchmark_python_single_threaded(files: List[str]) -> Tuple[List[float], List[float], int, int]:
    """Benchmark Python single-threaded processing"""
    if not RUST_AVAILABLE:
        return [], [], 0, len(files)
    
    def process_file(file_path: str) -> Tuple[bool, float]:
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
    memory_samples = []
    success_count = 0
    error_count = 0
    
    process = psutil.Process()
    initial_memory = process.memory_info().rss / (1024 * 1024)
    
    for file_path in files:
        memory_samples.append(process.memory_info().rss / (1024 * 1024))
        success, elapsed_time = process_file(file_path)
        times.append(elapsed_time)
        
        if success:
            success_count += 1
        else:
            error_count += 1
    
    # Normalize memory usage (subtract initial)
    memory_usage = [mem - initial_memory for mem in memory_samples]
    
    return times, memory_usage, success_count, error_count


def benchmark_python_multiprocessing(files: List[str], max_workers: int) -> Tuple[List[float], List[float], int, int]:
    """Benchmark Python multiprocessing"""
    if not RUST_AVAILABLE:
        return [], [], 0, len(files)
    
    def extract_exif_worker(file_path: str) -> Tuple[str, Dict[str, str], float, bool]:
        start_time = time.time()
        try:
            reader = FastExifReader()
            metadata = reader.read_file(file_path)
            end_time = time.time()
            return file_path, metadata, end_time - start_time, True
        except Exception as e:
            end_time = time.time()
            return file_path, {}, end_time - start_time, False
    
    times = []
    memory_samples = []
    success_count = 0
    error_count = 0
    
    process = psutil.Process()
    initial_memory = process.memory_info().rss / (1024 * 1024)
    
    with ProcessPoolExecutor(max_workers=max_workers) as executor:
        future_to_file = {executor.submit(extract_exif_worker, f): f for f in files}
        
        for future in as_completed(future_to_file):
            memory_samples.append(process.memory_info().rss / (1024 * 1024))
            try:
                file_path, metadata, processing_time, success = future.result()
                times.append(processing_time)
                
                if success:
                    success_count += 1
                else:
                    error_count += 1
            except Exception:
                error_count += 1
    
    # Normalize memory usage
    memory_usage = [mem - initial_memory for mem in memory_samples]
    
    return times, memory_usage, success_count, error_count


def benchmark_rust_single_threaded(files: List[str]) -> Tuple[List[float], List[float], int, int]:
    """Benchmark Rust single-threaded processing"""
    if not RUST_AVAILABLE:
        return [], [], 0, len(files)
    
    times = []
    memory_samples = []
    success_count = 0
    error_count = 0
    
    process = psutil.Process()
    initial_memory = process.memory_info().rss / (1024 * 1024)
    
    reader = FastExifReader()
    
    for file_path in files:
        memory_samples.append(process.memory_info().rss / (1024 * 1024))
        start_time = time.time()
        try:
            metadata = reader.read_file(file_path)
            end_time = time.time()
            times.append(end_time - start_time)
            success_count += 1
        except Exception:
            end_time = time.time()
            times.append(end_time - start_time)
            error_count += 1
    
    # Normalize memory usage
    memory_usage = [mem - initial_memory for mem in memory_samples]
    
    return times, memory_usage, success_count, error_count


def benchmark_rust_multiprocessing(files: List[str], max_workers: int) -> Tuple[List[float], List[float], int, int]:
    """Benchmark Rust multiprocessing using Rayon"""
    if not RUST_AVAILABLE:
        return [], [], 0, len(files)
    
    process = psutil.Process()
    initial_memory = process.memory_info().rss / (1024 * 1024)
    
    start_time = time.time()
    
    try:
        # Use the Rust multiprocessing function
        results = process_files_parallel(files, max_workers)
        
        end_time = time.time()
        total_time = end_time - start_time
        
        # Extract results
        times = []
        success_count = 0
        error_count = 0
        
        for file_path, result in results['results'].items():
            times.append(result['processing_time'])
            if result['success']:
                success_count += 1
            else:
                error_count += 1
        
        # Memory usage (approximate)
        final_memory = process.memory_info().rss / (1024 * 1024)
        memory_usage = [final_memory - initial_memory]
        
        return times, memory_usage, success_count, error_count
        
    except Exception as e:
        print(f"Error in Rust multiprocessing: {e}")
        return [], [], 0, len(files)


def benchmark_python_multiprocessing_class(files: List[str], max_workers: int) -> Tuple[List[float], List[float], int, int]:
    """Benchmark Python multiprocessing using the class wrapper"""
    if not RUST_AVAILABLE:
        return [], [], 0, len(files)
    
    process = psutil.Process()
    initial_memory = process.memory_info().rss / (1024 * 1024)
    
    start_time = time.time()
    
    try:
        reader = PythonMultiprocessingExifReader(max_workers=max_workers)
        results = reader.read_files(files)
        
        end_time = time.time()
        total_time = end_time - start_time
        
        # Extract results
        times = []
        success_count = 0
        error_count = 0
        
        for file_path, result in results['results'].items():
            times.append(result['processing_time'])
            if result['success']:
                success_count += 1
            else:
                error_count += 1
        
        # Memory usage
        final_memory = process.memory_info().rss / (1024 * 1024)
        memory_usage = [final_memory - initial_memory]
        
        return times, memory_usage, success_count, error_count
        
    except Exception as e:
        print(f"Error in Python multiprocessing class: {e}")
        return [], [], 0, len(files)


def run_comprehensive_benchmark(directory: str, file_counts: List[int], 
                              thread_counts: List[int], num_runs: int = 3) -> None:
    """Run comprehensive benchmark comparing all methods"""
    
    # Get all files
    all_files = get_image_files(directory)
    
    if not all_files:
        print(f"No image files found in {directory}")
        return
    
    results = BenchmarkResults()
    
    # Define methods to test
    methods = [
        ("Python Single-threaded", benchmark_python_single_threaded),
        ("Python Multiprocessing", benchmark_python_multiprocessing),
        ("Python Multiprocessing Class", benchmark_python_multiprocessing_class),
        ("Rust Single-threaded", benchmark_rust_single_threaded),
        ("Rust Multiprocessing", benchmark_rust_multiprocessing),
    ]
    
    for file_count in file_counts:
        print(f"\n{'='*80}")
        print(f"BENCHMARKING {file_count} FILES")
        print(f"{'='*80}")
        
        test_files = all_files[:file_count]
        
        for thread_count in thread_counts:
            print(f"\nTesting with {thread_count} threads...")
            
            for method_name, benchmark_func in methods:
                print(f"  {method_name}...", end=" ", flush=True)
                
                all_times = []
                all_memory = []
                all_success = 0
                all_errors = 0
                
                for run in range(num_runs):
                    gc.collect()  # Clean up memory between runs
                    
                    if "Single-threaded" in method_name:
                        times, memory, success, errors = benchmark_func(test_files)
                    else:
                        times, memory, success, errors = benchmark_func(test_files, thread_count)
                    
                    all_times.extend(times)
                    all_memory.extend(memory)
                    all_success += success
                    all_errors += errors
                
                if all_times:
                    results.add_result(method_name, file_count, thread_count, 
                                     all_times, all_memory, all_success, all_errors)
                    avg_time = statistics.mean(all_times)
                    print(f"Avg: {avg_time:.3f}s, Success: {all_success}/{len(test_files)*num_runs}")
                else:
                    print("Failed")
    
    # Print summary
    results.print_summary()
    
    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    results_file = f"rust_vs_python_benchmark_{timestamp}.json"
    results.save_json(results_file)


def main():
    parser = argparse.ArgumentParser(description="Rust vs Python multiprocessing benchmark")
    parser.add_argument("directory", help="Directory containing image files")
    parser.add_argument("--file-counts", nargs="+", type=int, default=[10, 50, 100],
                       help="File counts to test (default: 10 50 100)")
    parser.add_argument("--thread-counts", nargs="+", type=int, default=[1, 2, 4, 8],
                       help="Thread counts to test (default: 1 2 4 8)")
    parser.add_argument("--runs", type=int, default=3,
                       help="Number of runs per test (default: 3)")
    parser.add_argument("--max-files", type=int,
                       help="Maximum files to scan from directory")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.directory):
        print(f"Directory not found: {args.directory}")
        return 1
    
    # Check availability
    print("Checking implementations...")
    if RUST_AVAILABLE:
        print("✓ Rust implementation available")
    else:
        print("✗ Rust implementation not available")
        return 1
    
    print(f"\nStarting comprehensive benchmark:")
    print(f"  File counts: {args.file_counts}")
    print(f"  Thread counts: {args.thread_counts}")
    print(f"  Runs per test: {args.runs}")
    
    run_comprehensive_benchmark(args.directory, args.file_counts, 
                               args.thread_counts, args.runs)
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
