#!/usr/bin/env python3
"""
Setup script for fast-exif-reader package.
This is a fallback setup script for environments that don't support pyproject.toml.
"""

from setuptools import setup, find_packages
import os
import versioneer

# Read the README file
def read_readme():
    with open("README.md", "r", encoding="utf-8") as fh:
        return fh.read()

# Read requirements
def read_requirements():
    requirements = []
    if os.path.exists("requirements.txt"):
        with open("requirements.txt", "r", encoding="utf-8") as fh:
            requirements = [line.strip() for line in fh if line.strip() and not line.startswith("#")]
    return requirements

setup(
    name="fast-exif-reader",
    version=versioneer.get_version(),
    cmdclass=versioneer.get_cmdclass(),
    author="Your Name",
    author_email="your.email@example.com",
    description="Fast EXIF reader optimized for Canon 70D and Nikon Z50 II",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    url="https://github.com/dapperfu/fast-exif-rs",
    project_urls={
        "Bug Tracker": "https://github.com/dapperfu/fast-exif-rs/issues",
        "Documentation": "https://github.com/dapperfu/fast-exif-rs#readme",
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Rust",
        "Topic :: Multimedia :: Graphics",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    keywords=["exif", "metadata", "image", "canon", "nikon", "rust", "performance"],
    packages=find_packages(where="python"),
    package_dir={"": "python"},
    python_requires=">=3.8",
    install_requires=read_requirements(),
    extras_require={
        "dev": [
            "pytest>=6.0",
            "pytest-benchmark>=3.0",
            "psutil>=5.8",
        ],
    },
    include_package_data=True,
    zip_safe=False,
)

