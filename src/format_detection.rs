use crate::types::ExifError;

/// Unified format detection for read and write paths.
pub struct FormatDetector;

impl FormatDetector {
    /// Detect the format of image/video data with comprehensive support.
    pub fn detect_format(data: &[u8]) -> Result<String, ExifError> {
        if data.len() < 4 {
            return Err(ExifError::InvalidExif("File too small".to_string()));
        }

        if data[0] == 0xFF && data[1] == 0xD8 {
            return Ok("JPEG".to_string());
        }

        if (data[0] == 0x49 && data[1] == 0x49) || (data[0] == 0x4D && data[1] == 0x4D) {
            return Self::detect_tiff_based_format(data);
        }

        if data.len() >= 12 {
            let header = &data[4..12];
            if header == b"ftypheic"
                || header == b"ftypheix"
                || header == b"ftypmif1"
                || header == b"ftypmsf1"
                || header == b"ftyphevc"
                || header == b"ftypavci"
                || header == b"ftypavcs"
                || header == b"ftyphif1"
            {
                return Ok("HEIF".to_string());
            }
        }

        if data.len() >= 8 {
            if data[0] == 0x89
                && data[1] == 0x50
                && data[2] == 0x4E
                && data[3] == 0x47
                && data[4] == 0x0D
                && data[5] == 0x0A
                && data[6] == 0x1A
                && data[7] == 0x0A
            {
                return Ok("PNG".to_string());
            }
        }

        if data.len() >= 2 && data[0] == 0x42 && data[1] == 0x4D {
            return Ok("BMP".to_string());
        }

        if data.len() >= 6 && (data[0..6] == *b"GIF87a" || data[0..6] == *b"GIF89a") {
            return Ok("GIF".to_string());
        }

        if data.len() >= 12 && data[0..4] == *b"RIFF" && data[8..12] == *b"WEBP" {
            return Ok("WEBP".to_string());
        }

        if data.len() >= 4 && data[0] == 0x1A && data[1] == 0x45 && data[2] == 0xDF && data[3] == 0xA3 {
            return Ok("MKV".to_string());
        }

        if data.len() >= 12 && data[0..4] == *b"RIFF" && data[8..12] == *b"AVI " {
            return Ok("AVI".to_string());
        }

        if data.len() >= 16
            && data[0..16] == *b"\x30\x26\xB2\x75\x8E\x66\xCF\x11\xA6\xD9\x00\xAA\x00\x62\xCE\x6C"
        {
            return Ok("WMV".to_string());
        }

        if data.len() >= 12 && data[0..4] == *b"RIFF" && data[8..12] == *b"WEBM" {
            return Ok("WEBM".to_string());
        }

        if data.len() >= 8 {
            let atom_type = &data[4..8];

            if atom_type == b"ftyp" && data.len() >= 12 {
                let brand = &data[8..12];

                if brand == b"3gp4" || brand == b"3gp5" || brand == b"3g2a" {
                    return Ok("3GP".to_string());
                }

                if brand == b"qt  " || brand == b"CAEP" {
                    return Ok("MOV".to_string());
                }

                if brand == b"mp41" || brand == b"mp42" || brand == b"isom" || brand == b"avc1" {
                    return Ok("MP4".to_string());
                }

                return Ok("MOV".to_string());
            }

            if atom_type == b"moov" || atom_type == b"mdat" {
                return Ok("MOV".to_string());
            }
        }

        if Self::is_sony_arw(data) {
            return Ok("ARW".to_string());
        }

        if Self::is_fuji_raf(data) {
            return Ok("RAF".to_string());
        }

        if Self::is_samsung_srw(data) {
            return Ok("SRW".to_string());
        }

        if Self::is_pentax_pef(data) {
            return Ok("PEF".to_string());
        }

        if Self::is_panasonic_rw2(data) {
            return Ok("RW2".to_string());
        }

        Err(ExifError::UnsupportedFormat("Unknown format".to_string()))
    }

    fn detect_tiff_based_format(data: &[u8]) -> Result<String, ExifError> {
        if Self::is_canon_cr2(data) {
            return Ok("CR2".to_string());
        }

        if Self::is_canon_cr3(data) {
            return Ok("CR3".to_string());
        }

        if Self::is_nikon_nef(data) {
            return Ok("NEF".to_string());
        }

        if Self::is_sony_arw(data) {
            return Ok("ARW".to_string());
        }

        if Self::is_fuji_raf(data) {
            return Ok("RAF".to_string());
        }

        if Self::is_samsung_srw(data) {
            return Ok("SRW".to_string());
        }

        if Self::is_olympus_orf(data) {
            return Ok("ORF".to_string());
        }

        if Self::is_pentax_pef(data) {
            return Ok("PEF".to_string());
        }

        if Self::is_panasonic_rw2(data) {
            return Ok("RW2".to_string());
        }

        if Self::is_dng_file(data) {
            return Ok("DNG".to_string());
        }

        Ok("TIFF".to_string())
    }

    pub fn is_canon_cr2(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(5).any(|w| w == b"Canon")
    }

    pub fn is_canon_jpeg(data: &[u8]) -> bool {
        let search_len = std::cmp::min(8192, data.len());
        data[..search_len].windows(5).any(|w| w == b"Canon")
    }

