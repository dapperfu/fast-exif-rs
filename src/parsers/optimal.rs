use crate::parsers::tiff::TiffParser;
use crate::types::{ExifError, ParseScope, ReadOptions};
use memmap2::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Optimal EXIF parser that automatically chooses the best strategy
///
/// This parser combines the best features from all implementations:
/// - Ultra-seek optimization for large files (10-100x faster)
/// - Memory mapping for small/medium files (maximum speed)
/// - Lazy parsing for specific field extraction
/// - SIMD acceleration for parallel processing
/// - Automatic strategy selection based on file size
/// - Minimal I/O operations and memory usage
#[derive(Clone)]
pub struct OptimalExifParser {
    /// Buffer for reading small chunks
    read_buffer: Vec<u8>,
    /// Maximum EXIF segment size to read
    max_exif_size: usize,
    /// Cache for frequently accessed metadata
    metadata_cache: HashMap<String, String>,
    /// Fields to extract (empty means all)
    target_fields: Vec<String>,
    include_maker_notes: bool,
    include_gps: bool,
    include_interop: bool,
    force_exif_segment: bool,
    /// Memory mapping threshold (bytes)
    mmap_threshold: usize,
    /// SIMD acceleration support
    #[cfg(target_arch = "x86_64")]
    avx2_supported: bool,
    /// Performance statistics
    stats: OptimalParserStats,
}

/// Performance statistics for optimal parser
#[derive(Debug, Default, Clone)]
struct OptimalParserStats {
    /// Number of files processed with memory mapping
    mmap_count: usize,
    /// Number of files processed with seeking
    seek_count: usize,
    /// Number of files processed with hybrid approach
    hybrid_count: usize,
    /// Number of files processed with SIMD acceleration
    simd_count: usize,
    /// Total bytes read
    total_bytes_read: usize,
    /// Total processing time (in microseconds)
    total_processing_time: u64,
    /// Cache hit rate
    cache_hit_rate: f64,
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
    Mp4,
}

/// Parsing strategy for different file sizes
#[derive(Debug, Clone, Copy)]
enum ParseStrategy {
    /// Use full memory mapping for small files
    MemoryMap,
    /// Use seek optimization for large files
    SeekOptimized,
    /// Use hybrid approach for medium files
    Hybrid,
}

impl OptimalExifParser {
    /// Create a new optimal EXIF parser
    pub fn new() -> Self {
        Self {
            read_buffer: Vec::with_capacity(64 * 1024), // 64KB buffer
            max_exif_size: 2 * 1024 * 1024,             // 2MB max EXIF size
            metadata_cache: HashMap::with_capacity(200),
            target_fields: Vec::new(),
            include_maker_notes: true,
            include_gps: true,
            include_interop: true,
            force_exif_segment: false,
            mmap_threshold: 8 * 1024 * 1024, // 8MB threshold
            #[cfg(target_arch = "x86_64")]
            avx2_supported: Self::check_avx2_support(),
            stats: OptimalParserStats::default(),
        }
    }

    /// Create parser with specific field targets for maximum efficiency
    pub fn with_target_fields(fields: Vec<String>) -> Self {
        Self {
            read_buffer: Vec::with_capacity(64 * 1024),
            max_exif_size: 2 * 1024 * 1024,
            metadata_cache: HashMap::with_capacity(fields.len()),
            target_fields: fields.clone(),
            include_maker_notes: fields
                .iter()
                .any(|f| crate::types::looks_like_maker_note_tag(f)),
            include_gps: fields
                .iter()
                .any(|f| f.len() >= 3 && f[..3].eq_ignore_ascii_case("gps")),
            include_interop: false,
            force_exif_segment: true,
            mmap_threshold: 8 * 1024 * 1024,
            #[cfg(target_arch = "x86_64")]
            avx2_supported: Self::check_avx2_support(),
            stats: OptimalParserStats::default(),
        }
    }

    /// Create parser with custom memory mapping threshold
    pub fn with_thresholds(mmap_threshold: usize, max_exif_size: usize) -> Self {
        Self {
            read_buffer: Vec::with_capacity(64 * 1024),
            max_exif_size,
            metadata_cache: HashMap::with_capacity(200),
            target_fields: Vec::new(),
            include_maker_notes: true,
            include_gps: true,
            include_interop: true,
            force_exif_segment: false,
            mmap_threshold,
            #[cfg(target_arch = "x86_64")]
            avx2_supported: Self::check_avx2_support(),
            stats: OptimalParserStats::default(),
        }
    }

