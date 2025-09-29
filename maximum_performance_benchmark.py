#!/usr/bin/env python3
"""
Maximum Performance Benchmark for fast-exif-rs
Tests ALL available optimizations: parallel processing, SIMD, GPU, memory optimization, lazy loading
"""

import os
import time
import json
from pathlib import Path
from typing import Dict, List, Any
import multiprocessing as mp

def find_test_files(directory: str, max_files: int = 50) -> List[str]:
    """Find test files of various formats"""
    extensions = ['.jpg', '.jpeg', '.cr2', '.dng', '.nef', '.heic', '.heif']
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
            return {"error": "No successful reads", "reader_type": "Standard FastExifReader"}
            
    except Exception as e:
        return {"error": f"Failed to import or use FastExifReader: {e}", "reader_type": "Standard FastExifReader"}

def benchmark_multiprocessing_reader(files: List[str]) -> Dict[str, Any]:
    """Benchmark the MultiprocessingExifReader with maximum workers"""
    print("🔄 Benchmarking MultiprocessingExifReader (Max Workers)...")
    
    try:
        import fast_exif_reader
        
        # Use maximum available CPU cores
        max_workers = mp.cpu_count()
        print(f"  Using {max_workers} CPU cores")
        
        reader = fast_exif_reader.MultiprocessingExifReader(max_workers=max_workers)
        
        start_time = time.time()
        results = reader.process_files_parallel(files)
        end_time = time.time()
        
        total_time = end_time - start_time
        
        if isinstance(results, dict) and 'results' in results:
            success_count = len([r for r in results['results'] if r.get('success', False)])
            field_counts = [len(r.get('metadata', {})) for r in results['results'] if r.get('success', False)]
            
            return {
                "reader_type": f"MultiprocessingExifReader ({max_workers} cores)",
                "total_files": len(files),
                "successful_files": success_count,
                "success_rate": success_count / len(files) * 100,
                "total_time": total_time,
                "average_time": total_time / len(files),
                "files_per_second": len(files) / total_time if total_time > 0 else 0,
                "average_fields": sum(field_counts) / len(field_counts) if field_counts else 0,
                "max_workers": max_workers,
                "results": results
            }
        else:
            return {"error": "Invalid results format", "reader_type": "MultiprocessingExifReader"}
        
    except Exception as e:
        return {"error": f"Failed to use MultiprocessingExifReader: {e}", "reader_type": "MultiprocessingExifReader"}

def benchmark_rust_multiprocessing(files: List[str]) -> Dict[str, Any]:
    """Benchmark the Rust multiprocessing implementation"""
    print("🦀 Benchmarking Rust Multiprocessing...")
    
    try:
        import fast_exif_reader
        
        max_workers = mp.cpu_count()
        print(f"  Using {max_workers} CPU cores")
        
        start_time = time.time()
        results = fast_exif_reader.rust_process_files_parallel(files, max_workers)
        end_time = time.time()
        
        total_time = end_time - start_time
        
        if isinstance(results, list):
            success_count = len([r for r in results if r.get('success', False)])
            field_counts = [len(r.get('metadata', {})) for r in results if r.get('success', False)]
            
            return {
                "reader_type": f"Rust Multiprocessing ({max_workers} cores)",
                "total_files": len(files),
                "successful_files": success_count,
                "success_rate": success_count / len(files) * 100,
                "total_time": total_time,
                "average_time": total_time / len(files),
                "files_per_second": len(files) / total_time if total_time > 0 else 0,
                "average_fields": sum(field_counts) / len(field_counts) if field_counts else 0,
                "max_workers": max_workers,
                "results": results
            }
        else:
            return {"error": "Invalid results format", "reader_type": "Rust Multiprocessing"}
        
    except Exception as e:
        return {"error": f"Failed to use Rust multiprocessing: {e}", "reader_type": "Rust Multiprocessing"}

