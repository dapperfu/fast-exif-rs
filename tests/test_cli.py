#!/usr/bin/env python3
"""
Test script for the Fast EXIF CLI tool
"""

import subprocess
from pathlib import Path


def test_cli():
    """Test the CLI tool with various options"""

    # Get the path to the CLI
    cli_path = Path(__file__).parent / "venv" / "bin" / "python"
    cli_module = "-m cli.fast_exif_cli"

    print(f"Using CLI path: {cli_path}")
    print(f"CLI module: {cli_module}")

    print("Testing Fast EXIF CLI Tool")
    print("=" * 50)

    # Test 1: Help
    print("\n1. Testing help:")
    try:
        result = subprocess.run([str(cli_path), cli_module, "--help"], capture_output=True, text=True)
        if result.returncode == 0:
            print("✓ Help command works")
        else:
            print("✗ Help command failed")
    except Exception as e:
        print(f"✗ Error running help: {e}")

    # Test 2: Version
    print("\n2. Testing version:")
    try:
        result = subprocess.run([str(cli_path), cli_module, "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print(f"✓ Version: {result.stdout.strip()}")
        else:
            print("✗ Version command failed")
    except Exception as e:
        print(f"✗ Error running version: {e}")

    # Test 3: No arguments (should show error)
    print("\n3. Testing no arguments:")
    try:
        result = subprocess.run([str(cli_path), cli_module], capture_output=True, text=True)
        if result.returncode == 1 and "No files specified" in result.stderr:
            print("✓ Error handling works correctly")
        else:
            print("✗ Error handling failed")
    except Exception as e:
        print(f"✗ Error testing no arguments: {e}")

    # Test 4: Non-existent file
    print("\n4. Testing non-existent file:")
    try:
        result = subprocess.run(
            [str(cli_path), cli_module, "nonexistent.jpg"],
            capture_output=True,
            text=True,
        )
        if result.returncode == 1:
            print("✓ Non-existent file handling works")
        else:
            print("✗ Non-existent file handling failed")
    except Exception as e:
        print(f"✗ Error testing non-existent file: {e}")

    print("\n" + "=" * 50)
    print("CLI tool testing completed!")


if __name__ == "__main__":
    test_cli()
