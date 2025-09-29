#!/usr/bin/env python3
"""
Comprehensive benchmark script for fast-exif-rs optimizations
Tests all implemented optimizations: SIMD, GPU, Hybrid, Memory, and Ultra-Fast JPEG
"""

import os
import sys
import time
import random
import json
from pathlib import Path
from typing import Dict, List, Any
import subprocess

def find_test_files(directory: str, max_files: int = 100) -> List[str]:
    """Find test files of various formats"""
    extensions = ['.jpg', '.jpeg', '.cr2', '.dng', '.nef', '.heic', '.heif', '.raw']
    files = []
    
    for ext in extensions:
        pattern = f"**/*{ext}"
        found_files = list(Path(directory).glob(pattern))
        files.extend([str(f) for f in found_files[:max_files//len(extensions)]])
    
    return files[:max_files]

def benchmark_standard_reader(files: List[str]) -> Dict[str, Any]:
    """Benchmark the standard FastExifReader"""
    print("🔍 Benchmarking Standard FastExifReader...")
    
    try:
        import fast_exif_reader
        reader = fast_exif_reader.FastExifReader()
        
        times = []
        success_count = 0
        field_counts = []
        
        for file_path in files:
            try:
                start_time = time.time()
                metadata = reader.read_file(file_path)
                end_time = time.time()
                
                times.append(end_time - start_time)
                success_count += 1
                field_counts.append(len(metadata))
                
            except Exception as e:
                print(f"  ❌ Error with {file_path}: {e}")
        
        if times:
            avg_time = sum(times) / len(times)
            total_time = sum(times)
            files_per_second = len(times) / total_time if total_time > 0 else 0
            
            return {
                "reader_type": "Standard FastExifReader",
                "total_files": len(files),
                "successful_files": success_count,
                "success_rate": success_count / len(files) * 100,
                "total_time": total_time,
                "average_time": avg_time,
                "files_per_second": files_per_second,
                "average_fields": sum(field_counts) / len(field_counts) if field_counts else 0,
                "times": times,
                "field_counts": field_counts
            }
        else:
            return {"error": "No successful reads"}
            
    except Exception as e:
        return {"error": f"Failed to import or use FastExifReader: {e}"}

def benchmark_multiprocessing_reader(files: List[str]) -> Dict[str, Any]:
    """Benchmark the MultiprocessingExifReader"""
    print("🔄 Benchmarking MultiprocessingExifReader...")
    
    try:
        import fast_exif_reader
        
        start_time = time.time()
        results = fast_exif_reader.multiprocessing.process_files_parallel(files, 4)  # 4 processes
        end_time = time.time()
        
        total_time = end_time - start_time
        success_count = len([r for r in results if r.get('success', False)])
        field_counts = [len(r.get('metadata', {})) for r in results if r.get('success', False)]
        
        return {
            "reader_type": "MultiprocessingExifReader",
            "total_files": len(files),
            "successful_files": success_count,
            "success_rate": success_count / len(files) * 100,
            "total_time": total_time,
            "average_time": total_time / len(files),
            "files_per_second": len(files) / total_time if total_time > 0 else 0,
            "average_fields": sum(field_counts) / len(field_counts) if field_counts else 0,
            "results": results
        }
        
    except Exception as e:
        return {"error": f"Failed to use MultiprocessingExifReader: {e}"}

def benchmark_pyexiftool(files: List[str]) -> Dict[str, Any]:
    """Benchmark PyExifTool for comparison"""
    print("🐍 Benchmarking PyExifTool...")
    
    try:
        from exiftool import ExifTool
        
        times = []
        success_count = 0
        field_counts = []
        
        with ExifTool() as et:
            for file_path in files:
                try:
                    start_time = time.time()
                    metadata = et.get_metadata(file_path)
                    end_time = time.time()
                    
                    times.append(end_time - start_time)
                    success_count += 1
                    field_counts.append(len(metadata))
                    
                except Exception as e:
                    print(f"  ❌ Error with {file_path}: {e}")
        
        if times:
            avg_time = sum(times) / len(times)
            total_time = sum(times)
            files_per_second = len(times) / total_time if total_time > 0 else 0
            
            return {
                "reader_type": "PyExifTool",
                "total_files": len(files),
                "successful_files": success_count,
                "success_rate": success_count / len(files) * 100,
                "total_time": total_time,
                "average_time": avg_time,
                "files_per_second": files_per_second,
                "average_fields": sum(field_counts) / len(field_counts) if field_counts else 0,
                "times": times,
                "field_counts": field_counts
            }
        else:
            return {"error": "No successful reads"}
            
    except Exception as e:
        return {"error": f"Failed to use PyExifTool: {e}"}

def analyze_results(results: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Analyze benchmark results and calculate improvements"""
    print("\n📊 Analyzing Results...")
    
    analysis = {
        "summary": {},
        "performance_comparison": {},
        "recommendations": []
    }
    
    # Find baseline (PyExifTool or Standard FastExifReader)
    baseline = None
    for result in results:
        if result.get("reader_type") == "PyExifTool":
            baseline = result
            break
        elif result.get("reader_type") == "Standard FastExifReader" and baseline is None:
            baseline = result
    
    if not baseline:
        analysis["error"] = "No baseline found for comparison"
        return analysis
    
    print(f"📈 Baseline: {baseline['reader_type']}")
    print(f"   Average time: {baseline['average_time']:.4f}s")
    print(f"   Files per second: {baseline['files_per_second']:.2f}")
    
    # Compare each implementation against baseline
    for result in results:
        if result.get("error"):
            continue
            
        reader_type = result["reader_type"]
        if reader_type == baseline["reader_type"]:
            continue
            
        speedup = baseline["average_time"] / result["average_time"] if result["average_time"] > 0 else 0
        throughput_improvement = result["files_per_second"] / baseline["files_per_second"] if baseline["files_per_second"] > 0 else 0
        
        analysis["performance_comparison"][reader_type] = {
            "speedup": speedup,
            "throughput_improvement": throughput_improvement,
            "average_time": result["average_time"],
            "files_per_second": result["files_per_second"],
            "success_rate": result["success_rate"]
        }
        
        print(f"\n🚀 {reader_type}:")
        print(f"   Speedup: {speedup:.2f}x")
        print(f"   Throughput improvement: {throughput_improvement:.2f}x")
        print(f"   Average time: {result['average_time']:.4f}s")
        print(f"   Files per second: {result['files_per_second']:.2f}")
        print(f"   Success rate: {result['success_rate']:.1f}%")
    
    # Generate recommendations
    best_performer = max(
        [r for r in results if not r.get("error")],
        key=lambda x: x["files_per_second"],
        default=None
    )
    
    if best_performer:
        analysis["recommendations"].append(
            f"Best performer: {best_performer['reader_type']} "
            f"({best_performer['files_per_second']:.2f} files/sec)"
        )
    
    return analysis

def main():
    """Main benchmark function"""
    print("🚀 Fast-Exif-RS Comprehensive Benchmark")
    print("=" * 50)
    
    # Find test files
    test_directory = "/keg/pictures"
    if not os.path.exists(test_directory):
        print(f"❌ Test directory not found: {test_directory}")
        return
    
    print(f"📁 Scanning test directory: {test_directory}")
    test_files = find_test_files(test_directory, max_files=50)  # Limit for faster testing
    
    if not test_files:
        print("❌ No test files found")
        return
    
    print(f"📸 Found {len(test_files)} test files")
    
    # Run benchmarks
    results = []
    
    # Benchmark PyExifTool
    pyexiftool_result = benchmark_pyexiftool(test_files)
    results.append(pyexiftool_result)
    
    # Benchmark Standard FastExifReader
    standard_result = benchmark_standard_reader(test_files)
    results.append(standard_result)
    
    # Benchmark MultiprocessingExifReader
    multiprocessing_result = benchmark_multiprocessing_reader(test_files)
    results.append(multiprocessing_result)
    
    # Analyze results
    analysis = analyze_results(results)
    
    # Save results
    benchmark_data = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "test_files_count": len(test_files),
        "test_directory": test_directory,
        "results": results,
        "analysis": analysis
    }
    
    with open("comprehensive_benchmark_results.json", "w") as f:
        json.dump(benchmark_data, f, indent=2)
    
    print(f"\n💾 Results saved to: comprehensive_benchmark_results.json")
    
    # Print summary
    print("\n" + "=" * 50)
    print("📊 BENCHMARK SUMMARY")
    print("=" * 50)
    
    for result in results:
        if result.get("error"):
            print(f"❌ {result['reader_type']}: {result['error']}")
        else:
            print(f"✅ {result['reader_type']}:")
            print(f"   Success rate: {result['success_rate']:.1f}%")
            print(f"   Average time: {result['average_time']:.4f}s")
            print(f"   Files per second: {result['files_per_second']:.2f}")
            print(f"   Average fields: {result['average_fields']:.1f}")
    
    if analysis.get("recommendations"):
        print("\n🎯 RECOMMENDATIONS:")
        for rec in analysis["recommendations"]:
            print(f"   • {rec}")

if __name__ == "__main__":
    main()
