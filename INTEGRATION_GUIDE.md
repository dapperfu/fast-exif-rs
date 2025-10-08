# Integration Guide for fast-exif-reader

This crate is a **pure Rust library** ready for integration into other Rust projects.

## 🚀 Quick Integration

### 1. Add to Cargo.toml

```toml
[dependencies]
fast-exif-reader = { path = "../fast-exif-rs" }
# or from git:
# fast-exif-reader = { git = "https://github.com/dapperfu/fast-exif-rs" }
```

### 2. Basic Usage

```rust
use fast_exif_reader::{FastExifReader, ExifError};
use std::collections::HashMap;

fn main() -> Result<(), ExifError> {
    let mut reader = FastExifReader::new();
    
    // Read EXIF from file
    let metadata = reader.read_file("photo.jpg")?;
    
    // Extract specific fields
    if let Some(make) = metadata.get("Make") {
        println!("Camera: {}", make);
    }
    
    Ok(())
}
```

## 🔥 Advanced Usage

### Optimal Parser
```rust
use fast_exif_reader::parsers::OptimalExifParser;

// Automatic strategy selection
let mut parser = OptimalExifParser::new();
let metadata = parser.parse_file("photo.jpg")?;

// With specific target fields for maximum efficiency
let mut parser = OptimalExifParser::with_target_fields(
    vec!["Make".to_string(), "Model".to_string(), "DateTime".to_string()]
);
let metadata = parser.parse_file("photo.jpg")?;
```

### Batch Processing
```rust
use fast_exif_reader::parsers::OptimalBatchProcessor;

let mut processor = OptimalBatchProcessor::new(50);
let results = processor.process_files(&file_paths)?;
```

### Parallel Processing
```rust
use fast_exif_reader::FastExifReader;

let mut reader = FastExifReader::new();
let results = reader.read_files_parallel(file_paths)?;
```

## 📊 Performance Monitoring

```rust
use fast_exif_reader::parsers::OptimalExifParser;

let mut parser = OptimalExifParser::new();
let metadata = parser.parse_file("photo.jpg")?;

// Get performance statistics
let stats = parser.get_stats();
println!("Parser stats: {:?}", stats);
```

## 🎯 Format Support

The library supports comprehensive format detection and parsing:

- **JPEG** - Standard JPEG EXIF metadata
- **RAW** - Canon CR2, Nikon NEF, Olympus ORF, Adobe DNG
- **HEIF/HEIC** - Modern image format with EXIF
- **PNG** - PNG format with EXIF support
- **TIFF** - TIFF-based EXIF parsing
- **Video** - MOV, MP4, 3GP video format parsing
- **BMP** - BMP format parsing
- **MKV** - Matroska video format parsing

## 🔧 Error Handling

```rust
use fast_exif_reader::{FastExifReader, ExifError};

let mut reader = FastExifReader::new();

match reader.read_file("photo.jpg") {
    Ok(metadata) => {
        println!("Found {} EXIF fields", metadata.len());
    }
    Err(ExifError::InvalidExif(msg)) => {
        println!("Invalid EXIF data: {}", msg);
    }
    Err(ExifError::IoError(err)) => {
        println!("File I/O error: {}", err);
    }
    Err(e) => {
        println!("Other error: {:?}", e);
    }
}
```

## 📝 Writing EXIF Data

```rust
use fast_exif_reader::{FastExifWriter, ExifError};
use std::collections::HashMap;

let writer = FastExifWriter::new();
let mut metadata = HashMap::new();
metadata.insert("Make".to_string(), "Canon".to_string());
metadata.insert("Model".to_string(), "EOS R5".to_string());

writer.write_exif("input.jpg", "output.jpg", &metadata)?;
```

## 📋 Copying EXIF Data

```rust
use fast_exif_reader::{FastExifCopier, ExifError};

let mut copier = FastExifCopier::new();
copier.copy_high_priority_exif("source.jpg", "target.jpg", "output.jpg")?;
```

## 🚀 Performance Tips

1. **Use OptimalExifParser** - Automatically chooses the best strategy
2. **Specify Target Fields** - Only parse what you need
3. **Batch Processing** - Process multiple files together
4. **Monitor Performance** - Use statistics to optimize

## 📚 Examples

See the `examples/` directory for complete working examples:

- `basic_usage.rs` - Basic EXIF reading and writing
- Additional examples demonstrate advanced usage patterns

## 🔗 API Reference

The main API components:

- `FastExifReader` - Main EXIF reading interface
- `OptimalExifParser` - High-performance parser with automatic optimization
- `OptimalBatchProcessor` - Batch processing for multiple files
- `FastExifWriter` - EXIF writing capabilities
- `FastExifCopier` - EXIF copying between files

## 🎉 Ready to Use

The library is production-ready with comprehensive error handling, performance optimization, and extensive format support. Simply add it to your `Cargo.toml` and start reading EXIF metadata!