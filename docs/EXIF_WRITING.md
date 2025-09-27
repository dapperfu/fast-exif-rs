# EXIF Writing in Fast-EXIF-RS

## Overview

Fast-EXIF-RS now includes EXIF writing capabilities focused on high-priority fields for copying metadata between images. This is particularly useful for copying EXIF data to extracted or processed images.

## Features

### High-Priority Fields Supported

The EXIF writer focuses on the most commonly needed fields for copying metadata:

#### DateTime Fields
- `DateTime` - Image modification time
- `DateTimeOriginal` - When the photo was taken
- `DateTimeDigitized` - When the image was digitized

#### Camera Information
- `Make` - Camera manufacturer (e.g., "Canon", "Nikon")
- `Model` - Camera model (e.g., "EOS 70D", "D850")
- `Software` - Software used to process the image

#### Exposure Settings
- `ExposureTime` - Shutter speed (e.g., "1/60", "4.0")
- `FNumber` - Aperture value (e.g., "4.0", "2.8")
- `ISO` - ISO sensitivity (e.g., "100", "800")
- `FocalLength` - Lens focal length (e.g., "50.0 mm")

#### Image Properties
- `Orientation` - Image orientation (1-8)
- `XResolution` - Horizontal resolution
- `YResolution` - Vertical resolution
- `ResolutionUnit` - Resolution unit (inches, cm, etc.)

## Usage

### Basic EXIF Writing

```python
import fast_exif_reader

# Create a writer
writer = fast_exif_reader.FastExifWriter()

# Define metadata to write
metadata = {
    "Make": "Canon",
    "Model": "EOS 70D",
    "DateTime": "2023:12:25 12:00:00",
    "DateTimeOriginal": "2023:12:25 12:00:00",
    "ExposureTime": "1/60",
    "FNumber": "4.0",
    "ISO": "100",
    "FocalLength": "50.0 mm"
}

# Write EXIF to a JPEG file
writer.write_jpeg_exif("input.jpg", "output.jpg", metadata)
```

### Copying High-Priority EXIF Fields

```python
# Copy high-priority fields from source to target image
writer.copy_high_priority_exif("source.jpg", "target.jpg", "output.jpg")
```

### Working with Bytes

```python
# Read input image
with open("input.jpg", "rb") as f:
    input_data = f.read()

# Write EXIF to bytes
output_data = writer.write_jpeg_exif_to_bytes(input_data, metadata)

# Save output
with open("output.jpg", "wb") as f:
    f.write(output_data)
```

## API Reference

### FastExifWriter

#### Constructor
```python
writer = fast_exif_reader.FastExifWriter()
```

#### Methods

##### `write_jpeg_exif(input_path, output_path, metadata)`
Write EXIF metadata to a JPEG file.

**Parameters:**
- `input_path` (str): Path to input JPEG file
- `output_path` (str): Path for output JPEG file
- `metadata` (dict): Dictionary of EXIF field names and values

**Returns:** None

**Raises:** RuntimeError if writing fails

##### `write_jpeg_exif_to_bytes(input_data, metadata)`
Write EXIF metadata to JPEG bytes.

**Parameters:**
- `input_data` (bytes): Input JPEG data
- `metadata` (dict): Dictionary of EXIF field names and values

**Returns:** bytes - Output JPEG data with EXIF

**Raises:** RuntimeError if writing fails

##### `copy_high_priority_exif(source_path, target_path, output_path)`
Copy high-priority EXIF fields from source to target image.

**Parameters:**
- `source_path` (str): Path to source image with EXIF data
- `target_path` (str): Path to target image
- `output_path` (str): Path for output image

**Returns:** None

**Raises:** RuntimeError if copying fails

##### `copy_high_priority_exif_to_bytes(source_data, target_data)`
Copy high-priority EXIF fields from source to target bytes.

**Parameters:**
- `source_data` (bytes): Source image data
- `target_data` (bytes): Target image data

