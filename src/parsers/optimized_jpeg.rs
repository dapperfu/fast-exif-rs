use crate::format_detection::FormatDetector;
use crate::parsers::tiff::TiffParser;
use crate::types::ExifError;
use crate::memory_optimization::MemoryPool;
use std::collections::HashMap;

/// Memory-optimized JPEG EXIF parser
pub struct OptimizedJpegParser {
    /// Memory pool for reuse
    memory_pool: MemoryPool,
    /// Pre-allocated buffer for EXIF segment extraction
    exif_buffer: Vec<u8>,
}

impl OptimizedJpegParser {
    /// Create a new optimized JPEG parser
    pub fn new() -> Self {
        Self {
            memory_pool: MemoryPool::new(),
            exif_buffer: Vec::with_capacity(1024 * 1024), // 1MB buffer
        }
    }
    
    /// Parse EXIF data from JPEG format with memory optimization
    pub fn parse_jpeg_exif_optimized(
        &mut self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        // Clear and reuse buffer
        self.exif_buffer.clear();
        
        // Find EXIF segment efficiently
        if let Some(exif_data) = self.find_jpeg_exif_segment_optimized(data) {
            // Use optimized TIFF parsing
            TiffParser::parse_tiff_exif(exif_data, metadata)?;
        } else {
            // No EXIF segment found - extract basic file information
            self.extract_basic_jpeg_info_optimized(data, metadata);
        }
        
        // Extract JFIF information with optimization
        self.extract_jfif_info_optimized(data, metadata);
        
        // Detect camera make efficiently
        if !metadata.contains_key("Make") {
            if let Some(make) = FormatDetector::detect_camera_make(data) {
                metadata.insert("Make".to_string(), make);
            }
        }
        
        // Extract camera-specific metadata with optimization
        self.extract_camera_specific_metadata_optimized(data, metadata);
        
        // Add computed fields efficiently
        self.add_computed_fields_optimized(metadata);
        
        // Post-process fields with optimization
        self.post_process_problematic_fields_optimized(metadata);
        
        Ok(())
    }
    
    /// Find JPEG EXIF segment with memory optimization
    fn find_jpeg_exif_segment_optimized<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        // Use pre-allocated buffer for efficient searching
        let mut best_exif_segment: Option<&[u8]> = None;
        let mut best_segment_size = 0;
        
