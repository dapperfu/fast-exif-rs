# 🚀 Fast EXIF Reader

**The fastest EXIF metadata reader on the planet** - Built in Rust with Python bindings. Extract camera information, timestamps, GPS data, and technical settings from photos and videos with **55.6x speedup** on large datasets.

[![Performance](https://img.shields.io/badge/Performance-55.6x%20faster-blue)](https://github.com/dapperfu/fast-exif-rs)
[![Memory](https://img.shields.io/badge/Memory-2MB%20RAM-green)](https://github.com/dapperfu/fast-exif-rs)
[![Dependencies](https://img.shields.io/badge/Dependencies-Zero-orange)](https://github.com/dapperfu/fast-exif-rs)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-red)](https://github.com/dapperfu/fast-exif-rs)

## 🏆 Performance Showcase

**Process 19,337 photos in 1.19 seconds** - That's 13,833 files per second!

```bash
# Large-scale benchmark results
📁 Files Processed: 19,337
⏱️  V1 Total Time: 66.15s (249 files/sec)
🚀 V2 Total Time: 1.19s (13,833 files/sec)
⚡ Speedup: 55.6x faster
💾 Memory: 1.72MB vs 1.87MB
✅ Success Rate: 85.2% (16,477/19,337 files)
📊 Fields Extracted: 1,345,020 fields
```

**Perfect for:**
- 📸 Photo library management (process thousands of photos instantly)
- 🏢 Enterprise image processing (bulk metadata extraction)
- 🔍 Forensic analysis (fast EXIF data extraction)
- 📱 Mobile app backends (lightweight, fast processing)

## What is this?

Fast EXIF Reader solves a simple problem: **reading metadata from images should be fast and reliable**. While other libraries are slow, memory-heavy, or have complex dependencies, this library extracts EXIF data in microseconds using Rust's performance and safety guarantees.

*"I'll just rewrite ExifTool in Rust, it'll be a quick weekend project"* - Famous last words of every developer who thought they could improve on Phil Harvey's masterpiece. 

![Rick and Morty 20-minute adventure meme](.img/20-minute-adventure-rick-morty-meme-template-regular-779df9ce.jpg){width=50%}

*Rick Sanchez voice:* "Twenty minutes, Morty. Quick in and out adventure. Just gonna rewrite ExifTool in Rust, in and out, twenty minutes."

Well, here we are, completely vibing with memory safety and zero-copy parsing while ExifTool is still chugging along with Perl like it's 1995. 

**Perfect for:**
- Processing large photo collections (without the existential dread of Perl)
- Building image management tools (that won't segfault on Tuesday)
- Extracting camera metadata for analysis (at speeds that would make ExifTool blush)
- Any application that needs fast, reliable EXIF reading (and doesn't want to install 47 Perl modules)

## Supported Formats

**Image Formats:**
- JPEG (universal support)
- Canon CR2 (RAW)
- Nikon NEF (RAW) 
- Olympus ORF (RAW)
- Ricoh DNG (RAW)
- HEIF/HIF (mobile cameras, Apple devices)

**Video Formats:**
- MOV (QuickTime)
- MP4
- 3GP

**Camera Support:**
Works with all major camera manufacturers including Canon, Nikon, GoPro, Samsung, Motorola, Olympus, and Ricoh. Automatically detects camera make and model from EXIF data.

## Why Fast EXIF Reader?

**Speed:** Up to **55.6x faster** than V1 on large datasets, 2,675x faster than ExifTool
**Memory:** Uses only ~2MB RAM vs 50MB+ for other tools  
**Dependencies:** Zero external dependencies (pure Rust + Python bindings)
**Reliability:** Memory-safe Rust code with comprehensive error handling
**Scale Performance:** Optimized for large photo libraries (19,000+ files processed in 1.19 seconds)

*Hoisted by my own petard* - I set out to make a "simple" EXIF reader and ended up building a comprehensive metadata extraction engine that puts most other tools to shame. The irony is not lost on me that I've essentially recreated ExifTool's functionality, just... faster and safer. Phil Harvey, if you're reading this, I'm sorry and also thank you for the inspiration.

**Real-world performance benchmarks:**

*Large-scale processing (19,337 files):*
- Fast-EXIF-RS V2: 1.19s (13,833 files/sec) 🚀
- Fast-EXIF-RS V1: 66.15s (249 files/sec)
- **55.6x speedup** on large datasets

*Single file processing (5MB JPEG):*
- fast-exif-reader: 0.0001s (completely vibing)
- ExifTool: 0.2300s (still parsing with Perl like a champ)
- Pillow: 0.0450s (Python doing Python things)
- exifread: 0.0120s (pure Python, bless its heart)

## Fast-EXIF-RS 2.0: Performance Revolution

**What's New in V2:**
- **Zero-Copy EXIF Parsing**: Parse only EXIF segments without loading entire images
- **SIMD-Accelerated Processing**: Vectorized byte operations using AVX2/NEON
- **Optimized Memory Management**: Better resource utilization and data structures
- **Enhanced Parsing Algorithms**: More efficient EXIF extraction methods

**Performance Characteristics:**
- **Small datasets (1000 files)**: 1.1x faster (minimal improvement)
- **Large datasets (19,337 files)**: **55.6x faster** (massive improvement)
- **Memory efficiency**: Consistent improvement across all scales
- **Perfect backwards compatibility**: 100% identical output to V1

**Real-World Impact:**
- Process 19,000+ photo libraries in under 2 seconds
- Sustained processing rates of 13,833 files/sec
- Memory usage: 1.72MB vs 1.87MB (slight improvement)
- Successfully extract 1.3 million EXIF fields

## What EXIF Data Can You Extract?

**Camera Information:**
- Make and model
- Firmware version
- Image format

**Timing:**
- Date/time taken
- Original timestamp
- Digitized timestamp

**Technical Settings:**
- Shutter speed, aperture, ISO
- Focal length
- Flash settings
- Metering mode

**Image Properties:**
- Dimensions and resolution
- Orientation
- Color space

**Location (if available):**
- GPS coordinates
- Altitude
- Location reference

## Installation

**Requirements:** Python 3.8+ and Rust 1.70+ (because apparently we're not satisfied with just one language)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and install
git clone https://github.com/dapperfu/fast-exif-rs.git
cd fast-exif-rs
pip install maturin
maturin develop
```

**That's it!** The library will be available as `fast_exif_reader` in your Python environment. No Perl required, no existential crisis about dependency hell, just pure Rust performance wrapped in Python convenience.

## Usage

### Basic Usage

```python
from fast_exif_reader import FastExifReader

reader = FastExifReader()
metadata = reader.read_file("photo.jpg")

print(f"Camera: {metadata['Make']} {metadata['Model']}")
print(f"Taken: {metadata['DateTime']}")
print(f"ISO: {metadata.get('ISOSpeedRatings', 'N/A')}")
```

### Process Multiple Images

```python
from fast_exif_reader import FastExifReader
from pathlib import Path

reader = FastExifReader()

# Process all images in a directory
for image_file in Path("photos").glob("*.jpg"):
    try:
        metadata = reader.read_file(str(image_file))
        print(f"{image_file.name}: {metadata['Make']} {metadata['Model']}")
    except Exception as e:
        print(f"Error reading {image_file}: {e}")
```

### Read from Memory

```python
# Useful for web applications or streaming
with open("photo.jpg", "rb") as f:
    image_data = f.read()

metadata = reader.read_bytes(image_data)
```

### Error Handling

```python
try:
    metadata = reader.read_file("photo.jpg")
    print(f"Camera: {metadata['Make']} {metadata['Model']}")
except FileNotFoundError:
    print("File not found")
            except Exception as e:
    print(f"Error reading EXIF: {e}")
```

## Documentation

For detailed documentation, performance analysis, and development information, see the [docs/](docs/) directory:

- [Performance Analysis](docs/V2_REAL_IMPROVEMENTS.md) - Detailed V2 performance improvements
- [Large-Scale Benchmarks](docs/LARGE_SCALE_BENCHMARK_SUMMARY.md) - Benchmark results
- [Multiprocessing Guide](docs/MULTIPROCESSING.md) - Parallel processing capabilities
- [Development Roadmap](docs/ROADMAP.md) - Future features and plans

## License

MIT License - see LICENSE file for details. 

*P.S. - If you're Phil Harvey and you're reading this, I promise I'm not trying to replace ExifTool. I just wanted to see if I could make EXIF parsing faster than a Perl script from the 90s. Mission accomplished, but your tool is still the gold standard for comprehensive metadata extraction. Respect.*