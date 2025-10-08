# Makefile for fast-exif-rs project
# Builds and installs exiftool-rs binary to ~/.local/bin/

# Variables
CARGO := cargo
INSTALL_DIR := ${HOME}/.local/bin
BINARY_NAME := exiftool-rs
TARGET_DIR := exiftool-rs/target/release

# Default target
.PHONY: all
all: build

# Build the release binary
.PHONY: build
build:
	cd exiftool-rs && ${CARGO} build --release

# Install the binary to ~/.local/bin/
.PHONY: install
install: build
	@echo "Installing ${BINARY_NAME} to ${INSTALL_DIR}"
	@mkdir -p ${INSTALL_DIR}
	@cp ${TARGET_DIR}/${BINARY_NAME} ${INSTALL_DIR}/
	@echo "Installation complete. ${BINARY_NAME} is now available in ${INSTALL_DIR}"
	@echo "Make sure ${INSTALL_DIR} is in your PATH"

# Clean build artifacts
.PHONY: clean
clean:
	${CARGO} clean

# Run tests
.PHONY: test
test:
	${CARGO} test

# Check code without building
.PHONY: check
check:
	${CARGO} check

# Format code
.PHONY: fmt
fmt:
	${CARGO} fmt

# Lint code
.PHONY: clippy
clippy:
	${CARGO} clippy

# Uninstall the binary
.PHONY: uninstall
uninstall:
	@echo "Removing ${BINARY_NAME} from ${INSTALL_DIR}"
	@rm -f ${INSTALL_DIR}/${BINARY_NAME}
	@echo "Uninstallation complete"

# Show help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build     - Build the release binary"
	@echo "  install   - Install exiftool-rs to ~/.local/bin/"
	@echo "  uninstall - Remove exiftool-rs from ~/.local/bin/"
	@echo "  clean     - Clean build artifacts"
	@echo "  test      - Run tests"
	@echo "  check     - Check code without building"
	@echo "  fmt       - Format code"
	@echo "  clippy    - Lint code"
	@echo "  help      - Show this help message"
