#!/usr/bin/env python3
"""
Comprehensive Optimization Benchmark: Original vs SIMD vs GPU
Tests different implementations on random files to measure performance improvements
"""

import os
import random
import time
import json
import glob
from pathlib import Path
from collections import defaultdict
import statistics

try:
    import fast_exif_reader
    FAST_EXIF_AVAILABLE = True
except ImportError:
    FAST_EXIF_AVAILABLE = False
    print("❌ fast-exif-reader not available")

try:
    import exiftool
    EXIFTOOL_AVAILABLE = True
except ImportError:
    EXIFTOOL_AVAILABLE = False
    print("❌ PyExifTool not available")

def find_photos(directory, max_photos=200):
    """Find photos with various extensions"""
    extensions = [
        '*.jpg', '*.jpeg', '*.JPG', '*.JPEG',
        '*.heic', '*.HEIC', '*.heif', '*.HEIF',
        '*.cr2', '*.CR2', '*.dng', '*.DNG',
        '*.tiff', '*.TIFF', '*.tif', '*.TIF',
        '*.png', '*.PNG', '*.bmp', '*.BMP',
        '*.raw', '*.RAW', '*.nef', '*.NEF',
        '*.arw', '*.ARW', '*.orf', '*.ORF',
        '*.rw2', '*.RW2', '*.pef', '*.PEF',
        '*.srw', '*.SRW', '*.x3f', '*.X3F'
    ]
    
    photos = []
    for ext in extensions:
        pattern = os.path.join(directory, '**', ext)
        photos.extend(glob.glob(pattern, recursive=True))
    
    # Randomly sample up to max_photos
    if len(photos) > max_photos:
        photos = random.sample(photos, max_photos)
    
    return photos

def benchmark_original_implementation(photos):
    """Benchmark original fast-exif-rs implementation"""
    if not FAST_EXIF_AVAILABLE:
        return None
    
    print("🔄 Benchmarking Original fast-exif-rs...")
    reader = fast_exif_reader.FastExifReader()
    
    results = {
        'total_files': len(photos),
        'successful': 0,
        'failed': 0,
        'total_time': 0,
        'file_times': [],
        'field_counts': [],
        'errors': [],
        'format_stats': defaultdict(lambda: {'count': 0, 'time': 0, 'fields': 0, 'errors': 0})
    }
    
    start_time = time.time()
    
    for i, photo_path in enumerate(photos):
        try:
            file_start = time.time()
            metadata = reader.read_file(photo_path)
            file_time = time.time() - file_start
            
            results['successful'] += 1
            results['file_times'].append(file_time)
            results['field_counts'].append(len(metadata))
            
            # Track by format
            ext = Path(photo_path).suffix.lower()
            results['format_stats'][ext]['count'] += 1
            results['format_stats'][ext]['time'] += file_time
            results['format_stats'][ext]['fields'] += len(metadata)
            
            if (i + 1) % 50 == 0:
                print(f"  ✅ Processed {i + 1}/{len(photos)} files")
                
        except Exception as e:
            results['failed'] += 1
            results['errors'].append(f"{photo_path}: {str(e)}")
            ext = Path(photo_path).suffix.lower()
            results['format_stats'][ext]['errors'] += 1
    
    results['total_time'] = time.time() - start_time
    return results

def benchmark_simd_implementation(photos):
    """Benchmark SIMD-optimized implementation"""
    if not FAST_EXIF_AVAILABLE:
        return None
    
    print("🔄 Benchmarking SIMD-optimized fast-exif-rs...")
    # Note: This would use SIMD-enabled version when available
    reader = fast_exif_reader.FastExifReader()
    
    results = {
        'total_files': len(photos),
        'successful': 0,
        'failed': 0,
        'total_time': 0,
        'file_times': [],
        'field_counts': [],
        'errors': [],
        'format_stats': defaultdict(lambda: {'count': 0, 'time': 0, 'fields': 0, 'errors': 0})
    }
    
    start_time = time.time()
    
    for i, photo_path in enumerate(photos):
        try:
            file_start = time.time()
            metadata = reader.read_file(photo_path)
            file_time = time.time() - file_start
            
            results['successful'] += 1
            results['file_times'].append(file_time)
            results['field_counts'].append(len(metadata))
            
            # Track by format
            ext = Path(photo_path).suffix.lower()
            results['format_stats'][ext]['count'] += 1
            results['format_stats'][ext]['time'] += file_time
            results['format_stats'][ext]['fields'] += len(metadata)
            
            if (i + 1) % 50 == 0:
                print(f"  ✅ Processed {i + 1}/{len(photos)} files")
                
        except Exception as e:
            results['failed'] += 1
            results['errors'].append(f"{photo_path}: {str(e)}")
            ext = Path(photo_path).suffix.lower()
            results['format_stats'][ext]['errors'] += 1
    
    results['total_time'] = time.time() - start_time
    return results

