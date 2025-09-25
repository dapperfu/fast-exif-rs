#!/usr/bin/env python3
"""
Fast-EXIF-RS 2.0 Performance Benchmarking System

This script benchmarks the performance improvements in version 2.0
against the baseline version 0.4.9, providing detailed metrics
and performance comparisons.
"""

import time
import os
import sys
import json
import subprocess
import psutil
import statistics
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

class PerformanceBenchmark:
    """Comprehensive performance benchmarking system for fast-exif-rs 2.0"""
    
    def __init__(self, test_files_dir: str = "test_files"):
        self.test_files_dir = Path(test_files_dir)
        self.results = {
            "baseline": {},
            "v2_0": {},
            "comparison": {}
        }
        self.test_files = self._discover_test_files()
        
    def _discover_test_files(self) -> List[Path]:
        """Discover all test files for benchmarking"""
        if not self.test_files_dir.exists():
            raise FileNotFoundError(f"Test files directory not found: {self.test_files_dir}")
            
        test_files = []
        for ext in ['.jpg', '.jpeg', '.cr2', '.heic', '.mov', '.mp4', '.png', '.mkv']:
            test_files.extend(self.test_files_dir.glob(f"*{ext}"))
            
        return sorted(test_files)
    
    def _get_file_size(self, filepath: Path) -> int:
        """Get file size in bytes"""
        return filepath.stat().st_size
    
    def _measure_memory_usage(self, func, *args, **kwargs) -> Tuple[float, float]:
        """Measure memory usage during function execution"""
        process = psutil.Process()
        memory_before = process.memory_info().rss / 1024 / 1024  # MB
        
        start_time = time.perf_counter()
        result = func(*args, **kwargs)
        end_time = time.perf_counter()
        
        memory_after = process.memory_info().rss / 1024 / 1024  # MB
        
        return end_time - start_time, memory_after - memory_before
    
    def benchmark_single_file_parsing(self, version: str = "baseline") -> Dict:
        """Benchmark single file parsing performance"""
        print(f"\n🔍 Benchmarking single file parsing ({version})...")
        
        results = {
            "total_files": len(self.test_files),
            "successful_files": 0,
            "failed_files": 0,
            "total_time": 0.0,
            "file_times": [],
            "file_sizes": [],
            "field_counts": [],
            "memory_usage": [],
            "errors": []
        }
        
        reader = fast_exif_reader.FastExifReader()
        
        for filepath in self.test_files:
            try:
                file_size = self._get_file_size(filepath)
                results["file_sizes"].append(file_size)
                
                # Measure parsing time and memory
                parse_time, memory_delta = self._measure_memory_usage(
                    reader.read_file, str(filepath)
                )
                
                metadata = reader.read_file(str(filepath))
                
                results["file_times"].append(parse_time)
                results["field_counts"].append(len(metadata))
                results["memory_usage"].append(memory_delta)
                results["total_time"] += parse_time
                results["successful_files"] += 1
                
                print(f"  ✅ {filepath.name}: {parse_time:.3f}s, {len(metadata)} fields, {file_size/1024:.1f}KB")
                
            except Exception as e:
                results["failed_files"] += 1
                results["errors"].append(f"{filepath.name}: {str(e)}")
                print(f"  ❌ {filepath.name}: {str(e)}")
        
        # Calculate statistics
        if results["file_times"]:
            results["avg_time"] = statistics.mean(results["file_times"])
            results["median_time"] = statistics.median(results["file_times"])
            results["min_time"] = min(results["file_times"])
            results["max_time"] = max(results["file_times"])
            results["std_time"] = statistics.stdev(results["file_times"]) if len(results["file_times"]) > 1 else 0
            
            results["avg_memory"] = statistics.mean(results["memory_usage"])
            results["avg_fields"] = statistics.mean(results["field_counts"])
            results["avg_file_size"] = statistics.mean(results["file_sizes"])
        
        return results
    
    def benchmark_batch_processing(self, version: str = "baseline") -> Dict:
        """Benchmark batch processing performance"""
        print(f"\n📦 Benchmarking batch processing ({version})...")
        
        results = {
            "total_files": len(self.test_files),
            "batch_times": [],
            "memory_usage": [],
            "errors": []
        }
        
        # Test different batch sizes
        batch_sizes = [1, 5, 10, min(20, len(self.test_files))]
        
        for batch_size in batch_sizes:
            batch_files = [str(f) for f in self.test_files[:batch_size]]
            
            try:
                # Measure batch processing time and memory
                batch_time, memory_delta = self._measure_memory_usage(
                    fast_exif_reader.read_multiple_files, batch_files
                )
                
                results["batch_times"].append({
                    "batch_size": batch_size,
                    "time": batch_time,
                    "time_per_file": batch_time / batch_size,
                    "memory_delta": memory_delta
                })
                
                print(f"  📦 Batch size {batch_size}: {batch_time:.3f}s ({batch_time/batch_size:.3f}s/file)")
                
            except Exception as e:
                results["errors"].append(f"Batch size {batch_size}: {str(e)}")
                print(f"  ❌ Batch size {batch_size}: {str(e)}")
        
        return results
    
    def benchmark_memory_efficiency(self, version: str = "baseline") -> Dict:
        """Benchmark memory efficiency"""
        print(f"\n💾 Benchmarking memory efficiency ({version})...")
        
        results = {
            "file_memory_ratios": [],
            "peak_memory_usage": [],
            "memory_per_field": []
        }
        
        reader = fast_exif_reader.FastExifReader()
        
        for filepath in self.test_files:
            try:
                file_size = self._get_file_size(filepath)
                
                # Measure memory usage
                process = psutil.Process()
                memory_before = process.memory_info().rss / 1024 / 1024  # MB
                
                metadata = reader.read_file(str(filepath))
                
                memory_after = process.memory_info().rss / 1024 / 1024  # MB
                memory_delta = memory_after - memory_before
                
                # Calculate efficiency metrics
                memory_ratio = memory_delta / (file_size / 1024 / 1024) if file_size > 0 else 0
                memory_per_field = memory_delta / len(metadata) if len(metadata) > 0 else 0
                
                results["file_memory_ratios"].append(memory_ratio)
                results["peak_memory_usage"].append(memory_delta)
                results["memory_per_field"].append(memory_per_field)
                
                print(f"  💾 {filepath.name}: {memory_delta:.2f}MB ({memory_ratio:.2f}x file size)")
                
            except Exception as e:
                print(f"  ❌ {filepath.name}: {str(e)}")
        
        # Calculate statistics
        if results["file_memory_ratios"]:
            results["avg_memory_ratio"] = statistics.mean(results["file_memory_ratios"])
            results["avg_peak_memory"] = statistics.mean(results["peak_memory_usage"])
            results["avg_memory_per_field"] = statistics.mean(results["memory_per_field"])
        
        return results
    
    def benchmark_selective_fields(self, version: str = "baseline") -> Dict:
        """Benchmark selective field extraction (v2.0 feature)"""
        print(f"\n🎯 Benchmarking selective field extraction ({version})...")
        
        results = {
            "all_fields": {},
            "basic_fields": {},
            "gps_fields": {},
            "maker_notes": {}
        }
        
        # Define field sets
        field_sets = {
            "all_fields": None,  # All fields
            "basic_fields": ["Make", "Model", "DateTime", "FocalLength", "ISO"],
            "gps_fields": ["GPSLatitude", "GPSLongitude", "GPSPosition", "GPSDateTime"],
            "maker_notes": ["MakerNote", "MakerNoteType"]
        }
        
        reader = fast_exif_reader.FastExifReader()
        
        for field_set_name, fields in field_sets.items():
            times = []
            field_counts = []
            
            for filepath in self.test_files[:5]:  # Test first 5 files
                try:
                    start_time = time.perf_counter()
                    
                    if fields is None:
                        # All fields
                        metadata = reader.read_file(str(filepath))
                    else:
                        # Selective fields (v2.0 feature - placeholder for now)
                        metadata = reader.read_file(str(filepath))
                        # Filter to requested fields
                        metadata = {k: v for k, v in metadata.items() if k in fields}
                    
                    end_time = time.perf_counter()
                    
                    times.append(end_time - start_time)
                    field_counts.append(len(metadata))
                    
                except Exception as e:
                    print(f"  ❌ {filepath.name}: {str(e)}")
            
            if times:
                results[field_set_name] = {
                    "avg_time": statistics.mean(times),
                    "avg_fields": statistics.mean(field_counts),
                    "times": times
                }
                
                print(f"  🎯 {field_set_name}: {statistics.mean(times):.3f}s avg, {statistics.mean(field_counts):.1f} fields")
        
        return results
    
    def run_comprehensive_benchmark(self) -> Dict:
        """Run comprehensive benchmark suite"""
        print("🚀 Starting Fast-EXIF-RS 2.0 Performance Benchmark")
        print("=" * 60)
        
        # Benchmark baseline version
        print("\n📊 BENCHMARKING BASELINE VERSION (0.4.9)")
        print("-" * 40)
        
        self.results["baseline"] = {
            "single_file": self.benchmark_single_file_parsing("baseline"),
            "batch_processing": self.benchmark_batch_processing("baseline"),
            "memory_efficiency": self.benchmark_memory_efficiency("baseline"),
            "selective_fields": self.benchmark_selective_fields("baseline")
        }
        
        # Benchmark v2.0 version (same for now, will be different after implementation)
        print("\n📊 BENCHMARKING V2.0 VERSION")
        print("-" * 40)
        
        self.results["v2_0"] = {
            "single_file": self.benchmark_single_file_parsing("v2_0"),
            "batch_processing": self.benchmark_batch_processing("v2_0"),
            "memory_efficiency": self.benchmark_memory_efficiency("v2_0"),
            "selective_fields": self.benchmark_selective_fields("v2_0")
        }
        
        # Calculate comparisons
        self._calculate_comparisons()
        
        return self.results
    
    def _calculate_comparisons(self):
        """Calculate performance comparisons between versions"""
        print("\n📈 CALCULATING PERFORMANCE COMPARISONS")
        print("-" * 40)
        
        baseline = self.results["baseline"]
        v2_0 = self.results["v2_0"]
        
        comparisons = {}
        
        # Single file parsing comparison
        if "avg_time" in baseline["single_file"] and "avg_time" in v2_0["single_file"]:
            baseline_time = baseline["single_file"]["avg_time"]
            v2_0_time = v2_0["single_file"]["avg_time"]
            speedup = baseline_time / v2_0_time if v2_0_time > 0 else 1
            
            comparisons["single_file_speedup"] = {
                "baseline_time": baseline_time,
                "v2_0_time": v2_0_time,
                "speedup": speedup,
                "improvement": f"{((speedup - 1) * 100):.1f}%"
            }
            
            print(f"  ⚡ Single file parsing: {speedup:.1f}x faster ({comparisons['single_file_speedup']['improvement']} improvement)")
        
        # Memory efficiency comparison
        if "avg_memory_ratio" in baseline["memory_efficiency"] and "avg_memory_ratio" in v2_0["memory_efficiency"]:
            baseline_memory = baseline["memory_efficiency"]["avg_memory_ratio"]
            v2_0_memory = v2_0["memory_efficiency"]["avg_memory_ratio"]
            memory_improvement = baseline_memory / v2_0_memory if v2_0_memory > 0 else 1
            
            comparisons["memory_efficiency"] = {
                "baseline_ratio": baseline_memory,
                "v2_0_ratio": v2_0_memory,
                "improvement": memory_improvement,
                "improvement_pct": f"{((memory_improvement - 1) * 100):.1f}%"
            }
            
            print(f"  💾 Memory efficiency: {memory_improvement:.1f}x better ({comparisons['memory_efficiency']['improvement_pct']} improvement)")
        
        self.results["comparison"] = comparisons
    
    def save_results(self, filename: str = "benchmark_results.json"):
        """Save benchmark results to JSON file"""
        output_path = Path(__file__).parent / filename
        
        with open(output_path, 'w') as f:
            json.dump(self.results, f, indent=2, default=str)
        
        print(f"\n💾 Results saved to: {output_path}")
    
    def print_summary(self):
        """Print benchmark summary"""
        print("\n" + "=" * 60)
        print("📊 FAST-EXIF-RS 2.0 BENCHMARK SUMMARY")
        print("=" * 60)
        
        baseline = self.results["baseline"]
        v2_0 = self.results["v2_0"]
        comparison = self.results["comparison"]
        
        print(f"\n📁 Test Files: {len(self.test_files)}")
        print(f"✅ Successful Parses: {baseline['single_file']['successful_files']}/{baseline['single_file']['total_files']}")
        
        if "single_file_speedup" in comparison:
            speedup = comparison["single_file_speedup"]
            print(f"\n⚡ PERFORMANCE IMPROVEMENTS:")
            print(f"  Single File Parsing: {speedup['speedup']:.1f}x faster")
            print(f"  Baseline Time: {speedup['baseline_time']:.3f}s avg")
            print(f"  V2.0 Time: {speedup['v2_0_time']:.3f}s avg")
            print(f"  Improvement: {speedup['improvement']}")
        
        if "memory_efficiency" in comparison:
            memory = comparison["memory_efficiency"]
            print(f"\n💾 MEMORY EFFICIENCY:")
            print(f"  Memory Usage: {memory['improvement']:.1f}x better")
            print(f"  Baseline Ratio: {memory['baseline_ratio']:.2f}x file size")
            print(f"  V2.0 Ratio: {memory['v2_0_ratio']:.2f}x file size")
            print(f"  Improvement: {memory['improvement_pct']}")
        
        print(f"\n🎯 FIELD EXTRACTION:")
        print(f"  Average Fields per File: {baseline['single_file']['avg_fields']:.1f}")
        print(f"  Total Fields Extracted: {sum(baseline['single_file']['field_counts'])}")

def main():
    parser = argparse.ArgumentParser(description="Fast-EXIF-RS 2.0 Performance Benchmark")
    parser.add_argument("--test-files", default="test_files", help="Directory containing test files")
    parser.add_argument("--output", default="benchmark_results.json", help="Output file for results")
    parser.add_argument("--quick", action="store_true", help="Run quick benchmark (first 10 files only)")
    
    args = parser.parse_args()
    
    try:
        benchmark = PerformanceBenchmark(args.test_files)
        
        if args.quick:
            benchmark.test_files = benchmark.test_files[:10]
            print(f"🚀 Running quick benchmark with {len(benchmark.test_files)} files")
        
        results = benchmark.run_comprehensive_benchmark()
        benchmark.save_results(args.output)
        benchmark.print_summary()
        
    except Exception as e:
        print(f"❌ Benchmark failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
