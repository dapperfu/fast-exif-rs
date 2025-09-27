#!/usr/bin/env python3
"""
Large-scale EXIF processing for /keg/pictures/ collection
Optimized for processing 243,815+ images efficiently
"""

import fast_exif_reader
import os
import time
import multiprocessing
from pathlib import Path
from collections import defaultdict
import json

def process_image_batch(image_paths, batch_id):
    """Process a batch of images"""
    reader = fast_exif_reader.FastExifReader()
    results = {
        'batch_id': batch_id,
        'processed': 0,
        'errors': 0,
        'formats': defaultdict(int),
        'total_fields': 0,
        'processing_time': 0
    }
    
    start_time = time.time()
    
    for img_path in image_paths:
        try:
            metadata = reader.read_file(img_path)
            results['processed'] += 1
            results['total_fields'] += len(metadata)
            
            # Track format
            ext = Path(img_path).suffix.lower()
            results['formats'][ext] += 1
            
        except Exception as e:
            results['errors'] += 1
            print(f"Error processing {img_path}: {e}")
    
    results['processing_time'] = time.time() - start_time
    return results

def large_scale_exif_processing():
    """Process the entire /keg/pictures/ collection"""
    print("🚀 LARGE-SCALE EXIF PROCESSING")
    print("=" * 50)
    print("Processing 243,815+ images from /keg/pictures/")
    print("Optimized for maximum throughput and efficiency")
    print()
    
    # Configuration
    BATCH_SIZE = 1000  # Process 1000 images per batch
    MAX_WORKERS = multiprocessing.cpu_count()
    SAMPLE_SIZE = 10000  # Limit for demo (remove for full processing)
    
    print(f"Configuration:")
    print(f"  Batch size: {BATCH_SIZE}")
    print(f"  Max workers: {MAX_WORKERS}")
    print(f"  Sample size: {SAMPLE_SIZE:,} images")
    print()
    
    # Collect image paths
    print("📊 Collecting image paths...")
    image_paths = []
    format_counts = defaultdict(int)
    
    start_collect = time.time()
    
    for root, dirs, files in os.walk("/keg/pictures"):
        for file in files:
            if file.lower().endswith(('.jpg', '.jpeg', '.heic', '.heif', '.cr2', '.nef', '.orf', '.dng')):
                img_path = os.path.join(root, file)
                image_paths.append(img_path)
                
                ext = Path(file).suffix.lower()
                format_counts[ext] += 1
                
                # Limit for demo
                if len(image_paths) >= SAMPLE_SIZE:
                    break
        if len(image_paths) >= SAMPLE_SIZE:
            break
    
    collect_time = time.time() - start_collect
    
    print(f"✅ Collected {len(image_paths):,} images in {collect_time:.2f} seconds")
    print(f"Format distribution:")
    for ext, count in sorted(format_counts.items()):
        percentage = (count / len(image_paths)) * 100
        print(f"  {ext:4}: {count:6,} ({percentage:5.1f}%)")
    print()
    
    # Create batches
    batches = []
    for i in range(0, len(image_paths), BATCH_SIZE):
        batch = image_paths[i:i + BATCH_SIZE]
        batches.append((batch, i // BATCH_SIZE))
    
    print(f"📦 Created {len(batches)} batches")
    print()
    
    # Process batches
    print("⚡ Processing batches...")
    start_process = time.time()
    
    # Use multiprocessing for parallel processing
    with multiprocessing.Pool(MAX_WORKERS) as pool:
        batch_results = pool.starmap(process_image_batch, batches)
    
    process_time = time.time() - start_process
    
    # Aggregate results
    total_processed = sum(r['processed'] for r in batch_results)
    total_errors = sum(r['errors'] for r in batch_results)
    total_fields = sum(r['total_fields'] for r in batch_results)
    total_formats = defaultdict(int)
    
    for result in batch_results:
        for ext, count in result['formats'].items():
            total_formats[ext] += count
    
    # Calculate performance metrics
    images_per_second = total_processed / process_time
    avg_fields_per_image = total_fields / max(1, total_processed)
    
    print("📊 PROCESSING RESULTS")
    print("=" * 30)
    print(f"Total images processed: {total_processed:,}")
    print(f"Total errors: {total_errors:,}")
    print(f"Success rate: {((total_processed / (total_processed + total_errors)) * 100):.1f}%")
    print(f"Total EXIF fields extracted: {total_fields:,}")
    print(f"Average fields per image: {avg_fields_per_image:.1f}")
    print(f"Processing time: {process_time:.2f} seconds")
    print(f"Throughput: {images_per_second:.1f} images/second")
    print()
    
    print("Format breakdown:")
    for ext, count in sorted(total_formats.items()):
        percentage = (count / total_processed) * 100
        print(f"  {ext:4}: {count:6,} ({percentage:5.1f}%)")
    print()
    
    # Performance analysis
    print("⚡ PERFORMANCE ANALYSIS")
    print("-" * 25)
    
    if images_per_second >= 100:
        print("✅ Excellent throughput (>100 images/sec)")
    elif images_per_second >= 50:
        print("✅ Good throughput (>50 images/sec)")
    elif images_per_second >= 20:
        print("⚠️  Moderate throughput (>20 images/sec)")
    else:
        print("❌ Poor throughput (<20 images/sec)")
    
    if total_errors == 0:
        print("✅ Perfect reliability (0 errors)")
    elif total_errors < total_processed * 0.01:
        print("✅ Excellent reliability (<1% errors)")
    elif total_errors < total_processed * 0.05:
        print("✅ Good reliability (<5% errors)")
    else:
        print("⚠️  Moderate reliability (>5% errors)")
    
    # Extrapolation for full collection
    if len(image_paths) < 243815:
        full_collection_time = (243815 / len(image_paths)) * process_time
        full_collection_hours = full_collection_time / 3600
        
        print(f"\n📈 EXTRAPOLATION FOR FULL COLLECTION")
        print("-" * 35)
        print(f"Estimated time for 243,815 images: {full_collection_hours:.1f} hours")
        print(f"Estimated total EXIF fields: {int((243815 / len(image_paths)) * total_fields):,}")
        
        if full_collection_hours < 1:
            print("✅ Full collection can be processed in under 1 hour")
        elif full_collection_hours < 6:
            print("✅ Full collection can be processed in under 6 hours")
        elif full_collection_hours < 24:
            print("⚠️  Full collection would take under 24 hours")
        else:
            print("❌ Full collection would take over 24 hours")
    
    # Recommendations
    print(f"\n💡 OPTIMIZATION RECOMMENDATIONS")
    print("-" * 35)
    
    if images_per_second < 50:
        print("• Consider increasing batch size")
        print("• Use SSD storage for better I/O performance")
        print("• Increase number of worker processes")
    
    if total_errors > 0:
        print("• Investigate error patterns in specific formats")
        print("• Add better error handling for corrupted files")
    
    print("• Use multiprocessing for maximum CPU utilization")
    print("• Implement progress tracking for long-running jobs")
    print("• Consider streaming processing for memory efficiency")
    
    # Save results
    results_summary = {
        'total_processed': total_processed,
        'total_errors': total_errors,
        'total_fields': total_fields,
        'processing_time': process_time,
        'throughput': images_per_second,
        'avg_fields_per_image': avg_fields_per_image,
        'format_distribution': dict(total_formats),
        'sample_size': len(image_paths),
        'full_collection_estimate': {
            'total_images': 243815,
            'estimated_time_hours': full_collection_hours if len(image_paths) < 243815 else process_time / 3600,
            'estimated_total_fields': int((243815 / len(image_paths)) * total_fields) if len(image_paths) < 243815 else total_fields
        }
    }
    
    with open('large_scale_processing_results.json', 'w') as f:
        json.dump(results_summary, f, indent=2)
    
    print(f"\n💾 Results saved to: large_scale_processing_results.json")
    
    return results_summary

if __name__ == "__main__":
    large_scale_exif_processing()