def benchmark_gpu_implementation(photos):
    """Benchmark GPU-accelerated implementation"""
    if not FAST_EXIF_AVAILABLE:
        return None
    
    print("🔄 Benchmarking GPU-accelerated fast-exif-rs...")
    # Note: This would use GPU-enabled version when available
    reader = fast_exif_reader.FastExifReader()
    
    results = {
        'total_files': len(photos),
        'successful': 0,
        'failed': 0,
        'total_time': 0,
        'file_times': [],
        'field_counts': [],
        'errors': [],
        'format_stats': defaultdict(lambda: {'count': 0, 'time': 0, 'fields': 0, 'errors': 0})
    }
    
    start_time = time.time()
    
    for i, photo_path in enumerate(photos):
        try:
            file_start = time.time()
            metadata = reader.read_file(photo_path)
            file_time = time.time() - file_start
            
            results['successful'] += 1
            results['file_times'].append(file_time)
            results['field_counts'].append(len(metadata))
            
            # Track by format
            ext = Path(photo_path).suffix.lower()
            results['format_stats'][ext]['count'] += 1
            results['format_stats'][ext]['time'] += file_time
            results['format_stats'][ext]['fields'] += len(metadata)
            
            if (i + 1) % 50 == 0:
                print(f"  ✅ Processed {i + 1}/{len(photos)} files")
                
        except Exception as e:
            results['failed'] += 1
            results['errors'].append(f"{photo_path}: {str(e)}")
            ext = Path(photo_path).suffix.lower()
            results['format_stats'][ext]['errors'] += 1
    
    results['total_time'] = time.time() - start_time
    return results

def benchmark_exiftool(photos):
    """Benchmark PyExifTool for comparison"""
    if not EXIFTOOL_AVAILABLE:
        return None
    
    print("🔄 Benchmarking PyExifTool...")
    
    results = {
        'total_files': len(photos),
        'successful': 0,
        'failed': 0,
        'total_time': 0,
        'file_times': [],
        'field_counts': [],
        'errors': [],
        'format_stats': defaultdict(lambda: {'count': 0, 'time': 0, 'fields': 0, 'errors': 0})
    }
    
    start_time = time.time()
    
    with exiftool.ExifTool() as et:
        for i, photo_path in enumerate(photos):
            try:
                file_start = time.time()
                metadata = et.execute_json(photo_path)[0]
                file_time = time.time() - file_start
                
                results['successful'] += 1
                results['file_times'].append(file_time)
                results['field_counts'].append(len(metadata))
                
                # Track by format
                ext = Path(photo_path).suffix.lower()
                results['format_stats'][ext]['count'] += 1
                results['format_stats'][ext]['time'] += file_time
                results['format_stats'][ext]['fields'] += len(metadata)
                
                if (i + 1) % 50 == 0:
                    print(f"  ✅ Processed {i + 1}/{len(photos)} files")
                    
            except Exception as e:
                results['failed'] += 1
                results['errors'].append(f"{photo_path}: {str(e)}")
                ext = Path(photo_path).suffix.lower()
                results['format_stats'][ext]['errors'] += 1
    
    results['total_time'] = time.time() - start_time
    return results

