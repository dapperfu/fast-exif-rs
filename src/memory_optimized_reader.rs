use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::collections::HashMap;
use std::fs::File;
use memmap2::Mmap;
use crate::types::ExifError;
use crate::memory_optimization::{MemoryOptimizedReader, BatchMemoryOptimizer};
use crate::field_mapping::FieldMapper;
use crate::computed_fields::ComputedFields;
use crate::value_formatter::ValueFormatter;

/// Memory-optimized EXIF reader with advanced allocation management
#[pyclass]
pub struct MemoryOptimizedExifReader {
    /// Core memory-optimized reader
    reader: MemoryOptimizedReader,
    /// Batch optimizer for multiple files
    batch_optimizer: BatchMemoryOptimizer,
}

#[pymethods]
impl MemoryOptimizedExifReader {
    /// Create a new memory-optimized EXIF reader
    #[new]
    pub fn new() -> Self {
        Self {
            reader: MemoryOptimizedReader::new(),
            batch_optimizer: BatchMemoryOptimizer::new(50), // Process 50 files at a time
        }
    }
    
    /// Read EXIF data from file with memory optimization
    pub fn read_file(&mut self, file_path: &str) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let mut metadata = self.read_exif_optimized(file_path)?;
            
            // Add computed fields for 1:1 exiftool compatibility
            ComputedFields::add_computed_fields(&mut metadata);
            
            // Normalize field names to exiftool standard for 1:1 compatibility
            FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
            
            // Normalize values to match PyExifTool raw format
            ValueFormatter::normalize_values_to_exiftool(&mut metadata);
            
            Ok(metadata.into_pyobject(py)?.into())
        })
    }
    
    /// Read EXIF data from bytes with memory optimization
    pub fn read_bytes(&mut self, data: &[u8], file_extension: &str) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let mut metadata = self.reader.parse_exif_optimized(data, file_extension)?;
            
            // Add computed fields for 1:1 exiftool compatibility
            ComputedFields::add_computed_fields(&mut metadata);
            
            // Normalize field names to exiftool standard for 1:1 compatibility
            FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
            
            // Normalize values to match PyExifTool raw format
            ValueFormatter::normalize_values_to_exiftool(&mut metadata);
            
            Ok(metadata.into_pyobject(py)?.into())
        })
    }
    
    /// Process multiple files with batch memory optimization
    pub fn read_files_batch(&self, file_paths: Vec<String>) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let results = self.batch_optimizer.process_files_batch(
                &file_paths,
                |file_path| self.read_exif_optimized(file_path)
            )?;
            
            Ok(results.into_pyobject(py)?.into())
        })
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let stats = self.reader.get_memory_stats();
            let batch_stats = self.batch_optimizer.get_batch_stats();
            
            let mut result = HashMap::new();
            result.insert("reader_stats".to_string(), format!("{:?}", stats));
            result.insert("batch_stats".to_string(), format!("{:?}", batch_stats));
            
            Ok(result.into_pyobject(py)?.into())
        })
    }
    
    /// Clear memory caches
    pub fn clear_caches(&mut self) {
        self.reader.clear_caches();
    }
    
    /// Set batch size for processing
    pub fn set_batch_size(&mut self, batch_size: usize) {
        self.batch_optimizer = BatchMemoryOptimizer::new(batch_size);
    }
}

impl MemoryOptimizedExifReader {
    /// Read EXIF data with memory optimization
    fn read_exif_optimized(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let mut metadata = self.reader.parse_exif_optimized(&mmap, extension)?;
        
        // Add file system information
        Self::add_file_system_metadata(file_path, &mut metadata);
        
        Ok(metadata)
    }
    
    /// Add file system metadata
    fn add_file_system_metadata(file_path: &str, metadata: &mut HashMap<String, String>) {
        use std::path::Path;
        
        if let Some(file_name) = Path::new(file_path).file_name() {
            if let Some(name_str) = file_name.to_str() {
                metadata.insert("FileName".to_string(), name_str.to_string());
            }
        }
        
        if let Some(parent) = Path::new(file_path).parent() {
            if let Some(parent_str) = parent.to_str() {
                metadata.insert("Directory".to_string(), parent_str.to_string());
            }
        }
        
        // Add file size
        if let Ok(metadata_fs) = std::fs::metadata(file_path) {
            metadata.insert("FileSize".to_string(), metadata_fs.len().to_string());
        }
    }
}

impl Default for MemoryOptimizedExifReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark memory-optimized reader against standard reader
#[pyfunction]
pub fn benchmark_memory_optimization(file_paths: Vec<String>) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        use std::time::Instant;
        
        let mut standard_reader = crate::FastExifReader::new();
        let memory_reader = MemoryOptimizedExifReader::new();
        
        let mut standard_times = Vec::new();
        let mut memory_times = Vec::new();
        
        for file_path in file_paths {
            // Benchmark standard reader
            let start = Instant::now();
            let _ = standard_reader.read_file(&file_path);
            standard_times.push(start.elapsed().as_secs_f64());
            
            // Benchmark memory-optimized reader
            let start = Instant::now();
            let _ = memory_reader.read_file(&file_path);
            memory_times.push(start.elapsed().as_secs_f64());
        }
        
        let standard_avg = standard_times.iter().sum::<f64>() / standard_times.len() as f64;
        let memory_avg = memory_times.iter().sum::<f64>() / memory_times.len() as f64;
        let speedup = standard_avg / memory_avg;
        
        let mut results = HashMap::new();
        results.insert("standard_avg_time".to_string(), standard_avg.to_string());
        results.insert("memory_avg_time".to_string(), memory_avg.to_string());
        results.insert("speedup".to_string(), speedup.to_string());
        results.insert("files_tested".to_string(), standard_times.len().to_string());
        
        // Add memory statistics
        let memory_stats = memory_reader.get_memory_stats();
        results.insert("memory_stats".to_string(), format!("{:?}", memory_stats));
        
        Ok(results.into_pyobject(py)?.into())
    })
}

/// Memory usage profiler for EXIF operations
#[pyfunction]
pub fn profile_memory_usage(file_path: &str) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        use std::time::Instant;
        
        let memory_reader = MemoryOptimizedExifReader::new();
        
        // Profile memory usage
        let start_time = Instant::now();
        let start_stats = memory_reader.get_memory_stats();
        
        let _metadata = memory_reader.read_file(file_path)?;
        
        let end_time = Instant::now();
        let end_stats = memory_reader.get_memory_stats();
        
        let mut profile = HashMap::new();
        profile.insert("processing_time".to_string(), (end_time - start_time).as_secs_f64().to_string());
        profile.insert("start_stats".to_string(), format!("{:?}", start_stats));
        profile.insert("end_stats".to_string(), format!("{:?}", end_stats));
        profile.insert("file_path".to_string(), file_path.to_string());
        
        Ok(profile.into_pyobject(py)?.into())
    })
}
