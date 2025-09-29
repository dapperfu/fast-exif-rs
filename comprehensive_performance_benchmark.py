#!/usr/bin/env python3
"""
Comprehensive benchmark for fast-exif-rs with format-specific analysis
"""

import os
import time
import json
from pathlib import Path
from collections import defaultdict

def find_files_by_format(directory: str, max_per_format: int = 10) -> dict:
    """Find files grouped by format"""
    formats = {
        'JPEG': ['.jpg', '.jpeg'],
        'CR2': ['.cr2'],
        'DNG': ['.dng'],
        'HEIC': ['.heic', '.heif'],
        'NEF': ['.nef'],
        'RAW': ['.raw']
    }
    
    files_by_format = defaultdict(list)
    
    for format_name, extensions in formats.items():
        for ext in extensions:
            pattern = f"**/*{ext}"
            found_files = list(Path(directory).glob(pattern))
            files_by_format[format_name].extend([str(f) for f in found_files[:max_per_format]])
    
    # Remove empty formats
    return {k: v for k, v in files_by_format.items() if v}

def benchmark_format_performance(files: list, format_name: str) -> dict:
    """Benchmark performance for a specific format"""
    print(f"📸 Benchmarking {format_name} files ({len(files)} files)...")
    
    try:
        import fast_exif_reader
        reader = fast_exif_reader.FastExifReader()
        
        times = []
        success_count = 0
        field_counts = []
        file_sizes = []
        
        for file_path in files:
            try:
                # Get file size
                file_size = os.path.getsize(file_path)
                file_sizes.append(file_size)
                
                start_time = time.time()
                metadata = reader.read_file(file_path)
                end_time = time.time()
                
                times.append(end_time - start_time)
                success_count += 1
                field_counts.append(len(metadata))
                
            except Exception as e:
                print(f"  ❌ Error with {os.path.basename(file_path)}: {e}")
        
        if times:
            avg_time = sum(times) / len(times)
            total_time = sum(times)
            files_per_second = len(times) / total_time if total_time > 0 else 0
            avg_file_size = sum(file_sizes) / len(file_sizes) if file_sizes else 0
            
            return {
                "format": format_name,
                "total_files": len(files),
                "successful_files": success_count,
                "success_rate": success_count / len(files) * 100,
                "total_time": total_time,
                "average_time": avg_time,
                "files_per_second": files_per_second,
                "average_fields": sum(field_counts) / len(field_counts) if field_counts else 0,
                "average_file_size_mb": avg_file_size / (1024 * 1024),
                "times": times,
                "field_counts": field_counts,
                "file_sizes": file_sizes
            }
        else:
            return {"error": f"No successful reads for {format_name}", "format": format_name}
            
    except Exception as e:
        return {"error": f"Failed to benchmark {format_name}: {e}", "format": format_name}

def benchmark_batch_performance(files: list) -> dict:
    """Benchmark batch processing performance"""
    print(f"🔄 Benchmarking batch processing ({len(files)} files)...")
    
    try:
        import fast_exif_reader
        
        # Test different batch sizes
        batch_sizes = [1, 5, 10, len(files)]
        batch_results = {}
        
        for batch_size in batch_sizes:
            if batch_size > len(files):
                continue
                
            batch_files = files[:batch_size]
            
            start_time = time.time()
            results = []
            
            # Process files individually (simulating batch)
            reader = fast_exif_reader.FastExifReader()
            for file_path in batch_files:
                try:
                    metadata = reader.read_file(file_path)
                    results.append({"success": True, "fields": len(metadata)})
                except Exception as e:
                    results.append({"success": False, "error": str(e)})
            
            end_time = time.time()
            
            total_time = end_time - start_time
            success_count = len([r for r in results if r.get("success", False)])
            
            batch_results[f"batch_{batch_size}"] = {
                "batch_size": batch_size,
                "total_time": total_time,
                "average_time_per_file": total_time / batch_size,
                "files_per_second": batch_size / total_time if total_time > 0 else 0,
                "success_rate": success_count / batch_size * 100
            }
        
        return batch_results
        
    except Exception as e:
        return {"error": f"Failed to benchmark batch processing: {e}"}

