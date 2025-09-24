#!/usr/bin/env python3
"""
Dual-Path Multiprocessing Example

This script demonstrates both Python and Rust multiprocessing implementations
of Fast EXIF Reader, showing when to use each approach.
"""

import os
import sys
import time
import multiprocessing as mp
from pathlib import Path
from typing import List, Dict, Any

# Add the parent directory to path to import our module
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from fast_exif_reader import (
        # Python multiprocessing
        PythonMultiprocessingExifReader,
        python_extract_exif_batch,
        python_extract_exif_from_directory,
        # Rust multiprocessing
        RustMultiprocessingExifReader,
        rust_process_files_parallel,
        rust_process_directory_parallel,
        # Utility
        RUST_AVAILABLE,
    )

    print("✓ All imports successful")
except ImportError as e:
    print(f"✗ Import error: {e}")
    sys.exit(1)


def get_test_files(directory: str, max_files: int = 50) -> List[str]:
    """Get test image files from directory"""
    image_extensions = {
        ".jpg",
        ".jpeg",
        ".cr2",
        ".nef",
        ".heic",
        ".heif",
        ".tiff",
        ".tif",
    }
    files = []

    if not os.path.exists(directory):
        print(f"Directory {directory} not found")
        return []

    for root, dirs, filenames in os.walk(directory):
        for filename in filenames:
            if Path(filename).suffix.lower() in image_extensions:
                files.append(os.path.join(root, filename))
                if len(files) >= max_files:
                    break
        if len(files) >= max_files:
            break

    return files


def benchmark_implementation(name: str, func, file_paths: List[str], **kwargs) -> Dict[str, Any]:
    """Benchmark a multiprocessing implementation"""
    print(f"\n=== {name} ===")

    start_time = time.time()
    try:
        results = func(file_paths, **kwargs)
        end_time = time.time()

        processing_time = end_time - start_time
        stats = results["statistics"]

        print(f"✓ Success: {processing_time:.3f}s")
        print(f"  Files processed: {stats['total_files']}")
        print(f"  Success rate: {stats['success_rate']:.1f}%")
        print(f"  Files per second: {stats['files_per_second']:.1f}")
        print(f"  Average time per file: {stats['avg_processing_time']:.3f}s")

        return {
            "name": name,
            "success": True,
            "processing_time": processing_time,
            "files_per_second": stats["files_per_second"],
            "success_rate": stats["success_rate"],
            "total_files": stats["total_files"],
        }

    except Exception as e:
        end_time = time.time()
        print(f"✗ Failed: {e}")
        return {
            "name": name,
            "success": False,
            "processing_time": end_time - start_time,
            "error": str(e),
        }


def demonstrate_small_batch():
    """Demonstrate processing small batch of files"""
    print("\n" + "=" * 60)
    print("SMALL BATCH PROCESSING (10 files)")
    print("=" * 60)

    # Get small batch of files
    test_files = get_test_files("/keg/pictures/2015/05-May/", max_files=10)

    if not test_files:
        print("No test files found")
        return

    print(f"Testing with {len(test_files)} files")

    results = []

    # Python multiprocessing
    results.append(
        benchmark_implementation(
            "Python Multiprocessing",
            python_extract_exif_batch,
            test_files,
            max_workers=2,
        )
    )

    # Rust multiprocessing (if available)
    if RUST_AVAILABLE:
        results.append(
            benchmark_implementation(
                "Rust Multiprocessing",
                rust_process_files_parallel,
                test_files,
                max_workers=2,
            )
        )
    else:
        print("\n=== Rust Multiprocessing ===")
        print("✗ Rust implementation not available")

    # Compare results
    successful_results = [r for r in results if r["success"]]
    if len(successful_results) >= 2:
        fastest = min(successful_results, key=lambda x: x["processing_time"])
        print(f"\n🏆 Fastest: {fastest['name']} ({fastest['processing_time']:.3f}s)")

        for result in successful_results:
            if result["name"] != fastest["name"]:
                speedup = result["processing_time"] / fastest["processing_time"]
                print(f"   {result['name']}: {speedup:.2f}x slower")


