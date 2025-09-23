#!/usr/bin/env python3
"""
Comprehensive EXIF Benchmark

Benchmarks ExifTool, PIL/Pillow, and Fast EXIF Reader
with parallel processing and detailed statistics.

Usage:
    python comprehensive_benchmark.py [directory] [--file-counts 10 100 1000] [--runs 3]
"""

import os
import sys
import time
import subprocess
import multiprocessing as mp
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path
import statistics
import argparse
import json
from typing import List, Tuple, Dict, Any
from datetime import datetime

# Add the parent directory to path to import our module
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from fast_exif_reader import FastExifReader
    FAST_EXIF_AVAILABLE = True
except ImportError:
    print("Warning: fast_exif_reader not available")
    FAST_EXIF_AVAILABLE = False

try:
    from PIL import Image
    from PIL.ExifTags import TAGS
    PIL_AVAILABLE = True
except ImportError:
    print("Warning: PIL/Pillow not available")
    PIL_AVAILABLE = False
    TAGS = {}


class BenchmarkResults:
    """Container for benchmark results with detailed statistics"""
    
    def __init__(self):
        self.results = {}
        self.file_counts = []
        self.total_files = 0
        self.format_distribution = {}
        self.timestamp = datetime.now().isoformat()
    
    def add_result(self, method: str, file_count: int, times: List[float], 
                   success_count: int, error_count: int, total_fields: int = 0):
        if method not in self.results:
            self.results[method] = {}
        
        if times:
            self.results[method][file_count] = {
                'times': times,
                'total_time': sum(times),
                'avg_time': statistics.mean(times),
                'median_time': statistics.median(times),
                'min_time': min(times),
                'max_time': max(times),
                'std_dev': statistics.stdev(times) if len(times) > 1 else 0,
                'success_count': success_count,
                'error_count': error_count,
                'success_rate': success_count / (success_count + error_count) * 100,
                'files_per_second': (success_count + error_count) / sum(times),
                'total_fields': total_fields,
                'avg_fields_per_file': total_fields / success_count if success_count > 0 else 0
            }
        else:
            self.results[method][file_count] = {
                'times': [],
                'total_time': 0,
                'avg_time': 0,
                'median_time': 0,
                'min_time': 0,
                'max_time': 0,
                'std_dev': 0,
                'success_count': 0,
                'error_count': error_count,
                'success_rate': 0,
                'files_per_second': 0,
                'total_fields': 0,
                'avg_fields_per_file': 0
            }
    
    def print_summary(self):
        """Print a comprehensive summary of results"""
        print("\n" + "="*80)
        print("COMPREHENSIVE BENCHMARK SUMMARY")
        print("="*80)
        print(f"Timestamp: {self.timestamp}")
        print(f"Total files scanned: {self.total_files}")
        print(f"File format distribution: {self.format_distribution}")
        
        for file_count in sorted(self.file_counts):
            print(f"\n{'='*60}")
            print(f"RESULTS FOR {file_count} FILES")
            print(f"{'='*60}")
            
            # Get results for this file count
            method_results = []
            for method, data in self.results.items():
                if file_count in data:
                    method_results.append((method, data[file_count]))
            
            # Sort by files per second (descending)
            method_results.sort(key=lambda x: x[1]['files_per_second'], reverse=True)
            
            print(f"{'Method':<20} {'Total(s)':<8} {'Avg(s)':<8} {'Files/s':<8} {'Success%':<8} {'Fields':<8} {'Speedup':<8}")
            print("-" * 80)
            
            fastest_time = method_results[0][1]['total_time'] if method_results else 0
            
            for method, data in method_results:
                speedup = fastest_time / data['total_time'] if data['total_time'] > 0 else 0
                print(f"{method:<20} {data['total_time']:<8.2f} {data['avg_time']:<8.3f} "
                      f"{data['files_per_second']:<8.1f} {data['success_rate']:<8.1f} "
                      f"{data['total_fields']:<8} {speedup:<8.2f}x")
    
    def save_json(self, filename: str):
        """Save results to JSON file"""
        output = {
            'timestamp': self.timestamp,
            'total_files': self.total_files,
            'format_distribution': self.format_distribution,
            'file_counts': self.file_counts,
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


def test_exiftool(file_path: str) -> Tuple[bool, float, int, Dict[str, str]]:
    """Test ExifTool on a single file"""
    start_time = time.time()
    try:
        result = subprocess.run(
            ['exiftool', '-s3', '-fast', file_path],
            capture_output=True,
            text=True,
            timeout=15,
            check=True
        )
        end_time = time.time()
        
        # Parse ExifTool output into dictionary
        metadata = {}
        for line in result.stdout.split('\n'):
            if ':' in line:
                key, value = line.split(':', 1)
                metadata[key.strip()] = value.strip()
        
        field_count = len(metadata)
        return True, end_time - start_time, field_count, metadata
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, FileNotFoundError):
        end_time = time.time()
        return False, end_time - start_time, 0, {}


