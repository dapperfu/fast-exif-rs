#!/usr/bin/env python3
"""
Comprehensive Benchmark: fast-exif-rs vs PyExifTool
Tests 1000 random photos from /keg/pictures/ across various formats
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

def find_photos(directory, max_photos=1000):
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

def benchmark_fast_exif(photos):
    """Benchmark fast-exif-rs"""
    if not FAST_EXIF_AVAILABLE:
        return None
    
    print("🔄 Benchmarking fast-exif-rs...")
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
            
            if (i + 1) % 100 == 0:
                print(f"  ✅ Processed {i + 1}/{len(photos)} files")
                
        except Exception as e:
            results['failed'] += 1
            results['errors'].append(f"{photo_path}: {str(e)}")
            ext = Path(photo_path).suffix.lower()
            results['format_stats'][ext]['errors'] += 1
    
    results['total_time'] = time.time() - start_time
    return results

def benchmark_exiftool(photos):
    """Benchmark PyExifTool"""
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
                
                if (i + 1) % 100 == 0:
                    print(f"  ✅ Processed {i + 1}/{len(photos)} files")
                    
            except Exception as e:
                results['failed'] += 1
                results['errors'].append(f"{photo_path}: {str(e)}")
                ext = Path(photo_path).suffix.lower()
                results['format_stats'][ext]['errors'] += 1
    
    results['total_time'] = time.time() - start_time
    return results

def analyze_results(fast_results, exiftool_results):
    """Analyze and compare results"""
    print("\n" + "="*80)
    print("📊 COMPREHENSIVE BENCHMARK RESULTS")
    print("="*80)
    
    if not fast_results or not exiftool_results:
        print("❌ Cannot compare - one or both libraries failed")
        return
    
    # Overall performance
    print(f"\n🎯 OVERALL PERFORMANCE:")
    print(f"  📁 Total files tested: {fast_results['total_files']}")
    print(f"  ⚡ fast-exif-rs: {fast_results['total_time']:.2f}s total ({fast_results['total_time']/fast_results['total_files']:.4f}s avg)")
    print(f"  ⚡ PyExifTool: {exiftool_results['total_time']:.2f}s total ({exiftool_results['total_time']/exiftool_results['total_files']:.4f}s avg)")
    
    speedup = exiftool_results['total_time'] / fast_results['total_time']
    print(f"  🚀 Speedup: {speedup:.2f}x {'faster' if speedup > 1 else 'slower'}")
    
    # Success rates
    print(f"\n✅ SUCCESS RATES:")
    fast_success_rate = fast_results['successful'] / fast_results['total_files'] * 100
    exiftool_success_rate = exiftool_results['successful'] / exiftool_results['total_files'] * 100
    print(f"  📈 fast-exif-rs: {fast_results['successful']}/{fast_results['total_files']} ({fast_success_rate:.1f}%)")
    print(f"  📈 PyExifTool: {exiftool_results['successful']}/{exiftool_results['total_files']} ({exiftool_success_rate:.1f}%)")
    
    # Field counts
    print(f"\n📊 FIELD COUNTS:")
    fast_avg_fields = statistics.mean(fast_results['field_counts']) if fast_results['field_counts'] else 0
    exiftool_avg_fields = statistics.mean(exiftool_results['field_counts']) if exiftool_results['field_counts'] else 0
    print(f"  📈 fast-exif-rs: {fast_avg_fields:.1f} fields avg")
    print(f"  📈 PyExifTool: {exiftool_avg_fields:.1f} fields avg")
    
    # Format breakdown
    print(f"\n📁 FORMAT BREAKDOWN:")
    all_formats = set(fast_results['format_stats'].keys()) | set(exiftool_results['format_stats'].keys())
    
    for fmt in sorted(all_formats):
        fast_stats = fast_results['format_stats'][fmt]
        exiftool_stats = exiftool_results['format_stats'][fmt]
        
        if fast_stats['count'] > 0 and exiftool_stats['count'] > 0:
            fast_avg_time = fast_stats['time'] / fast_stats['count']
            exiftool_avg_time = exiftool_stats['time'] / exiftool_stats['count']
            speedup_fmt = exiftool_avg_time / fast_avg_time if fast_avg_time > 0 else 0
            
            print(f"  {fmt:>6}: {fast_stats['count']:>3} files | "
                  f"fast-exif: {fast_avg_time:.4f}s | "
                  f"exiftool: {exiftool_avg_time:.4f}s | "
                  f"{speedup_fmt:.2f}x")
    
    # Error analysis
    if fast_results['errors'] or exiftool_results['errors']:
        print(f"\n❌ ERRORS:")
        print(f"  fast-exif-rs errors: {len(fast_results['errors'])}")
        print(f"  PyExifTool errors: {len(exiftool_results['errors'])}")
        
        if fast_results['errors']:
            print(f"\n  fast-exif-rs error samples:")
            for error in fast_results['errors'][:5]:
                print(f"    • {error}")
        
        if exiftool_results['errors']:
            print(f"\n  PyExifTool error samples:")
            for error in exiftool_results['errors'][:5]:
                print(f"    • {error}")
    
    # Performance percentiles
    print(f"\n📈 PERFORMANCE PERCENTILES:")
    fast_times = sorted(fast_results['file_times'])
    exiftool_times = sorted(exiftool_results['file_times'])
    
    percentiles = [50, 90, 95, 99]
    for p in percentiles:
        fast_p = fast_times[int(len(fast_times) * p / 100)] if fast_times else 0
        exiftool_p = exiftool_times[int(len(exiftool_times) * p / 100)] if exiftool_times else 0
        print(f"  P{p:>2}: fast-exif {fast_p:.4f}s | exiftool {exiftool_p:.4f}s")

def main():
    print("🚀 Starting Comprehensive Benchmark Test")
    print("="*60)
    
    # Find photos
    photos_dir = "/keg/pictures"
    print(f"🔍 Scanning {photos_dir} for photos...")
    
    photos = find_photos(photos_dir, 1000)
    print(f"📸 Found {len(photos)} photos to test")
    
    if len(photos) == 0:
        print("❌ No photos found!")
        return
    
    # Show sample of formats
    formats = defaultdict(int)
    for photo in photos[:100]:  # Sample first 100
        ext = Path(photo).suffix.lower()
        formats[ext] += 1
    
    print(f"📁 Format distribution (sample):")
    for fmt, count in sorted(formats.items()):
        print(f"  {fmt}: {count} files")
    
    # Run benchmarks
    print(f"\n🔄 Running benchmarks on {len(photos)} photos...")
    
    fast_results = benchmark_fast_exif(photos)
    exiftool_results = benchmark_exiftool(photos)
    
    # Analyze results
    analyze_results(fast_results, exiftool_results)
    
    # Save detailed results
    results = {
        'fast_exif': fast_results,
        'exiftool': exiftool_results,
        'photos_tested': len(photos),
        'timestamp': time.time()
    }
    
    with open('benchmark_1000_results.json', 'w') as f:
        json.dump(results, f, indent=2, default=str)
    
    print(f"\n💾 Detailed results saved to: benchmark_1000_results.json")
    print("🎉 Benchmark complete!")

if __name__ == "__main__":
    main()

