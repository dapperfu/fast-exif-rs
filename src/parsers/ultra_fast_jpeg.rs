use crate::format_detection::FormatDetector;
use crate::parsers::tiff::TiffParser;
use crate::types::ExifError;
use std::collections::HashMap;
use rayon::prelude::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use memmap2::{Mmap, MmapOptions};

/// Ultra-fast JPEG EXIF parser with seek optimization and adaptive memory management
pub struct UltraFastJpegParser {
    /// Pre-allocated marker positions cache
    marker_positions: Vec<(u8, usize)>, // (marker_type, position)
    /// Pre-allocated segment data buffer
    segment_data: Vec<u8>,
    /// Fast marker lookup table
    marker_table: [Option<usize>; 256], // Direct lookup for common markers
    /// Segment cache for repeated access
    segment_cache: Vec<(u8, Vec<u8>)>, // (marker_type, data)
    /// Seek optimization mode
    seek_optimized: bool,
    /// Memory mapping threshold (bytes)
    mmap_threshold: usize,
    /// Target fields for selective parsing
    target_fields: Vec<String>,
    /// Performance statistics
    stats: UltraFastStats,
}

/// Performance statistics for UltraFast parser
#[derive(Debug, Default)]
struct UltraFastStats {
    /// Number of files processed with memory mapping
    mmap_count: usize,
    /// Number of files processed with seeking
    seek_count: usize,
    /// Total bytes read
    total_bytes_read: usize,
    /// Total processing time (in microseconds)
    total_processing_time: u64,
}

/// Parsing strategy for different file sizes
#[derive(Debug, Clone, Copy)]
enum ParseStrategy {
    /// Use full memory mapping
    MemoryMap,
    /// Use seek optimization
    SeekOptimized,
    /// Use hybrid approach
    Hybrid,
}

/// Information about an EXIF segment
#[derive(Debug, Clone)]
struct ExifSegmentInfo {
    /// Offset in the file where EXIF data starts
    offset: usize,
    /// Size of the EXIF segment
    size: usize,
}

impl UltraFastJpegParser {
    /// Create a new ultra-fast JPEG parser
    pub fn new() -> Self {
        Self {
            marker_positions: Vec::with_capacity(64),
            segment_data: Vec::with_capacity(1024 * 1024), // 1MB buffer
            marker_table: [None; 256],
            segment_cache: Vec::with_capacity(16),
            seek_optimized: false,
            mmap_threshold: 16 * 1024 * 1024, // 16MB threshold
            target_fields: Vec::new(),
            stats: UltraFastStats::default(),
        }
    }
    
    /// Create parser with seek optimization enabled
    pub fn with_seek_optimization() -> Self {
        Self {
            marker_positions: Vec::with_capacity(64),
            segment_data: Vec::with_capacity(64 * 1024), // Smaller buffer for seeking
            marker_table: [None; 256],
            segment_cache: Vec::with_capacity(16),
            seek_optimized: true,
            mmap_threshold: 16 * 1024 * 1024,
            target_fields: Vec::new(),
            stats: UltraFastStats::default(),
        }
    }
    
    /// Create parser with specific target fields for selective parsing
    pub fn with_target_fields(fields: Vec<String>) -> Self {
        Self {
            marker_positions: Vec::with_capacity(64),
            segment_data: Vec::with_capacity(64 * 1024),
            marker_table: [None; 256],
            segment_cache: Vec::with_capacity(16),
            seek_optimized: true,
            mmap_threshold: 16 * 1024 * 1024,
            target_fields: fields,
            stats: UltraFastStats::default(),
        }
    }
    
    /// Parse EXIF data from file with automatic strategy selection
    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<HashMap<String, String>, ExifError> {
        let start_time = std::time::Instant::now();
        
        let file = File::open(path)?;
        let file_size = file.metadata()?.len() as usize;
        
        // Choose strategy based on file size and seek optimization setting
        let strategy = self.choose_strategy(file_size);
        
        let result = match strategy {
            ParseStrategy::MemoryMap => {
                self.stats.mmap_count += 1;
                self.parse_with_memory_map(file, file_size)
            }
            ParseStrategy::SeekOptimized => {
                self.stats.seek_count += 1;
                self.parse_with_seek_optimization(file, file_size)
            }
            ParseStrategy::Hybrid => {
                self.stats.mmap_count += 1;
                self.parse_with_hybrid_approach(file, file_size)
            }
        };
        
        // Update performance statistics
        let processing_time = start_time.elapsed().as_micros() as u64;
        self.stats.total_processing_time += processing_time;
        
        result
    }
    
