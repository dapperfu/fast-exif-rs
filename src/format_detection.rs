use crate::types::ExifError;

/// Format detection utilities for various image and video formats
pub struct FormatDetector;

impl FormatDetector {
    /// Detect the format of image/video data
    pub fn detect_format(data: &[u8]) -> Result<String, ExifError> {
        if data.len() < 4 {
            return Err(ExifError::InvalidExif("File too small".to_string()));
        }

        // Check for JPEG
        if data[0] == 0xFF && data[1] == 0xD8 {
            return Ok("JPEG".to_string());
        }

        // Check for TIFF/CR2 (Canon RAW)
        if (data[0] == 0x49 && data[1] == 0x49) || (data[0] == 0x4D && data[1] == 0x4D) {
            // Check if it's CR2 by looking for Canon-specific markers
            if Self::is_canon_cr2(data) {
                return Ok("CR2".to_string());
            } else if Self::is_nikon_nef(data) {
                return Ok("NEF".to_string());
            } else if Self::is_olympus_raw(data) {
                return Ok("ORF".to_string());
            } else if Self::is_dng_file(data) {
                return Ok("DNG".to_string());
            } else {
                return Ok("TIFF".to_string());
            }
        }

        // Check for HEIF/HIF
        if data.len() >= 12 {
            let header = &data[4..12];
            if header == b"ftypheic"
                || header == b"ftypheix"
                || header == b"ftypmif1"
                || header == b"ftypmsf1"
                || header == b"ftyphevc"
                || header == b"ftypavci"
                || header == b"ftypavcs"
            {
                return Ok("HEIF".to_string());
            }
        }

        // Check for QuickTime/MOV/MP4/3GP format
        if data.len() >= 8 {
            // QuickTime files start with atom size (4 bytes) followed by atom type
            let atom_type = &data[4..8];

            if atom_type == b"ftyp" && data.len() >= 12 {
                let brand = &data[8..12];

                // Check for 3GP format first
                if brand == b"3gp4" || brand == b"3gp5" || brand == b"3g2a" {
                    return Ok("3GP".to_string());
                }

                // Check for MOV format (QuickTime) first
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

        Err(ExifError::UnsupportedFormat("Unknown format".to_string()))
    }

    /// Check if data represents a Canon CR2 file
    pub fn is_canon_cr2(data: &[u8]) -> bool {
        // CR2 files have Canon-specific markers
        // Look for "Canon" in the first 1KB
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(5).any(|w| w == b"Canon")
    }

    /// Check if data represents a Canon JPEG file
    pub fn is_canon_jpeg(data: &[u8]) -> bool {
        // Check for Canon-specific markers in JPEG files
        // Look for "Canon" in EXIF data or maker notes
        let search_len = std::cmp::min(8192, data.len());
        data[..search_len].windows(5).any(|w| w == b"Canon")
    }

    /// Check if data represents a Nikon NEF file
    pub fn is_nikon_nef(data: &[u8]) -> bool {
        // NEF files have Nikon-specific markers
        // Look for "Nikon" in the first 1KB
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(5).any(|w| w == b"Nikon")
    }

    /// Check if data represents an Olympus RAW file
    pub fn is_olympus_raw(data: &[u8]) -> bool {
        // Olympus RAW files have Olympus-specific markers
        // Look for "OLYMPUS" in the first 1KB
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(7).any(|w| w == b"OLYMPUS")
    }

    /// Check if data represents a DNG file
    pub fn is_dng_file(data: &[u8]) -> bool {
        // DNG (Digital Negative) files are TIFF-based but have specific characteristics
        // Look for DNG-specific markers in the first 8KB
        let search_len = std::cmp::min(8192, data.len());

        // Debug: Check for Samsung first (most common case)
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
        // Detect camera make from various markers in the file
        let search_len = std::cmp::min(8192, data.len());

        // Check for Canon
        if data[..search_len].windows(5).any(|w| w == b"Canon") {
            return Some("Canon".to_string());
        }

        // Check for Nikon (both NIKON CORPORATION and NIKON)
        if data[..search_len].windows(5).any(|w| w == b"Nikon")
            || data[..search_len]
                .windows(15)
                .any(|w| w == b"NIKON CORPORATION")
        {
            return Some("NIKON CORPORATION".to_string());
        }

        // Check for GoPro
        if data[..search_len].windows(6).any(|w| w == b"GoPro") {
            return Some("GoPro".to_string());
        }

        // Check for Samsung
        if data[..search_len].windows(7).any(|w| w == b"Samsung")
            || data[..search_len].windows(7).any(|w| w == b"SAMSUNG")
        {
            return Some("Samsung".to_string());
        }

        // Check for Motorola
        if data[..search_len].windows(8).any(|w| w == b"Motorola") {
            return Some("Motorola".to_string());
        }

        // Check for Olympus
        if data[..search_len].windows(7).any(|w| w == b"OLYMPUS") {
            return Some("OLYMPUS OPTICAL CO.,LTD".to_string());
        }

        // Check for Ricoh
        if data[..search_len].windows(5).any(|w| w == b"RICOH") {
            return Some("RICOH".to_string());
        }

        None
    }

    /// Check if TIFF data contains valid EXIF data
    pub fn is_valid_exif_data(data: &[u8]) -> bool {
        // Check if TIFF data contains valid EXIF data
        // EXIF data should have specific tags like DateTime, Make, Model, etc.

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
}
