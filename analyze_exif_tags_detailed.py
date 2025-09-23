#!/usr/bin/env python3
"""
Detailed EXIF tag analysis script
"""

import sys
from pathlib import Path

def analyze_exif_tags_detailed(file_path: str):
    """Analyze EXIF tags in detail"""
    print(f"Detailed EXIF Tag Analysis: {file_path}")
    print("=" * 80)
    
    with open(file_path, 'rb') as f:
        data = f.read()
    
    # Find the valid EXIF block (position 2052)
    exif_pos = 2052
    exif_start = exif_pos + 4 + 2  # Skip 'Exif' and padding
    
    print(f"Analyzing EXIF Block at position {exif_pos}:")
    print("-" * 40)
    
    if exif_start + 8 < len(data):
        tiff_header = data[exif_start:exif_start+8]
        print(f"TIFF header: {tiff_header.hex()}")
        
        # Check byte order
        if tiff_header[:2] == b'II':
            print("Byte order: Little-endian")
            is_little_endian = True
        else:
            print("Byte order: Big-endian")
            is_little_endian = False
        
        # Read IFD offset
        ifd_offset = int.from_bytes(tiff_header[4:8], byteorder='little' if is_little_endian else 'big')
        print(f"IFD offset: {ifd_offset}")
        
        # Read IFD entries
        ifd_pos = exif_start + ifd_offset
        if ifd_pos + 2 < len(data):
            entry_count = int.from_bytes(data[ifd_pos:ifd_pos+2], byteorder='little' if is_little_endian else 'big')
            print(f"IFD entry count: {entry_count}")
            
            print("\nAll EXIF tags in this block:")
            print("-" * 40)
            
            for j in range(entry_count):
                entry_pos = ifd_pos + 2 + (j * 12)
                if entry_pos + 12 <= len(data):
                    entry = data[entry_pos:entry_pos+12]
                    tag = int.from_bytes(entry[0:2], byteorder='little' if is_little_endian else 'big')
                    tag_type = int.from_bytes(entry[2:4], byteorder='little' if is_little_endian else 'big')
                    count = int.from_bytes(entry[4:8], byteorder='little' if is_little_endian else 'big')
                    value_offset = int.from_bytes(entry[8:12], byteorder='little' if is_little_endian else 'big')
                    
                    # Get tag name
                    tag_name = get_tag_name(tag)
                    
                    print(f"  Tag {j+1:2d}: {tag_name} (0x{tag:04X})")
                    print(f"           Type: {tag_type}, Count: {count}, Offset: {value_offset}")
                    
                    # Try to read the actual value
                    value_str = read_tag_value(data, exif_start, tag_type, count, value_offset, is_little_endian)
                    if value_str:
                        print(f"           Value: {value_str}")
                    
                    # Special attention to exposure-related tags
                    if tag in [0x829A, 0xA402, 0x8822, 0x9204]:
                        print(f"           *** EXPOSURE TAG ***")
                    
                    print()

def get_tag_name(tag: int) -> str:
    """Get human-readable tag name"""
    tag_names = {
        0x010F: "Make",
        0x0110: "Model", 
        0x0112: "Orientation",
        0x011A: "XResolution",
        0x011B: "YResolution",
        0x0128: "ResolutionUnit",
        0x0131: "Software",
        0x0132: "DateTime",
        0x013E: "WhitePoint",
        0x013F: "PrimaryChromaticities",
        0x0211: "YCbCrCoefficients",
        0x0213: "YCbCrPositioning",
        0x0214: "ReferenceBlackWhite",
        0x8298: "Copyright",
        0x829A: "ExposureTime",
        0x829D: "FNumber",
        0x8822: "ExposureProgram",
        0x8827: "ISOSpeedRatings",
        0x8828: "OECF",
        0x9000: "ExifVersion",
        0x9003: "DateTimeOriginal",
        0x9004: "DateTimeDigitized",
        0x9101: "ComponentsConfiguration",
        0x9102: "CompressedBitsPerPixel",
        0x9201: "ShutterSpeedValue",
        0x9202: "ApertureValue",
        0x9203: "BrightnessValue",
        0x9204: "ExposureBiasValue",
        0x9205: "MaxApertureValue",
        0x9206: "SubjectDistance",
        0x9207: "MeteringMode",
        0x9208: "LightSource",
        0x9209: "Flash",
        0x920A: "FocalLength",
        0x920B: "FlashEnergy",
        0x920C: "SpatialFrequencyResponse",
        0x920D: "Noise",
        0x920E: "FocalPlaneXResolution",
        0x920F: "FocalPlaneYResolution",
        0x9210: "FocalPlaneResolutionUnit",
        0x9211: "ImageNumber",
        0x9212: "SecurityClassification",
        0x9213: "ImageHistory",
        0x9214: "SubjectArea",
        0x9215: "ExposureIndex",
        0x9216: "TIFF/EPStandardID",
        0x9217: "SensingMethod",
        0x9286: "UserComment",
        0x9290: "SubSecTime",
        0x9291: "SubSecTimeOriginal",
        0x9292: "SubSecTimeDigitized",
        0xA000: "FlashpixVersion",
        0xA001: "ColorSpace",
        0xA002: "PixelXDimension",
        0xA003: "PixelYDimension",
        0xA004: "RelatedSoundFile",
        0xA20B: "FlashEnergy",
        0xA20C: "SpatialFrequencyResponse",
        0xA20E: "FocalPlaneXResolution",
        0xA20F: "FocalPlaneYResolution",
        0xA210: "FocalPlaneResolutionUnit",
        0xA214: "SubjectLocation",
        0xA215: "ExposureIndex",
        0xA217: "SensingMethod",
        0xA300: "FileSource",
        0xA301: "SceneType",
        0xA302: "CFAPattern",
        0xA401: "CustomRendered",
        0xA402: "ExposureMode",
        0xA403: "WhiteBalance",
        0xA404: "DigitalZoomRatio",
        0xA405: "FocalLengthIn35mmFilm",
        0xA406: "SceneCaptureType",
        0xA407: "GainControl",
        0xA408: "Contrast",
        0xA409: "Saturation",
        0xA40A: "Sharpness",
        0xA40B: "DeviceSettingDescription",
        0xA40C: "SubjectDistanceRange",
        0xA420: "ImageUniqueID",
        0xA430: "CameraOwnerName",
        0xA431: "BodySerialNumber",
        0xA432: "LensSpecification",
        0xA433: "LensMake",
        0xA434: "LensModel",
        0xA435: "LensSerialNumber",
    }
    return tag_names.get(tag, f"Unknown_{tag:04X}")

