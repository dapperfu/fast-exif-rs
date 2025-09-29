#!/usr/bin/env python3
"""
Test script to debug field mapping issues - direct test
"""

import sys
import os
sys.path.insert(0, '/projects/fast-exif-rs/python')

from fast_exif_reader import FastExifReader

def test_field_mapping_direct():
    reader = FastExifReader()
    
    # Test with a HEIF file
    file_path = "/keg/pictures/2025/09-Sep/20250921_120522.130.hif"
    
    print("Testing field mapping directly...")
    print(f"File: {file_path}")
    
    try:
        # Call the method directly
        metadata = reader.read_file(file_path)
        
        print(f"\nTotal fields: {len(metadata)}")
        
        # Check for specific fields
        datetime_fields = []
        for key, value in metadata.items():
            if "date" in key.lower() or "time" in key.lower():
                datetime_fields.append((key, value))
        
        print(f"\nDateTime fields found:")
        for key, value in datetime_fields:
            print(f"  {key}: {value}")
        
        # Check for the specific fields we're looking for
        target_fields = ["Create Date", "Date/Time Original", "Modify Date"]
        print(f"\nTarget fields:")
        for field in target_fields:
            if field in metadata:
                print(f"  ✓ {field}: {metadata[field]}")
            else:
                print(f"  ✗ {field}: NOT FOUND")
        
        # Check for internal field names
        internal_fields = ["CreateDate", "DateTimeOriginal", "ModifyDate"]
        print(f"\nInternal fields:")
        for field in internal_fields:
            if field in metadata:
                print(f"  ✓ {field}: {metadata[field]}")
            else:
                print(f"  ✗ {field}: NOT FOUND")
                
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_field_mapping_direct()
