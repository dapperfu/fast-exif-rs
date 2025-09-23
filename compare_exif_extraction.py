#!/usr/bin/env python3
"""
Comprehensive EXIF extraction comparison between fast-exif-reader and exiftool
Randomly samples 2 files of each extension and compares field extraction
"""

import sys
import os
import subprocess
import random
import time
from pathlib import Path
from collections import defaultdict

# Add the python directory to the path
sys.path.insert(0, 'python')
from fast_exif_reader import FastExifReader

def run_exiftool(file_path):
    """Run exiftool on a file and return parsed output"""
    try:
        result = subprocess.run(['exiftool', '-s', file_path], 
                              capture_output=True, text=True, timeout=30)
        if result.returncode == 0:
            # Parse exiftool output into dictionary
            fields = {}
            for line in result.stdout.strip().split('\n'):
                if ':' in line:
                    key, value = line.split(':', 1)
                    fields[key.strip()] = value.strip()
            return fields
        else:
            return {}
    except Exception as e:
        print(f"exiftool error for {file_path}: {e}")
        return {}

def find_random_samples():
    """Find 2 random samples of each file extension"""
    print("🔍 Finding random samples of each file type...")
    
    # Get all unique extensions
    cmd = "find /keg/pictures/20* -type f | sed 's/.*\\.//' | sort | uniq"
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=60)
        if result.returncode == 0:
            extensions = [ext.strip() for ext in result.stdout.strip().split('\n') if ext.strip()]
            print(f"Found {len(extensions)} file types: {extensions}")
        else:
            print("Error getting extensions")
            return {}
    except Exception as e:
        print(f"Error getting extensions: {e}")
        return {}
    
    # Get 2 random samples for each extension
    samples = {}
    for ext in extensions:
        cmd_sample = f"find /keg/pictures/20* -name '*.{ext}' | shuf | head -2"
        try:
            result = subprocess.run(cmd_sample, shell=True, capture_output=True, text=True, timeout=30)
            if result.returncode == 0 and result.stdout.strip():
                files = result.stdout.strip().split('\n')
                samples[ext] = files
                print(f"  {ext}: {len(files)} samples")
        except Exception as e:
            print(f"  Error getting {ext} samples: {e}")
    
    return samples

def compare_extraction(file_path, extension):
    """Compare EXIF extraction between fast-exif-reader and exiftool"""
    print(f"\n{'='*80}")
    print(f"📁 Testing {extension.upper()}: {os.path.basename(file_path)}")
    print(f"{'='*80}")
    
    reader = FastExifReader()
    
    # Test fast-exif-reader
    fast_start = time.time()
    try:
        fast_result = reader.read_file(file_path)
        fast_time = time.time() - fast_start
        fast_success = True
    except Exception as e:
        fast_result = {}
        fast_time = 0
        fast_success = False
        print(f"❌ FAST-EXIF-READER ERROR: {e}")
    
    # Test exiftool
    exiftool_start = time.time()
    exiftool_result = run_exiftool(file_path)
    exiftool_time = time.time() - exiftool_start
    exiftool_success = len(exiftool_result) > 0
    
    if not exiftool_success:
        print(f"❌ EXIFTOOL ERROR: No fields extracted")
    
    # Results summary
    print(f"\n📊 EXTRACTION RESULTS:")
    print(f"  Fast-exif-reader: {'✅ SUCCESS' if fast_success else '❌ FAILED'}")
    print(f"    Fields: {len(fast_result)}")
    print(f"    Time: {fast_time:.3f}s")
    
    print(f"  Exiftool: {'✅ SUCCESS' if exiftool_success else '❌ FAILED'}")
    print(f"    Fields: {len(exiftool_result)}")
    print(f"    Time: {exiftool_time:.3f}s")
    
    if fast_success and exiftool_success:
        speed_ratio = exiftool_time / fast_time if fast_time > 0 else float('inf')
        print(f"    Speed advantage: {speed_ratio:.1f}x faster")
    
    # Field comparison
    if fast_success and exiftool_success:
        print(f"\n🔍 FIELD COMPARISON:")
        
        # Common fields
        fast_keys = set(fast_result.keys())
        exiftool_keys = set(exiftool_result.keys())
        common_keys = fast_keys & exiftool_keys
        fast_only = fast_keys - exiftool_keys
        exiftool_only = exiftool_keys - fast_keys
        
        print(f"  Common fields: {len(common_keys)}")
        print(f"  Fast-exif only: {len(fast_only)}")
        print(f"  Exiftool only: {len(exiftool_only)}")
        
        # Show key fields comparison
        key_fields = ['DateTimeOriginal', 'CreateDate', 'Make', 'Model', 'Format']
        print(f"\n📋 KEY FIELDS COMPARISON:")
        for field in key_fields:
            fast_val = fast_result.get(field, 'NOT FOUND')
            exiftool_val = exiftool_result.get(field, 'NOT FOUND')
            match = "✅" if fast_val == exiftool_val else "❌"
            print(f"  {field}:")
            print(f"    Fast-exif: {fast_val}")
            print(f"    Exiftool:  {exiftool_val}")
            print(f"    Match: {match}")
        
        # Show SubSec fields
        fast_subsec = [k for k in fast_result.keys() if 'SubSec' in k]
        exiftool_subsec = [k for k in exiftool_result.keys() if 'SubSec' in k]
        
        if fast_subsec or exiftool_subsec:
            print(f"\n⏱️  SUBSEC FIELDS:")
            print(f"  Fast-exif SubSec fields ({len(fast_subsec)}):")
            for field in sorted(fast_subsec):
                print(f"    {field}: {fast_result[field]}")
            
            print(f"  Exiftool SubSec fields ({len(exiftool_subsec)}):")
            for field in sorted(exiftool_subsec):
                print(f"    {field}: {exiftool_result[field]}")
        
        # Show differences in common fields
        if common_keys:
            print(f"\n🔍 FIELD VALUE DIFFERENCES:")
            differences = 0
            for key in sorted(common_keys):
                if fast_result[key] != exiftool_result[key]:
                    differences += 1
                    if differences <= 5:  # Show first 5 differences
                        print(f"  {key}:")
                        print(f"    Fast-exif: {fast_result[key]}")
                        print(f"    Exiftool:  {exiftool_result[key]}")
            
            if differences > 5:
                print(f"  ... and {differences - 5} more differences")
            
            print(f"  Total differences: {differences}/{len(common_keys)} fields")
    
    return {
        'extension': extension,
        'file': os.path.basename(file_path),
        'fast_success': fast_success,
        'fast_fields': len(fast_result),
        'fast_time': fast_time,
        'exiftool_success': exiftool_success,
        'exiftool_fields': len(exiftool_result),
        'exiftool_time': exiftool_time,
        'fast_result': fast_result,
        'exiftool_result': exiftool_result
    }

