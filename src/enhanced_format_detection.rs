use crate::types::ExifError;

/// Enhanced format detection utilities for comprehensive image and video format support
pub struct EnhancedFormatDetector;

impl EnhancedFormatDetector {
    /// Detect the format of image/video data with comprehensive support
    pub fn detect_format(data: &[u8]) -> Result<String, ExifError> {
        if data.len() < 4 {
            return Err(ExifError::InvalidExif("File too small".to_string()));
        }

        // Check for JPEG variants
        if data[0] == 0xFF && data[1] == 0xD8 {
            return Ok("JPEG".to_string());
        }

        // Check for TIFF-based formats (RAW files)
        if (data[0] == 0x49 && data[1] == 0x49) || (data[0] == 0x4D && data[1] == 0x4D) {
            return Self::detect_tiff_based_format(data);
        }

        // Check for HEIF/HIF variants
        if data.len() >= 12 {
            let header = &data[4..12];
            println!("DEBUG: Checking HEIF header: {:?}", header);
            if header == b"ftypheic"
                || header == b"ftypheix"
                || header == b"ftypmif1"
                || header == b"ftypmsf1"
                || header == b"ftyphevc"
                || header == b"ftypavci"
                || header == b"ftypavcs"
                || header == b"ftyphif1"  // Hasselblad HIF
            {
                println!("DEBUG: Detected HEIF format with header: {:?}", header);
                return Ok("HEIF".to_string());
            }
        }

        // Check for PNG format
        if data.len() >= 8 {
            if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47
                && data[4] == 0x0D && data[5] == 0x0A && data[6] == 0x1A && data[7] == 0x0A
            {
                return Ok("PNG".to_string());
            }
        }

        // Check for BMP format
        if data.len() >= 2 {
            if data[0] == 0x42 && data[1] == 0x4D {
                return Ok("BMP".to_string());
            }
        }

        // Check for GIF format
        if data.len() >= 6 {
            if data[0..6] == *b"GIF87a" || data[0..6] == *b"GIF89a" {
                return Ok("GIF".to_string());
            }
        }

        // Check for WEBP format
        if data.len() >= 12 {
            if data[0..4] == *b"RIFF" && data[8..12] == *b"WEBP" {
                return Ok("WEBP".to_string());
            }
        }

        // Check for MKV format
        if data.len() >= 4 {
            if data[0] == 0x1A && data[1] == 0x45 && data[2] == 0xDF && data[3] == 0xA3 {
                return Ok("MKV".to_string());
            }
        }

        // Check for AVI format
        if data.len() >= 12 {
            if data[0..4] == *b"RIFF" && data[8..12] == *b"AVI " {
                return Ok("AVI".to_string());
            }
        }

        // Check for WMV format (ASF container)
        if data.len() >= 16 {
            if data[0..16] == *b"\x30\x26\xB2\x75\x8E\x66\xCF\x11\xA6\xD9\x00\xAA\x00\x62\xCE\x6C" {
                return Ok("WMV".to_string());
            }
        }

        // Check for WEBM format
        if data.len() >= 12 {
            if data[0..4] == *b"RIFF" && data[8..12] == *b"WEBM" {
                return Ok("WEBM".to_string());
            }
        }

        // Check for QuickTime/MOV/MP4/3GP format
        if data.len() >= 8 {
            let atom_type = &data[4..8];

            if atom_type == b"ftyp" && data.len() >= 12 {
                let brand = &data[8..12];

                // Check for 3GP format
                if brand == b"3gp4" || brand == b"3gp5" || brand == b"3g2a" {
                    return Ok("3GP".to_string());
                }

                // Check for MOV format (QuickTime)
                if brand == b"qt  " || brand == b"CAEP" {
                    return Ok("MOV".to_string());
                }

                // Check for MP4 format (ISO Base Media File Format)
                if brand == b"mp41" || brand == b"mp42" || brand == b"isom" || brand == b"avc1" {
                    return Ok("MP4".to_string());
                }

                // Default to MOV for other QuickTime formats
                return Ok("MOV".to_string());
            }

            // Check for other QuickTime atoms
            if atom_type == b"moov" || atom_type == b"mdat" {
                return Ok("MOV".to_string());
            }
        }

        // Check for Sony ARW format
        if Self::is_sony_arw(data) {
            return Ok("ARW".to_string());
        }

        // Check for Fuji RAF format
        if Self::is_fuji_raf(data) {
            return Ok("RAF".to_string());
        }

        // Check for Samsung SRW format
        if Self::is_samsung_srw(data) {
            return Ok("SRW".to_string());
        }

        // Check for Pentax PEF format
        if Self::is_pentax_pef(data) {
            return Ok("PEF".to_string());
        }

        // Check for Panasonic RW2 format
        if Self::is_panasonic_rw2(data) {
            return Ok("RW2".to_string());
        }

        Err(ExifError::UnsupportedFormat("Unknown format".to_string()))
    }