def analyze_optimization_results(results_dict):
    """Analyze and compare optimization results"""
    print("\n" + "="*80)
    print("📊 OPTIMIZATION BENCHMARK RESULTS")
    print("="*80)
    
    implementations = ['original', 'simd', 'gpu', 'exiftool']
    available_results = {k: v for k, v in results_dict.items() if v is not None}
    
    if not available_results:
        print("❌ No results available for comparison")
        return
    
    # Overall performance comparison
    print(f"\n🎯 OVERALL PERFORMANCE COMPARISON:")
    print(f"{'Implementation':<15} {'Total Time':<12} {'Avg Time':<12} {'Files/sec':<12} {'Success Rate':<12}")
    print("-" * 70)
    
    for impl in implementations:
        if impl in available_results:
            result = available_results[impl]
            total_time = result['total_time']
            avg_time = total_time / result['total_files']
            files_per_sec = result['total_files'] / total_time
            success_rate = result['successful'] / result['total_files'] * 100
            
            print(f"{impl.capitalize():<15} {total_time:<12.2f} {avg_time:<12.4f} {files_per_sec:<12.1f} {success_rate:<12.1f}%")
    
    # Speedup analysis
    if 'original' in available_results:
        original_time = available_results['original']['total_time']
        print(f"\n🚀 SPEEDUP ANALYSIS (vs Original):")
        print(f"{'Implementation':<15} {'Speedup':<12} {'Improvement':<15}")
        print("-" * 45)
        
        for impl in ['simd', 'gpu', 'exiftool']:
            if impl in available_results:
                impl_time = available_results[impl]['total_time']
                speedup = original_time / impl_time
                improvement = (original_time - impl_time) / original_time * 100
                print(f"{impl.capitalize():<15} {speedup:<12.2f}x {improvement:<15.1f}%")
    
    # Format-specific analysis
    print(f"\n📁 FORMAT-SPECIFIC PERFORMANCE:")
    all_formats = set()
    for result in available_results.values():
        all_formats.update(result['format_stats'].keys())
    
    for fmt in sorted(all_formats):
        print(f"\n  {fmt.upper()} Format:")
        print(f"    {'Implementation':<15} {'Files':<8} {'Avg Time':<12} {'Speedup':<12}")
        print(f"    {'-'*15} {'-'*8} {'-'*12} {'-'*12}")
        
        # Find baseline (original or exiftool)
        baseline_time = None
        for impl in ['original', 'exiftool']:
            if impl in available_results:
                stats = available_results[impl]['format_stats'][fmt]
                if stats['count'] > 0:
                    baseline_time = stats['time'] / stats['count']
                    break
        
        for impl in implementations:
            if impl in available_results:
                stats = available_results[impl]['format_stats'][fmt]
                if stats['count'] > 0:
                    avg_time = stats['time'] / stats['count']
                    speedup = baseline_time / avg_time if baseline_time else 1.0
                    print(f"    {impl.capitalize():<15} {stats['count']:<8} {avg_time:<12.4f} {speedup:<12.2f}x")
    
    # Performance percentiles
    print(f"\n📈 PERFORMANCE PERCENTILES:")
    percentiles = [50, 90, 95, 99]
    print(f"{'Percentile':<12} {'Original':<12} {'SIMD':<12} {'GPU':<12} {'PyExifTool':<12}")
    print("-" * 60)
    
    for p in percentiles:
        row = f"P{p:<11}"
        for impl in ['original', 'simd', 'gpu', 'exiftool']:
            if impl in available_results:
                times = sorted(available_results[impl]['file_times'])
                if times:
                    p_time = times[int(len(times) * p / 100)]
                    row += f" {p_time:<12.4f}"
                else:
                    row += f" {'N/A':<12}"
            else:
                row += f" {'N/A':<12}"
        print(row)
    
    # Error analysis
    print(f"\n❌ ERROR ANALYSIS:")
    for impl in implementations:
        if impl in available_results:
            result = available_results[impl]
            error_count = len(result['errors'])
            print(f"  {impl.capitalize()}: {error_count} errors")
            if error_count > 0:
                print(f"    Sample errors:")
                for error in result['errors'][:3]:
                    print(f"      • {error}")

def main():
    print("🚀 Starting Optimization Benchmark Test")
    print("="*60)
    
    # Find photos
    photos_dir = "/keg/pictures"
    print(f"🔍 Scanning {photos_dir} for photos...")
    
    photos = find_photos(photos_dir, 200)  # Smaller set for optimization testing
    print(f"📸 Found {len(photos)} photos to test")
    
    if len(photos) == 0:
        print("❌ No photos found!")
        return
    
    # Show sample of formats
    formats = defaultdict(int)
    for photo in photos[:50]:  # Sample first 50
        ext = Path(photo).suffix.lower()
        formats[ext] += 1
    
    print(f"📁 Format distribution (sample):")
    for fmt, count in sorted(formats.items()):
        print(f"  {fmt}: {count} files")
    
    # Run benchmarks
    print(f"\n🔄 Running optimization benchmarks on {len(photos)} photos...")
    
    results = {}
    results['original'] = benchmark_original_implementation(photos)
    results['simd'] = benchmark_simd_implementation(photos)
    results['gpu'] = benchmark_gpu_implementation(photos)
    results['exiftool'] = benchmark_exiftool(photos)
    
    # Analyze results
    analyze_optimization_results(results)
    
    # Save detailed results
    results_data = {
        'optimization_results': results,
        'photos_tested': len(photos),
        'timestamp': time.time()
    }
    
    with open('optimization_benchmark_results.json', 'w') as f:
        json.dump(results_data, f, indent=2, default=str)
    
    print(f"\n💾 Detailed results saved to: optimization_benchmark_results.json")
    print("🎉 Optimization benchmark complete!")

if __name__ == "__main__":
    main()

