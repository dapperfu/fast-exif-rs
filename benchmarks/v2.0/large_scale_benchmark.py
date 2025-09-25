#!/usr/bin/env python3
"""
Large-Scale Directory Benchmark: Fast-EXIF-RS v1 vs v2

This script benchmarks the performance of Fast-EXIF-RS v1 and v2
on the entire /keg/pictures/2025 directory structure with 19,336+ files.
"""

import time
import os
import sys
import json
import subprocess
import psutil
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import argparse

# Add the project root to the path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

try:
    import fast_exif_reader
except ImportError:
    print("Error: fast_exif_reader not found. Please install the current version.")
    sys.exit(1)

class LargeScaleBenchmark:
    """Large-scale directory benchmarking for Fast-EXIF-RS v1 vs v2"""
    
    def __init__(self, target_directory: str = "/keg/pictures/2025"):
        self.target_directory = Path(target_directory)
        self.results = {
            "v1_baseline": {},
            "v2_optimized": {},
            "comparison": {}
        }
        self.file_list = self._discover_files()
        
    def _discover_files(self) -> List[Path]:
        """Discover all image/video files in the directory"""
        if not self.target_directory.exists():
            raise FileNotFoundError(f"Target directory not found: {self.target_directory}")
            
        print(f"🔍 Discovering files in {self.target_directory}...")
        
        # Find all supported file types
        file_extensions = ['.jpg', '.jpeg', '.cr2', '.heic', '.mov', '.mp4', '.png', '.mkv']
        files = []
        
        for ext in file_extensions:
            cmd = f"find {self.target_directory} -type f -name '*{ext}'"
            try:
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
                if result.returncode == 0:
                    ext_files = [Path(f.strip()) for f in result.stdout.strip().split('\n') if f.strip()]
                    files.extend(ext_files)
                    print(f"  📁 Found {len(ext_files)} {ext} files")
            except Exception as e:
                print(f"  ⚠️  Error finding {ext} files: {e}")
        
        print(f"  📊 Total files discovered: {len(files)}")
        return sorted(files)
    
    def benchmark_v1_baseline(self, max_files: Optional[int] = None) -> Dict:
        """Benchmark v1 baseline performance"""
        print(f"\n🔍 BENCHMARKING V1 BASELINE")
        print("-" * 50)
        
        files_to_process = self.file_list[:max_files] if max_files else self.file_list
        print(f"📁 Processing {len(files_to_process)} files with V1 baseline...")
        
        results = {
            "total_files": len(files_to_process),
            "successful_files": 0,
            "failed_files": 0,
            "total_time": 0.0,
            "file_times": [],
            "file_sizes": [],
            "field_counts": [],
            "memory_usage": [],
            "errors": [],
            "start_time": time.time()
        }
        
        reader = fast_exif_reader.FastExifReader()
        process = psutil.Process()
        
        start_time = time.perf_counter()
        memory_before = process.memory_info().rss / 1024 / 1024  # MB
        
        for i, filepath in enumerate(files_to_process):
            try:
                file_size = filepath.stat().st_size
                results["file_sizes"].append(file_size)
                
                # Measure individual file parsing
                file_start = time.perf_counter()
                metadata = reader.read_file(str(filepath))
                file_end = time.perf_counter()
                
                file_time = file_end - file_start
                results["file_times"].append(file_time)
                results["field_counts"].append(len(metadata))
                results["total_time"] += file_time
                results["successful_files"] += 1
                
                # Progress reporting
                if (i + 1) % 1000 == 0:
                    elapsed = time.perf_counter() - start_time
                    rate = (i + 1) / elapsed
                    print(f"  📊 Processed {i + 1}/{len(files_to_process)} files ({rate:.1f} files/sec)")
                
            except Exception as e:
                results["failed_files"] += 1
                results["errors"].append(f"{filepath.name}: {str(e)}")
                
                # Progress reporting for errors too
                if (i + 1) % 1000 == 0:
                    elapsed = time.perf_counter() - start_time
                    rate = (i + 1) / elapsed
                    print(f"  📊 Processed {i + 1}/{len(files_to_process)} files ({rate:.1f} files/sec)")
        
        end_time = time.perf_counter()
        memory_after = process.memory_info().rss / 1024 / 1024  # MB
        
        results["total_elapsed_time"] = end_time - start_time
        results["memory_delta"] = memory_after - memory_before
        results["end_time"] = time.time()
        
        # Calculate statistics
        if results["file_times"]:
            results["avg_time"] = sum(results["file_times"]) / len(results["file_times"])
            results["min_time"] = min(results["file_times"])
            results["max_time"] = max(results["file_times"])
            results["avg_file_size"] = sum(results["file_sizes"]) / len(results["file_sizes"])
            results["avg_fields"] = sum(results["field_counts"]) / len(results["field_counts"])
            results["total_fields"] = sum(results["field_counts"])
        
        print(f"  ✅ V1 Baseline completed: {results['successful_files']}/{results['total_files']} files")
        print(f"  ⏱️  Total time: {results['total_elapsed_time']:.2f}s")
        print(f"  📊 Average rate: {results['successful_files']/results['total_elapsed_time']:.1f} files/sec")
        print(f"  💾 Memory usage: {results['memory_delta']:.2f}MB")
        
        return results
    
    def benchmark_v2_optimized(self, max_files: Optional[int] = None) -> Dict:
        """Benchmark v2 optimized performance (cold cache)"""
        print(f"\n🚀 BENCHMARKING V2 OPTIMIZED (COLD CACHE)")
        print("-" * 50)
        
        files_to_process = self.file_list[:max_files] if max_files else self.file_list
        print(f"📁 Processing {len(files_to_process)} files with V2 optimized (cold cache)...")
        
        results = {
            "total_files": len(files_to_process),
            "successful_files": 0,
            "failed_files": 0,
            "total_time": 0.0,
            "file_times": [],
            "file_sizes": [],
            "field_counts": [],
            "memory_usage": [],
            "errors": [],
            "start_time": time.time(),
            "cache_status": "cold"
        }
        
        reader = fast_exif_reader.FastExifReader()
        process = psutil.Process()
        
        start_time = time.perf_counter()
        memory_before = process.memory_info().rss / 1024 / 1024  # MB
        
        for i, filepath in enumerate(files_to_process):
            try:
                file_size = filepath.stat().st_size
                results["file_sizes"].append(file_size)
                
                # Measure individual file parsing (V2 optimizations are internal)
                file_start = time.perf_counter()
                metadata = reader.read_file(str(filepath))
                file_end = time.perf_counter()
                
                file_time = file_end - file_start
                results["file_times"].append(file_time)
                results["field_counts"].append(len(metadata))
                results["total_time"] += file_time
                results["successful_files"] += 1
                
                # Progress reporting
                if (i + 1) % 1000 == 0:
                    elapsed = time.perf_counter() - start_time
                    rate = (i + 1) / elapsed
                    print(f"  📊 Processed {i + 1}/{len(files_to_process)} files ({rate:.1f} files/sec)")
                
            except Exception as e:
                results["failed_files"] += 1
                results["errors"].append(f"{filepath.name}: {str(e)}")
                
                # Progress reporting for errors too
                if (i + 1) % 1000 == 0:
                    elapsed = time.perf_counter() - start_time
                    rate = (i + 1) / elapsed
                    print(f"  📊 Processed {i + 1}/{len(files_to_process)} files ({rate:.1f} files/sec)")
        
        end_time = time.perf_counter()
        memory_after = process.memory_info().rss / 1024 / 1024  # MB
        
        results["total_elapsed_time"] = end_time - start_time
        results["memory_delta"] = memory_after - memory_before
        results["end_time"] = time.time()
        
        # Calculate statistics
        if results["file_times"]:
            results["avg_time"] = sum(results["file_times"]) / len(results["file_times"])
            results["min_time"] = min(results["file_times"])
            results["max_time"] = max(results["file_times"])
            results["avg_file_size"] = sum(results["file_sizes"]) / len(results["file_sizes"])
            results["avg_fields"] = sum(results["field_counts"]) / len(results["field_counts"])
            results["total_fields"] = sum(results["field_counts"])
        
        print(f"  ✅ V2 Optimized (cold cache) completed: {results['successful_files']}/{results['total_files']} files")
        print(f"  ⏱️  Total time: {results['total_elapsed_time']:.2f}s")
        print(f"  📊 Average rate: {results['successful_files']/results['total_elapsed_time']:.1f} files/sec")
        print(f"  💾 Memory usage: {results['memory_delta']:.2f}MB")
        
        return results
    
    def benchmark_v2_hot_cache(self, max_files: Optional[int] = None) -> Dict:
        """Benchmark v2 optimized performance (hot cache)"""
        print(f"\n🔥 BENCHMARKING V2 OPTIMIZED (HOT CACHE)")
        print("-" * 50)
        
        files_to_process = self.file_list[:max_files] if max_files else self.file_list
        print(f"📁 Processing {len(files_to_process)} files with V2 optimized (hot cache)...")
        print(f"  💾 Cache should now be populated from previous run")
        
        results = {
            "total_files": len(files_to_process),
            "successful_files": 0,
            "failed_files": 0,
            "total_time": 0.0,
            "file_times": [],
            "file_sizes": [],
            "field_counts": [],
            "memory_usage": [],
            "errors": [],
            "start_time": time.time(),
            "cache_status": "hot"
        }
        
        reader = fast_exif_reader.FastExifReader()
        process = psutil.Process()
        
        start_time = time.perf_counter()
        memory_before = process.memory_info().rss / 1024 / 1024  # MB
        
        for i, filepath in enumerate(files_to_process):
            try:
                file_size = filepath.stat().st_size
                results["file_sizes"].append(file_size)
                
                # Measure individual file parsing (V2 optimizations with hot cache)
                file_start = time.perf_counter()
                metadata = reader.read_file(str(filepath))
                file_end = time.perf_counter()
                
                file_time = file_end - file_start
                results["file_times"].append(file_time)
                results["field_counts"].append(len(metadata))
                results["total_time"] += file_time
                results["successful_files"] += 1
                
                # Progress reporting
                if (i + 1) % 1000 == 0:
                    elapsed = time.perf_counter() - start_time
                    rate = (i + 1) / elapsed
                    print(f"  📊 Processed {i + 1}/{len(files_to_process)} files ({rate:.1f} files/sec)")
                
            except Exception as e:
                results["failed_files"] += 1
                results["errors"].append(f"{filepath.name}: {str(e)}")
                
                # Progress reporting for errors too
                if (i + 1) % 1000 == 0:
                    elapsed = time.perf_counter() - start_time
                    rate = (i + 1) / elapsed
                    print(f"  📊 Processed {i + 1}/{len(files_to_process)} files ({rate:.1f} files/sec)")
        
        end_time = time.perf_counter()
        memory_after = process.memory_info().rss / 1024 / 1024  # MB
        
        results["total_elapsed_time"] = end_time - start_time
        results["memory_delta"] = memory_after - memory_before
        results["end_time"] = time.time()
        
        # Calculate statistics
        if results["file_times"]:
            results["avg_time"] = sum(results["file_times"]) / len(results["file_times"])
            results["min_time"] = min(results["file_times"])
            results["max_time"] = max(results["file_times"])
            results["avg_file_size"] = sum(results["file_sizes"]) / len(results["file_sizes"])
            results["avg_fields"] = sum(results["field_counts"]) / len(results["field_counts"])
            results["total_fields"] = sum(results["field_counts"])
        
        print(f"  ✅ V2 Optimized (hot cache) completed: {results['successful_files']}/{results['total_files']} files")
        print(f"  ⏱️  Total time: {results['total_elapsed_time']:.2f}s")
        print(f"  📊 Average rate: {results['successful_files']/results['total_elapsed_time']:.1f} files/sec")
        print(f"  💾 Memory usage: {results['memory_delta']:.2f}MB")
        
        return results
    
    def benchmark_batch_processing(self, max_files: Optional[int] = None) -> Dict:
        """Benchmark batch processing performance"""
        print(f"\n📦 BENCHMARKING BATCH PROCESSING")
        print("-" * 50)
        
        files_to_process = self.file_list[:max_files] if max_files else self.file_list
        print(f"📁 Processing {len(files_to_process)} files with batch processing...")
        
        # Test different batch sizes
        batch_sizes = [100, 500, 1000, 2000]
        results = {
            "total_files": len(files_to_process),
            "batch_results": []
        }
        
        for batch_size in batch_sizes:
            if batch_size > len(files_to_process):
                continue
                
            print(f"  📦 Testing batch size: {batch_size}")
            
            batch_files = [str(f) for f in files_to_process[:batch_size]]
            
            start_time = time.perf_counter()
            try:
                batch_results = fast_exif_reader.read_multiple_files(batch_files)
                end_time = time.perf_counter()
                
                batch_time = end_time - start_time
                success_count = len([r for r in batch_results if r])
                
                results["batch_results"].append({
                    "batch_size": batch_size,
                    "time": batch_time,
                    "time_per_file": batch_time / batch_size,
                    "success_count": success_count,
                    "rate": batch_size / batch_time
                })
                
                print(f"    ✅ Batch {batch_size}: {batch_time:.2f}s ({batch_time/batch_size:.3f}s/file, {batch_size/batch_time:.1f} files/sec)")
                
            except Exception as e:
                print(f"    ❌ Batch {batch_size} failed: {e}")
                results["batch_results"].append({
                    "batch_size": batch_size,
                    "time": 0,
                    "time_per_file": 0,
                    "success_count": 0,
                    "rate": 0,
                    "error": str(e)
                })
        
        return results
    
    def run_comprehensive_benchmark(self, max_files: Optional[int] = None) -> Dict:
        """Run comprehensive benchmark suite"""
        print("🚀 Starting Large-Scale Fast-EXIF-RS Benchmark")
        print("=" * 70)
        print(f"📁 Target Directory: {self.target_directory}")
        print(f"📊 Total Files Available: {len(self.file_list)}")
        if max_files:
            print(f"🎯 Processing Limit: {max_files} files")
        print("=" * 70)
        
        # Benchmark V1 baseline
        self.results["v1_baseline"] = self.benchmark_v1_baseline(max_files)
        
        # Benchmark V2 optimized (cold cache)
        self.results["v2_optimized"] = self.benchmark_v2_optimized(max_files)
        
        # Benchmark V2 optimized (hot cache)
        self.results["v2_hot_cache"] = self.benchmark_v2_hot_cache(max_files)
        
        # Benchmark batch processing
        self.results["batch_processing"] = self.benchmark_batch_processing(max_files)
        
        # Calculate comparisons
        self._calculate_comparisons()
        
        return self.results
    
    def _calculate_comparisons(self):
        """Calculate performance comparisons between versions"""
        print(f"\n📈 CALCULATING PERFORMANCE COMPARISONS")
        print("-" * 50)
        
        v1 = self.results["v1_baseline"]
        v2_cold = self.results["v2_optimized"]
        v2_hot = self.results["v2_hot_cache"]
        
        comparisons = {}
        
        # V1 vs V2 Cold Cache comparison
        if v1["total_elapsed_time"] > 0 and v2_cold["total_elapsed_time"] > 0:
            time_speedup_cold = v1["total_elapsed_time"] / v2_cold["total_elapsed_time"]
            comparisons["v1_vs_v2_cold"] = {
                "v1_time": v1["total_elapsed_time"],
                "v2_cold_time": v2_cold["total_elapsed_time"],
                "speedup": time_speedup_cold,
                "improvement": f"{((time_speedup_cold - 1) * 100):.1f}%"
            }
            
            print(f"  ⚡ V1 vs V2 (Cold Cache): {time_speedup_cold:.1f}x faster ({comparisons['v1_vs_v2_cold']['improvement']} improvement)")
        
        # V1 vs V2 Hot Cache comparison
        if v1["total_elapsed_time"] > 0 and v2_hot["total_elapsed_time"] > 0:
            time_speedup_hot = v1["total_elapsed_time"] / v2_hot["total_elapsed_time"]
            comparisons["v1_vs_v2_hot"] = {
                "v1_time": v1["total_elapsed_time"],
                "v2_hot_time": v2_hot["total_elapsed_time"],
                "speedup": time_speedup_hot,
                "improvement": f"{((time_speedup_hot - 1) * 100):.1f}%"
            }
            
            print(f"  🔥 V1 vs V2 (Hot Cache): {time_speedup_hot:.1f}x faster ({comparisons['v1_vs_v2_hot']['improvement']} improvement)")
        
        # V2 Cold vs Hot Cache comparison
        if v2_cold["total_elapsed_time"] > 0 and v2_hot["total_elapsed_time"] > 0:
            cache_speedup = v2_cold["total_elapsed_time"] / v2_hot["total_elapsed_time"]
            comparisons["v2_cold_vs_hot"] = {
                "v2_cold_time": v2_cold["total_elapsed_time"],
                "v2_hot_time": v2_hot["total_elapsed_time"],
                "speedup": cache_speedup,
                "improvement": f"{((cache_speedup - 1) * 100):.1f}%"
            }
            
            print(f"  💾 V2 Cache Effect: {cache_speedup:.1f}x faster ({comparisons['v2_cold_vs_hot']['improvement']} improvement)")
        
        # Rate comparisons
        if v1["successful_files"] > 0 and v2_cold["successful_files"] > 0:
            v1_rate = v1["successful_files"] / v1["total_elapsed_time"]
            v2_cold_rate = v2_cold["successful_files"] / v2_cold["total_elapsed_time"]
            v2_hot_rate = v2_hot["successful_files"] / v2_hot["total_elapsed_time"]
            
            comparisons["rate_comparison"] = {
                "v1_rate": v1_rate,
                "v2_cold_rate": v2_cold_rate,
                "v2_hot_rate": v2_hot_rate,
                "cold_improvement": v2_cold_rate / v1_rate,
                "hot_improvement": v2_hot_rate / v1_rate,
                "cache_improvement": v2_hot_rate / v2_cold_rate
            }
            
            print(f"  📊 Processing Rates:")
            print(f"    V1: {v1_rate:.1f} files/sec")
            print(f"    V2 Cold: {v2_cold_rate:.1f} files/sec")
            print(f"    V2 Hot: {v2_hot_rate:.1f} files/sec")
            print(f"    Cache Improvement: {v2_hot_rate/v2_cold_rate:.1f}x faster")
        
        # Memory comparison
        if v1["memory_delta"] > 0 and v2_cold["memory_delta"] > 0:
            memory_improvement = v1["memory_delta"] / v2_cold["memory_delta"]
            comparisons["memory_improvement"] = {
                "v1_memory": v1["memory_delta"],
                "v2_cold_memory": v2_cold["memory_delta"],
                "v2_hot_memory": v2_hot["memory_delta"],
                "improvement": memory_improvement,
                "improvement_pct": f"{((memory_improvement - 1) * 100):.1f}%"
            }
            
            print(f"  💾 Memory Usage:")
            print(f"    V1: {v1['memory_delta']:.2f}MB")
            print(f"    V2 Cold: {v2_cold['memory_delta']:.2f}MB")
            print(f"    V2 Hot: {v2_hot['memory_delta']:.2f}MB")
        
        self.results["comparison"] = comparisons
    
    def save_results(self, filename: str = "large_scale_benchmark_results.json"):
        """Save benchmark results to JSON file"""
        output_path = Path(__file__).parent / filename
        
        with open(output_path, 'w') as f:
            json.dump(self.results, f, indent=2, default=str)
        
        print(f"\n💾 Results saved to: {output_path}")
    
    def print_summary(self):
        """Print benchmark summary"""
        print("\n" + "=" * 70)
        print("📊 LARGE-SCALE BENCHMARK SUMMARY")
        print("=" * 70)
        
        v1 = self.results["v1_baseline"]
        v2_cold = self.results["v2_optimized"]
        v2_hot = self.results["v2_hot_cache"]
        comparison = self.results["comparison"]
        
        print(f"\n📁 Files Processed: {v1['total_files']}")
        print(f"✅ V1 Success Rate: {v1['successful_files']}/{v1['total_files']} ({v1['successful_files']/v1['total_files']*100:.1f}%)")
        print(f"✅ V2 Cold Success Rate: {v2_cold['successful_files']}/{v2_cold['total_files']} ({v2_cold['successful_files']/v2_cold['total_files']*100:.1f}%)")
        print(f"✅ V2 Hot Success Rate: {v2_hot['successful_files']}/{v2_hot['total_files']} ({v2_hot['successful_files']/v2_hot['total_files']*100:.1f}%)")
        
        print(f"\n⏱️  PERFORMANCE COMPARISON:")
        print(f"  V1 Total Time: {v1['total_elapsed_time']:.2f}s")
        print(f"  V2 Cold Total Time: {v2_cold['total_elapsed_time']:.2f}s")
        print(f"  V2 Hot Total Time: {v2_hot['total_elapsed_time']:.2f}s")
        
        if "v1_vs_v2_cold" in comparison:
            cold_speedup = comparison["v1_vs_v2_cold"]
            print(f"  ⚡ V1 vs V2 (Cold): {cold_speedup['speedup']:.1f}x faster ({cold_speedup['improvement']})")
        
        if "v1_vs_v2_hot" in comparison:
            hot_speedup = comparison["v1_vs_v2_hot"]
            print(f"  🔥 V1 vs V2 (Hot): {hot_speedup['speedup']:.1f}x faster ({hot_speedup['improvement']})")
        
        if "v2_cold_vs_hot" in comparison:
            cache_speedup = comparison["v2_cold_vs_hot"]
            print(f"  💾 Cache Effect: {cache_speedup['speedup']:.1f}x faster ({cache_speedup['improvement']})")
        
        if "rate_comparison" in comparison:
            rate = comparison["rate_comparison"]
            print(f"\n📊 PROCESSING RATES:")
            print(f"  V1: {rate['v1_rate']:.1f} files/sec")
            print(f"  V2 Cold: {rate['v2_cold_rate']:.1f} files/sec")
            print(f"  V2 Hot: {rate['v2_hot_rate']:.1f} files/sec")
            print(f"  Cache Improvement: {rate['cache_improvement']:.1f}x faster")
        
        print(f"\n💾 MEMORY USAGE:")
        print(f"  V1 Memory Delta: {v1['memory_delta']:.2f}MB")
        print(f"  V2 Cold Memory Delta: {v2_cold['memory_delta']:.2f}MB")
        print(f"  V2 Hot Memory Delta: {v2_hot['memory_delta']:.2f}MB")
        
        if "memory_improvement" in comparison:
            memory = comparison["memory_improvement"]
            print(f"  💾 Memory Improvement: {memory['improvement']:.1f}x better")
        
        print(f"\n📊 FIELD EXTRACTION:")
        print(f"  V1 Total Fields: {v1.get('total_fields', 0)}")
        print(f"  V2 Cold Total Fields: {v2_cold.get('total_fields', 0)}")
        print(f"  V2 Hot Total Fields: {v2_hot.get('total_fields', 0)}")
        print(f"  Average Fields per File: {v1.get('avg_fields', 0):.1f}")
        
        print(f"\n🏆 PERFORMANCE HIGHLIGHTS:")
        if "v1_vs_v2_hot" in comparison:
            hot_speedup = comparison["v1_vs_v2_hot"]["speedup"]
            print(f"  🚀 Maximum Speedup: {hot_speedup:.1f}x faster (V2 with hot cache)")
        if "v2_cold_vs_hot" in comparison:
            cache_speedup = comparison["v2_cold_vs_hot"]["speedup"]
            print(f"  💾 Cache Benefit: {cache_speedup:.1f}x faster (hot vs cold cache)")
        if "rate_comparison" in comparison:
            hot_rate = comparison["rate_comparison"]["v2_hot_rate"]
            print(f"  ⚡ Peak Performance: {hot_rate:.1f} files/sec (V2 with hot cache)")

def main():
    parser = argparse.ArgumentParser(description="Large-Scale Fast-EXIF-RS Benchmark")
    parser.add_argument("--target-dir", default="/keg/pictures/2025", help="Target directory to benchmark")
    parser.add_argument("--max-files", type=int, help="Maximum number of files to process")
    parser.add_argument("--output", default="large_scale_benchmark_results.json", help="Output file for results")
    
    args = parser.parse_args()
    
    try:
        benchmark = LargeScaleBenchmark(args.target_dir)
        
        if args.max_files:
            print(f"🎯 Limiting benchmark to {args.max_files} files")
        
        results = benchmark.run_comprehensive_benchmark(args.max_files)
        benchmark.save_results(args.output)
        benchmark.print_summary()
        
    except Exception as e:
        print(f"❌ Benchmark failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