    /// Detect TIFF-based format (RAW files)
    fn detect_tiff_based_format(data: &[u8]) -> Result<String, ExifError> {
        // Check for Canon CR2
        if Self::is_canon_cr2(data) {
            return Ok("CR2".to_string());
        }

        // Check for Canon CR3
        if Self::is_canon_cr3(data) {
            return Ok("CR3".to_string());
        }

        // Check for Nikon NEF
        if Self::is_nikon_nef(data) {
            return Ok("NEF".to_string());
        }

        // Check for Sony ARW
        if Self::is_sony_arw(data) {
            return Ok("ARW".to_string());
        }

        // Check for Fuji RAF
        if Self::is_fuji_raf(data) {
            return Ok("RAF".to_string());
        }

        // Check for Samsung SRW
        if Self::is_samsung_srw(data) {
            return Ok("SRW".to_string());
        }

        // Check for Olympus ORF
        if Self::is_olympus_orf(data) {
            return Ok("ORF".to_string());
        }

        // Check for Pentax PEF
        if Self::is_pentax_pef(data) {
            return Ok("PEF".to_string());
        }

        // Check for Panasonic RW2
        if Self::is_panasonic_rw2(data) {
            return Ok("RW2".to_string());
        }

        // Check for DNG
        if Self::is_dng_file(data) {
            return Ok("DNG".to_string());
        }

        // Default to TIFF
        Ok("TIFF".to_string())
    }

    /// Check if data represents a Canon CR2 file
    pub fn is_canon_cr2(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(5).any(|w| w == b"Canon")
    }

    /// Check if data represents a Canon CR3 file
    pub fn is_canon_cr3(data: &[u8]) -> bool {
        // CR3 files are based on ISO Base Media File Format
        if data.len() >= 8 {
            let atom_type = &data[4..8];
            if atom_type == b"ftyp" && data.len() >= 12 {
                let brand = &data[8..12];
                return brand == b"crx " || brand == b"crx1";
            }
        }
        false
    }

    /// Check if data represents a Nikon NEF file
    pub fn is_nikon_nef(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(5).any(|w| w == b"Nikon")
    }

    /// Check if data represents a Sony ARW file
    pub fn is_sony_arw(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(4).any(|w| w == b"Sony")
            || data[..search_len].windows(4).any(|w| w == b"SONY")
    }

    /// Check if data represents a Fuji RAF file
    pub fn is_fuji_raf(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(6).any(|w| w == b"FUJIFILM")
            || data[..search_len].windows(5).any(|w| w == b"Fuji")
    }

    /// Check if data represents a Samsung SRW file
    pub fn is_samsung_srw(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(7).any(|w| w == b"Samsung")
            || data[..search_len].windows(7).any(|w| w == b"SAMSUNG")
    }

    /// Check if data represents an Olympus ORF file
    pub fn is_olympus_orf(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(7).any(|w| w == b"OLYMPUS")
    }

    /// Check if data represents a Pentax PEF file
    pub fn is_pentax_pef(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(6).any(|w| w == b"PENTAX")
    }

    /// Check if data represents a Panasonic RW2 file
    pub fn is_panasonic_rw2(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(9).any(|w| w == b"Panasonic")
    }

