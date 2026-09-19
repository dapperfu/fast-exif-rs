//! Fast EXIF Reader - Pure Rust Implementation
//! 
//! A high-performance EXIF metadata extraction library written in Rust.
//! Provides comprehensive support for image and video formats with exceptional performance.

use std::collections::HashMap;

// Module declarations
mod format_detection;
pub mod parsers;
mod types;
mod utils;
mod writer;
mod exif_encode;
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
pub use types::{ExifError, ExifResult, ParseScope, ProcessingStats, ReadOptions};
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
        self.read_file_with_options(file_path, &ReadOptions::full())
    }

    /// Read EXIF data with control over which tag groups are parsed.
    ///
    /// [`ReadOptions::datetime`] and [`ReadOptions::tags`] skip maker notes and
    /// (unless requested) GPS, which is the usual way to trade tag coverage for
    /// speed.
    pub fn read_file_with_options(
        &mut self,
        file_path: &str,
        options: &ReadOptions,
    ) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.parser.parse_file_with_options(file_path, options)?;
        if options.include_file_system {
            Self::add_file_system_metadata(file_path, &mut metadata);
        }
        if options.include_computed_fields {
            crate::computed_fields::ComputedFields::add_computed_fields(&mut metadata);
        }
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        crate::value_formatter::ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        if let Some(wanted) = &options.wanted_tags {
            let wanted_n = crate::types::wanted_normalized_set(wanted);
            metadata.retain(|key, _| crate::types::tag_is_wanted_normalized(key, &wanted_n));
        }
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
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let datetime = Self::timestamp_to_datetime(metadata_fs.ctime() as u64);
                metadata.insert("FileInodeChangeDate".to_string(), datetime);
            }
            #[cfg(not(unix))]
            {
                if let Ok(modified) = metadata_fs.modified() {
                    if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                        metadata.insert(
                            "FileInodeChangeDate".to_string(),
                            Self::timestamp_to_datetime(duration.as_secs()),
                        );
                    }
                }
            }
        }
    }
    
    /// Convert Unix timestamp to ExifTool file-date format (local + offset).
    fn timestamp_to_datetime(timestamp: u64) -> String {
        use chrono::{Local, TimeZone};
        match Local.timestamp_opt(timestamp as i64, 0).single() {
            Some(dt) => dt.format("%Y:%m:%d %H:%M:%S%:z").to_string(),
            None => String::new(),
        }
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
