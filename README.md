# Fast EXIF Reader

A high-performance EXIF metadata reader built in Rust with Python bindings, optimized for Canon 70D and Nikon Z50 II cameras.

## Features

- **Ultra-fast EXIF parsing** - Optimized for Canon 70D and Nikon Z50 II
- **Multiple format support** - RAW (CR2/NEF), HIF, and JPEG
- **Python bindings** - Easy integration with Python applications
- **Memory efficient** - Uses memory mapping for large files
- **Camera-specific optimizations** - Tailored for target cameras

## Performance

This library delivers exceptional performance improvements over ExifTool:

- **2,675x faster** than ExifTool on average
- **Sub-millisecond** EXIF reading times
- **Optimized for Nikon Z50 II** and Canon 70D cameras

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/dapperfu/fast-exif-rs.git
cd fast-exif-rs

# Install maturin for building Python extensions
pip install maturin

# Build and install
maturin develop
```

## Usage

```python
from fast_exif_reader import FastExifReader

# Create reader instance
reader = FastExifReader()

# Read from file
metadata = reader.read_file("image.jpg")
print(metadata)
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
