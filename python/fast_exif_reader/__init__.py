"""
Fast EXIF Reader - Optimized for Canon 70D and Nikon Z50 II

A high-performance EXIF metadata reader built in Rust with Python bindings.
Optimized for Canon 70D and Nikon Z50 II cameras in RAW, HIF, and JPEG formats.

Example:
    >>> from fast_exif_reader import FastExifReader
    >>> reader = FastExifReader()
    >>> metadata = reader.read_file("image.jpg")
    >>> print(metadata["Make"])
    Canon
"""

# Import from the compiled Rust module
try:
    from .fast_exif_reader import (
        FastExifReader,
        MultiprocessingExifReader,
        process_files_parallel,
        process_directory_parallel
    )
except ImportError:
    # Fallback to Python implementation if Rust module not available
    from .multiprocessing import (
        MultiprocessingExifReader,
        extract_exif_batch,
        extract_exif_from_directory,
        read_multiple_files,
        read_directory
    )
    FastExifReader = None
    process_files_parallel = None
    process_directory_parallel = None

# Also import Python multiprocessing functions for comparison
from .multiprocessing import (
    extract_exif_batch as python_extract_exif_batch,
    extract_exif_from_directory as python_extract_exif_from_directory,
    read_multiple_files,
    read_directory as python_read_directory
)

__version__ = "0.1.0"
__author__ = "Your Name"
__email__ = "your.email@example.com"
__license__ = "MIT"

__all__ = [
    "FastExifReader",
    "MultiprocessingExifReader", 
    "process_files_parallel",
    "process_directory_parallel",
    "extract_exif_batch",
    "extract_exif_from_directory",
    "read_multiple_files",
    "read_directory",
    "python_extract_exif_batch",
    "python_extract_exif_from_directory",
    "python_read_directory"
]

# Version info
VERSION = __version__
from . import _version
__version__ = _version.get_versions()['version']
