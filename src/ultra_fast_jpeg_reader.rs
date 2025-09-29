use std::collections::HashMap;
use std::fs::File;
use memmap2::Mmap;
use crate::types::ExifError;
use crate::parsers::ultra_fast_jpeg::{UltraFastJpegParser, UltraFastBatchProcessor};
use crate::field_mapping::FieldMapper;
use crate::computed_fields::ComputedFields;
use crate::value_formatter::ValueFormatter;

/// Ultra-fast JPEG EXIF reader with completely rewritten algorithms
pub struct UltraFastJpegReader {
    /// Ultra-fast JPEG parser
    parser: UltraFastJpegParser,
    /// Batch processor for multiple files
    batch_processor: UltraFastBatchProcessor,
}

impl UltraFastJpegReader {
    /// Create a new ultra-fast JPEG reader
    pub fn new() -> Self {
        Self {
            parser: UltraFastJpegParser::new(),
            batch_processor: UltraFastBatchProcessor::new(100), // Process 100 files at a time
        }
    }
    
    /// Read EXIF data from JPEG file with ultra-fast parsing
    pub fn read_file(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.read_jpeg_exif_fast(file_path)?;
        
        // Add computed fields for comprehensive metadata
        ComputedFields::add_computed_fields(&mut metadata);
        
        // Normalize field names to standard format
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        
        // Normalize values to standard format
        ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        
        Ok(metadata)
    }
    
    /// Read EXIF data from JPEG bytes with ultra-fast parsing
    pub fn read_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.read_jpeg_exif_from_bytes(data)?;
        
        // Add computed fields for comprehensive metadata
        ComputedFields::add_computed_fields(&mut metadata);
        
        // Normalize field names to standard format
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        
        // Normalize values to standard format
        ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        
        Ok(metadata)
    }
    
    /// Process multiple JPEG files in batch with ultra-fast algorithms
    pub fn read_files_batch(&mut self, file_paths: Vec<String>) -> Result<Vec<HashMap<String, String>>, ExifError> {
        let processed_results = self.batch_processor.process_jpeg_files_ultra_fast(&file_paths)?;
        
        // Post-process each result
        let mut final_results = Vec::new();
        for mut metadata in processed_results {
            // Add computed fields for comprehensive metadata
            ComputedFields::add_computed_fields(&mut metadata);
            
            // Normalize field names to standard format
            FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
            
            // Normalize values to standard format
            ValueFormatter::normalize_values_to_exiftool(&mut metadata);
            
            final_results.push(metadata);
        }
        
        Ok(final_results)
    }
    
    /// Get performance statistics for the ultra-fast parser
    pub fn get_stats(&self) -> Result<HashMap<String, String>, ExifError> {
        Ok(self.parser.get_ultra_fast_stats())
    }
    
    /// Get batch processing statistics
    pub fn get_batch_stats(&self) -> Result<HashMap<String, String>, ExifError> {
        Ok(self.batch_processor.get_ultra_fast_stats())
    }
    
    /// Internal method to read JPEG EXIF data from file
    fn read_jpeg_exif_fast(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        let mut metadata = HashMap::new();
        self.parser.parse_jpeg_exif_ultra_fast(&mmap, &mut metadata)?;
        
        Ok(metadata)
    }
    
    /// Internal method to read JPEG EXIF data from bytes
    fn read_jpeg_exif_from_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();
        self.parser.parse_jpeg_exif_ultra_fast(data, &mut metadata)?;
        
        Ok(metadata)
    }
}

impl Default for UltraFastJpegReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark ultra-fast JPEG processing
pub fn benchmark_ultra_fast_jpeg(file_paths: Vec<String>) -> Result<HashMap<String, String>, ExifError> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    let mut reader = UltraFastJpegReader::new();
    
    let mut total_files = 0;
    let mut successful_files = 0;
    let mut total_fields = 0;
    
    for file_path in &file_paths {
        total_files += 1;
        match reader.read_file(file_path) {
            Ok(metadata) => {
                successful_files += 1;
                total_fields += metadata.len();
            }
            Err(_) => {
                // Count as failed but continue processing
            }
        }
    }
    
    let total_time = start_time.elapsed().as_secs_f64();
    let files_per_second = if total_time > 0.0 {
        total_files as f64 / total_time
    } else {
        0.0
    };
    
    let success_rate = if total_files > 0 {
        (successful_files as f64 / total_files as f64) * 100.0
    } else {
        0.0
    };
    
    let avg_fields_per_file = if successful_files > 0 {
        total_fields as f64 / successful_files as f64
    } else {
        0.0
    };
    
    let mut results = HashMap::new();
    results.insert("total_files".to_string(), total_files.to_string());
    results.insert("successful_files".to_string(), successful_files.to_string());
    results.insert("failed_files".to_string(), (total_files - successful_files).to_string());
    results.insert("success_rate".to_string(), format!("{:.2}%", success_rate));
    results.insert("total_time".to_string(), format!("{:.3}s", total_time));
    results.insert("files_per_second".to_string(), format!("{:.1}", files_per_second));
    results.insert("total_fields_extracted".to_string(), total_fields.to_string());
    results.insert("avg_fields_per_file".to_string(), format!("{:.1}", avg_fields_per_file));
    
    Ok(results)
}

/// Profile ultra-fast JPEG processing for a single file
pub fn profile_ultra_fast_jpeg(file_path: &str) -> Result<HashMap<String, String>, ExifError> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    let mut reader = UltraFastJpegReader::new();
    
    let metadata = reader.read_file(file_path)?;
    let processing_time = start_time.elapsed().as_secs_f64();
    
    let mut profile = HashMap::new();
    profile.insert("file_path".to_string(), file_path.to_string());
    profile.insert("processing_time".to_string(), format!("{:.6}s", processing_time));
    profile.insert("fields_extracted".to_string(), metadata.len().to_string());
    profile.insert("success".to_string(), "true".to_string());
    
    // Add some sample metadata fields
    if let Some(make) = metadata.get("Make") {
        profile.insert("camera_make".to_string(), make.clone());
    }
    if let Some(model) = metadata.get("Model") {
        profile.insert("camera_model".to_string(), model.clone());
    }
    if let Some(date_time) = metadata.get("DateTime") {
        profile.insert("date_taken".to_string(), date_time.clone());
    }
    
    Ok(profile)
}