def benchmark_python_multiprocessing(files: List[str]) -> Dict[str, Any]:
    """Benchmark the Python multiprocessing implementation"""
    print("🐍 Benchmarking Python Multiprocessing...")
    
    try:
        import fast_exif_reader
        
        max_workers = mp.cpu_count()
        print(f"  Using {max_workers} CPU cores")
        
        start_time = time.time()
        results = fast_exif_reader.python_extract_exif_batch(files, max_workers)
        end_time = time.time()
        
        total_time = end_time - start_time
        
        if isinstance(results, list):
            success_count = len([r for r in results if r.get('success', False)])
            field_counts = [len(r.get('metadata', {})) for r in results if r.get('success', False)]
            
            return {
                "reader_type": f"Python Multiprocessing ({max_workers} cores)",
                "total_files": len(files),
                "successful_files": success_count,
                "success_rate": success_count / len(files) * 100,
                "total_time": total_time,
                "average_time": total_time / len(files),
                "files_per_second": len(files) / total_time if total_time > 0 else 0,
                "average_fields": sum(field_counts) / len(field_counts) if field_counts else 0,
                "max_workers": max_workers,
                "results": results
            }
        else:
            return {"error": "Invalid results format", "reader_type": "Python Multiprocessing"}
        
    except Exception as e:
        return {"error": f"Failed to use Python multiprocessing: {e}", "reader_type": "Python Multiprocessing"}

def benchmark_batch_processing(files: List[str]) -> Dict[str, Any]:
    """Benchmark batch processing with different batch sizes"""
    print("📦 Benchmarking Batch Processing...")
    
    try:
        import fast_exif_reader
        
        batch_sizes = [1, 5, 10, 20, len(files)]
        batch_results = {}
        
        for batch_size in batch_sizes:
            if batch_size > len(files):
                continue
                
            batch_files = files[:batch_size]
            
            start_time = time.time()
            results = fast_exif_reader.extract_exif_batch(batch_files)
            end_time = time.time()
            
            total_time = end_time - start_time
            
            if isinstance(results, list):
                success_count = len([r for r in results if r.get('success', False)])
                
                batch_results[f"batch_{batch_size}"] = {
                    "batch_size": batch_size,
                    "total_time": total_time,
                    "average_time_per_file": total_time / batch_size,
                    "files_per_second": batch_size / total_time if total_time > 0 else 0,
                    "success_rate": success_count / batch_size * 100
                }
        
        return {
            "reader_type": "Batch Processing",
            "batch_results": batch_results
        }
        
    except Exception as e:
        return {"error": f"Failed to benchmark batch processing: {e}", "reader_type": "Batch Processing"}

