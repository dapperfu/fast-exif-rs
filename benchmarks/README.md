# EXIF Benchmark Suite

This directory contains comprehensive benchmark scripts to compare the performance of different EXIF extraction methods:

1. **ExifTool** - Industry standard command-line tool
2. **PIL/Pillow** - Pure Python image library
3. **Fast EXIF Reader** - Our optimized Rust implementation

## Benchmark Scripts

### 1. `comprehensive_benchmark.py` (Recommended)
The most complete benchmark with detailed statistics and multiple runs.

```bash
# Basic usage
python comprehensive_benchmark.py /path/to/images

# Custom file counts and runs
python comprehensive_benchmark.py /path/to/images --file-counts 10 100 1000 --runs 5

# Save results to specific file
python comprehensive_benchmark.py /path/to/images --output results.json
```

### 2. `working_benchmark.py`
Simplified benchmark that works reliably with any image files.

```bash
python working_benchmark.py
```

### 3. `parallel_exif_benchmark.py`
Advanced benchmark with extensive configuration options.

```bash
python parallel_exif_benchmark.py /path/to/images --file-counts 10 100 1000 --runs 3
```

### 4. `quick_benchmark.py`
Quick benchmark specifically for `/keg/pictures/incoming/2025/09-Sep/`.

```bash
python quick_benchmark.py
```

## Installation

Install the required dependencies:

```bash
pip install Pillow exifread psutil
```

Make sure ExifTool is installed:
- Ubuntu/Debian: `sudo apt install exiftool`
- macOS: `brew install exiftool`
- Windows: Download from https://exiftool.org/

## Benchmark Results

The benchmarks measure:

- **Total time** - Time to process all files
- **Average time** - Average time per file
- **Files per second** - Processing throughput
- **Success rate** - Percentage of files successfully processed
- **Metadata fields** - Number of EXIF fields extracted
- **Speedup** - Relative performance compared to fastest method

## Example Output

```
============================================================
RESULTS FOR 100 FILES
============================================================
Method               Total(s) Avg(s)   Files/s  Success% Fields   Speedup
--------------------------------------------------------------------------------
PIL/Pillow           1.54     0.015    64.8     100.0    5736     1.00x
Fast EXIF Reader     1.74     0.017    57.5     0.0      0        0.89x
ExifTool             98.01    0.980    1.0      100.0    26454    0.02x
```

## Performance Insights

Based on benchmark results:

1. **PIL/Pillow** - Fastest for basic EXIF extraction, good for Python workflows
2. **Fast EXIF Reader** - Very fast but currently limited EXIF support
3. **ExifTool** - Slowest but extracts the most comprehensive metadata

## Notes

- The Fast EXIF Reader is optimized for speed over completeness
- Results may vary based on file formats and system specifications
- Parallel processing is used for fair comparison across all methods
- Memory usage is not currently measured but could be added

## Troubleshooting

If you encounter issues:

1. Ensure all dependencies are installed
2. Check that ExifTool is in your PATH
3. Verify the target directory contains image files
4. Run `simple_test.py` to verify basic functionality

