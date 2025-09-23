#!/usr/bin/env python3
"""
Example usage of FastExifReader with multiprocessing support.

This script demonstrates how to use the multiprocessing functionality
to avoid pickle errors when processing multiple files.
"""

import time
from pathlib import Path
from fast_exif_reader import (
    FastExifReader,
    MultiprocessingExifReader,
    extract_exif_batch,
    read_directory
)


def example_single_threaded():
    """Example of single-threaded processing (may cause pickle errors with multiprocessing)."""
    print("=== Single-threaded processing ===")
    
    reader = FastExifReader()
    
    # This would cause pickle errors if passed to multiprocessing
    # because FastExifReader instances aren't pickleable by default
    try:
        metadata = reader.read_file("sample_image.jpg")
        print(f"Successfully read metadata: {len(metadata)} fields")
    except FileNotFoundError:
        print("Sample image not found - skipping single-threaded example")


def example_multiprocessing_worker():
    """Example using multiprocessing with worker functions."""
    print("\n=== Multiprocessing with worker functions ===")
    
    # List of files to process
    file_paths = [
        "/keg/pictures/2015/05-May/20150503_093231.jpg",
        "/keg/pictures/2015/05-May/20150527_202045.jpg", 
        "/keg/pictures/2015/05-May/20150522_101905.jpg"
    ]
    
    # Filter to only existing files
    existing_files = [f for f in file_paths if Path(f).exists()]
    
    if not existing_files:
        print("No existing files found - creating dummy example")
        # For demonstration, we'll show the API even without real files
        print("Would process files:", file_paths)
        return
    
    print(f"Processing {len(existing_files)} files...")
    
    start_time = time.time()
    results = extract_exif_batch(existing_files, max_workers=2)
    end_time = time.time()
    
    print(f"Processing completed in {end_time - start_time:.2f} seconds")
    print(f"Success rate: {results['statistics']['success_rate']:.1f}%")
    print(f"Files per second: {results['statistics']['files_per_second']:.1f}")
    
    # Show sample results
    for file_path, result in list(results['results'].items())[:2]:
        print(f"\nFile: {Path(file_path).name}")
        print(f"Success: {result['success']}")
        if result['success']:
            print(f"Fields extracted: {len(result['metadata'])}")
            if 'Make' in result['metadata']:
                print(f"Camera: {result['metadata']['Make']} {result['metadata'].get('Model', '')}")


def example_multiprocessing_class():
    """Example using the MultiprocessingExifReader class."""
    print("\n=== MultiprocessingExifReader class ===")
    
    reader = MultiprocessingExifReader(max_workers=2)
    
    # Process a directory
    directory = "/keg/pictures/2015/05-May/"
    if Path(directory).exists():
        print(f"Processing directory: {directory}")
        
        start_time = time.time()
        results = reader.read_directory(directory, max_files=10)
        end_time = time.time()
        
        print(f"Directory processing completed in {end_time - start_time:.2f} seconds")
        print(f"Files processed: {results['statistics']['total_files']}")
        print(f"Success rate: {results['statistics']['success_rate']:.1f}%")
    else:
        print(f"Directory {directory} not found - skipping directory example")


def example_error_handling():
    """Example showing proper error handling."""
    print("\n=== Error handling example ===")
    
    # Mix of existing and non-existing files
    file_paths = [
        "/keg/pictures/2015/05-May/20150503_093231.jpg",  # May exist
        "/nonexistent/file.jpg",  # Won't exist
        "/keg/pictures/2015/05-May/20150527_202045.jpg",  # May exist
    ]
    
    results = extract_exif_batch(file_paths)
    
    print("Results summary:")
    for file_path, result in results['results'].items():
        status = "✓" if result['success'] else "✗"
        print(f"  {status} {Path(file_path).name}: {result.get('error', 'Success')}")


def main():
    """Run all examples."""
    print("FastExifReader Multiprocessing Examples")
    print("=" * 50)
    
    example_single_threaded()
    example_multiprocessing_worker()
    example_multiprocessing_class()
    example_error_handling()
    
    print("\n" + "=" * 50)
    print("Examples completed!")
    print("\nKey points:")
    print("1. Use extract_exif_batch() or MultiprocessingExifReader for multiprocessing")
    print("2. Worker functions create new FastExifReader instances to avoid pickle issues")
    print("3. The pickle support in FastExifReader allows it to be serialized if needed")
    print("4. Error handling is built into the multiprocessing functions")


if __name__ == "__main__":
    main()
