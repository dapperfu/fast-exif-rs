use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use memmap2::{Mmap, MmapOptions};
use crate::types::ExifError;

/// Adaptive memory mapping strategy for optimal performance
/// 
/// This parser automatically chooses between memory mapping and seeking
/// based on file size and system capabilities for maximum efficiency.
pub struct AdaptiveMemoryParser {
    /// Threshold for switching between strategies (in bytes)
    memory_map_threshold: usize,
    /// Maximum memory map size (in bytes)
    max_mmap_size: usize,
    /// Buffer for small file operations
    read_buffer: Vec<u8>,
    /// Cache for parsed metadata
    metadata_cache: HashMap<String, String>,
    /// Performance statistics
    stats: PerformanceStats,
}

/// Performance statistics for the adaptive parser
#[derive(Debug, Default)]
struct PerformanceStats {
    /// Number of files processed with memory mapping
    mmap_count: usize,
    /// Number of files processed with seeking
    seek_count: usize,
    /// Total bytes read
    total_bytes_read: usize,
    /// Total processing time (in microseconds)
    total_processing_time: u64,
}

impl AdaptiveMemoryParser {
    /// Create a new adaptive memory parser
    pub fn new() -> Self {
        Self {
            memory_map_threshold: 16 * 1024 * 1024, // 16MB threshold
            max_mmap_size: 256 * 1024 * 1024, // 256MB max mmap
            read_buffer: Vec::with_capacity(64 * 1024), // 64KB buffer
            metadata_cache: HashMap::with_capacity(200),
            stats: PerformanceStats::default(),
        }
    }
    
    /// Create parser with custom thresholds
    pub fn with_thresholds(mmap_threshold: usize, max_mmap_size: usize) -> Self {
        Self {
            memory_map_threshold: mmap_threshold,
            max_mmap_size,
            read_buffer: Vec::with_capacity(64 * 1024),
            metadata_cache: HashMap::with_capacity(200),
            stats: PerformanceStats::default(),
        }
    }
    
    /// Parse EXIF data with adaptive memory strategy
    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<HashMap<String, String>, ExifError> {
        let start_time = std::time::Instant::now();
        
        // Clear cache for new file
        self.metadata_cache.clear();
        
        let file = File::open(path)?;
        let file_size = file.metadata()?.len() as usize;
        
        // Choose strategy based on file size and system capabilities
        let strategy = self.choose_strategy(file_size);
        
        let result = match strategy {
            MemoryStrategy::MemoryMap => {
                self.stats.mmap_count += 1;
                self.parse_with_memory_map(file, file_size)
            }
            MemoryStrategy::SelectiveSeek => {
                self.stats.seek_count += 1;
                self.parse_with_selective_seek(file, file_size)
            }
            MemoryStrategy::Hybrid => {
                self.stats.mmap_count += 1;
                self.parse_with_hybrid_approach(file, file_size)
            }
        };
        
        // Update performance statistics
        let processing_time = start_time.elapsed().as_micros() as u64;
        self.stats.total_processing_time += processing_time;
        
        result
    }
    
    /// Choose the optimal memory strategy based on file size and system
    fn choose_strategy(&self, file_size: usize) -> MemoryStrategy {
        // For very small files, use seeking
        if file_size < 1024 * 1024 { // < 1MB
            return MemoryStrategy::SelectiveSeek;
        }
        
        // For medium files, use memory mapping if available
        if file_size <= self.memory_map_threshold {
            return MemoryStrategy::MemoryMap;
        }
        
        // For large files, use hybrid approach
        if file_size <= self.max_mmap_size {
            return MemoryStrategy::Hybrid;
        }
        
        // For very large files, use selective seeking
        MemoryStrategy::SelectiveSeek
    }
    
    /// Parse using full memory mapping
    fn parse_with_memory_map(&mut self, file: File, _file_size: usize) -> Result<HashMap<String, String>, ExifError> {
        // Create memory map
        let mmap = unsafe { Mmap::map(&file)? };
        
        // Use existing ultra-fast parser logic
        self.parse_memory_mapped_data(&mmap)?;
        
        Ok(self.metadata_cache.clone())
    }
    
    /// Parse using selective seeking
    fn parse_with_selective_seek(&mut self, mut file: File, file_size: usize) -> Result<HashMap<String, String>, ExifError> {
        // Locate EXIF segment with minimal reading
        let exif_info = self.locate_exif_segment(&mut file, file_size)?;
        
        // Read only the EXIF segment
        let exif_data = self.read_exif_segment(&mut file, &exif_info)?;
        
        // Parse EXIF data
        self.parse_exif_data(&exif_data)?;
        
        Ok(self.metadata_cache.clone())
    }
    