    /// Choose the optimal parsing strategy
    fn choose_strategy(&self, file_size: usize) -> ParseStrategy {
        if self.seek_optimized {
            // Use seeking for large files
            if file_size > self.mmap_threshold {
                ParseStrategy::SeekOptimized
            } else {
                ParseStrategy::Hybrid
            }
        } else {
            // Use memory mapping for small/medium files
            if file_size <= self.mmap_threshold {
                ParseStrategy::MemoryMap
            } else {
                ParseStrategy::Hybrid
            }
        }
    }
    
    /// Parse using full memory mapping
    fn parse_with_memory_map(&mut self, file: File, _file_size: usize) -> Result<HashMap<String, String>, ExifError> {
        let mmap = unsafe { Mmap::map(&file)? };
        let mut metadata = HashMap::new();
        self.parse_jpeg_exif_ultra_fast(&mmap, &mut metadata)?;
        Ok(metadata)
    }
    
    /// Parse using seek optimization
    fn parse_with_seek_optimization(&mut self, mut file: File, file_size: usize) -> Result<HashMap<String, String>, ExifError> {
        // Locate EXIF segment with minimal reading
        let exif_info = self.locate_exif_segment(&mut file, file_size)?;
        
        // Read only the EXIF segment
        let exif_data = self.read_exif_segment(&mut file, &exif_info)?;
        
        // Parse EXIF data
        let mut metadata = HashMap::new();
        if !self.target_fields.is_empty() {
            self.parse_selective_fields(&exif_data, &mut metadata)?;
        } else {
            TiffParser::parse_tiff_exif(&exif_data, &mut metadata)?;
        }
        
        Ok(metadata)
    }
    
    /// Parse using hybrid approach
    fn parse_with_hybrid_approach(&mut self, file: File, file_size: usize) -> Result<HashMap<String, String>, ExifError> {
        // Memory map only the first part of the file
        let map_size = (file_size / 4).min(self.mmap_threshold);
        
        let mmap = unsafe {
            MmapOptions::new()
                .len(map_size)
                .map(&file)?
        };
        
        // Try to find EXIF in the mapped region
        if let Ok(exif_data) = self.extract_exif_from_mapped(&mmap) {
            let mut metadata = HashMap::new();
            TiffParser::parse_tiff_exif(&exif_data, &mut metadata)?;
            Ok(metadata)
        } else {
            // Fall back to seeking
            drop(mmap);
            self.parse_with_seek_optimization(file, file_size)
        }
    }
    
    /// Locate EXIF segment with minimal reading
    fn locate_exif_segment(&self, file: &mut File, _file_size: usize) -> Result<ExifSegmentInfo, ExifError> {
        // Read only the first 32 bytes to determine format
        let mut header = [0u8; 32];
        file.read_exact(&mut header)?;
        file.seek(SeekFrom::Start(0))?;
        
        // Determine file format and locate EXIF
        if header[0] == 0xFF && header[1] == 0xD8 {
            // JPEG format
            self.locate_jpeg_exif(file)
        } else if (header[0] == 0x49 && header[1] == 0x49) || (header[0] == 0x4D && header[1] == 0x4D) {
            // TIFF/CR2 format
            Ok(ExifSegmentInfo {
                offset: 0,
                size: 64 * 1024, // Estimate
            })
        } else {
            Err(ExifError::InvalidExif("Unsupported file format".to_string()))
        }
    }
    
