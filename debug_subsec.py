#!/usr/bin/env python3
"""
Debug script to analyze SubSecTime field data
"""

import sys
from pathlib import Path
from fast_exif_reader import FastExifReader

def debug_subsec_fields(file_path: str):
    """Debug SubSecTime fields to understand the data format"""
    print(f"Debugging SubSecTime fields: {file_path}")
    print("=" * 60)
    
    reader = FastExifReader()
    
    try:
        metadata = reader.read_file(file_path)
        
        # Look for sub-second time fields
        subsec_fields = ['SubSecTime', 'SubSecTimeOriginal', 'SubSecTimeDigitized']
        
        print("Sub-second time fields:")
        print("-" * 30)
        for field in subsec_fields:
            if field in metadata:
                value = metadata[field]
                print(f"{field}: '{value}'")
                
                # Show raw bytes if it contains replacement characters
                if '��' in value:
                    print(f"  Contains Unicode replacement characters")
                    print(f"  Length: {len(value)}")
                    print(f"  Raw bytes: {[ord(c) for c in value]}")
            else:
                print(f"{field}: Not found")
        
        print("\nOther potentially problematic fields:")
        print("-" * 40)
        problematic_fields = ['UserComment', 'ComponentsConfiguration', 'LensSpecification']
        for field in problematic_fields:
            if field in metadata:
                value = metadata[field]
                print(f"{field}: '{value}'")
                if '��' in value:
                    print(f"  Contains Unicode replacement characters")
                    print(f"  Length: {len(value)}")
                    print(f"  Raw bytes: {[ord(c) for c in value]}")
        
        # Show all fields that contain replacement characters
        print("\nAll fields with Unicode replacement characters:")
        print("-" * 50)
        for key, value in metadata.items():
            if '��' in value:
                print(f"{key}: '{value}'")
                
    except Exception as e:
        print(f"Error reading file: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python debug_subsec.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    debug_subsec_fields(file_path)
