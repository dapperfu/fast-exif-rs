use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use memmap2::Mmap;
use crate::types::ExifError;
use crate::parsers::{HybridExifParser, PerformanceAwareParser};
use crate::field_mapping::FieldMapper;
use crate::computed_fields::ComputedFields;
use crate::value_formatter::ValueFormatter;

/// High-performance hybrid EXIF reader that automatically selects optimal parsing approach
#[pyclass]
pub struct HybridExifReader {
    hybrid_parser: HybridExifParser,
    performance_parser: PerformanceAwareParser,
}

#[pymethods]
impl HybridExifReader {
    /// Create a new hybrid EXIF reader
    #[new]
    pub fn new() -> Self {
        Self {
            hybrid_parser: HybridExifParser::new(),
            performance_parser: PerformanceAwareParser::new(),
        }
    }
    
    /// Read EXIF data from file using hybrid approach
    pub fn read_file(&mut self, file_path: &str) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let mut metadata = self.read_exif_hybrid(file_path)?;
            
            // Add computed fields for 1:1 exiftool compatibility
            ComputedFields::add_computed_fields(&mut metadata);
            
            // Normalize field names to exiftool standard for 1:1 compatibility
            FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
            
            // Normalize values to match PyExifTool raw format
            ValueFormatter::normalize_values_to_exiftool(&mut metadata);
            
            Ok(metadata.into_pyobject(py)?.into())
        })
    }
    
    /// Read EXIF data from bytes using hybrid approach
    pub fn read_bytes(&mut self, data: &[u8], file_extension: &str) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let mut metadata = self.read_exif_bytes_hybrid(data, file_extension)?;
            
            // Add computed fields for 1:1 exiftool compatibility
            ComputedFields::add_computed_fields(&mut metadata);
            
            // Normalize field names to exiftool standard for 1:1 compatibility
            FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
            
            // Normalize values to match PyExifTool raw format
            ValueFormatter::normalize_values_to_exiftool(&mut metadata);
            
            Ok(metadata.into_pyobject(py)?.into())
        })
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let stats = self.hybrid_parser.get_performance_stats();
            Ok(stats.into_pyobject(py)?.into())
        })
    }
    
    /// Set file size threshold for performance decisions
    pub fn set_file_size_threshold(&mut self, threshold: usize) {
        self.performance_parser.set_file_size_threshold(threshold);
    }
}

impl HybridExifReader {
    /// Read EXIF data from file using hybrid approach
    fn read_exif_hybrid(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        // Determine file extension
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let mut metadata = HashMap::new();
        
        // Use performance-aware parsing
        self.performance_parser.parse_with_performance_awareness(
            &mmap,
            extension,
            &mut metadata,
        )?;
        
        Ok(metadata)
    }
    
    /// Read EXIF data from bytes using hybrid approach
    fn read_exif_bytes_hybrid(
        &mut self,
        data: &[u8],
        file_extension: &str,
    ) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();
        
        // Use performance-aware parsing
        self.performance_parser.parse_with_performance_awareness(
            data,
            file_extension,
            &mut metadata,
        )?;
        
        Ok(metadata)
    }
}

impl Default for HybridExifReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark hybrid reader against standard reader
#[pyfunction]
pub fn benchmark_hybrid_vs_standard(file_paths: Vec<String>) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        use std::time::Instant;
        
        let mut hybrid_reader = HybridExifReader::new();
        let mut standard_reader = crate::FastExifReader::new();
        
        let mut hybrid_times = Vec::new();
        let mut standard_times = Vec::new();
        
        for file_path in file_paths {
            // Benchmark hybrid reader
            let start = Instant::now();
            let _ = hybrid_reader.read_file(&file_path);
            hybrid_times.push(start.elapsed().as_secs_f64());
            
            // Benchmark standard reader
            let start = Instant::now();
            let _ = standard_reader.read_file(&file_path);
            standard_times.push(start.elapsed().as_secs_f64());
        }
        
        let hybrid_avg = hybrid_times.iter().sum::<f64>() / hybrid_times.len() as f64;
        let standard_avg = standard_times.iter().sum::<f64>() / standard_times.len() as f64;
        let speedup = standard_avg / hybrid_avg;
        
        let results = HashMap::from([
            ("hybrid_avg_time".to_string(), hybrid_avg.to_string()),
            ("standard_avg_time".to_string(), standard_avg.to_string()),
            ("speedup".to_string(), speedup.to_string()),
            ("files_tested".to_string(), hybrid_times.len().to_string()),
        ]);
        
        Ok(results.into_pyobject(py)?.into())
    })
}

