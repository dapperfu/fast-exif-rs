#!/usr/bin/env python3
"""
Corrected Quick Performance Benchmark for fast-exif-rs
Tests key optimizations with proper API usage
"""

import os
import time
import multiprocessing as mp
from pathlib import Path

def find_test_files(directory: str, max_files: int = 5) -> list:
    """Find a small set of test files"""
    extensions = ['.jpg', '.jpeg', '.cr2', '.dng', '.heic']
    files = []
    
    for ext in extensions:
        pattern = f"**/*{ext}"
        found_files = list(Path(directory).glob(pattern))
        if found_files:
            files.append(str(found_files[0]))
            break
    
    return files[:max_files]

def benchmark_standard(files: list) -> dict:
    """Benchmark standard reader"""
    print("🔍 Standard FastExifReader...")
    
    import fast_exif_reader
    reader = fast_exif_reader.FastExifReader()
    
    start_time = time.time()
    success_count = 0
    field_counts = []
    
    for file_path in files:
        try:
            metadata = reader.read_file(file_path)
            success_count += 1
            field_counts.append(len(metadata))
        except Exception as e:
            print(f"  ❌ Error: {e}")
    
    total_time = time.time() - start_time
    
    return {
        "type": "Standard",
        "files_per_sec": len(files) / total_time if total_time > 0 else 0,
        "avg_time": total_time / len(files),
        "success_rate": success_count / len(files) * 100,
        "avg_fields": sum(field_counts) / len(field_counts) if field_counts else 0
    }

def benchmark_multiprocessing(files: list) -> dict:
    """Benchmark multiprocessing reader with correct API"""
    print("🔄 MultiprocessingExifReader...")
    
    import fast_exif_reader
    reader = fast_exif_reader.MultiprocessingExifReader(max_workers=mp.cpu_count())
    
    start_time = time.time()
    results = reader.read_files(files)
    total_time = time.time() - start_time
    
    # Extract results from the correct format
    success_count = 0
    field_counts = []
    
    if 'results' in results:
        for result in results['results']:
            if hasattr(result, 'success') and result.success:
                success_count += 1
                if hasattr(result, 'metadata'):
                    field_counts.append(len(result.metadata))
    
    return {
        "type": f"Multiprocessing ({mp.cpu_count()} cores)",
        "files_per_sec": len(files) / total_time if total_time > 0 else 0,
        "avg_time": total_time / len(files),
        "success_rate": success_count / len(files) * 100,
        "avg_fields": sum(field_counts) / len(field_counts) if field_counts else 0
    }

def benchmark_rust_parallel(files: list) -> dict:
    """Benchmark Rust parallel processing"""
    print("🦀 Rust Parallel Processing...")
    
    import fast_exif_reader
    
    start_time = time.time()
    results = fast_exif_reader.rust_process_files_parallel(files, mp.cpu_count())
    total_time = time.time() - start_time
    
    success_count = 0
    field_counts = []
    
    for key, value in results.items():
        if key != 'stats' and hasattr(value, 'success'):
            if value.success:
                success_count += 1
                if hasattr(value, 'metadata'):
                    field_counts.append(len(value.metadata))
    
    return {
        "type": f"Rust Parallel ({mp.cpu_count()} cores)",
        "files_per_sec": len(files) / total_time if total_time > 0 else 0,
        "avg_time": total_time / len(files),
        "success_rate": success_count / len(files) * 100,
        "avg_fields": sum(field_counts) / len(field_counts) if field_counts else 0
    }

def benchmark_batch_processing(files: list) -> dict:
    """Benchmark batch processing"""
    print("📦 Batch Processing...")
    
    import fast_exif_reader
    
    start_time = time.time()
    results = fast_exif_reader.extract_exif_batch(files)
    total_time = time.time() - start_time
    
    if isinstance(results, list):
        success_count = len([r for r in results if r.get('success', False)])
        field_counts = [len(r.get('metadata', {})) for r in results if r.get('success', False)]
        
        return {
            "type": "Batch Processing",
            "files_per_sec": len(files) / total_time if total_time > 0 else 0,
            "avg_time": total_time / len(files),
            "success_rate": success_count / len(files) * 100,
            "avg_fields": sum(field_counts) / len(field_counts) if field_counts else 0
        }
    else:
        return {
            "type": "Batch Processing",
            "files_per_sec": 0,
            "avg_time": 0,
            "success_rate": 0,
            "avg_fields": 0
        }

def main():
    """Quick benchmark"""
    print("🚀 Fast-Exif-RS Quick Performance Test")
    print("=" * 40)
    
    # Find test files
    test_directory = "/keg/pictures"
    if not os.path.exists(test_directory):
        print(f"❌ Test directory not found: {test_directory}")
        return
    
    test_files = find_test_files(test_directory, max_files=3)
    if not test_files:
        print("❌ No test files found")
        return
    
    print(f"📸 Testing with {len(test_files)} files")
    print(f"🖥️  CPU cores: {mp.cpu_count()}")
    print()
    
    # Run benchmarks
    results = []
    
    try:
        results.append(benchmark_standard(test_files))
    except Exception as e:
        print(f"❌ Standard benchmark failed: {e}")
    
    try:
        results.append(benchmark_multiprocessing(test_files))
    except Exception as e:
        print(f"❌ Multiprocessing benchmark failed: {e}")
    
    try:
        results.append(benchmark_rust_parallel(test_files))
    except Exception as e:
        print(f"❌ Rust parallel benchmark failed: {e}")
    
    try:
        results.append(benchmark_batch_processing(test_files))
    except Exception as e:
        print(f"❌ Batch processing benchmark failed: {e}")
    
    # Results
    print("\n📊 RESULTS:")
    print("-" * 40)
    
    best_throughput = 0
    best_performer = None
    
    for result in results:
        print(f"{result['type']}:")
        print(f"  {result['files_per_sec']:.1f} files/sec")
        print(f"  {result['avg_time']:.3f}s per file")
        print(f"  {result['success_rate']:.0f}% success")
        print(f"  {result['avg_fields']:.0f} fields avg")
        print()
        
        if result['files_per_sec'] > best_throughput:
            best_throughput = result['files_per_sec']
            best_performer = result['type']
    
    if best_performer:
        print(f"🏆 Best performer: {best_performer} ({best_throughput:.1f} files/sec)")
    
    # Check if parallel processing helps
    if len(results) > 1:
        standard_result = next((r for r in results if r['type'] == 'Standard'), None)
        parallel_results = [r for r in results if 'cores' in r['type']]
        
        if standard_result and parallel_results:
            best_parallel = max(parallel_results, key=lambda x: x['files_per_sec'])
            speedup = best_parallel['files_per_sec'] / standard_result['files_per_sec']
            print(f"⚡ Best parallel speedup: {speedup:.1f}x ({best_parallel['type']})")
    
    # Summary
    print(f"\n📈 SUMMARY:")
    print(f"   • Maximum throughput: {best_throughput:.1f} files/sec")
    print(f"   • CPU cores available: {mp.cpu_count()}")
    print(f"   • Test files: {len(test_files)}")
    
    if best_throughput > 100:
        print(f"   • Performance: EXCELLENT (>100 files/sec)")
    elif best_throughput > 50:
        print(f"   • Performance: GOOD (>50 files/sec)")
    else:
        print(f"   • Performance: MODERATE (<50 files/sec)")

if __name__ == "__main__":
    main()
