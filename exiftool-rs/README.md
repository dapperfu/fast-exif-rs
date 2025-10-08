# EXIF Tool RS

A fast EXIF metadata extraction tool written in Rust, built on top of the `fast-exif-reader` library.

## Features

- **High Performance**: Built with Rust for maximum speed
- **Short Tags**: Compact output with abbreviated tag names
- **Known Parameters**: Comprehensive EXIF field definitions
- **Multiple Formats**: Text, JSON, and CSV output
- **Batch Processing**: Process multiple files and directories
- **Recursive Scanning**: Deep directory traversal
- **Performance Benchmarking**: Built-in speed testing and analysis
- **20+ Formats**: Support for images and videos

## Installation

```bash
cd exiftool-rs
cargo build --release
```

## Usage

### Basic Usage

```bash
# Extract EXIF data from a single file
./target/release/exiftool-rs extract photo.jpg

# Extract with short tags
./target/release/exiftool-rs extract photo.jpg --short

# Extract from multiple files
./target/release/exiftool-rs extract photo1.jpg photo2.jpg photo3.jpg

# Process directory recursively
./target/release/exiftool-rs extract /path/to/photos --recursive
```

### Output Formats

```bash
# Text format (default)
./target/release/exiftool-rs extract photo.jpg

# JSON format
./target/release/exiftool-rs extract photo.jpg --format json

# CSV format
./target/release/exiftool-rs extract photo.jpg --format csv
```

### Filtering and Options

```bash
# Show only specific tags
./target/release/exiftool-rs extract photo.jpg --tags Make Model DateTime

# Show filenames
./target/release/exiftool-rs extract photo.jpg --filenames

# Quiet mode (minimal output)
./target/release/exiftool-rs extract photo.jpg --quiet
```

### List Known Tags

```bash
# List all known EXIF tags
./target/release/exiftool-rs list-tags

# List with short names only
./target/release/exiftool-rs list-tags --short

# Filter by category
./target/release/exiftool-rs list-tags --category "Camera Settings"
```

### Show Tool Information

```bash
./target/release/exiftool-rs info
```

### Benchmark Performance

```bash
# Benchmark a single file
./target/release/exiftool-rs benchmark photo.jpg

# Benchmark multiple files
./target/release/exiftool-rs benchmark photo1.jpg photo2.jpg photo3.jpg

# Benchmark a directory recursively
./target/release/exiftool-rs benchmark /path/to/photos --recursive

# Run multiple iterations for more accurate timing
./target/release/exiftool-rs benchmark /path/to/photos --iterations 5

# Show detailed per-file timing
./target/release/exiftool-rs benchmark /path/to/photos --detailed

# Output benchmark results in JSON format
./target/release/exiftool-rs benchmark /path/to/photos --format json

# Output benchmark results in CSV format
./target/release/exiftool-rs benchmark /path/to/photos --format csv
```

## Supported Formats

### Image Formats
- JPEG, CR2, NEF, ARW, RAF, SRW, PEF, RW2, ORF, DNG
- HEIF/HEIC, PNG, BMP, GIF, WEBP

### Video Formats
- MOV, MP4, 3GP, AVI, WMV, WEBM, MKV

## EXIF Tag Categories

- **Camera**: Make, Model, SerialNumber
- **Image**: ImageWidth, ImageHeight, Orientation
- **DateTime**: DateTime, DateTimeOriginal, DateTimeDigitized
- **Camera Settings**: ExposureTime, FNumber, ISO, FocalLength, Flash, WhiteBalance
- **GPS**: GPSLatitude, GPSLongitude, GPSAltitude
- **File**: FileName, FileSize, Directory, SourceFile

## Examples

### Extract Basic Information
```bash
$ ./target/release/exiftool-rs extract photo.jpg --short --tags Make Model DateTime
Make: Canon
Model: EOS R5
DateTime: 2024:01:15 14:30:25
```

### JSON Output
```bash
$ ./target/release/exiftool-rs extract photo.jpg --format json
[
  {
    "filename": "photo.jpg",
    "metadata": {
      "Make": "Canon",
      "Model": "EOS R5",
      "DateTime": "2024:01:15 14:30:25",
      "ExposureTime": "1/125",
      "FNumber": "2.8",
      "ISO": "100"
    }
  }
]
```

### Batch Processing
```bash
$ ./target/release/exiftool-rs extract /photos --recursive --format csv
filename,tag,value
photo1.jpg,Make,Canon
photo1.jpg,Model,EOS R5
photo2.jpg,Make,Nikon
photo2.jpg,Model,D850
```

### Performance Benchmarking
```bash
$ ./target/release/exiftool-rs benchmark /photos --recursive --detailed

EXIF Extraction Benchmark
=========================
Files to process: 1,247
Iterations: 1

Benchmark Results
================
Total files processed: 1,247
Iterations: 1
Total time: 0.089s
Successful files: 1,247
Total EXIF fields: 15,234
Success rate: 100.0%

Performance Metrics
==================
Files per second: 14,011.2
Fields per second: 171,168
Average time per file: 0.071ms

Detailed File Timings
=====================
  IMG_001.CR2: 0.234ms (23 fields)
  IMG_002.NEF: 0.198ms (19 fields)
  IMG_003.ARW: 0.156ms (21 fields)
  ...
```

## Performance

Built on the high-performance `fast-exif-reader` library:
- **55.6x faster** than standard EXIF libraries
- **Automatic parallelization** for batch processing
- **Memory-mapped** file access
- **SIMD acceleration** for parsing

## License

MIT License - see LICENSE file for details.