**Returns:** bytes - Output image data with copied EXIF

**Raises:** RuntimeError if copying fails

## Field Validation

The writer includes validation for common field formats:

### DateTime Fields
- Format: `YYYY:MM:DD HH:MM:SS`
- Example: `"2023:12:25 12:00:00"`

### Exposure Time
- Fraction format: `"1/60"`, `"1/125"`
- Decimal format: `"4.0"`, `"0.5"`

### F-Number
- Decimal format: `"4.0"`, `"2.8"`

### ISO
- Positive integer: `"100"`, `"800"`

### Focal Length
- With units: `"50.0 mm"`
- Decimal: `"50.0"`

## Implementation Details

### Supported Formats
- **JPEG**: Full support for EXIF writing
- **Other formats**: Currently JPEG-only, with plans for TIFF/RAW support

### EXIF Structure
The writer creates standard EXIF segments with:
- APP1 marker (0xFF 0xE1)
- EXIF signature ("Exif\0\0")
- TIFF header with little-endian byte order
- IFD entries for high-priority fields
- Proper value storage for different data types

### Data Types Supported
- **ASCII** (type 2): Text fields like Make, Model, DateTime
- **SHORT** (type 3): Integer fields like ISO, Orientation
- **RATIONAL** (type 5): Fractional fields like ExposureTime, FNumber

## Examples

### Copy EXIF from RAW to JPEG
```python
# Read EXIF from RAW file
reader = fast_exif_reader.FastExifReader()
raw_metadata = reader.read_file("image.cr2")

# Filter to high-priority fields
high_priority = {k: v for k, v in raw_metadata.items() 
                if k in ["Make", "Model", "DateTime", "ExposureTime", "FNumber", "ISO"]}

# Write to processed JPEG
writer = fast_exif_reader.FastExifWriter()
writer.write_jpeg_exif("processed.jpg", "output.jpg", high_priority)
```

### Batch Processing
```python
import os
from pathlib import Path

def copy_exif_batch(source_dir, target_dir, output_dir):
    """Copy EXIF from source images to target images"""
    writer = fast_exif_reader.FastExifWriter()
    
    for source_file in Path(source_dir).glob("*.jpg"):
        target_file = Path(target_dir) / source_file.name
        output_file = Path(output_dir) / source_file.name
        
        if target_file.exists():
            try:
                writer.copy_high_priority_exif(
                    str(source_file), 
                    str(target_file), 
                    str(output_file)
                )
                print(f"Copied EXIF: {source_file.name}")
            except Exception as e:
                print(f"Failed {source_file.name}: {e}")

# Usage
copy_exif_batch("raw_images/", "processed_images/", "output_images/")
```

## Limitations

### Current Limitations
1. **JPEG-only**: Currently supports only JPEG format
2. **High-priority fields only**: Focuses on essential fields for copying
3. **Simplified validation**: Basic format validation, not comprehensive
4. **No maker notes**: Does not preserve manufacturer-specific data

### Future Enhancements
1. **TIFF/RAW support**: Extend to support RAW formats
2. **Complete field set**: Support all EXIF fields
3. **Maker notes preservation**: Maintain manufacturer-specific data
4. **Advanced validation**: Comprehensive field validation
5. **GPS data**: Support for GPS metadata

## Performance

The EXIF writer is optimized for:
- **Fast processing**: Minimal overhead for high-priority fields
- **Memory efficiency**: Streams data where possible
- **Reliability**: Focuses on commonly used, well-tested fields

## Error Handling

The writer provides clear error messages for:
- Invalid field formats
- Missing input files
- Corrupted EXIF data
- Unsupported field types

## Contributing

When adding new field support:
1. Add field to high-priority list in `utils.rs`
2. Implement validation in `validate_field_value`
3. Add normalization in `normalize_field_value`
4. Update tests and documentation

## See Also

- [EXIF Reading Documentation](README.md)
- [Performance Benchmarks](PERFORMANCE.md)
- [API Reference](API.md)