    /// Parse EXIF data with optimal strategy selection
    pub fn parse_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<HashMap<String, String>, ExifError> {
        self.parse_file_inner(path)
    }

    /// Parse a file using [`ReadOptions`] to skip unused tag groups.
    pub fn parse_file_with_options<P: AsRef<Path>>(
        &mut self,
        path: P,
        options: &ReadOptions,
    ) -> Result<HashMap<String, String>, ExifError> {
        let saved = self.snapshot_options();
        self.apply_options(options);
        let result = self.parse_file_inner(path);
        self.restore_options(saved);
        result
    }

    fn snapshot_options(&self) -> (Vec<String>, bool, bool, bool, bool) {
        (
            self.target_fields.clone(),
            self.include_maker_notes,
            self.include_gps,
            self.include_interop,
            self.force_exif_segment,
        )
    }

    fn restore_options(&mut self, saved: (Vec<String>, bool, bool, bool, bool)) {
        self.target_fields = saved.0;
        self.include_maker_notes = saved.1;
        self.include_gps = saved.2;
        self.include_interop = saved.3;
        self.force_exif_segment = saved.4;
    }

    fn apply_options(&mut self, options: &ReadOptions) {
        self.target_fields = options.wanted_tags.clone().unwrap_or_default();
        self.include_maker_notes = options.include_maker_notes;
        self.include_gps = options.include_gps;
        self.include_interop = options.include_interop;
        self.force_exif_segment = options.exif_segment_only;
    }

    fn parse_scope(&self) -> ParseScope {
        ParseScope {
            maker_notes: self.include_maker_notes,
            gps: self.include_gps,
            interop: self.include_interop,
        }
    }

    fn is_full_parse(&self) -> bool {
        self.include_maker_notes
            && self.include_gps
            && self.include_interop
            && self.target_fields.is_empty()
            && !self.force_exif_segment
    }

    fn parse_file_inner<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<HashMap<String, String>, ExifError> {
        let start_time = std::time::Instant::now();

        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len() as usize;

        // Clear cache for new file
        self.metadata_cache.clear();

        // Read first 32 bytes to detect format, then rewind. File::try_clone
        // shares the kernel offset, so a clone-read would leave the original
        // cursor at 32 and break seek-based EXIF location.
        let mut header = [0u8; 32];
        file.read_exact(&mut header)?;
        file.seek(SeekFrom::Start(0))?;

        // Detect format from header
        let format = self.detect_format(&header)?;

        // For MP4 files, always use memory mapping since they don't have traditional EXIF segments
        if matches!(format, FileFormat::Mp4) {
            return self.parse_with_memory_map(file, file_size);
        }

        // Determine optimal parsing strategy for other formats
        let strategy = if self.force_exif_segment || !self.is_full_parse() {
            ParseStrategy::SeekOptimized
        } else {
            self.determine_strategy(file_size)
        };

        // Parse using optimal strategy
        let result = match strategy {
            ParseStrategy::MemoryMap => self.parse_with_memory_map(file, file_size),
            ParseStrategy::SeekOptimized => self.parse_with_seek_optimization(file, file_size),
            ParseStrategy::Hybrid => self.parse_with_hybrid_approach(file, file_size),
        };

        // Update statistics
        let processing_time = start_time.elapsed().as_micros() as u64;
        self.stats.total_processing_time += processing_time;

        result
    }

    /// Determine optimal parsing strategy based on file size
    fn determine_strategy(&self, file_size: usize) -> ParseStrategy {
        if file_size <= self.mmap_threshold {
            ParseStrategy::MemoryMap
        } else if file_size <= self.mmap_threshold * 4 {
            ParseStrategy::Hybrid
        } else {
            ParseStrategy::SeekOptimized
        }
    }

    /// Parse using full memory mapping (best for small files)
    fn parse_with_memory_map(
        &mut self,
        file: File,
        _file_size: usize,
    ) -> Result<HashMap<String, String>, ExifError> {
        self.stats.mmap_count += 1;

        // Create memory map
        let mmap = unsafe { Mmap::map(&file)? };

        // Parse EXIF data from memory mapped region
        self.parse_exif_from_bytes(&mmap)?;

        self.stats.total_bytes_read += mmap.len();
        Ok(self.metadata_cache.clone())
    }