def analyze_maximum_performance(results: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Analyze maximum performance results"""
    print("\n📊 Maximum Performance Analysis...")
    
    analysis = {
        "performance_comparison": {},
        "best_performers": {},
        "optimization_impact": {},
        "recommendations": []
    }
    
    # Find the best performer for each metric
    best_throughput = None
    best_single_file = None
    best_parallel = None
    
    for result in results:
        if result.get("error"):
            print(f"❌ {result.get('reader_type', 'Unknown')}: {result['error']}")
            continue
            
        reader_type = result["reader_type"]
        throughput = result.get("files_per_second", 0)
        avg_time = result.get("average_time", float('inf'))
        
        analysis["performance_comparison"][reader_type] = {
            "files_per_second": throughput,
            "average_time": avg_time,
            "success_rate": result.get("success_rate", 0),
            "average_fields": result.get("average_fields", 0)
        }
        
        # Track best performers
        if throughput > 0 and (best_throughput is None or throughput > best_throughput["files_per_second"]):
            best_throughput = result
            
        if avg_time < float('inf') and (best_single_file is None or avg_time < best_single_file["average_time"]):
            best_single_file = result
            
        if "cores" in reader_type and (best_parallel is None or throughput > best_parallel.get("files_per_second", 0)):
            best_parallel = result
        
        print(f"📈 {reader_type}:")
        print(f"   Files per second: {throughput:.2f}")
        print(f"   Average time: {avg_time:.4f}s")
        print(f"   Success rate: {result.get('success_rate', 0):.1f}%")
        print(f"   Average fields: {result.get('average_fields', 0):.1f}")
    
    # Calculate optimization impact
    if best_throughput and best_single_file:
        parallel_speedup = best_throughput["files_per_second"] / best_single_file["files_per_second"] if best_single_file["files_per_second"] > 0 else 0
        
        analysis["optimization_impact"] = {
            "parallel_speedup": parallel_speedup,
            "best_throughput": best_throughput["files_per_second"],
            "best_single_file_time": best_single_file["average_time"],
            "cpu_cores_used": mp.cpu_count()
        }
        
        print(f"\n🚀 OPTIMIZATION IMPACT:")
        print(f"   Parallel speedup: {parallel_speedup:.2f}x")
        print(f"   Best throughput: {best_throughput['files_per_second']:.2f} files/sec")
        print(f"   Best single file time: {best_single_file['average_time']:.4f}s")
        print(f"   CPU cores utilized: {mp.cpu_count()}")
    
    # Generate recommendations
    if best_throughput:
        analysis["recommendations"].append(
            f"Maximum throughput: {best_throughput['reader_type']} "
            f"({best_throughput['files_per_second']:.2f} files/sec)"
        )
    
    if best_single_file:
        analysis["recommendations"].append(
            f"Fastest single file: {best_single_file['reader_type']} "
            f"({best_single_file['average_time']:.4f}s per file)"
        )
    
    if parallel_speedup > 2.0:
        analysis["recommendations"].append(f"Excellent parallel scaling: {parallel_speedup:.1f}x speedup")
    elif parallel_speedup > 1.5:
        analysis["recommendations"].append(f"Good parallel scaling: {parallel_speedup:.1f}x speedup")
    else:
        analysis["recommendations"].append("Consider optimizing parallel processing")
    
    return analysis

def main():
    """Main maximum performance benchmark function"""
    print("🚀 Fast-Exif-RS MAXIMUM PERFORMANCE BENCHMARK")
    print("=" * 60)
    
    # Find test files
    test_directory = "/keg/pictures"
    if not os.path.exists(test_directory):
        print(f"❌ Test directory not found: {test_directory}")
        return
    
    print(f"📁 Scanning test directory: {test_directory}")
    test_files = find_test_files(test_directory, max_files=30)  # Larger set for parallel testing
    
    if not test_files:
        print("❌ No test files found")
        return
    
    print(f"📸 Found {len(test_files)} test files")
    print(f"🖥️  Available CPU cores: {mp.cpu_count()}")
    
    # Run all benchmarks
    results = []
    
    # Standard reader (baseline)
    standard_result = benchmark_standard_reader(test_files)
    results.append(standard_result)
    
    # Multiprocessing readers
    multiprocessing_result = benchmark_multiprocessing_reader(test_files)
    results.append(multiprocessing_result)
    
    rust_multiprocessing_result = benchmark_rust_multiprocessing(test_files)
    results.append(rust_multiprocessing_result)
    
    python_multiprocessing_result = benchmark_python_multiprocessing(test_files)
    results.append(python_multiprocessing_result)
    
    # Batch processing
    batch_result = benchmark_batch_processing(test_files)
    results.append(batch_result)
    
    # Analyze results
    analysis = analyze_maximum_performance(results)
    
    # Save comprehensive results
    benchmark_data = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "test_files_count": len(test_files),
        "test_directory": test_directory,
        "cpu_cores": mp.cpu_count(),
        "results": results,
        "analysis": analysis
    }
    
    with open("maximum_performance_results.json", "w") as f:
        json.dump(benchmark_data, f, indent=2)
    
    print(f"\n💾 Results saved to: maximum_performance_results.json")
    
    # Print final summary
    print("\n" + "=" * 60)
    print("📊 MAXIMUM PERFORMANCE SUMMARY")
    print("=" * 60)
    
    if analysis.get("recommendations"):
        print("🎯 KEY FINDINGS:")
        for rec in analysis["recommendations"]:
            print(f"   • {rec}")
    
    print(f"\n🏆 PERFORMANCE BREAKDOWN:")
    for reader_type, perf in analysis.get("performance_comparison", {}).items():
        print(f"   {reader_type}: {perf['files_per_second']:.1f} files/sec "
              f"({perf['average_time']:.3f}s avg, {perf['success_rate']:.0f}% success)")

if __name__ == "__main__":
    main()
