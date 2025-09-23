#!/usr/bin/env python3
"""
Fast EXIF CLI - A high-performance EXIF metadata reader CLI tool

This CLI tool mimics exiftool's output format while leveraging the fast Rust backend
for optimal performance. It supports various output formats similar to exiftool.

Usage:
    fast-exif-cli image.jpg
    fast-exif-cli -s image.jpg          # Short format
    fast-exif-cli -S image.jpg         # Very short format  
    fast-exif-cli -t image.jpg         # Tab-delimited format
    fast-exif-cli -T image.jpg         # Table format
    fast-exif-cli -v image.jpg         # Verbose output
"""

import click
import sys
import os
from pathlib import Path
from typing import Dict, List, Optional, Any
import json

# Import our fast EXIF reader
try:
    from fast_exif_reader import FastExifReader
except ImportError:
    click.echo("Error: fast_exif_reader module not found. Please install the package first.", err=True)
    sys.exit(1)


class ExifFormatter:
    """Formats EXIF data to match exiftool's output styles"""
    
    def __init__(self, verbose: bool = False):
        self.verbose = verbose
    
    def format_default(self, metadata: Dict[str, str], filename: str) -> str:
        """Default format: Tag Name: Value"""
        lines = [f"======== {filename}"]
        lines.append("")
        
        for tag, value in sorted(metadata.items()):
            lines.append(f"{tag}: {value}")
        
        return "\n".join(lines)
    
    def format_short(self, metadata: Dict[str, str], filename: str) -> str:
        """Short format: TagName: Value"""
        lines = [f"======== {filename}"]
        lines.append("")
        
        for tag, value in sorted(metadata.items()):
            lines.append(f"{tag}: {value}")
        
        return "\n".join(lines)
    
    def format_very_short(self, metadata: Dict[str, str], filename: str) -> str:
        """Very short format: Value only"""
        lines = [f"======== {filename}"]
        lines.append("")
        
        for tag, value in sorted(metadata.items()):
            lines.append(value)
        
        return "\n".join(lines)
    
    def format_tab_delimited(self, metadata: Dict[str, str], filename: str) -> str:
        """Tab-delimited format: Description\tValue"""
        lines = [f"======== {filename}"]
        lines.append("")
        
        for tag, value in sorted(metadata.items()):
            lines.append(f"{tag}\t{value}")
        
        return "\n".join(lines)
    
    def format_table(self, metadata: Dict[str, str], filename: str) -> str:
        """Table format: Value only in table"""
        lines = [f"======== {filename}"]
        lines.append("")
        
        for tag, value in sorted(metadata.items()):
            lines.append(value)
        
        return "\n".join(lines)
    
    def format_json(self, metadata: Dict[str, str], filename: str) -> str:
        """JSON format"""
        output = {
            "SourceFile": filename,
            "ExifToolVersion": "fast-exif-cli 0.1.0",
            **metadata
        }
        return json.dumps(output, indent=2)


@click.command()
@click.argument('files', nargs=-1, type=click.Path(exists=True, path_type=Path))
@click.option('-s', '--short', 'output_format', flag_value='short', 
              help='Short format: TagName: Value')
@click.option('-S', '--very-short', 'output_format', flag_value='very_short',
              help='Very short format: Value only')
@click.option('-t', '--tab-delimited', 'output_format', flag_value='tab_delimited',
              help='Tab-delimited format: Description\\tValue')
@click.option('-T', '--table', 'output_format', flag_value='table',
              help='Table format: Value only')
@click.option('-j', '--json', 'output_format', flag_value='json',
              help='JSON format')
@click.option('-v', '--verbose', is_flag=True,
              help='Verbose output with additional information')
@click.option('-q', '--quiet', is_flag=True,
              help='Suppress error messages')
@click.option('-r', '--recursive', is_flag=True,
              help='Process directories recursively')
