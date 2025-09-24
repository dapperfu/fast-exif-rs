#!/usr/bin/env python3
"""
Benchmark script comparing fast-exif-reader with ExifTool
"""

import subprocess
import sys
import time
from pathlib import Path

from fast_exif_reader import FastExifReader


def benchmark_exiftool(file_path: str, iterations: int = 10) -> float:
    """Benchmark ExifTool performance"""
    start_time = time.time()

    for _ in range(iterations):
        try:
            subprocess.run(
                ["exiftool", "-json", file_path],
                capture_output=True,
                text=True,
                check=True,
            )
        except subprocess.CalledProcessError:
            return float("inf")

    end_time = time.time()
    return (end_time - start_time) / iterations


def benchmark_fast_reader(file_path: str, iterations: int = 10) -> float:
    """Benchmark fast-exif-reader performance"""
    reader = FastExifReader()

    start_time = time.time()

    for _ in range(iterations):
        try:
            reader.read_file(file_path)
        except Exception as e:
            print(f"Error: {e}")
            return float("inf")

    end_time = time.time()
    return (end_time - start_time) / iterations


def main():
    if len(sys.argv) != 2:
        print("Usage: python benchmark.py <image_file>")
        sys.exit(1)

    file_path = sys.argv[1]

    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)

    print(f"Benchmarking: {file_path}")
    print("=" * 50)

    # Benchmark ExifTool
    print("Benchmarking ExifTool...")
    exiftool_time = benchmark_exiftool(file_path)
    print(f"ExifTool average time: {exiftool_time:.4f} seconds")

    # Benchmark fast-exif-reader
    print("Benchmarking fast-exif-reader...")
    fast_reader_time = benchmark_fast_reader(file_path)
    print(f"fast-exif-reader average time: {fast_reader_time:.4f} seconds")

    # Calculate speedup
    if exiftool_time != float("inf") and fast_reader_time != float("inf"):
        speedup = exiftool_time / fast_reader_time
        print(f"Speedup: {speedup:.2f}x faster")
    else:
        print("Could not calculate speedup due to errors")


if __name__ == "__main__":
    main()
