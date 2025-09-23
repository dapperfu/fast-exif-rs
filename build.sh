#!/bin/bash

# Build script for fast-exif-reader

echo "Building fast-exif-reader..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi

# Check if maturin is installed
if ! command -v maturin &> /dev/null; then
    echo "Installing maturin..."
    pip install maturin
fi

# Build in development mode
echo "Building in development mode..."
maturin develop

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "You can now use the module in Python:"
    echo "  from fast_exif_reader import FastExifReader"
else
    echo "Build failed!"
    exit 1
fi