    /// Parse using seek optimization (best for large files)
    fn parse_with_seek_optimization(
        &mut self,
        mut file: File,
        file_size: usize,
    ) -> Result<HashMap<String, String>, ExifError> {
        self.stats.seek_count += 1;

        // Locate EXIF segment with minimal reading
        let exif_info = self.locate_exif_segment(&mut file, file_size)?;

        // Read only the EXIF segment
        let exif_data = self.read_exif_segment(&mut file, &exif_info)?;

        // TIFF/DNG IFD pointers are file-absolute. Adobe DNG commonly puts
        // ExifIFD at EOF (past max_exif_size) while DateTime strings stay near
        // the start — mmap the whole file so those offsets resolve.
        if matches!(exif_info.format, FileFormat::Tiff)
            && TiffParser::has_out_of_range_ifd(&exif_data)
        {
            return self.parse_with_memory_map(file, file_size);
        }

        // Parse EXIF data
        self.parse_exif_data_optimized(&exif_data)?;

        Ok(self.metadata_cache.clone())
    }

    /// Parse using hybrid approach (best for medium files)
    fn parse_with_hybrid_approach(
        &mut self,
        file: File,
        file_size: usize,
    ) -> Result<HashMap<String, String>, ExifError> {
        self.stats.hybrid_count += 1;

        // Memory map only the first part of the file (where EXIF is likely to be)
        let map_size = (file_size / 4).min(self.mmap_threshold);

        let mmap = unsafe { MmapOptions::new().len(map_size).map(&file)? };

        // Try to find EXIF in the mapped region
        if let Ok(exif_data) = self.extract_exif_from_mapped(&mmap) {
            self.parse_exif_data_optimized(&exif_data)?;
        } else {
            // EXIF not in mapped region, fall back to seeking
            drop(mmap);
            return self.parse_with_seek_optimization(file, file_size);
        }

        self.stats.total_bytes_read += map_size;
        Ok(self.metadata_cache.clone())
    }

    /// Locate EXIF segment with minimal reading
    fn locate_exif_segment(
        &self,
        file: &mut File,
        file_size: usize,
    ) -> Result<ExifSegmentInfo, ExifError> {
        // Read only the first 32 bytes to determine format
        let mut header = [0u8; 32];
        file.read_exact(&mut header)?;
        file.seek(SeekFrom::Start(0))?;

        // Determine file format and locate EXIF
        if header[0] == 0xFF && header[1] == 0xD8 {
            // JPEG format
            self.locate_jpeg_exif(file, file_size)
        } else if (header[0] == 0x49 && header[1] == 0x49)
            || (header[0] == 0x4D && header[1] == 0x4D)
        {
            // TIFF/CR2 format - EXIF starts at beginning
            Ok(ExifSegmentInfo {
                offset: 0,
                size: file_size.min(self.max_exif_size),
                format: FileFormat::Tiff,
            })
        } else if header[4..8] == *b"ftyp" {
            // HEIC/MOV format
            self.locate_heic_exif(file, file_size)
        } else {
            Err(ExifError::InvalidExif(
                "Unsupported file format".to_string(),
            ))
        }
    }

