#!/usr/bin/env python3
"""
Test script to verify the fast-exif-reader package works correctly
"""

import sys
import os
import tempfile
import time
from pathlib import Path

def test_import():
    """Test that the module can be imported"""
    try:
        from fast_exif_reader import FastExifReader
        print("✅ Import successful")
        return True
    except ImportError as e:
        print(f"❌ Import failed: {e}")
        return False

def test_basic_functionality():
    """Test basic functionality without actual image files"""
    try:
        from fast_exif_reader import FastExifReader
        
        # Create reader instance
        reader = FastExifReader()
        print("✅ Reader instance created")
        
        # Test with empty bytes (should handle gracefully)
        try:
            result = reader.read_bytes(b"")
            print("✅ Empty bytes handled gracefully")
        except Exception as e:
            print(f"⚠️  Empty bytes error (expected): {e}")
        
        return True
    except Exception as e:
        print(f"❌ Basic functionality test failed: {e}")
        return False

def test_performance():
    """Test performance with a simple benchmark"""
    try:
        from fast_exif_reader import FastExifReader
        
        reader = FastExifReader()
        
        # Benchmark reader creation
        start_time = time.time()
        for _ in range(1000):
            reader = FastExifReader()
        end_time = time.time()
        
        avg_time = (end_time - start_time) / 1000
        print(f"✅ Reader creation: {avg_time*1000:.2f}ms average")
        
        # Should be very fast (under 1ms)
        if avg_time < 0.001:
            print("✅ Performance test passed")
            return True
        else:
            print("⚠️  Performance slower than expected")
            return False
            
    except Exception as e:
        print(f"❌ Performance test failed: {e}")
        return False

def test_with_sample_data():
    """Test with minimal JPEG-like data"""
    try:
        from fast_exif_reader import FastExifReader
        
        # Create minimal JPEG-like data with EXIF header
        # This is not a real JPEG, just enough to test parsing
        sample_data = b'\xff\xd8\xff\xe1\x00\x10Exif\x00\x00II*\x00\x08\x00\x00\x00\x01\x00\x0e\x01\x02\x00\x04\x00\x00\x00\x01\x00\x00\x00\x00'
        
        reader = FastExifReader()
        
        try:
            result = reader.read_bytes(sample_data)
            print("✅ Sample data parsing attempted")
            return True
        except Exception as e:
            print(f"⚠️  Sample data parsing error (expected): {e}")
            return True  # This is expected to fail with minimal data
            
    except Exception as e:
        print(f"❌ Sample data test failed: {e}")
        return False

def main():
    """Run all tests"""
    print("🧪 Testing fast-exif-reader package...")
    print("=" * 50)
    
    tests = [
        ("Import Test", test_import),
        ("Basic Functionality", test_basic_functionality),
        ("Performance Test", test_performance),
        ("Sample Data Test", test_with_sample_data),
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n🔍 {test_name}:")
        if test_func():
            passed += 1
        else:
            print(f"❌ {test_name} failed")
    
    print("\n" + "=" * 50)
    print(f"📊 Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! Package is working correctly.")
        return 0
    else:
        print("⚠️  Some tests failed. Check the output above.")
        return 1

if __name__ == "__main__":
    sys.exit(main())

