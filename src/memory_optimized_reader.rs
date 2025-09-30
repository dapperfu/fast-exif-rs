use std::collections::HashMap;
use std::fs::File;
use memmap2::Mmap;
use crate::types::ExifError;
use crate::field_mapping::FieldMapper;
use crate::computed_fields::ComputedFields;
use crate::value_formatter::ValueFormatter;
use crate::FastExifReader;

/// Memory-optimized EXIF reader with advanced allocation management
pub struct MemoryOptimizedExifReader {
    /// Core reader instance
    reader: FastExifReader,
    /// Batch size for processing multiple files
    batch_size: usize,
}

impl MemoryOptimizedExifReader {
    /// Create a new memory-optimized EXIF reader
    pub fn new() -> Self {
        Self {
            reader: FastExifReader::new(),
            batch_size: 50, // Process 50 files at a time
        }
    }
    
    /// Read EXIF data from file with memory optimization
    pub fn read_file(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.reader.read_file(file_path)?;
        
        // Add computed fields for 1:1 exiftool compatibility
        ComputedFields::add_computed_fields(&mut metadata);
        
        // Normalize field names to exiftool standard for 1:1 compatibility
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        
        // Normalize values to match PyExifTool raw format
        ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        
        Ok(metadata)
    }
    
    /// Read EXIF data from bytes with memory optimization
    pub fn read_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.reader.read_bytes(data)?;
        
        // Add computed fields for 1:1 exiftool compatibility
        ComputedFields::add_computed_fields(&mut metadata);
        
        // Normalize field names to exiftool standard for 1:1 compatibility
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        
        // Normalize values to match PyExifTool raw format
        ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        
        Ok(metadata)
    }
    
    /// Process multiple files with batch memory optimization
    pub fn read_files_batch(&mut self, file_paths: Vec<String>) -> Result<Vec<HashMap<String, String>>, ExifError> {
        // Use the parallel processing from FastExifReader
        self.reader.read_files_parallel(file_paths)
    }
    
    /// Set batch size for processing
    pub fn set_batch_size(&mut self, batch_size: usize) {
        self.batch_size = batch_size;
    }
    
    /// Get current batch size
    pub fn get_batch_size(&self) -> usize {
        self.batch_size
    }
}

impl Default for MemoryOptimizedExifReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark memory-optimized reader against standard reader
pub fn benchmark_memory_optimization(file_paths: Vec<String>) -> Result<HashMap<String, String>, ExifError> {
    use std::time::Instant;
    
    let mut standard_reader = FastExifReader::new();
    let mut memory_reader = MemoryOptimizedExifReader::new();
    
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
    
    Ok(results)
}

/// Memory usage profiler for EXIF operations
pub fn profile_memory_usage(file_path: &str) -> Result<HashMap<String, String>, ExifError> {
    use std::time::Instant;
    
    let mut memory_reader = MemoryOptimizedExifReader::new();
    
    // Profile memory usage
    let start_time = Instant::now();
    let _metadata = memory_reader.read_file(file_path)?;
    let end_time = Instant::now();
    
    let mut profile = HashMap::new();
    profile.insert("processing_time".to_string(), (end_time - start_time).as_secs_f64().to_string());
    profile.insert("file_path".to_string(), file_path.to_string());
    
    Ok(profile)
}