#!/usr/bin/env python3
"""
Test script to verify Rust multiprocessing implementation works correctly.

This script tests:
1. Basic functionality of Rust multiprocessing
2. Comparison with Python multiprocessing
3. Memory usage and performance metrics
"""

import os
import sys
import time
import psutil
from pathlib import Path
import multiprocessing as mp

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
    print("✓ All imports successful")
except ImportError as e:
    print(f"✗ Import error: {e}")
    sys.exit(1)


def get_test_files(directory: str, max_files: int = 10) -> list:
    """Get test image files"""
    image_extensions = {'.jpg', '.jpeg', '.cr2', '.nef', '.heic', '.heif', '.tiff', '.tif'}
    files = []
    
    for root, dirs, filenames in os.walk(directory):
        for filename in filenames:
            if Path(filename).suffix.lower() in image_extensions:
                files.append(os.path.join(root, filename))
                if len(files) >= max_files:
                    break
        if len(files) >= max_files:
            break
    
    return files


def test_rust_multiprocessing_class():
    """Test Rust MultiprocessingExifReader class"""
    print("\n=== Testing Rust MultiprocessingExifReader Class ===")
    
    # Create test files (use dummy paths for now)
    test_files = [
        "/keg/pictures/2015/05-May/20150503_093231.jpg",
        "/keg/pictures/2015/05-May/20150527_202045.jpg", 
        "/keg/pictures/2015/05-May/20150522_101905.jpg"
    ]
    
    # Filter to only existing files
    existing_files = [f for f in test_files if os.path.exists(f)]
    
    if not existing_files:
        print("No test files found - creating dummy test")
        # Test with non-existent files to verify error handling
        reader = MultiprocessingExifReader(max_workers=2)
        try:
            results = reader.read_files(test_files)
            print(f"Results structure: {list(results.keys())}")
            print(f"Statistics: {results['statistics']}")
            print("✓ Rust MultiprocessingExifReader class works")
        except Exception as e:
            print(f"✗ Error: {e}")
    else:
        print(f"Testing with {len(existing_files)} existing files")
        reader = MultiprocessingExifReader(max_workers=2)
        
        start_time = time.time()
        results = reader.read_files(existing_files)
        end_time = time.time()
        
        print(f"Processing time: {end_time - start_time:.3f}s")
        print(f"Success rate: {results['statistics']['success_rate']:.1f}%")
        print(f"Files per second: {results['statistics']['files_per_second']:.1f}")
        print("✓ Rust MultiprocessingExifReader class works")


def test_rust_multiprocessing_function():
    """Test Rust process_files_parallel function"""
    print("\n=== Testing Rust process_files_parallel Function ===")
    
    test_files = [
        "/keg/pictures/2015/05-May/20150503_093231.jpg",
        "/keg/pictures/2015/05-May/20150527_202045.jpg", 
        "/keg/pictures/2015/05-May/20150522_101905.jpg"
    ]
    
    existing_files = [f for f in test_files if os.path.exists(f)]
    
    if not existing_files:
        print("No test files found - testing error handling")
        try:
            results = process_files_parallel(test_files, max_workers=2)
            print(f"Results structure: {list(results.keys())}")
            print(f"Statistics: {results['statistics']}")
            print("✓ Rust process_files_parallel function works")
        except Exception as e:
            print(f"✗ Error: {e}")
    else:
        print(f"Testing with {len(existing_files)} existing files")
        
        start_time = time.time()
        results = process_files_parallel(existing_files, max_workers=2)
        end_time = time.time()
        
        print(f"Processing time: {end_time - start_time:.3f}s")
        print(f"Success rate: {results['statistics']['success_rate']:.1f}%")
        print(f"Files per second: {results['statistics']['files_per_second']:.1f}")
        print("✓ Rust process_files_parallel function works")


