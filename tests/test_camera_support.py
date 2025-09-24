#!/usr/bin/env python3
"""
Test script to verify comprehensive camera support in fast-exif-rs
"""

from fast_exif_reader import FastExifReader


def test_camera_detection():
    """Test camera make and model detection"""
    reader = FastExifReader()

    # Test data for different camera makes
    test_cases = [
        {
            "name": "Canon EOS 70D",
            "make": "Canon",
            "model": "Canon EOS 70D",
            "format": "JPEG",
        },
        {
            "name": "Nikon Z50_2",
            "make": "NIKON CORPORATION",
            "model": "NIKON Z50_2",
            "format": "NEF",
        },
        {
            "name": "GoPro HERO5 Black",
            "make": "GoPro",
            "model": "HERO5 Black",
            "format": "JPEG",
        },
        {
            "name": "Samsung SM-N910T",
            "make": "Samsung",
            "model": "SM-N910T",
            "format": "HEIF",
        },
        {
            "name": "Motorola moto g(6)",
            "make": "Motorola",
            "model": "moto g(6)",
            "format": "HEIF",
        },
        {
            "name": "Olympus Camera",
            "make": "OLYMPUS OPTICAL CO.,LTD",
            "model": "Unknown",
            "format": "ORF",
        },
        {"name": "Ricoh Camera", "make": "RICOH", "model": "Unknown", "format": "DNG"},
    ]

    print("🎯 Testing Comprehensive Camera Support")
    print("=" * 50)

    for test_case in test_cases:
        print(f"\n📸 Testing {test_case['name']}:")
        print(f"   Expected Make: {test_case['make']}")
        print(f"   Expected Model: {test_case['model']}")
        print(f"   Expected Format: {test_case['format']}")

        # Test format detection
        try:
            # Create a minimal test data structure
            test_data = create_test_data(test_case)
            metadata = reader.read_bytes(test_data)

            detected_make = metadata.get("Make", "Unknown")
            detected_model = metadata.get("Model", "Unknown")
            detected_format = metadata.get("Format", "Unknown")

            print(f"   Detected Make: {detected_make}")
            print(f"   Detected Model: {detected_model}")
            print(f"   Detected Format: {detected_format}")

            # Check if detection matches expected
            make_match = detected_make == test_case["make"] or test_case["make"] in detected_make
            format_match = detected_format == test_case["format"]

            if make_match and format_match:
                print("   ✅ PASS")
            else:
                print("   ❌ FAIL")

        except Exception as e:
            print(f"   ❌ ERROR: {e}")

    print("\n" + "=" * 50)
    print("🎉 Camera support testing completed!")


def create_test_data(test_case):
    """Create minimal test data for camera detection"""
    # This is a simplified test - in real usage, actual image files would be used
    if test_case["format"] == "JPEG":
        # Minimal JPEG header with EXIF
        return (
            b"\xff\xd8\xff\xe1\x00\x16Exif\x00\x00II*\x00\x08\x00\x00\x00\x01\x00\x0f\x01\x02\x00\x05\x00\x00\x00\x1a\x00\x00\x00\x00\x00\x00\x00"
            + test_case["make"].encode()
            + b"\x00"
            + test_case["model"].encode()
            + b"\x00"
        )
    elif test_case["format"] == "NEF":
        # Minimal TIFF header for Nikon
        return (
            b"II*\x00\x08\x00\x00\x00\x01\x00\x0f\x01\x02\x00\x05\x00\x00\x00\x1a\x00\x00\x00\x00\x00\x00\x00"
            + test_case["make"].encode()
            + b"\x00"
            + test_case["model"].encode()
            + b"\x00"
        )
    elif test_case["format"] == "HEIF":
        # Minimal HEIF header
        return (
            b"\x00\x00\x00\x20ftypheic\x00\x00\x00\x00heic\x00\x00\x00\x00"
            + test_case["make"].encode()
            + b"\x00"
            + test_case["model"].encode()
            + b"\x00"
        )
    else:
        # Default TIFF-like header
        return (
            b"II*\x00\x08\x00\x00\x00\x01\x00\x0f\x01\x02\x00\x05\x00\x00\x00\x1a\x00\x00\x00\x00\x00\x00\x00"
            + test_case["make"].encode()
            + b"\x00"
            + test_case["model"].encode()
            + b"\x00"
        )


if __name__ == "__main__":
    test_camera_detection()
