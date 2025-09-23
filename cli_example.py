#!/usr/bin/env python3
"""
Example usage of the Fast EXIF CLI tool
"""

import subprocess
import sys
from pathlib import Path

def run_cli_example():
    """Demonstrate the CLI tool functionality"""
    
    print("Fast EXIF CLI Tool - Example Usage")
    print("=" * 50)
    
    # Get the CLI path
    cli_path = Path(__file__).parent / "venv" / "bin" / "python"
    cli_module = "-m"
    cli_module_arg = "cli.fast_exif_cli"
    
    print(f"\nCLI Path: {cli_path}")
    print(f"Module: {cli_module}")
    
    # Example 1: Show help
    print("\n1. Help Command:")
    print("-" * 20)
    try:
        result = subprocess.run([str(cli_path), cli_module, cli_module_arg, "--help"], 
                              capture_output=True, text=True)
        if result.returncode == 0:
            print("✓ Help command executed successfully")
            # Show first few lines of help
            help_lines = result.stdout.split('\n')[:10]
            for line in help_lines:
                if line.strip():
                    print(f"  {line}")
            print("  ...")
        else:
            print("✗ Help command failed")
    except Exception as e:
        print(f"✗ Error: {e}")
    
    # Example 2: Show version
    print("\n2. Version Command:")
    print("-" * 20)
    try:
        result = subprocess.run([str(cli_path), cli_module, cli_module_arg, "--version"], 
                              capture_output=True, text=True)
        if result.returncode == 0:
            print(f"✓ {result.stdout.strip()}")
        else:
            print("✗ Version command failed")
    except Exception as e:
        print(f"✗ Error: {e}")
    
    # Example 3: Test error handling
    print("\n3. Error Handling:")
    print("-" * 20)
    try:
        result = subprocess.run([str(cli_path), cli_module, cli_module_arg], 
                              capture_output=True, text=True)
        if result.returncode == 1:
            print("✓ Error handling works correctly")
            print(f"  Error message: {result.stderr.strip()}")
        else:
            print("✗ Error handling failed")
    except Exception as e:
        print(f"✗ Error: {e}")
    
    # Example 4: Test with non-existent file
    print("\n4. Non-existent File Handling:")
    print("-" * 20)
    try:
        result = subprocess.run([str(cli_path), cli_module, cli_module_arg, "nonexistent.jpg"], 
                              capture_output=True, text=True)
        if result.returncode == 1:
            print("✓ Non-existent file handling works")
        else:
            print("✗ Non-existent file handling failed")
    except Exception as e:
        print(f"✗ Error: {e}")
    
    print("\n" + "=" * 50)
    print("CLI Tool Examples Completed!")
    print("\nTo use the CLI tool:")
    print("1. Install the package: pip install -e .")
    print("2. Run: fast-exif-cli image.jpg")
    print("3. Or run: python -m cli.fast_exif_cli image.jpg")
    print("\nAvailable formats:")
    print("- Default: Tag Name: Value")
    print("- Short (-s): TagName: Value") 
    print("- Very Short (-S): Value only")
    print("- Tab-delimited (-t): Description\\tValue")
    print("- Table (-T): Value only")
    print("- JSON (-j): JSON format")

if __name__ == "__main__":
    run_cli_example()