def analyze_performance(results: dict) -> dict:
    """Analyze performance results"""
    print("\n📊 Performance Analysis...")
    
    analysis = {
        "format_performance": {},
        "overall_stats": {},
        "recommendations": []
    }
    
    # Analyze format-specific performance
    format_times = []
    format_throughputs = []
    
    for format_name, result in results.items():
        if isinstance(result, dict) and not result.get("error"):
            format_times.append(result["average_time"])
            format_throughputs.append(result["files_per_second"])
            
            analysis["format_performance"][format_name] = {
                "average_time": result["average_time"],
                "files_per_second": result["files_per_second"],
                "success_rate": result["success_rate"],
                "average_fields": result["average_fields"],
                "average_file_size_mb": result["average_file_size_mb"]
            }
            
            print(f"📈 {format_name}:")
            print(f"   Average time: {result['average_time']:.4f}s")
            print(f"   Files per second: {result['files_per_second']:.2f}")
            print(f"   Success rate: {result['success_rate']:.1f}%")
            print(f"   Average fields: {result['average_fields']:.1f}")
            print(f"   Average file size: {result['average_file_size_mb']:.2f}MB")
    
    # Overall statistics
    if format_times:
        analysis["overall_stats"] = {
            "fastest_format": min(results.keys(), key=lambda k: results[k].get("average_time", float('inf'))),
            "slowest_format": max(results.keys(), key=lambda k: results[k].get("average_time", 0)),
            "average_time_across_formats": sum(format_times) / len(format_times),
            "average_throughput_across_formats": sum(format_throughputs) / len(format_throughputs)
        }
        
        print(f"\n🏆 Fastest format: {analysis['overall_stats']['fastest_format']}")
        print(f"🐌 Slowest format: {analysis['overall_stats']['slowest_format']}")
        print(f"📊 Average time across formats: {analysis['overall_stats']['average_time_across_formats']:.4f}s")
        print(f"📊 Average throughput across formats: {analysis['overall_stats']['average_throughput_across_formats']:.2f} files/sec")
    
    # Generate recommendations
    if analysis["format_performance"]:
        best_performer = max(
            analysis["format_performance"].items(),
            key=lambda x: x[1]["files_per_second"]
        )
        
        analysis["recommendations"].append(
            f"Best performing format: {best_performer[0]} "
            f"({best_performer[1]['files_per_second']:.2f} files/sec)"
        )
        
        # Performance insights
        if analysis["overall_stats"]["average_throughput_across_formats"] > 100:
            analysis["recommendations"].append("Excellent overall performance (>100 files/sec)")
        elif analysis["overall_stats"]["average_throughput_across_formats"] > 50:
            analysis["recommendations"].append("Good overall performance (>50 files/sec)")
        else:
            analysis["recommendations"].append("Consider further optimizations")
    
    return analysis

def main():
    """Main benchmark function"""
    print("🚀 Fast-Exif-RS Comprehensive Performance Benchmark")
    print("=" * 60)
    
    # Find test files
    test_directory = "/keg/pictures"
    if not os.path.exists(test_directory):
        print(f"❌ Test directory not found: {test_directory}")
        return
    
    print(f"📁 Scanning test directory: {test_directory}")
    files_by_format = find_files_by_format(test_directory, max_per_format=8)
    
    if not files_by_format:
        print("❌ No test files found")
        return
    
    print(f"📸 Found files by format:")
    for format_name, files in files_by_format.items():
        print(f"   {format_name}: {len(files)} files")
    
    # Run benchmarks for each format
    results = {}
    
    for format_name, files in files_by_format.items():
        result = benchmark_format_performance(files, format_name)
        results[format_name] = result
    
    # Benchmark batch processing
    all_files = [f for files in files_by_format.values() for f in files]
    batch_results = benchmark_batch_performance(all_files)
    
    # Analyze results
    analysis = analyze_performance(results)
    
    # Save comprehensive results
    benchmark_data = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "test_directory": test_directory,
        "files_by_format": {k: len(v) for k, v in files_by_format.items()},
        "format_results": results,
        "batch_results": batch_results,
        "analysis": analysis
    }
    
    with open("comprehensive_performance_results.json", "w") as f:
        json.dump(benchmark_data, f, indent=2)
    
    print(f"\n💾 Results saved to: comprehensive_performance_results.json")
    
    # Print final summary
    print("\n" + "=" * 60)
    print("📊 FINAL PERFORMANCE SUMMARY")
    print("=" * 60)
    
    if analysis.get("recommendations"):
        print("🎯 KEY FINDINGS:")
        for rec in analysis["recommendations"]:
            print(f"   • {rec}")
    
    print(f"\n📈 PERFORMANCE BREAKDOWN:")
    for format_name, perf in analysis.get("format_performance", {}).items():
        print(f"   {format_name}: {perf['files_per_second']:.1f} files/sec "
              f"({perf['average_time']:.3f}s avg, {perf['success_rate']:.0f}% success)")

if __name__ == "__main__":
    main()