def main():
    print("🔍 COMPREHENSIVE EXIF EXTRACTION COMPARISON")
    print("=" * 80)
    print("Comparing fast-exif-reader vs exiftool on random samples")
    print("=" * 80)
    
    # Find random samples
    samples = find_random_samples()
    
    if not samples:
        print("❌ No samples found!")
        return
    
    # Test each sample
    results = []
    for extension, files in samples.items():
        for file_path in files:
            result = compare_extraction(file_path, extension)
            results.append(result)
    
    # Summary table
    print(f"\n{'='*80}")
    print("📊 SUMMARY TABLE")
    print(f"{'='*80}")
    print(f"{'Extension':<8} {'File':<30} {'Fast':<6} {'Exif':<6} {'Fast':<6} {'Exif':<6} {'Speed':<8}")
    print(f"{'':<8} {'':<30} {'Fields':<6} {'Fields':<6} {'Time':<6} {'Time':<6} {'Ratio':<8}")
    print("-" * 80)
    
    total_fast_success = 0
    total_exiftool_success = 0
    total_fast_fields = 0
    total_exiftool_fields = 0
    total_fast_time = 0
    total_exiftool_time = 0
    
    for result in results:
        ext = result['extension']
        file = result['file'][:28] + ".." if len(result['file']) > 30 else result['file']
        fast_success = "✅" if result['fast_success'] else "❌"
        exiftool_success = "✅" if result['exiftool_success'] else "❌"
        fast_fields = result['fast_fields']
        exiftool_fields = result['exiftool_fields']
        fast_time = f"{result['fast_time']:.3f}"
        exiftool_time = f"{result['exiftool_time']:.3f}"
        
        speed_ratio = "N/A"
        if result['fast_success'] and result['exiftool_success'] and result['fast_time'] > 0:
            ratio = result['exiftool_time'] / result['fast_time']
            speed_ratio = f"{ratio:.1f}x"
        
        print(f"{ext:<8} {file:<30} {fast_success:<6} {exiftool_success:<6} {fast_fields:<6} {exiftool_fields:<6} {speed_ratio:<8}")
        
        if result['fast_success']:
            total_fast_success += 1
            total_fast_fields += fast_fields
            total_fast_time += result['fast_time']
        
        if result['exiftool_success']:
            total_exiftool_success += 1
            total_exiftool_fields += exiftool_fields
            total_exiftool_time += result['exiftool_time']
    
    # Overall statistics
    print("-" * 80)
    print(f"{'TOTAL':<8} {'':<30} {total_fast_success:<6} {total_exiftool_success:<6} {total_fast_fields:<6} {total_exiftool_fields:<6} {'':<8}")
    
    if total_fast_time > 0 and total_exiftool_time > 0:
        overall_speed = total_exiftool_time / total_fast_time
        print(f"\n🚀 OVERALL PERFORMANCE:")
        print(f"  Fast-exif-reader: {total_fast_success}/{len(results)} files successful")
        print(f"  Exiftool: {total_exiftool_success}/{len(results)} files successful")
        print(f"  Average fields per file:")
        print(f"    Fast-exif-reader: {total_fast_fields/total_fast_success:.1f}")
        print(f"    Exiftool: {total_exiftool_fields/total_exiftool_success:.1f}")
        print(f"  Overall speed advantage: {overall_speed:.1f}x faster")

if __name__ == "__main__":
    main()