    /// Check if data represents a DNG file
    pub fn is_dng_file(data: &[u8]) -> bool {
        let search_len = std::cmp::min(8192, data.len());

        // Check for Samsung first (most common case)
        if data[..search_len]
            .windows(7)
            .any(|w| w.eq_ignore_ascii_case(b"samsung"))
        {
            return true;
        }

        // Check for DNG version tag (0xC612) or DNG-specific strings
        if data[..search_len].windows(3).any(|w| w == b"DNG") {
            return true;
        }

        // Check for Adobe-specific markers
        if data[..search_len].windows(5).any(|w| w == b"Adobe") {
            return true;
        }

        false
    }

    /// Detect camera make from various markers in the file
    pub fn detect_camera_make(data: &[u8]) -> Option<String> {
        let search_len = std::cmp::min(8192, data.len());

        // Check for Canon
        if data[..search_len].windows(5).any(|w| w == b"Canon") {
            return Some("Canon".to_string());
        }

        // Check for Nikon
        if data[..search_len].windows(5).any(|w| w == b"Nikon")
            || data[..search_len]
                .windows(15)
                .any(|w| w == b"NIKON CORPORATION")
        {
            return Some("NIKON CORPORATION".to_string());
        }

        // Check for Sony
        if data[..search_len].windows(4).any(|w| w == b"Sony")
            || data[..search_len].windows(4).any(|w| w == b"SONY")
        {
            return Some("SONY".to_string());
        }

        // Check for Fuji
        if data[..search_len].windows(6).any(|w| w == b"FUJIFILM")
            || data[..search_len].windows(5).any(|w| w == b"Fuji")
        {
            return Some("FUJIFILM".to_string());
        }

        // Check for Samsung
        if data[..search_len].windows(7).any(|w| w == b"Samsung")
            || data[..search_len].windows(7).any(|w| w == b"SAMSUNG")
        {
            return Some("Samsung".to_string());
        }

        // Check for Olympus
        if data[..search_len].windows(7).any(|w| w == b"OLYMPUS") {
            return Some("OLYMPUS OPTICAL CO.,LTD".to_string());
        }

        // Check for Pentax
        if data[..search_len].windows(6).any(|w| w == b"PENTAX") {
            return Some("PENTAX".to_string());
        }

        // Check for Panasonic
        if data[..search_len].windows(9).any(|w| w == b"Panasonic") {
            return Some("Panasonic".to_string());
        }

        // Check for GoPro
        if data[..search_len].windows(6).any(|w| w == b"GoPro") {
            return Some("GoPro".to_string());
        }

        // Check for Motorola
        if data[..search_len].windows(8).any(|w| w == b"Motorola") {
            return Some("Motorola".to_string());
        }

        // Check for Ricoh
        if data[..search_len].windows(5).any(|w| w == b"RICOH") {
            return Some("RICOH".to_string());
        }

        // Check for Hasselblad
        if data[..search_len].windows(10).any(|w| w == b"Hasselblad") {
            return Some("Hasselblad".to_string());
        }

        None
    }

    /// Check if TIFF data contains valid EXIF data
    pub fn is_valid_exif_data(data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }

        // Check for TIFF header
        let is_little_endian = data[0] == 0x49 && data[1] == 0x49;
        let is_big_endian = data[0] == 0x4D && data[1] == 0x4D;

        if !is_little_endian && !is_big_endian {
            return false;
        }

        // Check for TIFF magic number (42)
        let magic_offset = if is_little_endian {
            u16::from_le_bytes([data[2], data[3]])
        } else {
            u16::from_be_bytes([data[2], data[3]])
        };

        magic_offset == 42
    }

    /// Get supported formats list
    pub fn get_supported_formats() -> Vec<&'static str> {
        vec![
            // Image formats
            "JPEG", "PNG", "BMP", "GIF", "WEBP", "TIFF",
            // RAW formats
            "CR2", "CR3", "NEF", "ARW", "RAF", "SRW", "ORF", "PEF", "RW2", "DNG",
            // HEIF variants
            "HEIF", "HEIC", "HIF",
            // Video formats
            "MP4", "MOV", "3GP", "AVI", "WMV", "WEBM", "MKV",
        ]
    }

    /// Check if format is supported
    pub fn is_format_supported(format: &str) -> bool {
        Self::get_supported_formats().contains(&format)
    }
}
