//! Fast EXIF Reader - Pure Rust Implementation
//! 
//! A high-performance EXIF metadata extraction library written in Rust.
//! Provides comprehensive support for image and video formats with exceptional performance.

use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use rayon::prelude::*;

// Module declarations
mod format_detection;
pub mod parsers;
mod types;
mod utils;
mod writer;
mod exif_copier;

// Enhanced format support modules
mod enhanced_format_detection;
mod enhanced_raw_parser;
mod enhanced_video_parser;
mod enhanced_image_parser;
mod enhanced_cr2_parser;
mod enhanced_heif_parser;
mod enhanced_dng_parser;
mod field_mapping;
mod computed_fields;
mod value_formatter;

// Re-export commonly used types
pub use format_detection::FormatDetector;
pub use parsers::{OptimalExifParser, OptimalBatchProcessor, BmpParser, HeifParser, JpegParser, MkvParser, PngParser, RawParser, VideoParser};
pub use types::{ExifError, ExifResult, ProcessingStats};
pub use utils::ExifUtils;
pub use writer::ExifWriter;
pub use exif_copier::ExifCopier;

// Re-export enhanced parsers
pub use enhanced_format_detection::EnhancedFormatDetector;
pub use enhanced_raw_parser::EnhancedRawParser;
pub use enhanced_video_parser::EnhancedVideoParser;
pub use enhanced_image_parser::EnhancedImageParser;
pub use field_mapping::FieldMapper;

/// Fast EXIF reader with comprehensive multimedia support
#[derive(Clone)]
pub struct FastExifReader {
    /// Optimal parser for maximum performance
    parser: OptimalExifParser,
}

impl FastExifReader {
    /// Create a new FastExifReader instance
    pub fn new() -> Self {
        Self {
            parser: OptimalExifParser::new(),
        }
    }

    /// Read EXIF data from file path
    pub fn read_file(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.parser.parse_file(file_path)?;
        Self::add_file_system_metadata(file_path, &mut metadata);
        crate::computed_fields::ComputedFields::add_computed_fields(&mut metadata);
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        crate::value_formatter::ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        Ok(metadata)
    }

    /// Read EXIF data from bytes
    pub fn read_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.parser.parse_exif_from_bytes(data)?;
        crate::computed_fields::ComputedFields::add_computed_fields(&mut metadata);
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        crate::value_formatter::ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        Ok(metadata)
    }

    /// Read EXIF data from multiple files in parallel
    pub fn read_files_parallel(&mut self, file_paths: Vec<String>) -> Result<Vec<HashMap<String, String>>, ExifError> {
        let mut processor = OptimalBatchProcessor::new(50);
        processor.process_files(&file_paths)
    }

    /// Add file system metadata
    fn add_file_system_metadata(file_path: &str, metadata: &mut HashMap<String, String>) {
        use std::path::Path;
        use std::fs;
        use std::time::UNIX_EPOCH;
        
        let path = Path::new(file_path);
        
        // Add file name and directory
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                metadata.insert("FileName".to_string(), name_str.to_string());
            }
        }
        
        if let Some(parent) = path.parent() {
            if let Some(parent_str) = parent.to_str() {
                metadata.insert("Directory".to_string(), parent_str.to_string());
            }
        }
        
        // Add source file path
        metadata.insert("SourceFile".to_string(), file_path.to_string());
        
        // Add file metadata
        if let Ok(metadata_fs) = fs::metadata(file_path) {
            // File size
            metadata.insert("FileSize".to_string(), metadata_fs.len().to_string());
            
            // File permissions (Unix-style)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = metadata_fs.permissions();
                let mode = permissions.mode();
                metadata.insert("FilePermissions".to_string(), format!("{:o}", mode));
            }
            
            // File modification time
            if let Ok(modified) = metadata_fs.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    let timestamp = duration.as_secs();
                    let datetime = Self::timestamp_to_datetime(timestamp);
                    metadata.insert("FileModifyDate".to_string(), datetime);
                }
            }
            
            // File access time
            if let Ok(accessed) = metadata_fs.accessed() {
                if let Ok(duration) = accessed.duration_since(UNIX_EPOCH) {
                    let timestamp = duration.as_secs();
                    let datetime = Self::timestamp_to_datetime(timestamp);
                    metadata.insert("FileAccessDate".to_string(), datetime);
                }
            }
            
            // File creation time (if available)
            #[cfg(target_os = "macos")]
            {
                use std::os::macos::fs::MetadataExt;
                let created = metadata_fs.created();
                if let Ok(created) = created {
                    if let Ok(duration) = created.duration_since(UNIX_EPOCH) {
                        let timestamp = duration.as_secs();
                        let datetime = Self::timestamp_to_datetime(timestamp);
                        metadata.insert("FileInodeChangeDate".to_string(), datetime);
                    }
                }
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                // For other systems, use modification time as fallback
                if let Ok(modified) = metadata_fs.modified() {
                    if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                        let timestamp = duration.as_secs();
                        let datetime = Self::timestamp_to_datetime(timestamp);
                        metadata.insert("FileInodeChangeDate".to_string(), datetime);
                    }
                }
            }
        }
    }
    
    /// Convert Unix timestamp to EXIF datetime format
    fn timestamp_to_datetime(timestamp: u64) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let datetime = UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
        let system_time = SystemTime::from(datetime);
        
        // Format as "YYYY:MM:DD HH:MM:SS"
        let datetime_chrono = chrono::DateTime::<chrono::Utc>::from(system_time);
        datetime_chrono.format("%Y:%m:%d %H:%M:%S").to_string()
    }

}

