# Installation Guide

## Prerequisites

### Required Software

1. **Rust** (version 1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Python** (version 3.8+)
   ```bash
   # Ubuntu/Debian
   sudo apt update
   sudo apt install python3 python3-pip python3-dev
   
   # macOS
   brew install python3
   
   # Windows
   # Download from https://python.org
   ```

3. **Maturin** (Python-Rust build tool)
   ```bash
   pip install maturin
   ```

### System Dependencies

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install build-essential libssl-dev pkg-config
```

#### macOS
```bash
# Install Xcode command line tools
xcode-select --install
```

#### Windows
- Install Visual Studio Build Tools
- Install Windows SDK

## Installation Methods

### Method 1: Development Installation (Recommended)

```bash
# Clone the repository
git clone https://github.com/dapperfu/fast-exif-rs.git
cd fast-exif-rs

# Install in development mode
maturin develop

# Or install with development dependencies
maturin develop
pip install -e .[dev]
```

### Method 2: From Source

```bash
# Clone the repository
git clone https://github.com/dapperfu/fast-exif-rs.git
cd fast-exif-rs

# Build the package
maturin build

# Install the built wheel
pip install dist/fast_exif_reader-*.whl
```

### Method 3: Using Makefile

```bash
# Clone the repository
git clone https://github.com/dapperfu/fast-exif-rs.git
cd fast-exif-rs

# Set up development environment
make dev-setup

# Install the package
make install

# Or install with development dependencies
make install-dev
```


## Verification

After installation, verify the package works:

```python
# Test basic functionality
python -c "from fast_exif_reader import FastExifReader; print('Installation successful!')"

# Run tests
python -m pytest tests/ -v

# Run benchmarks
python examples/benchmark.py sample_image.jpg
```

## Troubleshooting

### Common Issues

#### 1. Rust Not Found
```
error: could not find `Cargo.toml` in `/path/to/project` or any parent directory
```
**Solution**: Install Rust using rustup.rs

#### 2. Python Development Headers Missing
```
error: Microsoft Visual C++ 14.0 is required
```
**Solution**: Install Visual Studio Build Tools (Windows) or build-essential (Linux)

#### 3. Maturin Not Found
```
maturin: command not found
```
**Solution**: Install maturin with `pip install maturin`

#### 4. Permission Denied
```
Permission denied (publickey)
```
**Solution**: Ensure you have write permissions to the Python environment

### Platform-Specific Issues

#### Windows
- Ensure Visual Studio Build Tools are installed
- Use Command Prompt or PowerShell (not Git Bash)
- May need to restart terminal after installing Rust

#### macOS
- Install Xcode command line tools: `xcode-select --install`
- May need to install additional dependencies via Homebrew

#### Linux
- Install build-essential: `sudo apt install build-essential`
- Ensure Python development headers are installed

## Development Setup

For contributors:

```bash
# Clone and set up development environment
git clone https://github.com/yourusername/fast-exif-reader.git
cd fast-exif-reader

# Install development dependencies
make dev-setup
make install-dev

# Run tests
make test

# Format code
make format

# Lint code
make lint
```

## Uninstallation

```bash
# Uninstall the package
pip uninstall fast-exif-reader

# Clean build artifacts
make clean
```

## Performance Notes

- The package is built with Rust optimizations enabled
- For maximum performance, use the release build: `maturin build --release`
- Development builds include debug symbols and are slower

## Support

If you encounter issues:

1. Check this installation guide
2. Verify all prerequisites are installed
3. Check the troubleshooting section
4. Open an issue on GitHub with:
   - Operating system
   - Python version
   - Rust version
   - Full error message
   - Steps to reproduce