    pub fn is_canon_cr3(data: &[u8]) -> bool {
        if data.len() >= 12 {
            let atom_type = &data[4..8];
            if atom_type == b"ftyp" {
                let brand = &data[8..12];
                return brand == b"crx " || brand == b"crx1";
            }
        }
        false
    }

    pub fn is_nikon_nef(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(5).any(|w| w == b"Nikon")
            || data[..search_len].windows(5).any(|w| w == b"NIKON")
            || data[..search_len]
                .windows(15)
                .any(|w| w == b"NIKON CORPORATION")
    }

    pub fn is_olympus_raw(data: &[u8]) -> bool {
        Self::is_olympus_orf(data)
    }

    pub fn is_olympus_orf(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(7).any(|w| w == b"OLYMPUS")
    }

    pub fn is_sony_arw(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(4).any(|w| w == b"Sony")
            || data[..search_len].windows(4).any(|w| w == b"SONY")
    }

    pub fn is_fuji_raf(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(6).any(|w| w == b"FUJIFILM")
            || data[..search_len].windows(5).any(|w| w == b"Fuji")
    }

    pub fn is_samsung_srw(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(7).any(|w| w == b"Samsung")
            || data[..search_len].windows(7).any(|w| w == b"SAMSUNG")
    }

    pub fn is_pentax_pef(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(6).any(|w| w == b"PENTAX")
    }

    pub fn is_panasonic_rw2(data: &[u8]) -> bool {
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(9).any(|w| w == b"Panasonic")
    }

    pub fn is_dng_file(data: &[u8]) -> bool {
        let search_len = std::cmp::min(8192, data.len());

        if data[..search_len]
            .windows(7)
            .any(|w| w.eq_ignore_ascii_case(b"samsung"))
        {
            return true;
        }

        if data[..search_len].windows(3).any(|w| w == b"DNG") {
            return true;
        }

        if data[..search_len].windows(5).any(|w| w == b"Adobe") {
            return true;
        }

        false
    }

    pub fn detect_camera_make(data: &[u8]) -> Option<String> {
        let search_len = std::cmp::min(8192, data.len());

        if data[..search_len].windows(5).any(|w| w == b"Canon") {
            return Some("Canon".to_string());
        }

        if data[..search_len].windows(5).any(|w| w == b"Nikon")
            || data[..search_len]
                .windows(15)
                .any(|w| w == b"NIKON CORPORATION")
        {
            return Some("NIKON CORPORATION".to_string());
        }

        if data[..search_len].windows(4).any(|w| w == b"Sony")
            || data[..search_len].windows(4).any(|w| w == b"SONY")
        {
            return Some("SONY".to_string());
        }

        if data[..search_len].windows(6).any(|w| w == b"FUJIFILM")
            || data[..search_len].windows(5).any(|w| w == b"Fuji")
        {
            return Some("FUJIFILM".to_string());
        }

        if data[..search_len].windows(7).any(|w| w == b"Samsung")
            || data[..search_len].windows(7).any(|w| w == b"SAMSUNG")
        {
            return Some("Samsung".to_string());
        }

        if data[..search_len].windows(7).any(|w| w == b"OLYMPUS") {
            return Some("OLYMPUS OPTICAL CO.,LTD".to_string());
        }

        if data[..search_len].windows(6).any(|w| w == b"PENTAX") {
            return Some("PENTAX".to_string());
        }

        if data[..search_len].windows(9).any(|w| w == b"Panasonic") {
            return Some("Panasonic".to_string());
        }

        if data[..search_len].windows(6).any(|w| w == b"GoPro") {
            return Some("GoPro".to_string());
        }

        if data[..search_len].windows(8).any(|w| w == b"Motorola") {
            return Some("Motorola".to_string());
        }

        if data[..search_len].windows(5).any(|w| w == b"RICOH") {
            return Some("RICOH".to_string());
        }

        if data[..search_len].windows(10).any(|w| w == b"Hasselblad") {
            return Some("Hasselblad".to_string());
        }

        None
    }

    pub fn is_valid_exif_data(data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }

        let is_little_endian = data[0] == 0x49 && data[1] == 0x49;
        let is_big_endian = data[0] == 0x4D && data[1] == 0x4D;

        if !is_little_endian && !is_big_endian {
            return false;
        }

        let magic_offset = if is_little_endian {
            u16::from_le_bytes([data[2], data[3]])
        } else {
            u16::from_be_bytes([data[2], data[3]])
        };

        magic_offset == 42
    }

    pub fn get_supported_formats() -> Vec<&'static str> {
        vec![
            "JPEG", "PNG", "BMP", "GIF", "WEBP", "TIFF",
            "CR2", "CR3", "NEF", "ARW", "RAF", "SRW", "ORF", "PEF", "RW2", "DNG",
            "HEIF", "HEIC", "HIF",
            "MP4", "MOV", "3GP", "AVI", "WMV", "WEBM", "MKV",
        ]
    }

    pub fn is_format_supported(format: &str) -> bool {
        Self::get_supported_formats().contains(&format)
    }
}

/// Backward-compatible alias for the unified format detector.
pub type EnhancedFormatDetector = FormatDetector;