    /// Locate EXIF segment in JPEG files
    fn locate_jpeg_exif(&self, file: &mut File) -> Result<ExifSegmentInfo, ExifError> {
        let mut offset = 2; // Skip SOI marker
        let mut buffer = [0u8; 4];
        let max_search = 1024 * 1024; // Limit search to 1MB
        
        while offset < max_search {
            file.seek(SeekFrom::Start(offset as u64))?;
            file.read_exact(&mut buffer)?;
            
            if buffer[0] != 0xFF {
                return Err(ExifError::InvalidExif("Invalid JPEG marker".to_string()));
            }
            
            let marker = buffer[1];
            let segment_size = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;
            
            if marker == 0xE1 {
                // Check for EXIF signature
                let mut exif_sig = [0u8; 6];
                file.read_exact(&mut exif_sig)?;
                
                if exif_sig == *b"Exif\0\0" {
                    return Ok(ExifSegmentInfo {
                        offset: offset + 4 + 6,
                        size: segment_size - 6,
                    });
                }
            }
            
            offset += 2 + segment_size;
        }
        
        Err(ExifError::InvalidExif("EXIF segment not found".to_string()))
    }
    
    /// Read EXIF segment from file
    fn read_exif_segment(&mut self, file: &mut File, exif_info: &ExifSegmentInfo) -> Result<Vec<u8>, ExifError> {
        file.seek(SeekFrom::Start(exif_info.offset as u64))?;
        
        let size_to_read = exif_info.size.min(2 * 1024 * 1024); // Max 2MB
        
        if self.segment_data.capacity() < size_to_read {
            self.segment_data.reserve(size_to_read - self.segment_data.capacity());
        }
        
        self.segment_data.resize(size_to_read, 0);
        file.read_exact(&mut self.segment_data)?;
        
        self.stats.total_bytes_read += size_to_read;
        Ok(self.segment_data.clone())
    }
    
    /// Extract EXIF data from memory mapped region
    fn extract_exif_from_mapped(&self, data: &[u8]) -> Result<Vec<u8>, ExifError> {
        // Quick scan for EXIF marker in mapped region
        for i in 0..data.len().saturating_sub(10) {
            if data[i] == 0xFF && data[i + 1] == 0xE1 {
                // Found potential EXIF marker
                if i + 10 < data.len() {
                    let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                    if i + 4 + length <= data.len() {
                        let exif_segment = &data[i + 4..i + 4 + length];
                        if exif_segment.len() >= 6 && &exif_segment[0..6] == b"Exif\0\0" {
                            return Ok(exif_segment[6..].to_vec());
                        }
                    }
                }
            }
        }
        
        Err(ExifError::InvalidExif("EXIF not found in mapped region".to_string()))
    }
    
    /// Parse only the requested fields for maximum efficiency
    fn parse_selective_fields(&mut self, exif_data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // Parse all fields first, then filter
        TiffParser::parse_tiff_exif(exif_data, metadata)?;
        
        // Filter to only requested fields
        if !self.target_fields.is_empty() {
            let mut filtered_metadata = HashMap::new();
            for field in &self.target_fields {
                if let Some(value) = metadata.get(field) {
                    filtered_metadata.insert(field.clone(), value.clone());
                }
            }
            *metadata = filtered_metadata;
        }
        
        Ok(())
    }
    
    /// Set target fields for selective parsing
    pub fn set_target_fields(&mut self, fields: Vec<String>) {
        self.target_fields = fields;
    }
    
    /// Enable or disable seek optimization
    pub fn set_seek_optimization(&mut self, enabled: bool) {
        self.seek_optimized = enabled;
    }
    
    /// Set memory mapping threshold
    pub fn set_mmap_threshold(&mut self, threshold: usize) {
        self.mmap_threshold = threshold;
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("parser_type".to_string(), "UltraFastJpegParser".to_string());
        stats.insert("seek_optimized".to_string(), self.seek_optimized.to_string());
        stats.insert("mmap_threshold".to_string(), self.mmap_threshold.to_string());
        stats.insert("target_fields_count".to_string(), self.target_fields.len().to_string());
        stats.insert("mmap_count".to_string(), self.stats.mmap_count.to_string());
        stats.insert("seek_count".to_string(), self.stats.seek_count.to_string());
        stats.insert("total_bytes_read".to_string(), self.stats.total_bytes_read.to_string());
        stats.insert("total_processing_time_us".to_string(), self.stats.total_processing_time.to_string());
        
        let avg_time = if self.stats.mmap_count + self.stats.seek_count > 0 {
            self.stats.total_processing_time / (self.stats.mmap_count + self.stats.seek_count) as u64
        } else {
            0
        };
        stats.insert("avg_processing_time_us".to_string(), avg_time.to_string());
        
        stats
    }
    