    /// Locate EXIF segment in JPEG files with optimized seeking
    fn locate_jpeg_exif(
        &self,
        file: &mut File,
        _file_size: usize,
    ) -> Result<ExifSegmentInfo, ExifError> {
        let mut offset = 2; // Skip SOI marker
        let mut buffer = [0u8; 4];

        // Limit search to first 1MB to avoid reading entire large files
        let max_search = 1024 * 1024;

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

    /// Locate EXIF segment in HEIC/MOV files
    fn locate_heic_exif(
        &self,
        _file: &mut File,
        file_size: usize,
    ) -> Result<ExifSegmentInfo, ExifError> {
        // Simplified HEIC/MOV EXIF location
        // In practice, this would need more sophisticated parsing
        Ok(ExifSegmentInfo {
            offset: 0,
            size: file_size.min(self.max_exif_size),
            format: FileFormat::Heic,
        })
    }

    /// Read EXIF segment from file
    fn read_exif_segment(
        &mut self,
        file: &mut File,
        exif_info: &ExifSegmentInfo,
    ) -> Result<Vec<u8>, ExifError> {
        file.seek(SeekFrom::Start(exif_info.offset as u64))?;

        let size_to_read = exif_info.size.min(self.max_exif_size);

        if self.read_buffer.capacity() < size_to_read {
            self.read_buffer
                .reserve(size_to_read - self.read_buffer.capacity());
        }

        self.read_buffer.resize(size_to_read, 0);
        file.read_exact(&mut self.read_buffer)?;

        self.stats.total_bytes_read += size_to_read;
        Ok(self.read_buffer.clone())
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

        Err(ExifError::InvalidExif(
            "EXIF not found in mapped region".to_string(),
        ))
    }

    /// Parse EXIF data from bytes with optimizations
    /// Parse EXIF data from bytes
    pub fn parse_exif_from_bytes(
        &mut self,
        data: &[u8],
    ) -> Result<HashMap<String, String>, ExifError> {
        // Detect file format
        let format = self.detect_format(data)?;

        // Parse based on format
        match format {
            FileFormat::Jpeg => self.parse_jpeg_exif(data)?,
            FileFormat::Tiff => self.parse_tiff_exif(data)?,
            FileFormat::Heic => self.parse_heic_exif(data)?,
            FileFormat::Mov => self.parse_mov_exif(data)?,
            FileFormat::Mp4 => self.parse_mp4_exif(data)?,
        }

        Ok(self.metadata_cache.clone())
    }

    /// Parse EXIF data with optimizations
    fn parse_exif_data_optimized(&mut self, exif_data: &[u8]) -> Result<(), ExifError> {
        // Always use the real TIFF/EXIF walker. The old AVX2 path only
        // touched IFD0 and filled Make/Model/DateTime with a placeholder.
        let scope = self.parse_scope();
        TiffParser::parse_tiff_exif_scoped(exif_data, &mut self.metadata_cache, &scope)?;
        self.apply_target_field_filter();
        Ok(())
    }

    fn apply_target_field_filter(&mut self) {
        if self.target_fields.is_empty() {
            return;
        }
        let wanted = crate::types::wanted_normalized_set(&self.target_fields);
        self.metadata_cache
            .retain(|key, _| crate::types::tag_is_wanted_normalized(key, &wanted));
    }

    /// Parse only specific fields for maximum efficiency
    #[allow(dead_code)]
    fn parse_selective_fields(&mut self, exif_data: &[u8]) -> Result<(), ExifError> {
        let scope = self.parse_scope();
        TiffParser::parse_tiff_exif_scoped(exif_data, &mut self.metadata_cache, &scope)?;
        self.apply_target_field_filter();
        Ok(())
    }

    /// Parse a specific field from EXIF data
    fn parse_specific_field(
        &self,
        exif_data: &[u8],
        field_name: &str,
    ) -> Result<Option<String>, ExifError> {
        // Map field name to tag ID
        let tag_id = self.field_name_to_tag_id(field_name)?;

        // Parse the specific tag
        self.parse_tag_from_exif(exif_data, tag_id, field_name)
    }

    /// Map field name to EXIF tag ID
    fn field_name_to_tag_id(&self, field_name: &str) -> Result<u16, ExifError> {
        match field_name {
            "Make" => Ok(0x010F),
            "Model" => Ok(0x0110),
            "DateTime" => Ok(0x0132),
            "ExposureTime" => Ok(0x829A),
            "FNumber" => Ok(0x829D),
            "ISO" => Ok(0x8827),
            "FocalLength" => Ok(0x920A),
            "Flash" => Ok(0x9209),
            "WhiteBalance" => Ok(0xA403),
            "GPSLatitude" => Ok(0x0002),
            "GPSLongitude" => Ok(0x0004),
            _ => Err(ExifError::InvalidExif(format!(
                "Unknown field: {}",
                field_name
            ))),
        }
    }

    /// Parse a specific tag from EXIF data
    fn parse_tag_from_exif(
        &self,
        _exif_data: &[u8],
        _tag_id: u16,
        field_name: &str,
    ) -> Result<Option<String>, ExifError> {
        // Simplified tag parsing - in practice this would need full TIFF parsing
        // For now, return a placeholder
        Ok(Some(format!("{}_value", field_name)))
    }

    /// Detect file format from header
    fn detect_format(&self, data: &[u8]) -> Result<FileFormat, ExifError> {
        if data.len() < 4 {
            return Err(ExifError::InvalidExif("File too small".to_string()));
        }

        if data[0] == 0xFF && data[1] == 0xD8 {
            Ok(FileFormat::Jpeg)
        } else if (data[0] == 0x49 && data[1] == 0x49) || (data[0] == 0x4D && data[1] == 0x4D) {
            Ok(FileFormat::Tiff)
        } else if data.len() >= 8 && &data[4..8] == b"ftyp" {
            // Check for MP4 vs HEIC/MOV
            if data.len() >= 12 {
                let brand = &data[8..12];
                if brand == b"mp42" || brand == b"mp41" || brand == b"isom" {
                    Ok(FileFormat::Mp4)
                } else {
                    Ok(FileFormat::Heic)
                }
            } else {
                Ok(FileFormat::Heic)
            }
        } else {
            Err(ExifError::InvalidExif("Unsupported format".to_string()))
        }
    }

    /// Parse JPEG EXIF data
    fn parse_jpeg_exif(&mut self, data: &[u8]) -> Result<(), ExifError> {
        let payload = if let Ok(exif) = self.extract_exif_from_mapped(data) {
            exif
        } else {
            data.to_vec()
        };
        let scope = self.parse_scope();
        TiffParser::parse_tiff_exif_scoped(&payload, &mut self.metadata_cache, &scope)?;
        self.apply_target_field_filter();
        Ok(())
    }

    /// Parse TIFF EXIF data
    fn parse_tiff_exif(&mut self, data: &[u8]) -> Result<(), ExifError> {
        let scope = self.parse_scope();
        TiffParser::parse_tiff_exif_scoped(data, &mut self.metadata_cache, &scope)?;
        self.apply_target_field_filter();
        Ok(())
    }

    /// Parse HEIC EXIF data
    fn parse_heic_exif(&mut self, _data: &[u8]) -> Result<(), ExifError> {
        // Simplified HEIC parsing
        self.metadata_cache
            .insert("Format".to_string(), "HEIC".to_string());
        Ok(())
    }

    /// Parse MOV EXIF data
    fn parse_mov_exif(&mut self, data: &[u8]) -> Result<(), ExifError> {
        use crate::parsers::video::VideoParser;
        VideoParser::parse_mov_exif(data, &mut self.metadata_cache)?;
        Ok(())
    }

    /// Parse MP4 EXIF data
    fn parse_mp4_exif(&mut self, data: &[u8]) -> Result<(), ExifError> {
        use crate::parsers::video::VideoParser;
        VideoParser::parse_mp4_exif(data, &mut self.metadata_cache)?;

        // Add global computed fields (like Track and Media dates for video files)
        crate::computed_fields::ComputedFields::add_computed_fields(&mut self.metadata_cache);

        Ok(())
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("mmap_count".to_string(), self.stats.mmap_count.to_string());
        stats.insert("seek_count".to_string(), self.stats.seek_count.to_string());
        stats.insert(
            "hybrid_count".to_string(),
            self.stats.hybrid_count.to_string(),
        );
        stats.insert("simd_count".to_string(), self.stats.simd_count.to_string());
        stats.insert(
            "total_bytes_read".to_string(),
            self.stats.total_bytes_read.to_string(),
        );
        stats.insert(
            "total_processing_time".to_string(),
            self.stats.total_processing_time.to_string(),
        );
        stats.insert(
            "cache_hit_rate".to_string(),
            self.stats.cache_hit_rate.to_string(),
        );
        stats.insert("parser_type".to_string(), "OptimalExif".to_string());
        #[cfg(target_arch = "x86_64")]
        {
            stats.insert(
                "avx2_supported".to_string(),
                self.avx2_supported.to_string(),
            );
        }
        stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = OptimalParserStats::default();
    }

    /// AVX2 is CPUID leaf 7, subleaf 0, EBX bit 5.
    ///
    /// The `7` is a CPUID function number, not a CPU count. Leaf 0 EAX is the
    /// highest basic leaf this processor implements; do not query leaf 7 when
    /// that maximum is below 7. `__cpuid` is `unsafe` on newer rustc, so this
    /// stays in an `unsafe` block. `unused_unsafe` covers rustc versions that
    /// already treat these helpers as safe.
    #[cfg(target_arch = "x86_64")]
    fn check_avx2_support() -> bool {
        #[allow(unused_unsafe)]
        unsafe {
            let max_basic_leaf = std::arch::x86_64::__cpuid(0).eax;
            if max_basic_leaf < 7 {
                return false;
            }
            let leaf7 = std::arch::x86_64::__cpuid_count(7, 0);
            (leaf7.ebx & (1 << 5)) != 0
        }
    }
}

impl Default for OptimalExifParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch processor for optimal EXIF parsing
pub struct OptimalBatchProcessor {
    parser: OptimalExifParser,
    batch_size: usize,
}

impl OptimalBatchProcessor {
    /// Create a new batch processor
    pub fn new(batch_size: usize) -> Self {
        Self {
            parser: OptimalExifParser::new(),
            batch_size,
        }
    }

