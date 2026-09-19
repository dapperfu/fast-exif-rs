# Variables
CARGO := cargo
INSTALL_DIR := ${HOME}/.local/bin
BINARY_NAME := exiftool-rs

.PHONY: all
all: build

.PHONY: build
build:
	${CARGO} build --release --manifest-path exiftool-rs/Cargo.toml

.PHONY: install
install:
	@mkdir -p ${INSTALL_DIR}
	${CARGO} install --path exiftool-rs --root ${HOME}/.local --force
	@echo "Installed ${BINARY_NAME} to ${INSTALL_DIR}"
	@echo "Make sure ${INSTALL_DIR} is in your PATH"

.PHONY: uninstall
uninstall:
	@rm -f ${INSTALL_DIR}/${BINARY_NAME}
	@echo "Removed ${BINARY_NAME} from ${INSTALL_DIR}"

.PHONY: clean
clean:
	${CARGO} clean
	${CARGO} clean --manifest-path exiftool-rs/Cargo.toml

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build     - Build the release binary"
	@echo "  install   - Install ${BINARY_NAME} to ~/.local/bin/"
	@echo "  uninstall - Remove ${BINARY_NAME} from ~/.local/bin/"
	@echo "  clean     - Clean build artifacts"
	@echo "  help      - Show this help message"