    /// Parse using hybrid approach (memory map + selective reading)
    fn parse_with_hybrid_approach(&mut self, file: File, file_size: usize) -> Result<HashMap<String, String>, ExifError> {
        // Memory map only the first part of the file (where EXIF is likely to be)
        let map_size = (file_size / 4).min(self.memory_map_threshold); // Map first quarter or threshold
        
        let mmap = unsafe {
            MmapOptions::new()
                .len(map_size)
                .map(&file)?
        };
        
        // Try to find EXIF in the mapped region
        if let Ok(exif_data) = self.extract_exif_from_mapped(&mmap) {
            self.parse_exif_data(&exif_data)?;
        } else {
            // EXIF not in mapped region, fall back to seeking
            drop(mmap); // Release the memory map
            return self.parse_with_selective_seek(file, file_size);
        }
        
        Ok(self.metadata_cache.clone())
    }
    
    /// Parse memory mapped data
    fn parse_memory_mapped_data(&mut self, data: &[u8]) -> Result<(), ExifError> {
        // Use existing ultra-fast parser logic
        use crate::parsers::ultra_fast_jpeg::UltraFastJpegParser;
        
        let mut parser = UltraFastJpegParser::new();
        parser.parse_jpeg_exif_ultra_fast(data, &mut self.metadata_cache)?;
        
        Ok(())
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
                format: FileFormat::Tiff,
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
                        format: FileFormat::Jpeg,
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
        
        if self.read_buffer.capacity() < size_to_read {
            self.read_buffer.reserve(size_to_read - self.read_buffer.capacity());
        }
        
        self.read_buffer.resize(size_to_read, 0);
        file.read_exact(&mut self.read_buffer)?;
        
        Ok(self.read_buffer.clone())
    }
    
    /// Parse EXIF data
    fn parse_exif_data(&mut self, exif_data: &[u8]) -> Result<(), ExifError> {
        use crate::parsers::tiff::TiffParser;
        TiffParser::parse_tiff_exif(exif_data, &mut self.metadata_cache)?;
        Ok(())
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("parser_type".to_string(), "AdaptiveMemory".to_string());
        stats.insert("mmap_threshold".to_string(), self.memory_map_threshold.to_string());
        stats.insert("max_mmap_size".to_string(), self.max_mmap_size.to_string());
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
    
    /// Set memory mapping threshold
    pub fn set_mmap_threshold(&mut self, threshold: usize) {
        self.memory_map_threshold = threshold;
    }
    
    /// Set maximum memory map size
    pub fn set_max_mmap_size(&mut self, max_size: usize) {
        self.max_mmap_size = max_size;
    }
}

impl Default for AdaptiveMemoryParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory strategy for parsing
#[derive(Debug, Clone, Copy)]
enum MemoryStrategy {
    /// Use full memory mapping
    MemoryMap,
    /// Use selective seeking
    SelectiveSeek,
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
    /// File format
    format: FileFormat,
}

/// Supported file formats
#[derive(Debug, Clone)]
enum FileFormat {
    Jpeg,
    Tiff,
    Heic,
    Mov,
}

/// Adaptive memory batch processor
pub struct AdaptiveMemoryBatchProcessor {
    /// Parser instance
    parser: AdaptiveMemoryParser,
    /// Batch size for processing
    batch_size: usize,
}

impl AdaptiveMemoryBatchProcessor {
    /// Create a new adaptive memory batch processor
    pub fn new(batch_size: usize) -> Self {
        Self {
            parser: AdaptiveMemoryParser::new(),
            batch_size,
        }
    }
    
    /// Process files with adaptive memory strategy
    pub fn process_files(&mut self, file_paths: &[String]) -> Result<Vec<HashMap<String, String>>, ExifError> {
        let mut results = Vec::with_capacity(file_paths.len());
        
        for file_path in file_paths {
            match self.parser.parse_file(file_path) {
                Ok(metadata) => results.push(metadata),
                Err(e) => {
                    eprintln!("Error processing {}: {}", file_path, e);
                    results.push(HashMap::new());
                }
            }
        }
        
        Ok(results)
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> HashMap<String, String> {
        self.parser.get_stats()
    }
}

impl Default for AdaptiveMemoryBatchProcessor {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_creation() {
        let parser = AdaptiveMemoryParser::new();
        assert_eq!(parser.memory_map_threshold, 16 * 1024 * 1024);
        assert_eq!(parser.max_mmap_size, 256 * 1024 * 1024);
    }
    
    #[test]
    fn test_custom_thresholds() {
        let parser = AdaptiveMemoryParser::with_thresholds(8 * 1024 * 1024, 128 * 1024 * 1024);
        assert_eq!(parser.memory_map_threshold, 8 * 1024 * 1024);
        assert_eq!(parser.max_mmap_size, 128 * 1024 * 1024);
    }
    
    #[test]
    fn test_strategy_selection() {
        let parser = AdaptiveMemoryParser::new();
        
        // Small files should use seeking
        assert!(matches!(parser.choose_strategy(512 * 1024), MemoryStrategy::SelectiveSeek));
        
        // Medium files should use memory mapping
        assert!(matches!(parser.choose_strategy(8 * 1024 * 1024), MemoryStrategy::MemoryMap));
        
        // Large files should use hybrid
        assert!(matches!(parser.choose_strategy(32 * 1024 * 1024), MemoryStrategy::Hybrid));
        
        // Very large files should use seeking
        assert!(matches!(parser.choose_strategy(512 * 1024 * 1024), MemoryStrategy::SelectiveSeek));
    }
}