def test_python_vs_rust_comparison():
    """Compare Python vs Rust multiprocessing"""
    print("\n=== Python vs Rust Multiprocessing Comparison ===")
    
    test_files = [
        "/keg/pictures/2015/05-May/20150503_093231.jpg",
        "/keg/pictures/2015/05-May/20150527_202045.jpg", 
        "/keg/pictures/2015/05-May/20150522_101905.jpg"
    ]
    
    existing_files = [f for f in test_files if os.path.exists(f)]
    
    if not existing_files:
        print("No test files found - skipping comparison")
        return
    
    print(f"Comparing with {len(existing_files)} files")
    
    # Test Python multiprocessing
    print("\nTesting Python multiprocessing...")
    start_time = time.time()
    try:
        python_results = extract_exif_batch(existing_files, max_workers=2)
        python_time = time.time() - start_time
        python_success_rate = python_results['statistics']['success_rate']
        python_files_per_sec = python_results['statistics']['files_per_second']
        print(f"  Python: {python_time:.3f}s, {python_success_rate:.1f}% success, {python_files_per_sec:.1f} files/s")
    except Exception as e:
        print(f"  Python error: {e}")
        python_time = float('inf')
        python_files_per_sec = 0
    
    # Test Rust multiprocessing
    print("\nTesting Rust multiprocessing...")
    start_time = time.time()
    try:
        rust_results = process_files_parallel(existing_files, max_workers=2)
        rust_time = time.time() - start_time
        rust_success_rate = rust_results['statistics']['success_rate']
        rust_files_per_sec = rust_results['statistics']['files_per_second']
        print(f"  Rust: {rust_time:.3f}s, {rust_success_rate:.1f}% success, {rust_files_per_sec:.1f} files/s")
    except Exception as e:
        print(f"  Rust error: {e}")
        rust_time = float('inf')
        rust_files_per_sec = 0
    
    # Compare results
    if python_time != float('inf') and rust_time != float('inf'):
        speedup = python_time / rust_time
        print(f"\nSpeedup: Rust is {speedup:.2f}x {'faster' if speedup > 1 else 'slower'} than Python")
        
        if python_files_per_sec > 0 and rust_files_per_sec > 0:
            throughput_ratio = rust_files_per_sec / python_files_per_sec
            print(f"Throughput: Rust is {throughput_ratio:.2f}x {'higher' if throughput_ratio > 1 else 'lower'} than Python")


def test_memory_usage():
    """Test memory usage of different implementations"""
    print("\n=== Memory Usage Test ===")
    
    process = psutil.Process()
    
    # Test single-threaded Rust
    print("Testing single-threaded Rust...")
    initial_memory = process.memory_info().rss / (1024 * 1024)
    reader = FastExifReader()
    final_memory = process.memory_info().rss / (1024 * 1024)
    rust_single_memory = final_memory - initial_memory
    print(f"  Single-threaded Rust: {rust_single_memory:.1f} MB")
    
    # Test multiprocessing Rust
    print("Testing multiprocessing Rust...")
    initial_memory = process.memory_info().rss / (1024 * 1024)
    reader = MultiprocessingExifReader(max_workers=2)
    final_memory = process.memory_info().rss / (1024 * 1024)
    rust_mp_memory = final_memory - initial_memory
    print(f"  Multiprocessing Rust: {rust_mp_memory:.1f} MB")
    
    print(f"Memory overhead: {rust_mp_memory - rust_single_memory:.1f} MB")


def test_directory_processing():
    """Test directory processing functionality"""
    print("\n=== Directory Processing Test ===")
    
    test_directory = "/keg/pictures/2015/05-May/"
    
    if not os.path.exists(test_directory):
        print(f"Test directory {test_directory} not found - skipping")
        return
    
    print(f"Testing directory: {test_directory}")
    
    # Test Rust directory processing
    try:
        start_time = time.time()
        results = process_directory_parallel(test_directory, max_files=5, max_workers=2)
        end_time = time.time()
        
        print(f"Rust directory processing: {end_time - start_time:.3f}s")
        print(f"Files processed: {results['statistics']['total_files']}")
        print(f"Success rate: {results['statistics']['success_rate']:.1f}%")
        print("✓ Rust directory processing works")
    except Exception as e:
        print(f"✗ Rust directory processing error: {e}")


def main():
    print("Rust Multiprocessing Implementation Test")
    print("=" * 50)
    
    print(f"System info:")
    print(f"  CPUs: {mp.cpu_count()}")
    print(f"  Memory: {psutil.virtual_memory().total / (1024**3):.1f} GB")
    print(f"  Python: {sys.version.split()[0]}")
    
    # Run all tests
    test_rust_multiprocessing_class()
    test_rust_multiprocessing_function()
    test_python_vs_rust_comparison()
    test_memory_usage()
    test_directory_processing()
    
    print("\n" + "=" * 50)
    print("All tests completed!")


if __name__ == "__main__":
    main()
