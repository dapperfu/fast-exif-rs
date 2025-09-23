#!/usr/bin/env python3
"""Test Rust multiprocessing with file output"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

with open('test_output.txt', 'w') as f:
    try:
        import fast_exif_reader
        f.write("✓ fast_exif_reader imported\n")
        
        # Check what's available
        available = [x for x in dir(fast_exif_reader) if not x.startswith('_')]
        f.write(f"Available: {available}\n")
        
        # Test MultiprocessingExifReader
        if 'MultiprocessingExifReader' in available:
            f.write("✓ MultiprocessingExifReader is available\n")
            try:
                reader = fast_exif_reader.MultiprocessingExifReader()
                f.write("✓ MultiprocessingExifReader created\n")
            except Exception as e:
                f.write(f"✗ Error creating MultiprocessingExifReader: {e}\n")
        else:
            f.write("✗ MultiprocessingExifReader not available\n")
        
        # Test process_files_parallel
        if 'process_files_parallel' in available:
            f.write("✓ process_files_parallel is available\n")
        else:
            f.write("✗ process_files_parallel not available\n")
            
    except Exception as e:
        f.write(f"✗ Import error: {e}\n")
        import traceback
        f.write(traceback.format_exc())

print("Test completed, check test_output.txt")