        for i in 0..data.len().saturating_sub(4) {
            if data[i] == 0xFF && data[i + 1] == 0xE1 {
                // Found EXIF marker
                if i + 4 < data.len() {
                    let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                    if i + 4 + length <= data.len() {
                        let segment_data = &data[i + 4..i + 4 + length];
                        
                        // Look for "Exif" identifier in this segment
                        for exif_start in 0..segment_data.len().saturating_sub(4) {
                            if &segment_data[exif_start..exif_start + 4] == b"Exif" {
                                let exif_data_start = exif_start + 4;
                                if exif_data_start < segment_data.len() {
                                    let exif_data = &segment_data[exif_data_start..];
                                    let exif_size = exif_data.len();
                                    
                                    // Use the largest EXIF segment (most complete metadata)
                                    if exif_size > best_segment_size {
                                        best_exif_segment = Some(exif_data);
                                        best_segment_size = exif_size;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        best_exif_segment
    }
    
    /// Extract basic JPEG information with optimization
    fn extract_basic_jpeg_info_optimized(
        &mut self,
        _data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) {
        metadata.insert("FileType".to_string(), "JPEG".to_string());
        metadata.insert("FileTypeExtension".to_string(), "jpg".to_string());
        metadata.insert("MIMEType".to_string(), "image/jpeg".to_string());
    }
    
    /// Extract JFIF information with optimization
    fn extract_jfif_info_optimized(
        &mut self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) {
        // Find JFIF segment efficiently
        for i in 0..data.len().saturating_sub(8) {
            if data[i] == 0xFF && data[i + 1] == 0xE0 {
                // Found JFIF marker
                if i + 8 < data.len() {
                    let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                    if i + 8 + length <= data.len() {
                        let jfif_data = &data[i + 4..i + 8 + length];
                        
                        // Extract JFIF version
                        if jfif_data.len() >= 5 {
                            let version = format!("{}.{}", jfif_data[4], jfif_data[5]);
                            metadata.insert("JFIFVersion".to_string(), version);
                        }
                        
                        // Extract resolution
                        if jfif_data.len() >= 9 {
                            let x_res = u16::from_be_bytes([jfif_data[6], jfif_data[7]]);
                            let y_res = u16::from_be_bytes([jfif_data[8], jfif_data[9]]);
                            metadata.insert("XResolution".to_string(), x_res.to_string());
                            metadata.insert("YResolution".to_string(), y_res.to_string());
                        }
                        
                        break;
                    }
                }
            }
        }
    }
    
    /// Extract camera-specific metadata with optimization
    fn extract_camera_specific_metadata_optimized(
        &mut self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) {
        // Use efficient pattern matching for camera detection
        if let Some(make) = FormatDetector::detect_camera_make(data) {
            metadata.insert("Make".to_string(), make);
        }
    }
    
    /// Add computed fields with optimization
    fn add_computed_fields_optimized(
        &mut self,
        metadata: &mut HashMap<String, String>,
    ) {
        // Add image dimensions efficiently
        if let (Some(width), Some(height)) = (
            metadata.get("ImageWidth"),
            metadata.get("ImageHeight")
        ) {
            if let (Ok(w), Ok(h)) = (width.parse::<u32>(), height.parse::<u32>()) {
                let megapixels = (w * h) as f64 / 1_000_000.0;
                metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));
            }
        }
    }
    
    /// Post-process problematic fields with optimization
    fn post_process_problematic_fields_optimized(
        &mut self,
        metadata: &mut HashMap<String, String>,
    ) {
        // Optimize field value processing
        if let Some(flash) = metadata.get("Flash") {
            let optimized_flash = match flash.as_str() {
                "Off, Did not fire" => "16",
                "On, Fired" => "9",
                _ => flash,
            };
            metadata.insert("Flash".to_string(), optimized_flash.to_string());
        }
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> crate::memory_optimization::MemoryStats {
        let pool_stats = self.memory_pool.get_stats();
        
        crate::memory_optimization::MemoryStats {
            pool_stats,
            buffer_capacity: self.exif_buffer.capacity(),
            metadata_capacity: 0, // Not applicable for parser
        }
    }
    
    /// Clear memory caches
    pub fn clear_caches(&mut self) {
        self.exif_buffer.clear();
    }
}

impl Default for OptimizedJpegParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch JPEG processor with memory optimization
pub struct BatchJpegProcessor {
    /// Optimized JPEG parser
    parser: OptimizedJpegParser,
    /// Batch size for processing
    batch_size: usize,
    /// Pre-allocated batch buffer
    batch_buffer: Vec<HashMap<String, String>>,
}

impl BatchJpegProcessor {
    /// Create a new batch JPEG processor
    pub fn new(batch_size: usize) -> Self {
        Self {
            parser: OptimizedJpegParser::new(),
            batch_size,
            batch_buffer: Vec::with_capacity(batch_size),
        }
    }
    
    /// Process JPEG files in batches
    pub fn process_jpeg_files(
        &mut self,
        file_paths: &[String],
    ) -> Result<Vec<HashMap<String, String>>, ExifError> {
        let mut results = Vec::with_capacity(file_paths.len());
        
        for chunk in file_paths.chunks(self.batch_size) {
            self.batch_buffer.clear();
            
            for file_path in chunk {
                let file = std::fs::File::open(file_path)?;
                let mmap = unsafe { memmap2::Mmap::map(&file)? };
                
                let mut metadata = HashMap::with_capacity(200);
                self.parser.parse_jpeg_exif_optimized(&mmap, &mut metadata)?;
                
                self.batch_buffer.push(metadata);
            }
            
            results.extend(self.batch_buffer.drain(..));
        }
        
        Ok(results)
    }
    
    /// Get processor statistics
    pub fn get_stats(&self) -> crate::memory_optimization::BatchStats {
        let memory_stats = self.parser.get_memory_stats();
        
        crate::memory_optimization::BatchStats {
            pool_stats: memory_stats.pool_stats,
            batch_size: self.batch_size,
            buffer_capacity: self.batch_buffer.capacity(),
        }
    }
}

impl Default for BatchJpegProcessor {
    fn default() -> Self {
        Self::new(50) // Default batch size of 50
    }
}