def demonstrate_large_batch():
    """Demonstrate processing large batch of files"""
    print("\n" + "=" * 60)
    print("LARGE BATCH PROCESSING (50 files)")
    print("=" * 60)

    # Get larger batch of files
    test_files = get_test_files("/keg/pictures/2015/05-May/", max_files=50)

    if not test_files:
        print("No test files found")
        return

    print(f"Testing with {len(test_files)} files")

    results = []

    # Python multiprocessing
    results.append(
        benchmark_implementation(
            "Python Multiprocessing",
            python_extract_exif_batch,
            test_files,
            max_workers=4,
        )
    )

    # Rust multiprocessing (if available)
    if RUST_AVAILABLE:
        results.append(
            benchmark_implementation(
                "Rust Multiprocessing",
                rust_process_files_parallel,
                test_files,
                max_workers=4,
            )
        )
    else:
        print("\n=== Rust Multiprocessing ===")
        print("✗ Rust implementation not available")

    # Compare results
    successful_results = [r for r in results if r["success"]]
    if len(successful_results) >= 2:
        fastest = min(successful_results, key=lambda x: x["processing_time"])
        print(f"\n🏆 Fastest: {fastest['name']} ({fastest['processing_time']:.3f}s)")

        for result in successful_results:
            if result["name"] != fastest["name"]:
                speedup = result["processing_time"] / fastest["processing_time"]
                print(f"   {result['name']}: {speedup:.2f}x slower")


def demonstrate_class_based_usage():
    """Demonstrate class-based usage patterns"""
    print("\n" + "=" * 60)
    print("CLASS-BASED USAGE PATTERNS")
    print("=" * 60)

    test_files = get_test_files("/keg/pictures/2015/05-May/", max_files=20)

    if not test_files:
        print("No test files found")
        return

    print(f"Testing with {len(test_files)} files")

    # Python class-based approach
    print("\n=== Python Class-Based ===")
    start_time = time.time()
    try:
        reader = PythonMultiprocessingExifReader(max_workers=2)
        results = reader.read_files(test_files)
        end_time = time.time()

        stats = results["statistics"]
        print(f"✓ Success: {end_time - start_time:.3f}s")
        print(f"  Files per second: {stats['files_per_second']:.1f}")
        print(f"  Success rate: {stats['success_rate']:.1f}%")
    except Exception as e:
        print(f"✗ Failed: {e}")

    # Rust class-based approach
    if RUST_AVAILABLE:
        print("\n=== Rust Class-Based ===")
        start_time = time.time()
        try:
            reader = RustMultiprocessingExifReader(max_workers=2)
            results = reader.read_files(test_files)
            end_time = time.time()

            stats = results["statistics"]
            print(f"✓ Success: {end_time - start_time:.3f}s")
            print(f"  Files per second: {stats['files_per_second']:.1f}")
            print(f"  Success rate: {stats['success_rate']:.1f}%")
        except Exception as e:
            print(f"✗ Failed: {e}")
    else:
        print("\n=== Rust Class-Based ===")
        print("✗ Rust implementation not available")


def demonstrate_directory_processing():
    """Demonstrate directory processing capabilities"""
    print("\n" + "=" * 60)
    print("DIRECTORY PROCESSING")
    print("=" * 60)

    test_directory = "/keg/pictures/2015/05-May/"

    if not os.path.exists(test_directory):
        print(f"Test directory {test_directory} not found")
        return

    print(f"Processing directory: {test_directory}")

    # Python directory processing
    print("\n=== Python Directory Processing ===")
    start_time = time.time()
    try:
        results = python_extract_exif_from_directory(test_directory, max_files=30, max_workers=2)
        end_time = time.time()

        stats = results["statistics"]
        print(f"✓ Success: {end_time - start_time:.3f}s")
        print(f"  Files found: {stats['total_files']}")
        print(f"  Files per second: {stats['files_per_second']:.1f}")
        print(f"  Success rate: {stats['success_rate']:.1f}%")
    except Exception as e:
        print(f"✗ Failed: {e}")

    # Rust directory processing
    if RUST_AVAILABLE:
        print("\n=== Rust Directory Processing ===")
        start_time = time.time()
        try:
            results = rust_process_directory_parallel(
                test_directory,
                extensions=[".jpg", ".cr2", ".nef"],
                max_files=30,
                max_workers=2,
            )
            end_time = time.time()

            stats = results["statistics"]
            print(f"✓ Success: {end_time - start_time:.3f}s")
            print(f"  Files found: {stats['total_files']}")
            print(f"  Files per second: {stats['files_per_second']:.1f}")
            print(f"  Success rate: {stats['success_rate']:.1f}%")
        except Exception as e:
            print(f"✗ Failed: {e}")
    else:
        print("\n=== Rust Directory Processing ===")
        print("✗ Rust implementation not available")


