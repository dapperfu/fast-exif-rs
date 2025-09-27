# Project Structure

This document describes the organized structure of the fast-exif-rs project.

## Top-Level Directory

The top-level directory contains only essential project files:

```
fast-exif-rs/
├── README.md                 # Main project documentation
├── LICENSE                   # Project license
├── INSTALL.md               # Installation instructions
├── Makefile                 # Build automation
├── Cargo.toml               # Rust project configuration
├── Cargo.lock               # Rust dependency lock file
├── pyproject.toml           # Python project configuration
├── setup.py                 # Python setup script
├── MANIFEST.in              # Python package manifest
├── versioneer.py            # Version management
├── build.rs                 # Rust build script
├── build.sh                 # Build script
└── requirements.txt          # Python dependencies
```

## Organized Subdirectories

### `src/` - Source Code
```
src/
├── lib.rs                   # Main library entry point
├── types.rs                 # Type definitions
├── utils.rs                 # Utility functions
├── format_detection.rs      # Original format detection
├── enhanced_format_detection.rs  # Enhanced format detection
├── enhanced_raw_parser.rs   # Additional RAW format parsers
├── enhanced_video_parser.rs # Additional video format parsers
├── enhanced_image_parser.rs # Additional image format parsers
├── multiprocessing.rs       # Parallel processing
├── writer.rs                # EXIF writing functionality
├── exif_copier.rs           # EXIF copying functionality
├── batch_writer.rs          # Batch writing operations
├── v2_reader.rs             # Version 2 reader
└── parsers/                 # Format-specific parsers
    ├── mod.rs
    ├── jpeg.rs
    ├── raw.rs
    ├── heif.rs
    ├── video.rs
    ├── png.rs
    ├── bmp.rs
    ├── tiff.rs
    ├── mkv.rs
    ├── maker_notes.rs
    ├── selective.rs
    ├── simd.rs
    └── zero_copy.rs
```

### `docs/` - Documentation
```
docs/
├── README.md                # Documentation index
├── PERFORMANCE.md           # Performance documentation
├── MULTIPROCESSING.md       # Multiprocessing guide
├── EXIF_WRITING.md          # EXIF writing guide
├── ENHANCED_EXIF_WRITING.md # Enhanced EXIF writing
├── FAST_EXIF_RS_V2_SUMMARY.md # Version 2 summary
├── ImplementationChecklist.md # Implementation checklist
├── LARGE_SCALE_BENCHMARK_SUMMARY.md # Large scale benchmarks
├── RUST_VS_PYTHON_ANALYSIS.md # Rust vs Python analysis
├── V2_REAL_IMPROVEMENTS.md  # Version 2 improvements
├── ROADMAP.md               # Project roadmap
├── analysis/                # Analysis documents
│   ├── EXIF_WRITE_ANALYSIS.md
│   ├── EXIF_WRITE_SUMMARY.md
│   ├── EXIFTOOL_IMPLEMENTATION_PLAN.md
│   ├── EXIFTOOL_REVERSE_ENGINEERING_GUIDE.md
│   └── JPEG_EXIF_WRITING_SUMMARY.md
├── benchmarks/              # Benchmark results and scripts
│   ├── benchmark_exif_writing.py
│   ├── simple_exif_benchmark.py
│   ├── test_format_support.py
│   ├── EXIF_WRITING_BENCHMARK_RESULTS.md
│   ├── ENHANCED_FORMAT_SUPPORT_SUMMARY.md
│   ├── exif_writing_benchmark_results.json
│   └── format_support_test_results.json
└── implementation/          # Implementation details
```

### `tests/` - Test Suite
```
tests/
├── test_camera_support.py
├── test_cli.py
├── test_heif_timestamps.py
├── test_performance.py
├── test_pickle.py
├── test_tiff_validation.py
├── test_comprehensive_exif_copying.py
├── test_exiftool_compatibility_comprehensive.py
└── test_exiftool_compatibility.py
```

### `examples/` - Usage Examples
```
examples/
├── simple_usage.py
├── exif_writing_example.py
├── enhanced_exif_writing_example.py
└── large_scale_exif_processing.py
```

### `cli/` - Command Line Interface
```
cli/
├── __init__.py
├── fast_exif_cli.py
├── README.md
└── requirements.txt
```

### `python/` - Python Package
```
python/
└── fast_exif_reader/
    ├── __init__.py
    ├── _version.py
    ├── multiprocessing.py
    ├── fast_exif_reader.cpython-312-x86_64-linux-gnu.so
    └── cli/
        ├── __init__.py
        ├── fast_exif_cli.py
        ├── README.md
        └── requirements.txt
```

### `scripts/` - Build and Utility Scripts
```
scripts/
├── build.sh
├── test_build.py
└── analyze_exiftool.sh
```

### `benchmarks/` - Performance Benchmarks
```
benchmarks/
├── README.md
├── requirements.txt
└── v2.0/
    ├── benchmark_results.json
    ├── demonstrate_v2_features.py
    ├── focused_v2_benchmark_results.json
    ├── focused_v2_benchmark.py
    ├── large_scale_benchmark_results.json
    ├── large_scale_benchmark.py
    └── performance_benchmark.py
```

### `test_data/` - Test Data Files
```
test_data/
├── output_with_exif.jpg
├── sample_image.jpg
└── target_image.jpg
```

### `test_files/` - Test Media Files (Gitignored)
```
test_files/
└── essential/
    ├── canon_eos_70d_2015.cr2
    ├── canon_eos_70d_2018.cr2
    ├── canon_powershot_sx280_hs_2021.jpg
    ├── canon_powershot_sx280_hs_video_2021.mp4
    ├── motorola_moto_g6_2019.jpg
    ├── nikon_coolpix_w100_2019.jpg
    ├── ricoh_theta_v_2019.jpg
    ├── samsung_galaxy_note4_2018.jpg
    └── samsung_galaxy_s10_2021.heic
```

## File Organization Principles

1. **Top-level cleanliness**: Only essential project files at the root
2. **Logical grouping**: Related files grouped in appropriate directories
3. **Clear naming**: Descriptive directory and file names
4. **Documentation**: Comprehensive documentation in `docs/`
5. **Separation of concerns**: Source code, tests, examples, and docs separated
6. **Gitignore compliance**: Temporary and generated files properly ignored

## Benefits of This Structure

- **Easier navigation**: Clear separation of different types of files
- **Better maintainability**: Related files grouped together
- **Cleaner repository**: Top-level directory is uncluttered
- **Professional appearance**: Follows standard project organization patterns
- **Easier onboarding**: New contributors can quickly understand the structure
- **Better tooling**: IDEs and tools work better with organized structures
