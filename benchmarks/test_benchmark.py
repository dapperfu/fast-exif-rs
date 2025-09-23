#!/usr/bin/env python3
"""
Test script to verify benchmark functionality
Creates sample image files and tests the benchmark
"""

import os
import sys
import tempfile
from PIL import Image
import numpy as np

# Add the parent directory to path to import our module
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from fast_exif_reader import FastExifReader
    FAST_EXIF_AVAILABLE = True
except ImportError:
    print("Warning: fast_exif_reader not available")
    FAST_EXIF_AVAILABLE = False


def create_test_images(directory: str, count: int = 10):
    """Create test JPEG images with EXIF data"""
    print(f"Creating {count} test images in {directory}")
    
    os.makedirs(directory, exist_ok=True)
    
    for i in range(count):
        # Create a simple test image
        img_array = np.random.randint(0, 255, (100, 100, 3), dtype=np.uint8)
        img = Image.fromarray(img_array)
        
        # Add some EXIF data
        exif_dict = {
            271: "Test Camera",  # Make
            272: f"Test Model {i}",  # Model
            306: "2023:01:01 12:00:00",  # DateTime
            33434: (1, 125),  # ExposureTime
            33437: (56, 10),  # FNumber
            34855: 400,  # ISOSpeedRatings
            37386: (50, 1),  # FocalLength
        }
        
        # Save with EXIF
        filename = os.path.join(directory, f"test_image_{i:03d}.jpg")
        img.save(filename, exif=exif_dict)
    
    print(f"Created {count} test images")


def test_fast_exif_reader():
    """Test our Fast EXIF Reader"""
    if not FAST_EXIF_AVAILABLE:
        print("Fast EXIF Reader not available")
        return False
    
    print("\nTesting Fast EXIF Reader...")
    
    # Create test directory
    with tempfile.TemporaryDirectory() as temp_dir:
        create_test_images(temp_dir, 5)
        
        # Test our reader
        reader = FastExifReader()
        
        for filename in os.listdir(temp_dir):
            if filename.endswith('.jpg'):
                file_path = os.path.join(temp_dir, filename)
                try:
                    metadata = reader.read_file(file_path)
                    print(f"  {filename}: {len(metadata)} metadata fields")
                    for key, value in list(metadata.items())[:3]:  # Show first 3 fields
                        print(f"    {key}: {value}")
                except Exception as e:
                    print(f"  Error reading {filename}: {e}")
                    return False
    
    print("✓ Fast EXIF Reader test passed")
    return True


def test_pil():
    """Test PIL EXIF reading"""
    print("\nTesting PIL...")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        create_test_images(temp_dir, 3)
        
        for filename in os.listdir(temp_dir):
            if filename.endswith('.jpg'):
                file_path = os.path.join(temp_dir, filename)
                try:
                    with Image.open(file_path) as img:
                        exif_data = img._getexif()
                        if exif_data:
                            print(f"  {filename}: {len(exif_data)} EXIF fields")
                        else:
                            print(f"  {filename}: No EXIF data")
                except Exception as e:
                    print(f"  Error reading {filename}: {e}")
                    return False
    
    print("✓ PIL test passed")
    return True


def main():
    print("Testing benchmark components...")
    
    # Test PIL
    if not test_pil():
        print("✗ PIL test failed")
        return 1
    
    # Test Fast EXIF Reader
    if not test_fast_exif_reader():
        print("✗ Fast EXIF Reader test failed")
        return 1
    
    print("\n✓ All tests passed!")
    print("\nYou can now run the benchmark with:")
    print("  python quick_benchmark.py")
    print("  python parallel_exif_benchmark.py /path/to/images")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())