def test_pil(file_path: str) -> Tuple[bool, float, int, Dict[str, str]]:
    """Test PIL on a single file"""
    if not PIL_AVAILABLE:
        return False, 0.0, 0, {}
    
    start_time = time.time()
    try:
        with Image.open(file_path) as img:
            exif_data = img._getexif()
            if exif_data:
                # Convert PIL EXIF to readable format
                metadata = {}
                for tag_id, value in exif_data.items():
                    tag = TAGS.get(tag_id, tag_id)
                    metadata[str(tag)] = str(value)
                field_count = len(metadata)
            else:
                metadata = {}
                field_count = 0
        end_time = time.time()
        return True, end_time - start_time, field_count, metadata
    except Exception:
        end_time = time.time()
        return False, end_time - start_time, 0, {}


def test_fast_exif(file_path: str) -> Tuple[bool, float, int, Dict[str, str]]:
    """Test our Rust implementation on a single file"""
    if not FAST_EXIF_AVAILABLE:
        return False, 0.0, 0, {}
    
    start_time = time.time()
    try:
        reader = FastExifReader()
        metadata = reader.read_file(file_path)
        field_count = len(metadata)
        end_time = time.time()
        return True, end_time - start_time, field_count, metadata
    except Exception:
        end_time = time.time()
        return False, end_time - start_time, 0, {}


def compare_metadata(exiftool_data: Dict[str, str], pil_data: Dict[str, str], 
                     fast_exif_data: Dict[str, str]) -> Dict[str, Any]:
    """Compare metadata from different methods"""
    comparison = {
        'exiftool_fields': len(exiftool_data),
        'pil_fields': len(pil_data),
        'fast_exif_fields': len(fast_exif_data),
        'common_fields': set(),
        'exiftool_only': set(),
        'pil_only': set(),
        'fast_exif_only': set(),
        'field_matches': 0,
        'field_mismatches': 0
    }
    
    # Normalize field names for comparison (case-insensitive)
    exiftool_norm = {k.lower(): v for k, v in exiftool_data.items()}
    pil_norm = {k.lower(): v for k, v in pil_data.items()}
    fast_exif_norm = {k.lower(): v for k, v in fast_exif_data.items()}
    
    all_fields = set(exiftool_norm.keys()) | set(pil_norm.keys()) | set(fast_exif_norm.keys())
    
    for field in all_fields:
        exiftool_val = exiftool_norm.get(field)
        pil_val = pil_norm.get(field)
        fast_exif_val = fast_exif_norm.get(field)
        
        if exiftool_val and pil_val and fast_exif_val:
            comparison['common_fields'].add(field)
            # Check if values match (normalize for comparison)
            if str(exiftool_val).strip().lower() == str(pil_val).strip().lower() == str(fast_exif_val).strip().lower():
                comparison['field_matches'] += 1
            else:
                comparison['field_mismatches'] += 1
        elif exiftool_val and pil_val:
            comparison['exiftool_only'].add(field)
        elif exiftool_val and fast_exif_val:
            comparison['exiftool_only'].add(field)
        elif pil_val and fast_exif_val:
            comparison['pil_only'].add(field)
        elif exiftool_val:
            comparison['exiftool_only'].add(field)
        elif pil_val:
            comparison['pil_only'].add(field)
        elif fast_exif_val:
            comparison['fast_exif_only'].add(field)
    
    return comparison


def benchmark_method(method_name: str, test_func, files: List[str], 
                    num_processes: int = None, num_runs: int = 1) -> Dict[str, Any]:
    """Benchmark a single method with multiple runs"""
    if num_processes is None:
        num_processes = min(mp.cpu_count(), len(files))
    
    print(f"\nBenchmarking {method_name} with {num_processes} processes ({num_runs} runs)...")
    
    all_times = []
    all_success_count = 0
    all_error_count = 0
    all_total_fields = 0
    all_metadata = []  # Store metadata for validation
    
    for run in range(num_runs):
        print(f"  Run {run + 1}/{num_runs}...", end=" ", flush=True)
        
        times = []
        success_count = 0
        error_count = 0
        total_fields = 0
        run_metadata = []
        
        with ProcessPoolExecutor(max_workers=num_processes) as executor:
            future_to_file = {executor.submit(test_func, f): f for f in files}
            
            for future in as_completed(future_to_file):
                success, elapsed_time, field_count, metadata = future.result()
                times.append(elapsed_time)
                run_metadata.append(metadata)
                if success:
                    success_count += 1
                    total_fields += field_count
                else:
                    error_count += 1
        
        all_times.extend(times)
        all_success_count += success_count
        all_error_count += error_count
        all_total_fields += total_fields
        all_metadata.extend(run_metadata)
        
        avg_time = statistics.mean(times) if times else 0
        print(f"Avg: {avg_time:.3f}s, Success: {success_count}/{len(files)}")
    
    return {
        'method': method_name,
        'times': all_times,
        'success_count': all_success_count,
        'error_count': all_error_count,
        'total_fields': all_total_fields,
        'metadata': all_metadata
    }


