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

from .fast_exif_reader import FastExifReader

__version__ = "0.1.0"
__author__ = "Your Name"
__email__ = "your.email@example.com"
__license__ = "MIT"

__all__ = ["FastExifReader"]

# Version info
VERSION = __version__