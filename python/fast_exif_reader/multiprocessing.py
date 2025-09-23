"""
Multiprocessing support for FastExifReader

This module provides utilities for using FastExifReader with multiprocessing,
including pickle support and worker functions.
"""

import multiprocessing as mp
from concurrent.futures import ProcessPoolExecutor, as_completed
from typing import List, Dict, Any, Tuple, Optional
import time
from pathlib import Path

from .fast_exif_reader import FastExifReader


def extract_exif_worker(file_path: str) -> Tuple[str, Dict[str, str], float, bool]:
    """
    Worker function for multiprocessing EXIF extraction.
    
    This function creates a new FastExifReader instance in each worker process,
    avoiding pickle issues.
    
    Args:
        file_path: Path to the image file
        
    Returns:
        Tuple of (file_path, metadata_dict, processing_time, success_flag)
    """
    start_time = time.time()
    try:
        reader = FastExifReader()
        metadata = reader.read_file(file_path)
        end_time = time.time()
        return file_path, metadata, end_time - start_time, True
    except Exception as e:
        end_time = time.time()
        return file_path, {}, end_time - start_time, False


def extract_exif_batch(file_paths: List[str], 
                      max_workers: Optional[int] = None,
                      timeout: Optional[float] = None) -> Dict[str, Any]:
    """
    Extract EXIF data from multiple files using multiprocessing.
    
    Args:
        file_paths: List of file paths to process
        max_workers: Maximum number of worker processes (default: CPU count)
        timeout: Timeout per file in seconds (default: None)
        
    Returns:
        Dictionary with results, statistics, and timing information
    """
    if max_workers is None:
        max_workers = min(mp.cpu_count(), len(file_paths))
    
    results = {}
    processing_times = []
    success_count = 0
    error_count = 0
    
    start_time = time.time()
    
    with ProcessPoolExecutor(max_workers=max_workers) as executor:
        # Submit all tasks
        future_to_file = {
            executor.submit(extract_exif_worker, file_path): file_path 
            for file_path in file_paths
        }
        
        # Collect results as they complete
        for future in as_completed(future_to_file, timeout=timeout):
            try:
                file_path, metadata, processing_time, success = future.result()
                results[file_path] = {
                    'metadata': metadata,
                    'processing_time': processing_time,
                    'success': success
                }
                processing_times.append(processing_time)
                
                if success:
                    success_count += 1
                else:
                    error_count += 1
                    
            except Exception as e:
                file_path = future_to_file[future]
                results[file_path] = {
                    'metadata': {},
                    'processing_time': 0.0,
                    'success': False,
                    'error': str(e)
                }
                error_count += 1
    
    total_time = time.time() - start_time
    
    return {
        'results': results,
        'statistics': {
            'total_files': len(file_paths),
            'success_count': success_count,
            'error_count': error_count,
            'success_rate': success_count / len(file_paths) * 100 if file_paths else 0,
            'total_time': total_time,
            'avg_processing_time': sum(processing_times) / len(processing_times) if processing_times else 0,
            'files_per_second': len(file_paths) / total_time if total_time > 0 else 0
        }
    }


def extract_exif_from_directory(directory: str,
                               extensions: List[str] = None,
                               max_files: Optional[int] = None,
                               max_workers: Optional[int] = None) -> Dict[str, Any]:
    """
    Extract EXIF data from all image files in a directory.
    
    Args:
        directory: Directory path to scan
        extensions: List of file extensions to include (default: common image formats)
        max_files: Maximum number of files to process (default: None for all)
        max_workers: Maximum number of worker processes (default: CPU count)
        
    Returns:
        Dictionary with results and statistics
    """
    if extensions is None:
        extensions = ['.jpg', '.jpeg', '.cr2', '.nef', '.heic', '.heif', '.tiff', '.tif', '.png', '.bmp']
    
    # Find all image files
    file_paths = []
    directory_path = Path(directory)
    
    if not directory_path.exists():
        raise FileNotFoundError(f"Directory not found: {directory}")
    
    for file_path in directory_path.rglob('*'):
        if file_path.is_file() and file_path.suffix.lower() in extensions:
            file_paths.append(str(file_path))
            if max_files and len(file_paths) >= max_files:
                break
    
    if not file_paths:
        return {
            'results': {},
            'statistics': {
                'total_files': 0,
                'success_count': 0,
                'error_count': 0,
                'success_rate': 0,
                'total_time': 0,
                'avg_processing_time': 0,
                'files_per_second': 0
            }
        }
    
    return extract_exif_batch(file_paths, max_workers=max_workers)


class MultiprocessingExifReader:
    """
    A wrapper class that provides multiprocessing support for FastExifReader.
    
    This class avoids pickle issues by creating new FastExifReader instances
    in worker processes rather than trying to pickle the reader itself.
    """
    
    def __init__(self, max_workers: Optional[int] = None):
        """
        Initialize the multiprocessing EXIF reader.
        
        Args:
            max_workers: Maximum number of worker processes (default: CPU count)
        """
        self.max_workers = max_workers or mp.cpu_count()
    
    def read_files(self, file_paths: List[str]) -> Dict[str, Any]:
        """
        Read EXIF data from multiple files using multiprocessing.
        
        Args:
            file_paths: List of file paths to process
            
        Returns:
            Dictionary with results and statistics
        """
        return extract_exif_batch(file_paths, max_workers=self.max_workers)
    
    def read_directory(self, directory: str, 
                      extensions: List[str] = None,
                      max_files: Optional[int] = None) -> Dict[str, Any]:
        """
        Read EXIF data from all image files in a directory.
        
        Args:
            directory: Directory path to scan
            extensions: List of file extensions to include
            max_files: Maximum number of files to process
            
        Returns:
            Dictionary with results and statistics
        """
        return extract_exif_from_directory(
            directory, 
            extensions=extensions, 
            max_files=max_files, 
            max_workers=self.max_workers
        )


# Convenience functions for backward compatibility
def read_multiple_files(file_paths: List[str], max_workers: Optional[int] = None) -> Dict[str, Any]:
    """Convenience function for reading multiple files."""
    return extract_exif_batch(file_paths, max_workers=max_workers)


def read_directory(directory: str, max_files: Optional[int] = None, max_workers: Optional[int] = None) -> Dict[str, Any]:
    """Convenience function for reading all files in a directory."""
    return extract_exif_from_directory(directory, max_files=max_files, max_workers=max_workers)