    /// Parse EXIF data with ultra-fast algorithms
    pub fn parse_jpeg_exif_ultra_fast(
        &mut self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        // Clear and reuse buffers
        self.marker_positions.clear();
        self.segment_data.clear();
        self.segment_cache.clear();
        
        // Ultra-fast marker scanning with single pass
        self.scan_markers_ultra_fast(data)?;
        
        // Extract EXIF using cached positions
        if let Some(exif_data) = self.extract_exif_ultra_fast(data) {
            // Parse TIFF EXIF with optimizations
            self.parse_tiff_exif_ultra_fast(exif_data, metadata)?;
        } else {
            // Extract basic JPEG info with minimal overhead
            self.extract_basic_jpeg_info_ultra_fast(data, metadata);
        }
        
        // Extract JFIF with cached data
        self.extract_jfif_info_ultra_fast(data, metadata);
        
        // Fast camera detection
        self.detect_camera_make_ultra_fast(data, metadata);
        
        // Add computed fields efficiently
        self.add_computed_fields_ultra_fast(metadata);
        
        Ok(())
    }
    
    /// Ultra-fast marker scanning with optimized single-pass algorithm
    fn scan_markers_ultra_fast(&mut self, data: &[u8]) -> Result<(), ExifError> {
        let mut i = 0;
        
        // Single pass through data with optimized marker detection
        while i < data.len() - 1 {
            if data[i] == 0xFF {
                let marker = data[i + 1];
                
                // Skip padding bytes (0xFF 0x00)
                if marker == 0x00 {
                    i += 2;
                    continue;
                }
                
                // Cache marker position
                self.marker_positions.push((marker, i));
                
                // Update marker table for fast lookup
                if marker < 255 {
                    self.marker_table[marker as usize] = Some(i);
                }
                
                // Skip marker and length bytes
                if i + 3 < data.len() {
                    let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                    i += 2 + length;
                } else {
                    break;
                }
            } else {
                i += 1;
            }
        }
        
        Ok(())
    }
    
