use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use memmap2::Mmap;
use crate::types::ExifError;
use crate::parsers::ultra_fast_jpeg::{UltraFastJpegParser, UltraFastBatchProcessor};
use crate::field_mapping::FieldMapper;
use crate::computed_fields::ComputedFields;
use crate::value_formatter::ValueFormatter;

/// Ultra-fast JPEG EXIF reader with completely rewritten algorithms
#[pyclass]
pub struct UltraFastJpegReader {
    /// Ultra-fast JPEG parser
    parser: UltraFastJpegParser,
    /// Batch processor for multiple files
    batch_processor: UltraFastBatchProcessor,
}

#[pymethods]
impl UltraFastJpegReader {
    /// Create a new ultra-fast JPEG reader
    #[new]
    pub fn new() -> Self {
        Self {
            parser: UltraFastJpegParser::new(),
            batch_processor: UltraFastBatchProcessor::new(100), // Process 100 files at a time
        }
    }
    
    /// Read EXIF data from JPEG file with ultra-fast parsing
    pub fn read_file(&mut self, file_path: &str) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let mut metadata = self.read_jpeg_exif_fast(file_path)?;
            
            // Add computed fields for 1:1 exiftool compatibility
            ComputedFields::add_computed_fields(&mut metadata);
            
            // Normalize field names to exiftool standard for 1:1 compatibility
            FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
            
            // Normalize values to match PyExifTool raw format
            ValueFormatter::normalize_values_to_exiftool(&mut metadata);
            
            Ok(metadata.into_pyobject(py)?.into())
        })
    }
    
    /// Read EXIF data from JPEG bytes with ultra-fast parsing
    pub fn read_bytes(&mut self, data: &[u8]) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let mut metadata = HashMap::new();
            self.parser.parse_jpeg_exif_ultra_fast(data, &mut metadata)?;
            
            // Add computed fields for 1:1 exiftool compatibility
            ComputedFields::add_computed_fields(&mut metadata);
            
            // Normalize field names to exiftool standard for 1:1 compatibility
            FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
            
            // Normalize values to match PyExifTool raw format
            ValueFormatter::normalize_values_to_exiftool(&mut metadata);
            
            Ok(metadata.into_pyobject(py)?.into())
        })
    }
    
    /// Process multiple JPEG files with ultra-fast batch processing
    pub fn read_files_batch(&mut self, file_paths: Vec<String>) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let results = self.batch_processor.process_jpeg_files_ultra_fast(&file_paths)?;
            
            // Process each result with computed fields and normalization
            let mut processed_results = Vec::with_capacity(results.len());
            for mut metadata in results {
                ComputedFields::add_computed_fields(&mut metadata);
                FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
                ValueFormatter::normalize_values_to_exiftool(&mut metadata);
                processed_results.push(metadata);
            }
            
            Ok(processed_results.into_pyobject(py)?.into())
        })
    }
    
    /// Get ultra-fast parser statistics
    pub fn get_stats(&self) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let stats = self.parser.get_ultra_fast_stats();
            Ok(stats.into_pyobject(py)?.into())
        })
    }
    
    /// Get batch processor statistics
    pub fn get_batch_stats(&self) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let stats = self.batch_processor.get_ultra_fast_stats();
            Ok(stats.into_pyobject(py)?.into())
        })
    }
    
    /// Clear caches for memory management
    pub fn clear_caches(&mut self) {
        self.parser.clear_caches();
    }
    
    /// Set batch size for processing
    pub fn set_batch_size(&mut self, batch_size: usize) {
        self.batch_processor = UltraFastBatchProcessor::new(batch_size);
    }
}

impl UltraFastJpegReader {
    /// Read JPEG EXIF data with ultra-fast parsing
    fn read_jpeg_exif_fast(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        let mut metadata = HashMap::new();
        self.parser.parse_jpeg_exif_ultra_fast(&mmap, &mut metadata)?;
        
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

impl Default for UltraFastJpegReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark ultra-fast JPEG reader against standard reader
#[pyfunction]
pub fn benchmark_ultra_fast_jpeg(file_paths: Vec<String>) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        use std::time::Instant;
        
        let mut standard_reader = crate::FastExifReader::new();
        let mut ultra_fast_reader = UltraFastJpegReader::new();
        
        let mut standard_times = Vec::new();
        let mut ultra_fast_times = Vec::new();
        
        for file_path in file_paths {
            // Benchmark standard reader
            let start = Instant::now();
            let _ = standard_reader.read_file(&file_path);
            standard_times.push(start.elapsed().as_secs_f64());
            
            // Benchmark ultra-fast reader
            let start = Instant::now();
            let _ = ultra_fast_reader.read_file(&file_path);
            ultra_fast_times.push(start.elapsed().as_secs_f64());
        }
        
        let standard_avg = standard_times.iter().sum::<f64>() / standard_times.len() as f64;
        let ultra_fast_avg = ultra_fast_times.iter().sum::<f64>() / ultra_fast_times.len() as f64;
        let speedup = standard_avg / ultra_fast_avg;
        
        let mut results = HashMap::new();
        results.insert("standard_avg_time".to_string(), standard_avg.to_string());
        results.insert("ultra_fast_avg_time".to_string(), ultra_fast_avg.to_string());
        results.insert("speedup".to_string(), speedup.to_string());
        results.insert("files_tested".to_string(), standard_times.len().to_string());
        
        // Add ultra-fast parser statistics
        let parser_stats = ultra_fast_reader.get_stats();
        results.insert("parser_stats".to_string(), format!("{:?}", parser_stats));
        
        Ok(results.into_pyobject(py)?.into())
    })
}

/// Profile ultra-fast JPEG parsing performance
#[pyfunction]
pub fn profile_ultra_fast_jpeg(file_path: &str) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        use std::time::Instant;
        
        let mut ultra_fast_reader = UltraFastJpegReader::new();
        
        // Profile parsing performance
        let start_time = Instant::now();
        let _metadata = ultra_fast_reader.read_file(file_path)?;
        let end_time = Instant::now();
        
        let mut profile = HashMap::new();
        profile.insert("parsing_time".to_string(), (end_time - start_time).as_secs_f64().to_string());
        profile.insert("file_path".to_string(), file_path.to_string());
        
        // Add parser statistics
        let stats = ultra_fast_reader.get_stats();
        profile.insert("parser_stats".to_string(), format!("{:?}", stats));
        
        Ok(profile.into_pyobject(py)?.into())
    })
}
