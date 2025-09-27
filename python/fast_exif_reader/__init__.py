"""
Fast EXIF Reader - Optimized for Canon 70D and Nikon Z50 II

A high-performance EXIF metadata reader built in Rust with Python bindings.
Optimized for Canon 70D and Nikon Z50 II cameras in RAW, HIF, and JPEG formats.

This module provides both Python and Rust multiprocessing implementations:

Python Multiprocessing:
- Uses ProcessPoolExecutor for parallel processing
- Good for small to medium file counts
- Familiar Python API and error handling

Rust Multiprocessing:
- Uses Rayon for data parallelism
- Optimized for large file counts and high throughput
- Better memory efficiency and thread safety

Example:
    >>> from fast_exif_reader import FastExifReader
    >>> reader = FastExifReader()
    >>> metadata = reader.read_file("image.jpg")
    >>> print(metadata["Make"])
    Canon
    
    # Python multiprocessing
    >>> from fast_exif_reader import python_extract_exif_batch
    >>> results = python_extract_exif_batch(file_paths, max_workers=4)
    
    # Rust multiprocessing  
    >>> from fast_exif_reader import rust_process_files_parallel
    >>> results = rust_process_files_parallel(file_paths, max_workers=4)
"""

# Import from the compiled Rust module
try:
    from .fast_exif_reader import (
        FastExifReader,
        FastExifWriter,
        FastExifCopier,
        MultiprocessingExifReader as RustMultiprocessingExifReader,
        BatchExifWriter,
        process_files_parallel as rust_process_files_parallel,
        process_directory_parallel as rust_process_directory_parallel,
        write_exif_batch_parallel as rust_write_exif_batch_parallel,
        copy_exif_batch_parallel as rust_copy_exif_batch_parallel
    )
    RUST_AVAILABLE = True
except ImportError:
    RUST_AVAILABLE = False
    FastExifReader = None
    FastExifWriter = None
    FastExifCopier = None
    RustMultiprocessingExifReader = None
    BatchExifWriter = None
    rust_process_files_parallel = None
    rust_process_directory_parallel = None
    rust_write_exif_batch_parallel = None
    rust_copy_exif_batch_parallel = None

# Import Python multiprocessing functions
from .multiprocessing import (
    MultiprocessingExifReader as PythonMultiprocessingExifReader,
    extract_exif_batch as python_extract_exif_batch,
    extract_exif_from_directory as python_extract_exif_from_directory,
    read_multiple_files as python_read_multiple_files,
    read_directory as python_read_directory
)

# Convenience aliases for backward compatibility
MultiprocessingExifReader = PythonMultiprocessingExifReader
extract_exif_batch = python_extract_exif_batch
extract_exif_from_directory = python_extract_exif_from_directory
read_multiple_files = python_read_multiple_files
read_directory = python_read_directory

__version__ = "0.1.0"
__author__ = "Your Name"
__email__ = "your.email@example.com"
__license__ = "MIT"

__all__ = [
    # Core functionality
    "FastExifReader",
    "FastExifWriter",
    "FastExifCopier",
    "BatchExifWriter",
    
    # Python multiprocessing (default/backward compatible)
    "MultiprocessingExifReader",
    "extract_exif_batch", 
    "extract_exif_from_directory",
    "read_multiple_files",
    "read_directory",
    
    # Explicit Python multiprocessing
    "PythonMultiprocessingExifReader",
    "python_extract_exif_batch",
    "python_extract_exif_from_directory", 
    "python_read_multiple_files",
    "python_read_directory",
    
    # Rust multiprocessing (if available)
    "RustMultiprocessingExifReader",
    "rust_process_files_parallel",
    "rust_process_directory_parallel",
    "rust_write_exif_batch_parallel",
    "rust_copy_exif_batch_parallel",
    
    # Utility
    "RUST_AVAILABLE"
]

# Version info
VERSION = __version__
from . import _version
__version__ = _version.get_versions()['version']