def read_tag_value(data: bytes, exif_start: int, tag_type: int, count: int, value_offset: int, is_little_endian: bool) -> str:
    """Read and format tag value"""
    try:
        if tag_type == 1:  # BYTE
            if count <= 4:
                return f"{value_offset}"
            else:
                value_pos = exif_start + value_offset
                if value_pos + count <= len(data):
                    return data[value_pos:value_pos+count].hex()
        elif tag_type == 2:  # ASCII
            if count <= 4:
                # Value stored directly in offset field
                value_bytes = value_offset.to_bytes(4, byteorder='little' if is_little_endian else 'big')
                return value_bytes[:count].decode('ascii', errors='ignore').rstrip('\x00')
            else:
                value_pos = exif_start + value_offset
                if value_pos + count <= len(data):
                    return data[value_pos:value_pos+count].decode('ascii', errors='ignore').rstrip('\x00')
        elif tag_type == 3:  # SHORT
            if count == 1 and value_offset < 65536:
                return f"{value_offset}"
            else:
                value_pos = exif_start + value_offset
                if value_pos + 2 <= len(data):
                    value = int.from_bytes(data[value_pos:value_pos+2], byteorder='little' if is_little_endian else 'big')
                    return f"{value}"
        elif tag_type == 4:  # LONG
            if count == 1 and value_offset < 4294967296:
                return f"{value_offset}"
            else:
                value_pos = exif_start + value_offset
                if value_pos + 4 <= len(data):
                    value = int.from_bytes(data[value_pos:value_pos+4], byteorder='little' if is_little_endian else 'big')
                    return f"{value}"
        elif tag_type == 5:  # RATIONAL
            value_pos = exif_start + value_offset
            if value_pos + 8 <= len(data):
                numerator = int.from_bytes(data[value_pos:value_pos+4], byteorder='little' if is_little_endian else 'big')
                denominator = int.from_bytes(data[value_pos+4:value_pos+8], byteorder='little' if is_little_endian else 'big')
                if denominator != 0:
                    return f"{numerator}/{denominator} ({numerator/denominator:.6f})"
                else:
                    return f"{numerator}/{denominator}"
        elif tag_type == 6:  # SBYTE
            return f"{value_offset}"
        elif tag_type == 7:  # UNDEFINED
            return f"UNDEFINED_{value_offset}"
        elif tag_type == 8:  # SSHORT
            return f"{value_offset}"
        elif tag_type == 9:  # SLONG
            return f"{value_offset}"
        elif tag_type == 10:  # SRATIONAL
            value_pos = exif_start + value_offset
            if value_pos + 8 <= len(data):
                numerator = int.from_bytes(data[value_pos:value_pos+4], byteorder='little' if is_little_endian else 'big')
                denominator = int.from_bytes(data[value_pos+4:value_pos+8], byteorder='little' if is_little_endian else 'big')
                if denominator != 0:
                    return f"{numerator}/{denominator} ({numerator/denominator:.6f})"
                else:
                    return f"{numerator}/{denominator}"
    except Exception as e:
        return f"Error: {e}"
    
    return "Unknown"

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python analyze_exif_tags_detailed.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    analyze_exif_tags_detailed(file_path)
