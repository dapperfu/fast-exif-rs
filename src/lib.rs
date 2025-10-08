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
mod parsers;
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
        let mut metadata = self.read_exif_fast(file_path)?;
        
        // Add computed fields for comprehensive metadata
        crate::computed_fields::ComputedFields::add_computed_fields(&mut metadata);
        
        // Normalize field names to standard format
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        
        // Normalize values to standard format
        crate::value_formatter::ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        
        Ok(metadata)
    }

    /// Read EXIF data from bytes
    pub fn read_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = self.read_exif_from_bytes(data)?;
        
        // Add computed fields for comprehensive metadata
        crate::computed_fields::ComputedFields::add_computed_fields(&mut metadata);
        
        // Normalize field names to standard format
        FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
        
        // Normalize values to standard format
        crate::value_formatter::ValueFormatter::normalize_values_to_exiftool(&mut metadata);
        
        Ok(metadata)
    }

    /// Read EXIF data from multiple files in parallel
    pub fn read_files_parallel(&mut self, file_paths: Vec<String>) -> Result<Vec<HashMap<String, String>>, ExifError> {
        // Use Rayon for true parallel processing across multiple files
        let results: Result<Vec<_>, _> = file_paths
            .par_iter()
            .map(|file_path| {
                let file = File::open(file_path)?;
                let mmap = unsafe { Mmap::map(&file)? };
                
                // Create a temporary reader for this thread
                let mut temp_reader = FastExifReader::new();
                let mut metadata = temp_reader.read_exif_from_bytes(&mmap)?;
                
                // Add file system information that exiftool provides
                Self::add_file_system_metadata(file_path, &mut metadata);
                
                // Add computed fields for 1:1 exiftool compatibility
                crate::computed_fields::ComputedFields::add_computed_fields(&mut metadata);
                
                // Normalize field names to exiftool standard for 1:1 compatibility
                FieldMapper::normalize_metadata_to_exiftool(&mut metadata);
                
                // Normalize values to match PyExifTool raw format
                crate::value_formatter::ValueFormatter::normalize_values_to_exiftool(&mut metadata);
                
                Ok(metadata)
            })
            .collect();
        
        results
    }

    /// Read EXIF data from file path (internal implementation)
    fn read_exif_fast(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let mut metadata = self.read_exif_from_bytes(&mmap)?;
        
        // Add file system information
        Self::add_file_system_metadata(file_path, &mut metadata);
        
        Ok(metadata)
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

    /// Read EXIF data from bytes (internal implementation)
    fn read_exif_from_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();

        // Detect file format
        let format = EnhancedFormatDetector::detect_format(data)?;
        metadata.insert("Format".to_string(), format.clone());

        // Parse EXIF based on format
        match format.as_str() {
            "JPEG" => JpegParser::parse_jpeg_exif(data, &mut metadata)?,
            "CR2" => enhanced_cr2_parser::EnhancedCr2Parser::parse_cr2_exif(data, &mut metadata)?,
            "NEF" => RawParser::parse_nef_exif(data, &mut metadata)?,
            "ARW" => {
                let arw_metadata = EnhancedRawParser::parse_sony_arw(data)?;
                metadata.extend(arw_metadata);
            },
            "RAF" => {
                let raf_metadata = EnhancedRawParser::parse_fuji_raf(data)?;
                metadata.extend(raf_metadata);
            },
            "SRW" => {
                let srw_metadata = EnhancedRawParser::parse_samsung_srw(data)?;
                metadata.extend(srw_metadata);
            },
            "PEF" => {
                let pef_metadata = EnhancedRawParser::parse_pentax_pef(data)?;
                metadata.extend(pef_metadata);
            },
            "RW2" => {
                let rw2_metadata = EnhancedRawParser::parse_panasonic_rw2(data)?;
                metadata.extend(rw2_metadata);
            },
            "ORF" => RawParser::parse_orf_exif(data, &mut metadata)?,
            "DNG" => enhanced_dng_parser::EnhancedDngParser::parse_dng_exif(data, &mut metadata)?,
            "HEIF" | "HIF" => enhanced_heif_parser::EnhancedHeifParser::parse_heif_exif(data, &mut metadata)?,
            "MOV" => VideoParser::parse_mov_exif(data, &mut metadata)?,
            "MP4" => VideoParser::parse_mp4_exif(data, &mut metadata)?,
            "3GP" => VideoParser::parse_3gp_exif(data, &mut metadata)?,
            "AVI" => {
                let avi_metadata = EnhancedVideoParser::parse_avi(data)?;
                metadata.extend(avi_metadata);
            },
            "WMV" => {
                let wmv_metadata = EnhancedVideoParser::parse_wmv(data)?;
                metadata.extend(wmv_metadata);
            },
            "WEBM" => {
                let webm_metadata = EnhancedVideoParser::parse_webm(data)?;
                metadata.extend(webm_metadata);
            },
            "PNG" => PngParser::parse_png_exif(data, &mut metadata)?,
            "BMP" => BmpParser::parse_bmp_exif(data, &mut metadata)?,
            "GIF" => {
                let gif_metadata = EnhancedImageParser::parse_gif(data)?;
                metadata.extend(gif_metadata);
            },
            "WEBP" => {
                let webp_metadata = EnhancedImageParser::parse_webp(data)?;
                metadata.extend(webp_metadata);
            },
            "MKV" => MkvParser::parse_mkv_exif(data, &mut metadata)?,
            _ => {
                return Err(ExifError::UnsupportedFormat(format!(
                    "Unsupported format: {}",
                    format
                )))
            }
        }

        Ok(metadata)
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
