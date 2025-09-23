# Fast EXIF CLI

A high-performance command-line interface for reading EXIF metadata from image files, built on top of the fast-exif-rs Rust backend.

## Features

- **High Performance**: Leverages Rust backend for fast metadata extraction
- **ExifTool Compatible**: Mimics exiftool's output formats and command-line interface
- **Multiple Formats**: Supports JPEG, CR2, NEF, HEIF, and other image formats
- **Flexible Output**: Multiple output formats including JSON, tab-delimited, and table formats
- **Recursive Processing**: Process entire directories of images

## Installation

```bash
# Install the package (includes CLI)
pip install -e .

# Or install CLI dependencies separately
pip install -r cli/requirements.txt
```

## Usage

### Basic Usage

```bash
# Read EXIF data from a single image
fast-exif-cli image.jpg

# Process multiple files
fast-exif-cli *.jpg

# Process directory recursively
fast-exif-cli -r /path/to/photos
```

### Output Formats

```bash
# Default format (Tag Name: Value)
fast-exif-cli image.jpg

# Short format (TagName: Value)
fast-exif-cli -s image.jpg

# Very short format (Value only)
fast-exif-cli -S image.jpg

# Tab-delimited format
fast-exif-cli -t image.jpg

# Table format
fast-exif-cli -T image.jpg

# JSON format
fast-exif-cli -j image.jpg
```

### Advanced Options

```bash
# Verbose output
fast-exif-cli -v image.jpg

# Quiet mode (suppress errors)
fast-exif-cli -q image.jpg

# Filter by file extensions
fast-exif-cli --ext jpg --ext cr2 /path/to/photos

# Recursive processing with specific extensions
fast-exif-cli -r --ext jpg --ext cr2 --ext nef /path/to/photos
```

## Command Line Options

- `-s, --short`: Short format output
- `-S, --very-short`: Very short format output
- `-t, --tab-delimited`: Tab-delimited format
- `-T, --table`: Table format
- `-j, --json`: JSON format
- `-v, --verbose`: Verbose output
- `-q, --quiet`: Suppress error messages
- `-r, --recursive`: Process directories recursively
- `--ext`: File extensions to process (can be used multiple times)
- `--version`: Show version information
- `--help`: Show help message

## Examples

### Basic EXIF Reading

```bash
$ fast-exif-cli sample.jpg
======== sample.jpg

Format: JPEG
Make: Canon
Model: EOS 70D
DateTime: 2023:01:15 14:30:25
ExposureTime: 1/125
FNumber: 4.0
ISOSpeedRatings: 400
FocalLength: 50
```

### JSON Output

```bash
$ fast-exif-cli -j sample.jpg
{
  "SourceFile": "sample.jpg",
  "ExifToolVersion": "fast-exif-cli 0.1.0",
  "Format": "JPEG",
  "Make": "Canon",
  "Model": "EOS 70D",
  "DateTime": "2023:01:15 14:30:25",
  "ExposureTime": "1/125",
  "FNumber": "4.0",
  "ISOSpeedRatings": "400",
  "FocalLength": "50"
}
```

### Batch Processing

```bash
# Process all images in a directory
$ fast-exif-cli -r --ext jpg --ext cr2 --ext nef /photos/

# Process with specific output format
$ fast-exif-cli -t *.jpg > metadata.txt
```

## Performance

The CLI tool leverages the high-performance Rust backend, providing:

- **Fast Processing**: Optimized for Canon 70D and Nikon Z50 II cameras
- **Memory Efficient**: Uses memory-mapped files for large images
- **Parallel Processing**: Supports batch processing of multiple files

## Compatibility

- **File Formats**: JPEG, CR2 (Canon RAW), NEF (Nikon RAW), HEIF, HIF, TIFF
- **Operating Systems**: Linux, macOS, Windows
- **Python Versions**: 3.8+

## Error Handling

The CLI provides comprehensive error handling:

- File not found errors
- Unsupported format errors
- Permission errors (with recursive processing)
- Invalid EXIF data errors

Use `-q` flag to suppress error messages, or `-v` for verbose error reporting.

## Development

To run the CLI in development mode:

```bash
# Make the script executable
chmod +x fast-exif-cli

# Run directly
./fast-exif-cli image.jpg

# Or use Python module
python -m cli.fast_exif_cli image.jpg
```