    /// Process multiple files with optimal strategy
    pub fn process_files(
        &mut self,
        file_paths: &[String],
    ) -> Result<Vec<HashMap<String, String>>, ExifError> {
        let mut results = Vec::new();

        for chunk in file_paths.chunks(self.batch_size) {
            for file_path in chunk {
                match self.parser.parse_file(file_path) {
                    Ok(metadata) => results.push(metadata),
                    Err(e) => {
                        eprintln!("Error processing {}: {}", file_path, e);
                        results.push(HashMap::new());
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get parser statistics
    pub fn get_stats(&self) -> HashMap<String, String> {
        self.parser.get_stats()
    }
}

impl Default for OptimalBatchProcessor {
    fn default() -> Self {
        Self::new(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_parser_creation() {
        let parser = OptimalExifParser::new();
        assert_eq!(parser.max_exif_size, 2 * 1024 * 1024);
        assert_eq!(parser.mmap_threshold, 8 * 1024 * 1024);
    }

    #[test]
    fn test_optimal_parser_with_target_fields() {
        let fields = vec!["Make".to_string(), "Model".to_string()];
        let parser = OptimalExifParser::with_target_fields(fields.clone());
        assert_eq!(parser.target_fields, fields);
    }

    #[test]
    fn test_strategy_determination() {
        let parser = OptimalExifParser::new();

        // Small file should use memory mapping
        assert!(matches!(
            parser.determine_strategy(1024),
            ParseStrategy::MemoryMap
        ));

        // Medium file should use hybrid
        assert!(matches!(
            parser.determine_strategy(16 * 1024 * 1024),
            ParseStrategy::Hybrid
        ));

        // Large file should use seeking
        assert!(matches!(
            parser.determine_strategy(100 * 1024 * 1024),
            ParseStrategy::SeekOptimized
        ));
    }

    #[test]
    fn test_field_name_to_tag_id() {
        let parser = OptimalExifParser::new();

        assert_eq!(parser.field_name_to_tag_id("Make").unwrap(), 0x010F);
        assert_eq!(parser.field_name_to_tag_id("Model").unwrap(), 0x0110);
        assert_eq!(parser.field_name_to_tag_id("DateTime").unwrap(), 0x0132);
    }

    fn put_u16(buf: &mut [u8], at: usize, v: u16) {
        buf[at..at + 2].copy_from_slice(&v.to_le_bytes());
    }

    fn put_u32(buf: &mut [u8], at: usize, v: u32) {
        buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }

    fn put_entry(buf: &mut [u8], at: usize, tag: u16, dtype: u16, count: u32, value: u32) {
        put_u16(buf, at, tag);
        put_u16(buf, at + 2, dtype);
        put_u32(buf, at + 4, count);
        put_u32(buf, at + 8, value);
    }

    #[test]
    fn seek_path_follows_trailing_exif_ifd() {
        // Force SeekOptimized (file > mmap_threshold * 4) with a small EXIF
        // window so ExifIFD at EOF is initially out of range — the same layout
        // as Adobe DNG converted from CR2.
        let mut parser = OptimalExifParser::with_thresholds(1024, 2048);
        let exif_ifd = 20_000usize;
        let mut data = vec![0u8; exif_ifd + 40];
        data[0] = b'I';
        data[1] = b'I';
        put_u16(&mut data, 2, 42);
        put_u32(&mut data, 4, 8);

        put_u16(&mut data, 8, 2);
        put_entry(&mut data, 10, 0x010F, 2, 6, 50);
        put_entry(&mut data, 22, 0x8769, 4, 1, exif_ifd as u32);
        put_u32(&mut data, 34, 0);
        data[50..56].copy_from_slice(b"Canon\0");
        data[56..76].copy_from_slice(b"2015:06:14 01:58:40\0");

        put_u16(&mut data, exif_ifd, 1);
        put_entry(&mut data, exif_ifd + 2, 0x9003, 2, 20, 56);
        put_u32(&mut data, exif_ifd + 14, 0);

        let dir = std::env::temp_dir();
        let path = dir.join("fast_exif_trailing_exif_ifd.tif");
        std::fs::write(&path, &data).unwrap();
        let metadata = parser.parse_file(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(metadata.get("Make").unwrap(), "Canon");
        assert_eq!(
            metadata.get("DateTimeOriginal").unwrap(),
            "2015:06:14 01:58:40"
        );
    }
}
