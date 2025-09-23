# GitHub Actions Workflows

This repository includes several GitHub Actions workflows for automated building, testing, and releasing.

## Workflows

### 1. CI (`ci.yml`)
**Triggers:** Push to main/develop branches, Pull requests
**Purpose:** Continuous integration testing

- **Test Matrix:** Tests on Ubuntu, Windows, macOS with Python 3.8-3.12
- **Linting:** Rust (cargo fmt, clippy) and Python (black, flake8)
- **Security:** Rust (cargo audit) and Python (safety) security audits
- **Build Test:** Ensures package builds and imports correctly

### 2. Build and Release (`build-and-release.yml`)
**Triggers:** Git tags (v*), Manual dispatch
**Purpose:** Create GitHub releases with pre-built wheels

- **Multi-platform Build:** Builds wheels for Linux, Windows, macOS
- **Source Distribution:** Creates source tarball
- **Release Creation:** Automatically creates GitHub releases
- **Installation Test:** Tests wheel installation and CLI functionality

### 3. PyPI Distribution (`pypi.yml`)
**Triggers:** Manual dispatch only
**Purpose:** Build and optionally upload to PyPI

- **PyPI Build:** Creates distribution packages for PyPI
- **Upload Option:** Can upload to PyPI (requires PYPI_TOKEN secret)
- **Installation Test:** Tests installation from PyPI

## Usage

### Creating a Release

1. **Tag a release:**
   ```bash
   git tag -a v0.3.5 -m "Release v0.3.5"
   git push origin v0.3.5
   ```

2. **Manual release:**
   - Go to Actions → Build and Release
   - Click "Run workflow"
   - Enter tag name (e.g., v0.3.5)

### Uploading to PyPI

1. **Set up PyPI token:**
   - Go to PyPI → Account Settings → API tokens
   - Create a new token
   - Add as repository secret: `PYPI_TOKEN`

2. **Upload:**
   - Go to Actions → Build for PyPI
   - Click "Run workflow"
   - Check "Upload to PyPI"
   - Click "Run workflow"

## Artifacts

### Release Artifacts
- **Wheels:** Pre-compiled packages for all platforms
- **Source:** Source distribution (tar.gz)
- **Release Notes:** Auto-generated with installation instructions

### Build Artifacts
- **Wheels:** Platform-specific wheel files
- **Source:** Source distribution
- **Logs:** Build and test logs

## Requirements

### Repository Secrets
- `PYPI_TOKEN`: PyPI API token for uploading packages

### Dependencies
- Rust toolchain (stable)
- Python 3.8-3.12
- maturin (for building Rust extensions)
- Various Python packages (pytest, black, flake8, etc.)

## Workflow Features

### Caching
- **Rust dependencies:** Cached for faster builds
- **Python dependencies:** Cached via pip

### Matrix Testing
- **OS Matrix:** Ubuntu, Windows, macOS
- **Python Matrix:** 3.8, 3.9, 3.10, 3.11, 3.12

### Security
- **Rust audit:** Checks for known vulnerabilities
- **Python safety:** Checks for known security issues
- **Dependency scanning:** Automated security scanning

### Quality Checks
- **Code formatting:** Rust (fmt) and Python (black)
- **Linting:** Rust (clippy) and Python (flake8)
- **Type checking:** Python type hints validation

## Troubleshooting

### Common Issues

1. **Build failures:**
   - Check Rust toolchain version
   - Verify Python version compatibility
   - Check for missing system dependencies

2. **Upload failures:**
   - Verify PYPI_TOKEN is set correctly
   - Check package name availability
   - Ensure version number is unique

3. **Test failures:**
   - Check test file paths
   - Verify import statements
   - Check for missing test dependencies

### Debugging

1. **Check workflow logs:**
   - Go to Actions tab
   - Click on failed workflow
   - Expand failed step to see logs

2. **Local testing:**
   ```bash
   # Test build locally
   make venv
   make build
   
   # Test installation
   make install
   fast-exif-cli --version
   ```

## Contributing

When contributing to this repository:

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Run tests locally:** `make test`
5. **Push to your fork**
6. **Create a pull request**

The CI workflow will automatically test your changes across multiple platforms and Python versions.
