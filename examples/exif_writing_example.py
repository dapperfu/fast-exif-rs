#!/usr/bin/env python3
"""
Example demonstrating EXIF writing functionality in fast-exif-rs.

This example shows how to:
1. Read EXIF data from a source image
2. Write high-priority EXIF fields to a target image
3. Copy metadata between images for extracted/processed images
"""

import sys
import os
from pathlib import Path

# Add the project root to the Python path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

try:
    import fast_exif_reader
except ImportError:
    print("Error: fast_exif_reader module not found. Please build the project first.")
    print("Run: maturin develop")
    sys.exit(1)

def copy_exif_metadata(source_path, target_path, output_path):
    """
    Copy high-priority EXIF metadata from source to target image.
    
    Args:
        source_path: Path to source image with EXIF data
        target_path: Path to target image (may or may not have EXIF)
        output_path: Path for output image with copied EXIF
    """
    print(f"Copying EXIF metadata from {source_path} to {target_path}")
    
    # Initialize reader and writer
    reader = fast_exif_reader.FastExifReader()
    writer = fast_exif_reader.FastExifWriter()
    
    # Read EXIF data from source image
    print("Reading EXIF data from source image...")
    try:
        source_metadata = reader.read_file(source_path)
        print(f"Found {len(source_metadata)} EXIF fields in source image")
    except Exception as e:
        print(f"Error reading source image: {e}")
        return False
    
    # Define high-priority fields to copy
    high_priority_fields = [
        # DateTime fields
        "DateTime",
        "DateTimeOriginal", 
        "DateTimeDigitized",
        
        # Camera information
        "Make",
        "Model",
        "Software",
        
        # Exposure settings
        "ExposureTime",
        "FNumber", 
        "ISO",
        "FocalLength",
        
        # Image properties
        "Orientation",
        "XResolution",
        "YResolution",
        "ResolutionUnit",
    ]
    
    # Filter metadata to only include high-priority fields
    filtered_metadata = {}
    for field in high_priority_fields:
        if field in source_metadata:
            filtered_metadata[field] = source_metadata[field]
            print(f"  {field}: {source_metadata[field]}")
    
    if not filtered_metadata:
        print("No high-priority EXIF fields found in source image")
        return False
    
    # Write EXIF metadata to target image
    print(f"\nWriting {len(filtered_metadata)} EXIF fields to output image...")
    try:
        writer.write_jpeg_exif(target_path, output_path, filtered_metadata)
        print(f"Successfully wrote EXIF metadata to {output_path}")
        return True
    except Exception as e:
        print(f"Error writing EXIF metadata: {e}")
        return False

def verify_exif_copy(output_path):
    """Verify that EXIF data was successfully copied."""
    print(f"\nVerifying EXIF data in {output_path}...")
    
    reader = fast_exif_reader.FastExifReader()
    try:
        output_metadata = reader.read_file(output_path)
        print(f"Output image contains {len(output_metadata)} EXIF fields:")
        
        # Show high-priority fields
        high_priority_fields = [
            "DateTime", "DateTimeOriginal", "DateTimeDigitized",
            "Make", "Model", "Software",
            "ExposureTime", "FNumber", "ISO", "FocalLength",
            "Orientation", "XResolution", "YResolution", "ResolutionUnit"
        ]
        
        for field in high_priority_fields:
            if field in output_metadata:
                print(f"  {field}: {output_metadata[field]}")
        
        return True
    except Exception as e:
        print(f"Error verifying output image: {e}")
        return False

def main():
    """Main function demonstrating EXIF writing capabilities."""
    print("Fast-EXIF-RS EXIF Writing Example")
    print("=" * 40)
    
    # Check if test files exist
    test_data_dir = Path(__file__).parent.parent / "test_data"
    if not test_data_dir.exists():
        print(f"Test data directory not found: {test_data_dir}")
        print("Please ensure test images are available")
        return
    
    # Look for sample images
    sample_images = list(test_data_dir.glob("*.jpg")) + list(test_data_dir.glob("*.jpeg"))
    if not sample_images:
        print(f"No sample images found in {test_data_dir}")
        return
    
    # Use first available image as source
    source_path = str(sample_images[0])
    print(f"Using source image: {source_path}")
    
    # For this example, we'll create a simple target image
    # In practice, this would be an extracted/processed image
    target_path = str(test_data_dir / "target_image.jpg")
    output_path = str(test_data_dir / "output_with_exif.jpg")
    
    # Check if target image exists, if not create a simple one
    if not os.path.exists(target_path):
        print(f"Target image not found: {target_path}")
        print("Creating a simple target image for demonstration...")
        
        # Create a minimal JPEG file (this is a simplified approach)
        # In practice, you would have an actual processed image
        try:
            import shutil
            shutil.copy(source_path, target_path)
            print(f"Created target image: {target_path}")
        except Exception as e:
            print(f"Error creating target image: {e}")
            return
    
    # Copy EXIF metadata
    success = copy_exif_metadata(source_path, target_path, output_path)
    
    if success:
        # Verify the copy
        verify_exif_copy(output_path)
        print(f"\nExample completed successfully!")
        print(f"Output image with copied EXIF: {output_path}")
    else:
        print("Example failed")

if __name__ == "__main__":
    main()
