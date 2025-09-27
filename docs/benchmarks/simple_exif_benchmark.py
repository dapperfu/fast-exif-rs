#!/usr/bin/env python3
"""
Simple benchmark script comparing fast-exif-rs EXIF writing performance against exiftool.
Uses existing test images from the test_files directory.
"""

import os
import sys
import time
import tempfile
import subprocess
import shutil
from pathlib import Path
import json
from typing import Dict, List, Tuple, Any
import statistics

# Add the project root to Python path
sys.path.insert(0, str(Path(__file__).parent))

try:
    import fast_exif_reader
    from fast_exif_reader import BatchExifWriter, rust_write_exif_batch_parallel
except ImportError:
    print("Error: fast_exif_reader not found. Make sure the Rust module is built.")
    print("Try running: source venv/bin/activate && maturin develop")
    sys.exit(1)

class SimpleExifBenchmark:
    def __init__(self):
        self.temp_dir = None
        self.results = {
            "fast_exif_rs": {},
            "exiftool": {},
            "comparison": {}
        }
        
    def setup_temp_environment(self):
        """Create temporary directory for test images"""
        self.temp_dir = tempfile.mkdtemp(prefix="exif_benchmark_")
        print(f"Created temporary directory: {self.temp_dir}")
        
    def cleanup_temp_environment(self):
        """Clean up temporary files"""
        if self.temp_dir and os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
            print(f"Cleaned up temporary directory: {self.temp_dir}")
            
    def get_test_images(self, count: int = 10) -> List[str]:
        """Get test images from test_files directory"""
        test_files_dir = Path(__file__).parent / "test_files" / "essential"
        
        if not test_files_dir.exists():
            print(f"Error: Test files directory not found: {test_files_dir}")
            return []
            
        # Get all image files
        image_extensions = ['.jpg', '.jpeg', '.cr2', '.heic', '.mp4']
        test_images = []
        
        for ext in image_extensions:
            images = list(test_files_dir.glob(f"*{ext}"))
            test_images.extend(images)
            
        # Limit to requested count
        test_images = test_images[:count]
        
        if not test_images:
            print("Error: No test images found")
            return []
            
        print(f"Found {len(test_images)} test images")
        return [str(img) for img in test_images]
        
    def prepare_exif_metadata(self) -> Dict[str, str]:
        """Prepare comprehensive EXIF metadata for writing tests"""
        return {
            # Camera settings
            "Make": "Benchmark Camera",
            "Model": "Test Model Pro",
            "DateTime": "2024:09:27 16:30:00",
            "DateTimeOriginal": "2024:09:27 16:30:00",
            "DateTimeDigitized": "2024:09:27 16:30:00",
            
            # Exposure settings
            "ExposureTime": "1/125",
            "FNumber": "2.8",
            "ISO": "400",
            "ExposureProgram": "Aperture Priority",
            "ExposureMode": "Auto",
            "ExposureBiasValue": "0/1",
            
            # Lens settings
            "FocalLength": "50mm",
            "FocalLengthIn35mmFilm": "75",
            "MaxApertureValue": "2.8",
            
            # Image settings
            "WhiteBalance": "Auto",
            "Flash": "No Flash",
            "MeteringMode": "Center-weighted average",
            "LightSource": "Unknown",
            "ColorSpace": "sRGB",
            
            # Technical
            "Software": "fast-exif-rs benchmark",
            "Artist": "Benchmark Test",
            "Copyright": "Test Copyright 2024",
        }
        
    def benchmark_fast_exif_rs_single(self, input_path: str, output_path: str, metadata: Dict[str, str]) -> Tuple[float, bool]:
        """Benchmark single file EXIF writing with fast-exif-rs"""
        try:
            start_time = time.time()
            
            # Use the writer directly
            writer = fast_exif_reader.FastExifWriter()
            result = writer.write_exif(input_path, output_path, metadata)
            
            end_time = time.time()
            processing_time = end_time - start_time
            
            # write_exif returns None on success, raises exception on failure
            success = result is None
            
            return processing_time, success
            
        except Exception as e:
            print(f"Error in fast-exif-rs single write: {e}")
            return 0.0, False
            
    def benchmark_fast_exif_rs_batch(self, operations: List[Dict]) -> Tuple[float, Dict]:
        """Benchmark batch EXIF writing with fast-exif-rs"""
        try:
            start_time = time.time()
            
            # Convert operations to the format expected by batch writer
            batch_ops = []
            for op in operations:
                batch_ops.append({
                    "input_path": op["input_path"],
                    "output_path": op["output_path"], 
                    "metadata": op["metadata"]
                })
            
            # Use batch writer
            results = rust_write_exif_batch_parallel(batch_ops, max_workers=None)
            
            end_time = time.time()
            processing_time = end_time - start_time
            
            return processing_time, results
            
        except Exception as e:
            print(f"Error in fast-exif-rs batch write: {e}")
            return 0.0, {}
            
    def benchmark_exiftool_single(self, input_path: str, output_path: str, metadata: Dict[str, str]) -> Tuple[float, bool]:
        """Benchmark single file EXIF writing with exiftool"""
        try:
            # Copy input to output first
            shutil.copy2(input_path, output_path)
            
            start_time = time.time()
            
            # Build exiftool command
            cmd = ["exiftool", "-overwrite_original"]
            
            # Add metadata tags
            for key, value in metadata.items():
                cmd.extend([f"-{key}={value}"])
            
            cmd.append(output_path)
            
            # Run exiftool
            result = subprocess.run(cmd, capture_output=True, text=True)
            
            end_time = time.time()
            processing_time = end_time - start_time
            
            success = result.returncode == 0
            
            return processing_time, success
            
        except Exception as e:
            print(f"Error in exiftool single write: {e}")
            return 0.0, False
            
    def benchmark_exiftool_batch(self, operations: List[Dict]) -> Tuple[float, List[bool]]:
        """Benchmark batch EXIF writing with exiftool"""
        try:
            start_time = time.time()
            
            # Copy all input files to output files first
            for op in operations:
                shutil.copy2(op["input_path"], op["output_path"])
            
            # Build exiftool command for batch processing
            cmd = ["exiftool", "-overwrite_original"]
            
            # Add metadata tags (same for all files)
            metadata = operations[0]["metadata"]
            for key, value in metadata.items():
                cmd.extend([f"-{key}={value}"])
            
            # Add all output files
            for op in operations:
                cmd.append(op["output_path"])
            
            # Run exiftool
            result = subprocess.run(cmd, capture_output=True, text=True)
            
            end_time = time.time()
            processing_time = end_time - start_time
            
            success = result.returncode == 0
            results = [success] * len(operations)
            
            return processing_time, results
            
        except Exception as e:
            print(f"Error in exiftool batch write: {e}")
            return 0.0, [False] * len(operations)
            
    def run_benchmark(self):
        """Run benchmark comparing both tools"""
        print("🚀 Starting EXIF Writing Benchmark")
        print("=" * 50)
        
        self.setup_temp_environment()
        
        try:
            # Get available test images
            test_images = self.get_test_images(20)  # Use up to 20 images
            
            if not test_images:
                print("No test images available. Exiting.")
                return
                
            metadata = self.prepare_exif_metadata()
            
            # Test scenarios based on available images
            test_scenarios = [
                {"name": "Small Batch", "count": min(5, len(test_images))},
                {"name": "Medium Batch", "count": min(10, len(test_images))},
                {"name": "Large Batch", "count": min(20, len(test_images))},
            ]
            
            for scenario in test_scenarios:
                if scenario['count'] == 0:
                    continue
                    
                print(f"\n📊 Testing {scenario['name']} ({scenario['count']} files)")
                print("-" * 40)
                
                # Use subset of test images
                scenario_images = test_images[:scenario['count']]
                
                # Prepare operations for fast-exif-rs
                fast_operations = []
                for i, input_path in enumerate(scenario_images):
                    output_path = os.path.join(self.temp_dir, f"output_fast_{i:03d}.jpg")
                    fast_operations.append({
                        "input_path": input_path,
                        "output_path": output_path,
                        "metadata": metadata
                    })
                
                # Prepare operations for exiftool
                exif_operations = []
                for i, input_path in enumerate(scenario_images):
                    output_path = os.path.join(self.temp_dir, f"output_exif_{i:03d}.jpg")
                    exif_operations.append({
                        "input_path": input_path,
                        "output_path": output_path,
                        "metadata": metadata
                    })
                
                # Benchmark fast-exif-rs
                print("Testing fast-exif-rs...")
                fast_times = []
                fast_successes = []
                
                # Single file tests (first 3 files)
                for i, op in enumerate(fast_operations[:3]):
                    time_taken, success = self.benchmark_fast_exif_rs_single(
                        op["input_path"], op["output_path"], op["metadata"]
                    )
                    fast_times.append(time_taken)
                    fast_successes.append(success)
                
                # Batch test
                batch_time, batch_results = self.benchmark_fast_exif_rs_batch(fast_operations)
                
                # Benchmark exiftool
                print("Testing exiftool...")
                exif_times = []
                exif_successes = []
                
                # Single file tests (first 3 files)
                for i, op in enumerate(exif_operations[:3]):
                    time_taken, success = self.benchmark_exiftool_single(
                        op["input_path"], op["output_path"], op["metadata"]
                    )
                    exif_times.append(time_taken)
                    exif_successes.append(success)
                
                # Batch test
                exif_batch_time, exif_batch_results = self.benchmark_exiftool_batch(exif_operations)
                
                # Calculate statistics
                fast_avg_single = statistics.mean(fast_times) if fast_times else 0
                exif_avg_single = statistics.mean(exif_times) if exif_times else 0
                
                fast_success_rate = sum(fast_successes) / len(fast_successes) if fast_successes else 0
                exif_success_rate = sum(exif_successes) / len(exif_successes) if exif_successes else 0
                
                # Store results
                scenario_name = scenario['name'].lower().replace(' ', '_')
                self.results["fast_exif_rs"][scenario_name] = {
                    "files": scenario['count'],
                    "single_file_avg_time": fast_avg_single,
                    "batch_time": batch_time,
                    "single_file_success_rate": fast_success_rate,
                    "batch_success": batch_results.get("stats", {}).get("success_rate", 0) / 100 if batch_results else 0,
                    "files_per_second": scenario['count'] / batch_time if batch_time > 0 else 0
                }
                
                self.results["exiftool"][scenario_name] = {
                    "files": scenario['count'],
                    "single_file_avg_time": exif_avg_single,
                    "batch_time": exif_batch_time,
                    "single_file_success_rate": exif_success_rate,
                    "batch_success": sum(exif_batch_results) / len(exif_batch_results) if exif_batch_results else 0,
                    "files_per_second": scenario['count'] / exif_batch_time if exif_batch_time > 0 else 0
                }
                
                # Calculate comparison metrics
                speedup_single = exif_avg_single / fast_avg_single if fast_avg_single > 0 else 0
                speedup_batch = exif_batch_time / batch_time if batch_time > 0 else 0
                
                self.results["comparison"][scenario_name] = {
                    "single_file_speedup": speedup_single,
                    "batch_speedup": speedup_batch,
                    "fast_exif_rs_faster": speedup_single > 1.0 or speedup_batch > 1.0
                }
                
                # Print results
                print(f"\n📈 Results for {scenario['name']}:")
                print(f"  fast-exif-rs:")
                print(f"    Single file avg: {fast_avg_single:.4f}s")
                print(f"    Batch time: {batch_time:.4f}s")
                print(f"    Files/sec: {scenario['count'] / batch_time:.2f}" if batch_time > 0 else "    Files/sec: N/A")
                print(f"    Success rate: {fast_success_rate:.2%}")
                
                print(f"  exiftool:")
                print(f"    Single file avg: {exif_avg_single:.4f}s")
                print(f"    Batch time: {exif_batch_time:.4f}s")
                print(f"    Files/sec: {scenario['count'] / exif_batch_time:.2f}" if exif_batch_time > 0 else "    Files/sec: N/A")
                print(f"    Success rate: {exif_success_rate:.2%}")
                
                print(f"  Performance:")
                print(f"    Single file speedup: {speedup_single:.2f}x")
                print(f"    Batch speedup: {speedup_batch:.2f}x")
                print(f"    Winner: {'fast-exif-rs' if speedup_single > 1.0 or speedup_batch > 1.0 else 'exiftool'}")
                
        finally:
            self.cleanup_temp_environment()
            
    def save_results(self, filename: str = "exif_writing_benchmark_results.json"):
        """Save benchmark results to JSON file"""
        results_path = Path(__file__).parent / filename
        
        # Add metadata
        self.results["metadata"] = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "python_version": sys.version,
            "fast_exif_rs_version": getattr(fast_exif_reader, '__version__', 'unknown'),
            "benchmark_type": "EXIF Writing Performance",
            "test_environment": "Existing Test Images"
        }
        
        with open(results_path, 'w') as f:
            json.dump(self.results, f, indent=2)
            
        print(f"\n💾 Results saved to: {results_path}")
        
    def print_summary(self):
        """Print benchmark summary"""
        print("\n" + "=" * 50)
        print("📊 BENCHMARK SUMMARY")
        print("=" * 50)
        
        total_fast_wins = 0
        total_exif_wins = 0
        
        for scenario, comparison in self.results["comparison"].items():
            if comparison["fast_exif_rs_faster"]:
                total_fast_wins += 1
            else:
                total_exif_wins += 1
                
        print(f"fast-exif-rs wins: {total_fast_wins}")
        print(f"exiftool wins: {total_exif_wins}")
        
        # Calculate overall performance
        all_speedups = []
        for comparison in self.results["comparison"].values():
            all_speedups.extend([comparison["single_file_speedup"], comparison["batch_speedup"]])
            
        avg_speedup = statistics.mean(all_speedups) if all_speedups else 0
        print(f"Average speedup: {avg_speedup:.2f}x")
        
        if avg_speedup > 1.0:
            print("🏆 fast-exif-rs is faster overall!")
        elif avg_speedup < 1.0:
            print("🏆 exiftool is faster overall!")
        else:
            print("🤝 Performance is roughly equivalent!")

def main():
    """Main benchmark execution"""
    print("🔬 EXIF Writing Performance Benchmark")
    print("Comparing fast-exif-rs vs exiftool")
    print("Using existing test images")
    print()
    
    # Check if exiftool is available
    try:
        subprocess.run(["exiftool", "-ver"], check=True, capture_output=True)
        print("✅ exiftool found")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("❌ exiftool not found. Please install exiftool to run this benchmark.")
        return
        
    # Check if fast_exif_reader is available
    try:
        import fast_exif_reader
        print("✅ fast_exif_reader found")
    except ImportError:
        print("❌ fast_exif_reader not found. Please build the Rust module first.")
        return
        
    # Run benchmark
    benchmark = SimpleExifBenchmark()
    benchmark.run_benchmark()
    benchmark.save_results()
    benchmark.print_summary()

if __name__ == "__main__":
    main()
