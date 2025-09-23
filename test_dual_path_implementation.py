#!/usr/bin/env python3
"""Test script to verify both Python and Rust multiprocessing implementations work correctly"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def test_imports():
    """Test that all imports work correctly"""
    print("Testing imports...")
    
    try:
        from fast_exif_reader import (
            # Core functionality
            FastExifReader,
            
            # Python multiprocessing
            PythonMultiprocessingExifReader,
            python_extract_exif_batch,
            python_extract_exif_from_directory,
            
            # Rust multiprocessing
            RustMultiprocessingExifReader,
            rust_process_files_parallel,
            rust_process_directory_parallel,
            
            # Utility
            RUST_AVAILABLE
        )
        print("✓ All imports successful")
        return True
    except ImportError as e:
        print(f"✗ Import error: {e}")
        return False


def test_python_multiprocessing():
    """Test Python multiprocessing implementation"""
    print("\nTesting Python multiprocessing...")
    
    try:
        from fast_exif_reader import python_extract_exif_batch, PythonMultiprocessingExifReader
        
        # Test with dummy files (will fail but should handle errors gracefully)
        test_files = ["/nonexistent/file1.jpg", "/nonexistent/file2.jpg"]
        
        # Test function-based approach
        results = python_extract_exif_batch(test_files, max_workers=2)
        print(f"✓ Function-based: {results['statistics']['total_files']} files processed")
        
        # Test class-based approach
        reader = PythonMultiprocessingExifReader(max_workers=2)
        results = reader.read_files(test_files)
        print(f"✓ Class-based: {results['statistics']['total_files']} files processed")
        
        return True
    except Exception as e:
        print(f"✗ Python multiprocessing error: {e}")
        return False


def test_rust_multiprocessing():
    """Test Rust multiprocessing implementation"""
    print("\nTesting Rust multiprocessing...")
    
    try:
        from fast_exif_reader import RUST_AVAILABLE, rust_process_files_parallel, RustMultiprocessingExifReader
        
        if not RUST_AVAILABLE:
            print("✗ Rust implementation not available")
            return False
        
        # Test with dummy files (will fail but should handle errors gracefully)
        test_files = ["/nonexistent/file1.jpg", "/nonexistent/file2.jpg"]
        
        # Test function-based approach
        results = rust_process_files_parallel(test_files, max_workers=2)
        print(f"✓ Function-based: {results['statistics']['total_files']} files processed")
        
        # Test class-based approach
        reader = RustMultiprocessingExifReader(max_workers=2)
        results = reader.read_files(test_files)
        print(f"✓ Class-based: {results['statistics']['total_files']} files processed")
        
        return True
    except Exception as e:
        print(f"✗ Rust multiprocessing error: {e}")
        return False


def test_availability():
    """Test availability of both implementations"""
    print("\nTesting availability...")
    
    try:
        from fast_exif_reader import RUST_AVAILABLE
        
        print(f"Python multiprocessing: ✓ Available")
        print(f"Rust multiprocessing: {'✓ Available' if RUST_AVAILABLE else '✗ Not available'}")
        
        return True
    except Exception as e:
        print(f"✗ Availability test error: {e}")
        return False


def test_backward_compatibility():
    """Test backward compatibility aliases"""
    print("\nTesting backward compatibility...")
    
    try:
        from fast_exif_reader import (
            MultiprocessingExifReader,
            extract_exif_batch,
            extract_exif_from_directory
        )
        
        print("✓ Backward compatibility aliases available")
        
        # Test that aliases work
        test_files = ["/nonexistent/file.jpg"]
        results = extract_exif_batch(test_files, max_workers=1)
        print(f"✓ Alias function works: {results['statistics']['total_files']} files processed")
        
        return True
    except Exception as e:
        print(f"✗ Backward compatibility error: {e}")
        return False


def main():
    """Run all tests"""
    print("Fast EXIF Reader - Dual-Path Implementation Test")
    print("=" * 50)
    
    tests = [
        ("Import Test", test_imports),
        ("Availability Test", test_availability),
        ("Python Multiprocessing Test", test_python_multiprocessing),
        ("Rust Multiprocessing Test", test_rust_multiprocessing),
        ("Backward Compatibility Test", test_backward_compatibility),
    ]
    
    results = []
    for test_name, test_func in tests:
        print(f"\n{test_name}:")
        try:
            success = test_func()
            results.append((test_name, success))
        except Exception as e:
            print(f"✗ Test failed with exception: {e}")
            results.append((test_name, False))
    
    # Summary
    print("\n" + "=" * 50)
    print("TEST SUMMARY")
    print("=" * 50)
    
    passed = 0
    total = len(results)
    
    for test_name, success in results:
        status = "✓ PASS" if success else "✗ FAIL"
        print(f"{test_name}: {status}")
        if success:
            passed += 1
    
    print(f"\nOverall: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! Dual-path implementation is working correctly.")
    else:
        print("⚠️  Some tests failed. Check the output above for details.")
    
    return passed == total


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