@click.option('--ext', 'extensions', multiple=True,
              help='File extensions to process (e.g., --ext jpg --ext cr2)')
@click.version_option(version='0.1.0', prog_name='fast-exif-cli', message='%(prog)s version %(version)s')
def main(files: tuple, output_format: str, verbose: bool, quiet: bool, 
         recursive: bool, extensions: tuple):
    """
    Fast EXIF CLI - High-performance EXIF metadata reader
    
    Mimics exiftool's output format while leveraging Rust backend for speed.
    
    Examples:
        fast-exif-cli image.jpg
        fast-exif-cli -s *.jpg
        fast-exif-cli -r --ext jpg --ext cr2 /path/to/photos
    """
    
    if not files:
        click.echo("Error: No files specified. Use --help for usage information.", err=True)
        sys.exit(1)
    
    # Set default format
    if not output_format:
        output_format = 'default'
    
    # Initialize formatter
    formatter = ExifFormatter(verbose=verbose)
    
    # Initialize EXIF reader
    reader = FastExifReader()
    
    # Process files
    processed_files = []
    errors = []
    
    for file_path in files:
        if file_path.is_file():
            if _should_process_file(file_path, extensions):
                processed_files.append(file_path)
        elif file_path.is_dir():
            if recursive:
                for subfile in _find_files_recursive(file_path, extensions):
                    processed_files.append(subfile)
            else:
                if not quiet:
                    click.echo(f"Warning: {file_path} is a directory. Use -r for recursive processing.", err=True)
        else:
            if not quiet:
                click.echo(f"Warning: {file_path} is not a valid file or directory.", err=True)
    
    if not processed_files:
        click.echo("Error: No valid files found to process.", err=True)
        sys.exit(1)
    
    # Process each file
    for file_path in processed_files:
        try:
            if verbose:
                click.echo(f"Processing: {file_path}", err=True)
            
            # Read EXIF data
            metadata = reader.read_file(str(file_path))
            
            # Format and output
            if output_format == 'default':
                output = formatter.format_default(metadata, str(file_path))
            elif output_format == 'short':
                output = formatter.format_short(metadata, str(file_path))
            elif output_format == 'very_short':
                output = formatter.format_very_short(metadata, str(file_path))
            elif output_format == 'tab_delimited':
                output = formatter.format_tab_delimited(metadata, str(file_path))
            elif output_format == 'table':
                output = formatter.format_table(metadata, str(file_path))
            elif output_format == 'json':
                output = formatter.format_json(metadata, str(file_path))
            else:
                output = formatter.format_default(metadata, str(file_path))
            
            click.echo(output)
            
            # Add separator between files (except for JSON format)
            if output_format != 'json' and len(processed_files) > 1:
                click.echo("")
                
        except Exception as e:
            error_msg = f"Error processing {file_path}: {str(e)}"
            errors.append(error_msg)
            if not quiet:
                click.echo(error_msg, err=True)
    
    # Report errors
    if errors and not quiet:
        click.echo(f"\nSummary: {len(errors)} error(s) occurred.", err=True)
    
    # Exit with error code if any errors occurred
    if errors:
        sys.exit(1)


def _should_process_file(file_path: Path, extensions: tuple) -> bool:
    """Check if file should be processed based on extensions"""
    if not extensions:
        # Default extensions if none specified
        default_extensions = {'.jpg', '.jpeg', '.cr2', '.nef', '.heic', '.heif', '.hif', '.tiff', '.tif'}
        return file_path.suffix.lower() in default_extensions
    
    return file_path.suffix.lower() in {ext.lower() for ext in extensions}


def _find_files_recursive(directory: Path, extensions: tuple) -> List[Path]:
    """Find files recursively in directory"""
    files = []
    
    try:
        for item in directory.rglob('*'):
            if item.is_file() and _should_process_file(item, extensions):
                files.append(item)
    except PermissionError:
        pass  # Skip directories we can't access
    
    return files


if __name__ == '__main__':
    main()
