# Makefile for fast-exif-reader

.PHONY: help install install-dev build test clean benchmark format lint

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## Install the package in development mode
	maturin develop

install-dev: ## Install with development dependencies
	maturin develop
	pip install -e .[dev]

build: ## Build the package
	maturin build

build-release: ## Build the package in release mode
	maturin build --release

test: ## Run tests
	python -m pytest tests/ -v

test-performance: ## Run performance tests
	python -m pytest tests/test_performance.py -v

benchmark: ## Run benchmarks
	python examples/benchmark.py

clean: ## Clean build artifacts
	cargo clean
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info/
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

format: ## Format code
	cargo fmt
	black python/ examples/ tests/

lint: ## Lint code
	cargo clippy
	flake8 python/ examples/ tests/

check: ## Check code without building
	cargo check

# Development targets
dev-setup: ## Set up development environment
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	pip install maturin pytest black flake8

# Documentation
docs: ## Generate documentation
	cargo doc --open

# Package distribution
dist: ## Create distribution packages
	maturin build --release
	twine check dist/*

upload-test: ## Upload to test PyPI
	twine upload --repository testpypi dist/*

upload: ## Upload to PyPI
	twine upload dist/*

