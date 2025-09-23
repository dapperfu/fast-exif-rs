#!/bin/bash

# Build script for fast-exif-reader
set -e

echo "🚀 Building fast-exif-reader..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "pyproject.toml" ]; then
    print_error "Please run this script from the fast-exif-reader root directory"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if maturin is installed
if ! command -v maturin &> /dev/null; then
    print_warning "maturin is not installed. Installing..."
    pip install maturin
fi

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    print_error "Python 3 is not installed"
    exit 1
fi

# Get Python version
PYTHON_VERSION=$(python3 -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
print_status "Using Python $PYTHON_VERSION"

# Check if Python version is supported
if python3 -c "import sys; exit(0 if sys.version_info >= (3, 8) else 1)"; then
    print_status "Python version is supported"
else
    print_error "Python 3.8+ is required"
    exit 1
fi

# Clean previous builds
print_status "Cleaning previous builds..."
cargo clean 2>/dev/null || true
rm -rf build/ dist/ *.egg-info/ 2>/dev/null || true

# Build the package
print_status "Building Rust extension..."
if [ "$1" = "--release" ]; then
    print_status "Building in release mode..."
    maturin build --release
else
    print_status "Building in development mode..."
    maturin develop
fi

if [ $? -eq 0 ]; then
    print_status "Build successful! 🎉"
    
    # Test the installation
    print_status "Testing installation..."
    if python3 -c "from fast_exif_reader import FastExifReader; print('✅ Import successful')" 2>/dev/null; then
        print_status "Package is ready to use!"
        echo ""
        echo "Usage example:"
        echo "  python3 -c \"from fast_exif_reader import FastExifReader; reader = FastExifReader(); print('Ready!')\""
        echo ""
        echo "Run tests:"
        echo "  python3 -m pytest tests/ -v"
        echo ""
        echo "Run benchmarks:"
        echo "  python3 examples/benchmark.py sample_image.jpg"
    else
        print_error "Import test failed"
        exit 1
    fi
else
    print_error "Build failed!"
    exit 1
fi

