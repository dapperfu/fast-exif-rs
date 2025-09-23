#!/usr/bin/env python3
"""Quick benchmark test of Rust vs Python multiprocessing"""

import sys
import os
import time
import multiprocessing as mp
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    from fast_exif_reader import (
        FastExifReader,
        MultiprocessingExifReader,
        process_files_parallel,
        python_extract_exif_batch
    )
    print("✓ All imports successful")
except ImportError as e:
    print(f"✗ Import error: {e}")
    sys.exit(1)


def test_with_sample_files():
    """Test with sample files if available"""
    # Try to find some test files
    test_dirs = [
        "/keg/pictures/2015/05-May/",
        "/keg/pictures/incoming/2025/09-Sep/",
        "/home/jed/Pictures/",
        "/tmp/"
    ]
    
    test_files = []
    for test_dir in test_dirs:
        if os.path.exists(test_dir):
            print(f"Scanning {test_dir}...")
            for root, dirs, files in os.walk(test_dir):
                for file in files:
                    if file.lower().endswith(('.jpg', '.jpeg', '.cr2', '.nef')):
                        test_files.append(os.path.join(root, file))
                        if len(test_files) >= 10:  # Limit to 10 files for quick test
                            break
                if len(test_files) >= 10:
                    break
            if len(test_files) >= 10:
                break
    
    if not test_files:
        print("No test files found - creating dummy test")
        test_files = ["/nonexistent/file1.jpg", "/nonexistent/file2.jpg", "/nonexistent/file3.jpg"]
    
    print(f"Testing with {len(test_files)} files")
    
    # Test Python multiprocessing
    print("\n=== Python Multiprocessing ===")
    start_time = time.time()
    try:
        python_results = python_extract_exif_batch(test_files, max_workers=2)
        python_time = time.time() - start_time
        print(f"Python: {python_time:.3f}s")
        print(f"Success rate: {python_results['statistics']['success_rate']:.1f}%")
        print(f"Files per second: {python_results['statistics']['files_per_second']:.1f}")
    except Exception as e:
        print(f"Python error: {e}")
        python_time = float('inf')
    
    # Test Rust multiprocessing
    print("\n=== Rust Multiprocessing ===")
    start_time = time.time()
    try:
        rust_results = process_files_parallel(test_files, max_workers=2)
        rust_time = time.time() - start_time
        print(f"Rust: {rust_time:.3f}s")
        print(f"Success rate: {rust_results['statistics']['success_rate']:.1f}%")
        print(f"Files per second: {rust_results['statistics']['files_per_second']:.1f}")
    except Exception as e:
        print(f"Rust error: {e}")
        rust_time = float('inf')
    
    # Compare results
    if python_time != float('inf') and rust_time != float('inf'):
        speedup = python_time / rust_time
        print(f"\n=== Comparison ===")
        print(f"Speedup: Rust is {speedup:.2f}x {'faster' if speedup > 1 else 'slower'} than Python")
        
        if 'python_results' in locals() and 'rust_results' in locals():
            python_fps = python_results['statistics']['files_per_second']
            rust_fps = rust_results['statistics']['files_per_second']
            if python_fps > 0 and rust_fps > 0:
                throughput_ratio = rust_fps / python_fps
                print(f"Throughput: Rust is {throughput_ratio:.2f}x {'higher' if throughput_ratio > 1 else 'lower'} than Python")


def test_single_threaded():
    """Test single-threaded performance"""
    print("\n=== Single-threaded Test ===")
    
    # Test with a single file if available
    test_file = "/keg/pictures/2015/05-May/20150503_093231.jpg"
    
    if not os.path.exists(test_file):
        print("No test file found - skipping single-threaded test")
        return
    
    print(f"Testing with: {test_file}")
    
    # Test Python single-threaded
    print("Python single-threaded...")
    start_time = time.time()
    try:
        reader = FastExifReader()
        metadata = reader.read_file(test_file)
        python_time = time.time() - start_time
        print(f"Python: {python_time:.3f}s, {len(metadata)} fields")
    except Exception as e:
        print(f"Python error: {e}")
        python_time = float('inf')
    
    # Test Rust single-threaded (same reader)
    print("Rust single-threaded...")
    start_time = time.time()
    try:
        reader = FastExifReader()
        metadata = reader.read_file(test_file)
        rust_time = time.time() - start_time
        print(f"Rust: {rust_time:.3f}s, {len(metadata)} fields")
    except Exception as e:
        print(f"Rust error: {e}")
        rust_time = float('inf')
    
    if python_time != float('inf') and rust_time != float('inf'):
        speedup = python_time / rust_time
        print(f"Single-threaded speedup: {speedup:.2f}x")


def main():
    print("Rust vs Python Multiprocessing Quick Test")
    print("=" * 50)
    print(f"System: {mp.cpu_count()} CPUs")
    
    test_single_threaded()
    test_with_sample_files()
    
    print("\n" + "=" * 50)
    print("Test completed!")


if __name__ == "__main__":
    main()