def demonstrate_error_handling():
    """Demonstrate error handling capabilities"""
    print("\n" + "=" * 60)
    print("ERROR HANDLING DEMONSTRATION")
    print("=" * 60)

    # Mix of existing and non-existing files
    test_files = [
        "/keg/pictures/2015/05-May/20150503_093231.jpg",  # May exist
        "/nonexistent/file1.jpg",  # Won't exist
        "/keg/pictures/2015/05-May/20150527_202045.jpg",  # May exist
        "/nonexistent/file2.jpg",  # Won't exist
    ]

    print("Testing error handling with mixed file list...")

    # Python error handling
    print("\n=== Python Error Handling ===")
    try:
        results = python_extract_exif_batch(test_files, max_workers=2)
        stats = results["statistics"]

        print(f"✓ Processed: {stats['total_files']} files")
        print(f"  Success: {stats['success_count']}")
        print(f"  Errors: {stats['error_count']}")
        print(f"  Success rate: {stats['success_rate']:.1f}%")

        # Show error details
        for file_path, result in results["results"].items():
            if not result["success"]:
                print(f"  Error in {os.path.basename(file_path)}: {result.get('error', 'Unknown')}")

    except Exception as e:
        print(f"✗ Failed: {e}")

    # Rust error handling
    if RUST_AVAILABLE:
        print("\n=== Rust Error Handling ===")
        try:
            results = rust_process_files_parallel(test_files, max_workers=2)
            stats = results["statistics"]

            print(f"✓ Processed: {stats['total_files']} files")
            print(f"  Success: {stats['success_count']}")
            print(f"  Errors: {stats['error_count']}")
            print(f"  Success rate: {stats['success_rate']:.1f}%")

            # Show error details
            for file_path, result in results["results"].items():
                if not result["success"]:
                    print(f"  Error in {os.path.basename(file_path)}: {result.get('error', 'Unknown')}")

        except Exception as e:
            print(f"✗ Failed: {e}")
    else:
        print("\n=== Rust Error Handling ===")
        print("✗ Rust implementation not available")


def demonstrate_hybrid_approach():
    """Demonstrate hybrid approach for optimal performance"""
    print("\n" + "=" * 60)
    print("HYBRID APPROACH - OPTIMAL PERFORMANCE")
    print("=" * 60)

    def process_files_optimally(file_paths: List[str], max_workers: int = 4) -> Dict[str, Any]:
        """Choose optimal implementation based on file count and availability"""
        file_count = len(file_paths)

        print(f"Processing {file_count} files...")

        if file_count < 20:
            print("  → Using Python multiprocessing (small batch, better error handling)")
            return python_extract_exif_batch(file_paths, max_workers=max_workers)
        elif RUST_AVAILABLE:
            print("  → Using Rust multiprocessing (large batch, better performance)")
            return rust_process_files_parallel(file_paths, max_workers=max_workers)
        else:
            print("  → Using Python multiprocessing (Rust not available)")
            return python_extract_exif_batch(file_paths, max_workers=max_workers)

    # Test with different file counts
    test_cases = [("Small batch", 10), ("Medium batch", 30), ("Large batch", 50)]

    for case_name, file_count in test_cases:
        print(f"\n--- {case_name} ({file_count} files) ---")

        test_files = get_test_files("/keg/pictures/2015/05-May/", max_files=file_count)

        if test_files:
            start_time = time.time()
            results = process_files_optimally(test_files, max_workers=2)
            end_time = time.time()

            stats = results["statistics"]
            print(f"✓ Completed in {end_time - start_time:.3f}s")
            print(f"  Files per second: {stats['files_per_second']:.1f}")
            print(f"  Success rate: {stats['success_rate']:.1f}%")


def main():
    """Main demonstration function"""
    print("Fast EXIF Reader - Dual-Path Multiprocessing Demonstration")
    print("=" * 70)

    print("System Information:")
    print(f"  CPUs: {mp.cpu_count()}")
    print(f"  Python: {sys.version.split()[0]}")
    print(f"  Rust Available: {RUST_AVAILABLE}")

    # Run all demonstrations
    demonstrate_small_batch()
    demonstrate_large_batch()
    demonstrate_class_based_usage()
    demonstrate_directory_processing()
    demonstrate_error_handling()
    demonstrate_hybrid_approach()

    print("\n" + "=" * 70)
    print("DEMONSTRATION COMPLETED")
    print("=" * 70)

    print("\nKey Takeaways:")
    print("• Python multiprocessing: Best for small batches and development")
    print("• Rust multiprocessing: Best for large batches and production")
    print("• Hybrid approach: Choose implementation based on file count")
    print("• Both implementations provide excellent error handling")
    print("• Use class-based approach for reusable processing objects")


if __name__ == "__main__":
    main()
