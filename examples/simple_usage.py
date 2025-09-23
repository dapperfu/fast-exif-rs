#!/usr/bin/env python3
"""
Simple usage example for fast-exif-reader
"""

from fast_exif_reader import FastExifReader
import json

def main():
    # Create reader instance
    reader = FastExifReader()
    
    # Example 1: Read from file
    print("Example 1: Reading from file")
    try:
        metadata = reader.read_file("sample_image.jpg")
        print("Metadata:")
        for key, value in metadata.items():
            print(f"  {key}: {value}")
    except Exception as e:
        print(f"Error reading file: {e}")
    
    print("\n" + "="*50 + "\n")
    
    # Example 2: Read from bytes
    print("Example 2: Reading from bytes")
    try:
        with open("sample_image.jpg", "rb") as f:
            image_data = f.read()
        
        metadata = reader.read_bytes(image_data)
        print("Metadata:")
        for key, value in metadata.items():
            print(f"  {key}: {value}")
    except Exception as e:
        print(f"Error reading bytes: {e}")
    
    print("\n" + "="*50 + "\n")
    
    # Example 3: JSON output
    print("Example 3: JSON output")
    try:
        metadata = reader.read_file("sample_image.jpg")
        json_output = json.dumps(metadata, indent=2)
        print(json_output)
    except Exception as e:
        print(f"Error generating JSON: {e}")

if __name__ == "__main__":
    main()

