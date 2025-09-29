use std::collections::HashMap;
use std::fs::File;
use memmap2::Mmap;
use crate::types::ExifError;
use crate::parsers::{HybridExifParser, PerformanceAwareParser};
use crate::field_mapping::FieldMapper;
use crate::computed_fields::ComputedFields;
use crate::value_formatter::ValueFormatter;
use rayon::prelude::*;

/// High-performance hybrid EXIF reader that automatically selects optimal parsing approach
pub struct HybridExifReader {
    hybrid_parser: HybridExifParser,
    performance_parser: PerformanceAwareParser,
}

impl HybridExifReader {
    /// Create a new hybrid EXIF reader
    pub fn new() -> Self {
        Self {
            hybrid_parser: HybridExifParser::new(),
            performance_parser: PerformanceAwareParser::new(),
        }
    }
    
    /// Read EXIF data from file using hybrid approach
    pub fn read_file(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.read_exif_hybrid(file_path)?;
        
        // Add computed fields for comprehensive metadata
        ComputedFields::add_computed_fields(&mut metadata);
        
        // Normalize field names to standard format
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        
        // Normalize values to standard format
        ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        
        Ok(metadata)
    }
    
    /// Read EXIF data from bytes using hybrid approach
    pub fn read_bytes(&mut self, data: &[u8], file_extension: &str) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.read_exif_bytes_hybrid(data, file_extension)?;
        
        // Add computed fields for comprehensive metadata
        ComputedFields::add_computed_fields(&mut metadata);
        
        // Normalize field names to standard format
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        
        // Normalize values to standard format
        ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        
        Ok(metadata)
    }
    
    /// Process multiple files in parallel using hybrid approach
    pub fn read_files_parallel(&mut self, file_paths: Vec<String>) -> Result<Vec<HashMap<String, String>>, ExifError> {
        // Use Rayon for true parallel processing
        let results: Result<Vec<_>, _> = file_paths
            .par_iter()
            .map(|file_path| {
                let file = File::open(file_path)?;
                let mmap = unsafe { Mmap::map(&file)? };
                
                let mut metadata = HashMap::new();
                // Create a temporary parser for this thread
                let mut temp_parser = HybridExifParser::new();
                temp_parser.parse_exif_hybrid(&mmap, "jpg", &mut metadata)?;
                
                // Add computed fields for comprehensive metadata
                ComputedFields::add_computed_fields(&mut metadata);
                
                // Normalize field names to standard format
                FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
                
                // Normalize values to standard format
                ValueFormatter::normalize_values_to_exiftool(&mut metadata);
                
                Ok(metadata)
            })
            .collect();
        
        results
    }
    
    /// Internal method to read EXIF data using hybrid approach
    fn read_exif_hybrid(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        let mut metadata = HashMap::new();
        self.hybrid_parser.parse_exif_hybrid(&mmap, "jpg", &mut metadata)?;
        
        Ok(metadata)
    }
    
    /// Internal method to read EXIF data from bytes using hybrid approach
    fn read_exif_bytes_hybrid(&mut self, data: &[u8], file_extension: &str) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();
        self.hybrid_parser.parse_exif_hybrid(data, file_extension, &mut metadata)?;
        
        Ok(metadata)
    }
}

impl Default for HybridExifReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark hybrid vs standard parsing approaches
pub fn benchmark_hybrid_vs_standard(file_paths: Vec<String>) -> Result<HashMap<String, String>, ExifError> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    let mut hybrid_reader = HybridExifReader::new();
    let mut standard_reader = crate::FastExifReader::new();
    
    let mut hybrid_total_time = 0.0;
    let mut standard_total_time = 0.0;
    let mut hybrid_successful = 0;
    let mut standard_successful = 0;
    
    for file_path in &file_paths {
        // Benchmark hybrid approach
        let hybrid_start = Instant::now();
        match hybrid_reader.read_file(file_path) {
            Ok(_) => {
                hybrid_successful += 1;
                hybrid_total_time += hybrid_start.elapsed().as_secs_f64();
            }
            Err(_) => {}
        }
        
        // Benchmark standard approach
        let standard_start = Instant::now();
        match standard_reader.read_file(file_path) {
            Ok(_) => {
                standard_successful += 1;
                standard_total_time += standard_start.elapsed().as_secs_f64();
            }
            Err(_) => {}
        }
    }
    
    let total_time = start_time.elapsed().as_secs_f64();
    
    let hybrid_files_per_second = if hybrid_total_time > 0.0 {
        hybrid_successful as f64 / hybrid_total_time
    } else {
        0.0
    };
    
    let standard_files_per_second = if standard_total_time > 0.0 {
        standard_successful as f64 / standard_total_time
    } else {
        0.0
    };
    
    let speedup = if standard_files_per_second > 0.0 {
        hybrid_files_per_second / standard_files_per_second
    } else {
        1.0
    };
    
    let mut results = HashMap::new();
    results.insert("total_files".to_string(), file_paths.len().to_string());
    results.insert("hybrid_successful".to_string(), hybrid_successful.to_string());
    results.insert("standard_successful".to_string(), standard_successful.to_string());
    results.insert("hybrid_total_time".to_string(), format!("{:.3}s", hybrid_total_time));
    results.insert("standard_total_time".to_string(), format!("{:.3}s", standard_total_time));
    results.insert("hybrid_files_per_second".to_string(), format!("{:.1}", hybrid_files_per_second));
    results.insert("standard_files_per_second".to_string(), format!("{:.1}", standard_files_per_second));
    results.insert("speedup".to_string(), format!("{:.2}x", speedup));
    results.insert("total_benchmark_time".to_string(), format!("{:.3}s", total_time));
    
    Ok(results)
}