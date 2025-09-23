#!/usr/bin/env python3
"""
Simple test script for the benchmark
"""

import os
import sys
import tempfile
from PIL import Image

# Add the parent directory to path to import our module
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from fast_exif_reader import FastExifReader
    FAST_EXIF_AVAILABLE = True
    print("✓ Fast EXIF Reader available")
except ImportError as e:
    print(f"✗ Fast EXIF Reader not available: {e}")
    FAST_EXIF_AVAILABLE = False

try:
    from PIL import Image
    PIL_AVAILABLE = True
    print("✓ PIL/Pillow available")
except ImportError as e:
    print(f"✗ PIL/Pillow not available: {e}")
    PIL_AVAILABLE = False


def create_test_image(filename: str):
    """Create a simple test JPEG image"""
    # Create a simple 100x100 RGB image
    img = Image.new('RGB', (100, 100), color='red')
    
    # Save without EXIF first
    img.save(filename)
    print(f"Created test image: {filename}")


def test_fast_exif_reader():
    """Test our Fast EXIF Reader"""
    if not FAST_EXIF_AVAILABLE:
        return False
    
    print("\nTesting Fast EXIF Reader...")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        test_file = os.path.join(temp_dir, "test.jpg")
        create_test_image(test_file)
        
        try:
            reader = FastExifReader()
            metadata = reader.read_file(test_file)
            print(f"  Successfully read {len(metadata)} metadata fields")
            for key, value in list(metadata.items())[:3]:  # Show first 3 fields
                print(f"    {key}: {value}")
            return True
        except Exception as e:
            print(f"  Error: {e}")
            return False


def test_pil():
    """Test PIL EXIF reading"""
    if not PIL_AVAILABLE:
        return False
    
    print("\nTesting PIL...")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        test_file = os.path.join(temp_dir, "test.jpg")
        create_test_image(test_file)
        
        try:
            with Image.open(test_file) as img:
                exif_data = img._getexif()
                if exif_data:
                    print(f"  Successfully read {len(exif_data)} EXIF fields")
                    return True
                else:
                    print("  No EXIF data found")
                    return False
        except Exception as e:
            print(f"  Error: {e}")
            return False


def main():
    print("Testing benchmark components...")
    
    # Test PIL
    pil_ok = test_pil()
    
    # Test Fast EXIF Reader
    fast_exif_ok = test_fast_exif_reader()
    
    if pil_ok and fast_exif_ok:
        print("\n✓ All tests passed!")
        print("\nYou can now run the benchmark with:")
        print("  python quick_benchmark.py")
        print("  python parallel_exif_benchmark.py /path/to/images")
        return 0
    else:
        print("\n✗ Some tests failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())