impl Default for FastExifReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Fast EXIF writer for adding/modifying EXIF metadata
#[derive(Clone)]
pub struct FastExifWriter {
    writer: ExifWriter,
}

impl FastExifWriter {
    /// Create a new FastExifWriter instance
    pub fn new() -> Self {
        Self {
            writer: ExifWriter::new(),
        }
    }

    /// Write EXIF metadata to an image file (auto-detects format)
    pub fn write_exif(
        &self,
        input_path: &str,
        output_path: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<(), ExifError> {
        self.writer.write_exif(input_path, output_path, metadata)
    }

    /// Write EXIF metadata to image bytes (auto-detects format)
    pub fn write_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        self.writer.write_exif_to_bytes(input_data, metadata)
    }

    /// Copy high-priority EXIF fields from source to target image
    pub fn copy_high_priority_exif(
        &self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> Result<(), ExifError> {
        self.writer.copy_high_priority_exif(source_path, target_path, output_path)
    }

    /// Copy high-priority EXIF fields from source bytes to target bytes
    pub fn copy_high_priority_exif_to_bytes(
        &self,
        source_data: &[u8],
        target_data: &[u8],
    ) -> Result<Vec<u8>, ExifError> {
        self.writer.copy_high_priority_exif_to_bytes(source_data, target_data)
    }
}

impl Default for FastExifWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Fast EXIF copier for copying metadata between images
#[derive(Clone)]
pub struct FastExifCopier {
    copier: ExifCopier,
}

impl FastExifCopier {
    /// Create a new FastExifCopier instance
    pub fn new() -> Self {
        Self {
            copier: ExifCopier::new(),
        }
    }

    /// Copy high-priority EXIF fields from source to target image
    pub fn copy_high_priority_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> Result<(), ExifError> {
        self.copier.copy_high_priority_exif(source_path, target_path, output_path)
    }

    /// Copy high-priority EXIF fields from source bytes to target bytes
    pub fn copy_high_priority_exif_to_bytes(
        &mut self,
        source_data: &[u8],
        target_data: &[u8],
    ) -> Result<Vec<u8>, ExifError> {
        self.copier.copy_high_priority_exif_to_bytes(source_data, target_data)
    }

    /// Copy all EXIF fields from source to target image
    pub fn copy_all_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> Result<(), ExifError> {
        self.copier.copy_all_exif(source_path, target_path, output_path)
    }

    /// Copy specific EXIF fields from source to target image
    pub fn copy_specific_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
        field_names: &[&str],
    ) -> Result<(), ExifError> {
        self.copier.copy_specific_exif(source_path, target_path, output_path, field_names)
    }

    /// Get available EXIF fields from source image
    pub fn get_available_fields(&mut self, source_path: &str) -> Result<Vec<String>, ExifError> {
        self.copier.get_available_fields(source_path)
    }

    /// Get high-priority EXIF fields from source image
    pub fn get_high_priority_fields(&mut self, source_path: &str) -> Result<HashMap<String, String>, ExifError> {
        self.copier.get_high_priority_fields(source_path)
    }
}

impl Default for FastExifCopier {
    fn default() -> Self {
        Self::new()
    }
}
