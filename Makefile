# Makefile for fast-exif-reader

.PHONY: help venv install install-dev build test clean benchmark format lint

# Virtual environment setup
venv: ## Create virtual environment
	python3 -m venv venv --copies
	venv/bin/pip install --upgrade pip
	venv/bin/pip install maturin click

# Ensure virtual environment exists
venv/bin/python:
	$(MAKE) venv

# Simple install that works with existing setup
install-simple: venv/bin/python ## Simple install using existing method
	venv/bin/pip install click
	# Note: Use 'maturin develop' manually if needed

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: venv/bin/python ## Install the package in development mode
	# Try alternative method using pip install -e .
	venv/bin/pip install -e .
	# If that fails, try manual maturin build
	@echo "If pip install failed, try: venv/bin/maturin develop"

install-dev: venv/bin/python ## Install with development dependencies
	venv/bin/pip install -e .
	venv/bin/pip install -e .[dev]

build: venv/bin/python ## Build the package
	venv/bin/pip install build
	venv/bin/python -m build

build-release: venv/bin/python ## Build the package in release mode
	venv/bin/pip install build
	venv/bin/python -m build

test: venv/bin/python ## Run tests
	venv/bin/python -m pytest tests/ -v

test-performance: venv/bin/python ## Run performance tests
	venv/bin/python -m pytest tests/test_performance.py -v

benchmark: venv/bin/python ## Run benchmarks
	venv/bin/python examples/benchmark.py

clean: ## Clean build artifacts
	cargo clean
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info/
	rm -rf venv/
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

format: venv/bin/python ## Format code
	cargo fmt
	venv/bin/black python/ examples/ tests/ cli/

lint: venv/bin/python ## Lint code
	cargo clippy
	venv/bin/flake8 python/ examples/ tests/ cli/

check: ## Check code without building
	cargo check

# Development targets
dev-setup: ## Set up development environment
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	$(MAKE) venv
	venv/bin/pip install pytest black flake8

# CLI specific targets
cli-test: venv/bin/python ## Test the CLI tool
	venv/bin/python tests/test_cli.py

cli-example: venv/bin/python ## Run CLI examples
	venv/bin/python cli_example.py

cli-install: venv/bin/python ## Install CLI dependencies
	venv/bin/pip install click

# Documentation
docs: ## Generate documentation
	cargo doc --open

# Package distribution
dist: ## Create distribution packages
	maturin build --release
	twine check dist/*