def main():
    parser = argparse.ArgumentParser(description="Comprehensive EXIF extraction benchmark")
    parser.add_argument("directory", help="Directory containing image files")
    parser.add_argument("--file-counts", nargs="+", type=int, default=[10, 100],
                       help="File counts to test (default: 10 100)")
    parser.add_argument("--runs", type=int, default=3,
                       help="Number of runs per test (default: 3)")
    parser.add_argument("--max-files", type=int,
                       help="Maximum files to scan from directory")
    parser.add_argument("--output", "-o", help="Output JSON file for results")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.directory):
        print(f"Directory not found: {args.directory}")
        return 1
    
    # Check dependencies
    print("Checking dependencies...")
    
    try:
        subprocess.run(['exiftool', '-ver'], capture_output=True, check=True)
        print("✓ ExifTool available")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("✗ ExifTool not found. Install from: https://exiftool.org/")
        return 1
    
    if PIL_AVAILABLE:
        print("✓ PIL/Pillow available")
    else:
        print("✗ PIL/Pillow not available")
    
    if FAST_EXIF_AVAILABLE:
        print("✓ Fast EXIF Reader available")
    else:
        print("✗ Fast EXIF Reader not available")
    
    # Get image files
    all_files = get_image_files(args.directory, args.max_files)
    
    if not all_files:
        print(f"No image files found in {args.directory}")
        return 1
    
    # Analyze file format distribution
    format_counts = {}
    for file_path in all_files:
        ext = Path(file_path).suffix.lower()
        format_counts[ext] = format_counts.get(ext, 0) + 1
    
    print("\nFile format distribution:")
    for ext, count in sorted(format_counts.items()):
        print(f"  {ext}: {count} files")
    
    # Initialize results container
    results = BenchmarkResults()
    results.total_files = len(all_files)
    results.format_distribution = format_counts
    
    # Run benchmarks
    for file_count in args.file_counts:
        print(f"\n{'='*60}")
        print(f"BENCHMARKING {file_count} FILES")
        print(f"{'='*60}")
        
        test_files = all_files[:file_count]
        results.file_counts.append(file_count)
        
        # Define methods to test
        methods = [
            ("ExifTool", test_exiftool),
        ]
        
        if PIL_AVAILABLE:
            methods.append(("PIL/Pillow", test_pil))
        
        if FAST_EXIF_AVAILABLE:
            methods.append(("Fast EXIF Reader", test_fast_exif))
        
        # Run benchmarks for each method
        method_results = {}
        for method_name, test_func in methods:
            benchmark_result = benchmark_method(method_name, test_func, test_files, 
                                              num_runs=args.runs)
            method_results[method_name] = benchmark_result
            results.add_result(method_name, file_count, 
                             benchmark_result['times'],
                             benchmark_result['success_count'],
                             benchmark_result['error_count'],
                             benchmark_result['total_fields'])
        
        # Validate metadata consistency for a sample of files
        if len(test_files) > 0 and len(method_results) >= 2:
            print(f"\nValidating metadata consistency for {min(5, len(test_files))} sample files...")
            
            validation_results = []
            sample_files = test_files[:5]  # Test first 5 files
            
            for file_path in sample_files:
                print(f"  Validating {os.path.basename(file_path)}...")
                
                # Get metadata from each method
                exiftool_data = {}
                pil_data = {}
                fast_exif_data = {}
                
                if 'ExifTool' in method_results:
                    success, _, _, metadata = test_exiftool(file_path)
                    if success:
                        exiftool_data = metadata
                
                if 'PIL/Pillow' in method_results:
                    success, _, _, metadata = test_pil(file_path)
                    if success:
                        pil_data = metadata
                
                if 'Fast EXIF Reader' in method_results:
                    success, _, _, metadata = test_fast_exif(file_path)
                    if success:
                        fast_exif_data = metadata
                
                # Compare metadata
                comparison = compare_metadata(exiftool_data, pil_data, fast_exif_data)
                validation_results.append({
                    'file': os.path.basename(file_path),
                    'comparison': comparison
                })
                
                print(f"    ExifTool: {comparison['exiftool_fields']} fields, "
                      f"PIL: {comparison['pil_fields']} fields, "
                      f"Fast EXIF: {comparison['fast_exif_fields']} fields")
            
            # Print validation summary
            print(f"\nValidation Summary:")
            print(f"  Files tested: {len(validation_results)}")
            
            total_matches = sum(r['comparison']['field_matches'] for r in validation_results)
            total_mismatches = sum(r['comparison']['field_mismatches'] for r in validation_results)
            
            if total_matches + total_mismatches > 0:
                accuracy = total_matches / (total_matches + total_mismatches) * 100
                print(f"  Field accuracy: {accuracy:.1f}% ({total_matches}/{total_matches + total_mismatches})")
            else:
                print(f"  Field accuracy: No comparable fields found")
            
            # Store validation results
            results.validation_results = validation_results
    
    # Print summary
    results.print_summary()
    
    # Save results if requested
    if args.output:
        results.save_json(args.output)
    else:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        results.save_json(f"benchmark_results_{timestamp}.json")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
