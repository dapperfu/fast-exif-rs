#!/usr/bin/env python3
"""
Performance tests for fast-exif-reader
"""

import time
import tempfile
import os
from pathlib import Path
import pytest
from fast_exif_reader import FastExifReader

class TestPerformance:
    """Performance test suite"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.reader = FastExifReader()
        self.test_files = self.create_test_files()
    
    def create_test_files(self):
        """Create test files for benchmarking"""
        # This would normally create actual test files
        # For now, we'll use placeholder paths
        return [
            "test_data/canon_70d_sample.cr2",
            "test_data/nikon_z50_sample.nef",
            "test_data/sample_image.jpg"
        ]
    
    def test_canon_70d_performance(self):
        """Test Canon 70D CR2 file performance"""
        if not os.path.exists("test_data/canon_70d_sample.cr2"):
            pytest.skip("Test file not available")
        
        start_time = time.time()
        metadata = self.reader.read_file("test_data/canon_70d_sample.cr2")
        end_time = time.time()
        
        parse_time = end_time - start_time
        
        # Should parse in under 0.01 seconds
        assert parse_time < 0.01, f"Parse time too slow: {parse_time:.4f}s"
        
        # Should have essential metadata
        assert "Make" in metadata
        assert "Model" in metadata
        assert metadata["Make"] == "Canon"
    
    def test_nikon_z50_performance(self):
        """Test Nikon Z50 II NEF file performance"""
        if not os.path.exists("test_data/nikon_z50_sample.nef"):
            pytest.skip("Test file not available")
        
        start_time = time.time()
        metadata = self.reader.read_file("test_data/nikon_z50_sample.nef")
        end_time = time.time()
        
        parse_time = end_time - start_time
        
        # Should parse in under 0.01 seconds
        assert parse_time < 0.01, f"Parse time too slow: {parse_time:.4f}s"
        
        # Should have essential metadata
        assert "Make" in metadata
        assert "Model" in metadata
        assert metadata["Make"] == "Nikon"
    
    def test_jpeg_performance(self):
        """Test JPEG file performance"""
        if not os.path.exists("test_data/sample_image.jpg"):
            pytest.skip("Test file not available")
        
        start_time = time.time()
        metadata = self.reader.read_file("test_data/sample_image.jpg")
        end_time = time.time()
        
        parse_time = end_time - start_time
        
        # Should parse in under 0.005 seconds
        assert parse_time < 0.005, f"Parse time too slow: {parse_time:.4f}s"
    
    def test_batch_processing_performance(self):
        """Test batch processing performance"""
        if not os.path.exists("test_data/sample_image.jpg"):
            pytest.skip("Test file not available")
        
        # Test processing multiple files
        start_time = time.time()
        
        for _ in range(100):
            metadata = self.reader.read_file("test_data/sample_image.jpg")
        
        end_time = time.time()
        
        total_time = end_time - start_time
        avg_time = total_time / 100
        
        # Should average under 0.005 seconds per file
        assert avg_time < 0.005, f"Average parse time too slow: {avg_time:.4f}s"
    
    def test_memory_usage(self):
        """Test memory usage efficiency"""
        import psutil
        import os
        
        process = psutil.Process(os.getpid())
        initial_memory = process.memory_info().rss
        
        # Process multiple files
        for _ in range(50):
            metadata = self.reader.read_file("test_data/sample_image.jpg")
        
        final_memory = process.memory_info().rss
        memory_increase = final_memory - initial_memory
        
        # Memory increase should be minimal (under 10MB)
        assert memory_increase < 10 * 1024 * 1024, f"Memory usage too high: {memory_increase / 1024 / 1024:.2f}MB"
    
    def test_bytes_vs_file_performance(self):
        """Test bytes vs file reading performance"""
        if not os.path.exists("test_data/sample_image.jpg"):
            pytest.skip("Test file not available")
        
        # Test file reading
        start_time = time.time()
        metadata1 = self.reader.read_file("test_data/sample_image.jpg")
        file_time = time.time() - start_time
        
        # Test bytes reading
        with open("test_data/sample_image.jpg", "rb") as f:
            data = f.read()
        
        start_time = time.time()
        metadata2 = self.reader.read_bytes(data)
        bytes_time = time.time() - start_time
        
        # Both methods should be fast
        assert file_time < 0.01, f"File reading too slow: {file_time:.4f}s"
        assert bytes_time < 0.01, f"Bytes reading too slow: {bytes_time:.4f}s"
        
        # Results should be identical
        assert metadata1 == metadata2

if __name__ == "__main__":
    pytest.main([__file__])