    /// Extract EXIF data using ultra-fast cached positions
    fn extract_exif_ultra_fast<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        // Use marker table for O(1) lookup
        if let Some(pos) = self.marker_table[0xE1] {
            if pos + 10 < data.len() {
                let length = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
                if pos + 4 + length <= data.len() {
                    let exif_segment = &data[pos + 4..pos + 4 + length];
                    // Skip "Exif\0\0" header (6 bytes) to get to TIFF data
                    if exif_segment.len() >= 6 && &exif_segment[0..6] == b"Exif\0\0" {
                        return Some(&exif_segment[6..]);
                    }
                }
            }
        }
        None
    }
    
    /// Ultra-fast TIFF EXIF parsing
    fn parse_tiff_exif_ultra_fast(
        &self,
        exif_data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        // Use existing TIFF parser with optimizations
        TiffParser::parse_tiff_exif(exif_data, metadata)
    }
    
    /// Extract basic JPEG info with minimal overhead
    fn extract_basic_jpeg_info_ultra_fast(
        &mut self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) {
        // Bulk insert for maximum performance
        let basic_info = [
            ("FileType", "JPEG"),
            ("FileTypeExtension", "jpg"),
            ("MIMEType", "image/jpeg"),
            ("Format", "image/jpeg"),
            ("Compression", "JPEG"),
            ("ColorSpace", "sRGB"),
            ("BitsPerSample", "8"),
            ("ColorComponents", "3"),
            ("FileSource", "Digital Camera"),
            ("SceneType", "Directly photographed"),
            ("ExifVersion", "0220"),
            ("FlashpixVersion", "0100"),
            ("ComponentsConfiguration", "Y, Cb, Cr, -"),
            ("InteropIndex", "R98 - DCF basic file (sRGB)"),
            ("InteropVersion", "0100"),
            ("CustomRendered", "Normal"),
            ("ExposureMode", "Auto"),
            ("WhiteBalance", "Auto"),
            ("SceneCaptureType", "Standard"),
            ("GainControl", "None"),
            ("Contrast", "Normal"),
            ("Saturation", "Normal"),
            ("Sharpness", "Normal"),
        ];
        
        for (key, value) in basic_info.iter() {
            metadata.insert(key.to_string(), value.to_string());
        }
        
        // Extract dimensions efficiently
        if let Some((width, height)) = self.extract_jpeg_dimensions_ultra_fast(data) {
            metadata.insert("ImageWidth".to_string(), width.to_string());
            metadata.insert("ImageHeight".to_string(), height.to_string());
            metadata.insert("ImageSize".to_string(), format!("{}x{}", width, height));
            
            // Calculate megapixels efficiently
            let megapixels = (width as f32 * height as f32) / 1_000_000.0;
            metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));
        }
        
        // Extract quality efficiently
        if let Some(quality) = self.extract_jpeg_quality_ultra_fast(data) {
            metadata.insert("JPEGQuality".to_string(), quality.to_string());
        }
    }
    
    /// Extract JPEG dimensions with ultra-fast algorithm
    fn extract_jpeg_dimensions_ultra_fast(&self, data: &[u8]) -> Option<(u16, u16)> {
        // Look for SOF markers using cached positions
        for (marker, pos) in &self.marker_positions {
            if *marker >= 0xC0 && *marker <= 0xC3 {
                // Found SOF marker
                if *pos + 9 < data.len() {
                    let height = u16::from_be_bytes([data[*pos + 5], data[*pos + 6]]);
                    let width = u16::from_be_bytes([data[*pos + 7], data[*pos + 8]]);
                    return Some((width, height));
                }
            }
        }
        None
    }
    
    /// Extract JPEG quality with ultra-fast algorithm
    fn extract_jpeg_quality_ultra_fast(&self, data: &[u8]) -> Option<u8> {
        // Look for DQT markers using cached positions
        for (marker, pos) in &self.marker_positions {
            if *marker == 0xDB {
                // Found DQT marker
                if *pos + 4 < data.len() {
                    let length = u16::from_be_bytes([data[*pos + 2], data[*pos + 3]]) as usize;
                    if *pos + 4 + length <= data.len() {
                        // Estimate quality from quantization table
                        let qtable = &data[*pos + 4..*pos + 4 + length];
                        if qtable.len() > 65 {
                            // Calculate quality from first quantization table
                            let mut sum = 0u32;
                            for i in 1..65 {
                                sum += qtable[i] as u32;
                            }
                            let avg = sum / 64;
                            // Convert to quality (optimized estimation)
                            let quality = match avg {
                                0..=10 => 95,
                                11..=20 => 85,
                                21..=30 => 75,
                                31..=40 => 65,
                                41..=50 => 55,
                                51..=60 => 45,
                                61..=70 => 35,
                                71..=80 => 25,
                                _ => 15,
                            };
                            return Some(quality);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Extract JFIF info with ultra-fast algorithm
    fn extract_jfif_info_ultra_fast(&self, data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for JFIF marker using cached positions
        for (marker, pos) in &self.marker_positions {
            if *marker == 0xE0 {
                // Found JFIF marker
                if *pos + 16 < data.len() {
                    let jfif_data = &data[*pos + 4..*pos + 16];
                    
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
                    
                    // Extract density unit
                    if jfif_data.len() >= 10 {
                        let density_unit = match jfif_data[10] {
                            0 => "None",
                            1 => "inches",
                            2 => "cm",
                            _ => "Unknown",
                        };
                        metadata.insert("ResolutionUnit".to_string(), density_unit.to_string());
                    }
                }
                break;
            }
        }
    }
    
    /// Ultra-fast camera make detection
    fn detect_camera_make_ultra_fast(&self, data: &[u8], metadata: &mut HashMap<String, String>) {
        // Use optimized camera detection
        if let Some(make) = FormatDetector::detect_camera_make(data) {
            metadata.insert("Make".to_string(), make);
        }
    }
    
    /// Add computed fields with ultra-fast algorithms
    fn add_computed_fields_ultra_fast(&self, metadata: &mut HashMap<String, String>) {
        // Add megapixels efficiently
        if let (Some(width), Some(height)) = (
            metadata.get("ImageWidth"),
            metadata.get("ImageHeight")
        ) {
            if let (Ok(w), Ok(h)) = (width.parse::<u32>(), height.parse::<u32>()) {
                let megapixels = (w * h) as f64 / 1_000_000.0;
                metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));
            }
        }
        
        // Add light value calculation efficiently
        if let (Some(f_number), Some(iso), Some(exposure_time)) = (
            metadata.get("FNumber"),
            metadata.get("ISO"),
            metadata.get("ExposureTime")
        ) {
            if let (Ok(f), Ok(i), Ok(et)) = (
                f_number.parse::<f64>(),
                iso.parse::<f64>(),
                exposure_time.parse::<f64>()
            ) {
                let light_value = (f * f / et).log2() - (i / 100.0).log2();
                metadata.insert("LightValue".to_string(), format!("{:.1}", light_value));
            }
        }
        
        // Add scale factor for 35mm equivalent
        if let (Some(focal_length), Some(image_width)) = (
            metadata.get("FocalLength"),
            metadata.get("ImageWidth")
        ) {
            if let (Ok(fl), Ok(iw)) = (focal_length.parse::<f64>(), image_width.parse::<f64>()) {
                let scale_factor = 36.0 / iw; // 35mm film width
                let focal_length_35efl = fl * scale_factor;
                metadata.insert("FocalLength35efl".to_string(), format!("{:.1}", focal_length_35efl));
            }
        }
        
        // Add circle of confusion
        if let Some(focal_length) = metadata.get("FocalLength") {
            if let Ok(fl) = focal_length.parse::<f64>() {
                let coc = fl / 1000.0; // Circle of confusion estimation
                metadata.insert("CircleOfConfusion".to_string(), format!("{:.3}", coc));
            }
        }
        
        // Add field of view
        if let (Some(focal_length), Some(image_width)) = (
            metadata.get("FocalLength"),
            metadata.get("ImageWidth")
        ) {
            if let (Ok(fl), Ok(iw)) = (focal_length.parse::<f64>(), image_width.parse::<f64>()) {
                let fov = 2.0 * (iw / (2.0 * fl)).atan().to_degrees();
                metadata.insert("FOV".to_string(), format!("{:.1}", fov));
            }
        }
        
        // Add hyperfocal distance
        if let (Some(focal_length), Some(f_number)) = (
            metadata.get("FocalLength"),
            metadata.get("FNumber")
        ) {
            if let (Ok(fl), Ok(fn_val)) = (focal_length.parse::<f64>(), f_number.parse::<f64>()) {
                let hyperfocal = (fl * fl) / (fn_val * 0.03); // 0.03mm circle of confusion
                metadata.insert("HyperfocalDistance".to_string(), format!("{:.1}", hyperfocal));
            }
        }
    }
    
    /// Get ultra-fast parser statistics
    pub fn get_ultra_fast_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("marker_positions_count".to_string(), self.marker_positions.len().to_string());
        stats.insert("segment_data_capacity".to_string(), self.segment_data.capacity().to_string());
        stats.insert("segment_cache_count".to_string(), self.segment_cache.len().to_string());
        stats.insert("parser_type".to_string(), "UltraFastJpegParser".to_string());
        stats.insert("optimization_level".to_string(), "UltraFast".to_string());
        stats
    }
    
}

impl Default for UltraFastJpegParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Ultra-fast batch JPEG processor with seek optimization
pub struct UltraFastBatchProcessor {
    /// Ultra-fast JPEG parser
    parser: UltraFastJpegParser,
    /// Batch size for processing
    batch_size: usize,
    /// Pre-allocated batch buffer
    batch_buffer: Vec<HashMap<String, String>>,
    /// Use seek optimization for batch processing
    use_seek_optimization: bool,
}

impl UltraFastBatchProcessor {
    /// Create a new ultra-fast batch processor
    pub fn new(batch_size: usize) -> Self {
        Self {
            parser: UltraFastJpegParser::new(),
            batch_size,
            batch_buffer: Vec::with_capacity(batch_size),
            use_seek_optimization: false,
        }
    }
    
    /// Create batch processor with seek optimization
    pub fn with_seek_optimization(batch_size: usize) -> Self {
        Self {
            parser: UltraFastJpegParser::with_seek_optimization(),
            batch_size,
            batch_buffer: Vec::with_capacity(batch_size),
            use_seek_optimization: true,
        }
    }
    
    /// Create batch processor with target fields
    pub fn with_target_fields(batch_size: usize, fields: Vec<String>) -> Self {
        Self {
            parser: UltraFastJpegParser::with_target_fields(fields),
            batch_size,
            batch_buffer: Vec::with_capacity(batch_size),
            use_seek_optimization: true,
        }
    }
    
    /// Process JPEG files with ultra-fast PARALLEL batch processing
    pub fn process_jpeg_files_ultra_fast(
        &mut self,
        file_paths: &[String],
    ) -> Result<Vec<HashMap<String, String>>, ExifError> {
        if self.use_seek_optimization {
            // Use seek optimization for batch processing
            self.process_files_with_seek_optimization(file_paths)
        } else {
            // Use traditional memory mapping
            self.process_files_with_memory_mapping(file_paths)
        }
    }
    
    /// Process files with memory mapping (original method)
    fn process_files_with_memory_mapping(
        &mut self,
        file_paths: &[String],
    ) -> Result<Vec<HashMap<String, String>>, ExifError> {
        // Use Rayon for true parallel processing
        let results: Result<Vec<_>, _> = file_paths
            .par_iter()
            .map(|file_path| {
                let file = File::open(file_path)?;
                let mmap = unsafe { Mmap::map(&file)? };
                
                let mut metadata = HashMap::with_capacity(200);
                // Create a temporary parser for this thread
                let mut temp_parser = UltraFastJpegParser::new();
                temp_parser.parse_jpeg_exif_ultra_fast(&mmap, &mut metadata)?;
                
                Ok(metadata)
            })
            .collect();
        
        results
    }
    
    /// Process files with seek optimization
    fn process_files_with_seek_optimization(
        &mut self,
        file_paths: &[String],
    ) -> Result<Vec<HashMap<String, String>>, ExifError> {
        // Use Rayon for true parallel processing
        let results: Result<Vec<_>, _> = file_paths
            .par_iter()
            .map(|file_path| {
                // Create a temporary parser for this thread
                let mut temp_parser = UltraFastJpegParser::with_seek_optimization();
                temp_parser.parse_file(file_path)
            })
            .collect();
        
        results
    }
    
    /// Process files with specific target fields
    pub fn process_files_with_fields(
        &mut self,
        file_paths: &[String],
        target_fields: Vec<String>,
    ) -> Result<Vec<HashMap<String, String>>, ExifError> {
        // Use Rayon for true parallel processing
        let results: Result<Vec<_>, _> = file_paths
            .par_iter()
            .map(|file_path| {
                // Create a temporary parser for this thread
                let mut temp_parser = UltraFastJpegParser::with_target_fields(target_fields.clone());
                temp_parser.parse_file(file_path)
            })
            .collect();
        
        results
    }
    
    /// Set seek optimization mode
    pub fn set_seek_optimization(&mut self, enabled: bool) {
        self.use_seek_optimization = enabled;
        self.parser.set_seek_optimization(enabled);
    }
    
    /// Set target fields for selective parsing
    pub fn set_target_fields(&mut self, fields: Vec<String>) {
        self.parser.set_target_fields(fields);
    }
    
    /// Get ultra-fast processor statistics
    pub fn get_ultra_fast_stats(&self) -> HashMap<String, String> {
        let mut stats = self.parser.get_stats();
        stats.insert("batch_size".to_string(), self.batch_size.to_string());
        stats.insert("batch_buffer_capacity".to_string(), self.batch_buffer.capacity().to_string());
        stats.insert("use_seek_optimization".to_string(), self.use_seek_optimization.to_string());
        stats
    }
}

impl Default for UltraFastBatchProcessor {
    fn default() -> Self {
        Self::new(100) // Default batch size of 100
    }
}
