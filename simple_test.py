#!/usr/bin/env python3
"""Simple test of Rust multiprocessing"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    import fast_exif_reader
    print("✓ fast_exif_reader imported")
    
    # Check what's available
    available = [x for x in dir(fast_exif_reader) if not x.startswith('_')]
    print(f"Available: {available}")
    
    # Test MultiprocessingExifReader
    if 'MultiprocessingExifReader' in available:
        print("✓ MultiprocessingExifReader is available")
        try:
            reader = fast_exif_reader.MultiprocessingExifReader()
            print("✓ MultiprocessingExifReader created")
        except Exception as e:
            print(f"✗ Error creating MultiprocessingExifReader: {e}")
    else:
        print("✗ MultiprocessingExifReader not available")
    
    # Test process_files_parallel
    if 'process_files_parallel' in available:
        print("✓ process_files_parallel is available")
    else:
        print("✗ process_files_parallel not available")
        
except Exception as e:
    print(f"✗ Import error: {e}")
    import traceback
    traceback.print_exc()
