#!/usr/bin/env python3
"""
Test if our TIFF validation is working by creating a simple test
"""


def test_tiff_validation():
    """Test if our TIFF validation is working"""
    print("Testing TIFF Validation")
    print("=" * 40)

    # Create test data with invalid TIFF header
    invalid_tiff = b"\x00\x00\x2a\x69\x6e\x66\x66\x02"  # Invalid TIFF header
    valid_tiff = b"II\x2a\x00\x08\x00\x00\x00"  # Valid TIFF header

    print("Invalid TIFF header:", invalid_tiff.hex())
    print("Valid TIFF header:", valid_tiff.hex())

    # Check byte order
    print("\nByte order check:")
    print(
        f"Invalid: {invalid_tiff[:2]} -> {'II' if invalid_tiff[:2] == b'II' else 'MM' if invalid_tiff[:2] == b'MM' else 'Unknown'}"
    )
    print(
        f"Valid: {valid_tiff[:2]} -> {'II' if valid_tiff[:2] == b'II' else 'MM' if valid_tiff[:2] == b'MM' else 'Unknown'}"
    )

    # Check TIFF version
    print("\nTIFF version check:")
    invalid_version = int.from_bytes(invalid_tiff[2:4], byteorder="little")
    valid_version = int.from_bytes(valid_tiff[2:4], byteorder="little")
    print(f"Invalid version: {invalid_version} -> {'Valid' if invalid_version == 42 else 'Invalid'}")
    print(f"Valid version: {valid_version} -> {'Valid' if valid_version == 42 else 'Invalid'}")


if __name__ == "__main__":
    test_tiff_validation()
