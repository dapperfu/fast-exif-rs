use crate::types::ExifError;
use std::collections::HashMap;

/// MKV parser for extracting metadata from Matroska video files
pub struct MkvParser;

impl MkvParser {
    /// Parse MKV metadata
    pub fn parse_mkv_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        if data.len() < 4 {
            return Err(ExifError::InvalidExif("MKV file too small".to_string()));
        }

        // Check MKV signature
        if data[0] != 0x1A || data[1] != 0x45 || data[2] != 0xDF || data[3] != 0xA3 {
            return Err(ExifError::InvalidExif("Invalid MKV signature".to_string()));
        }

        // Set format
        metadata.insert("Format".to_string(), "MKV".to_string());

        // Parse EBML structure
        let mut offset = 4; // Skip MKV signature
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                Self::process_mkv_element(element_id, element_data, metadata);
                offset += element_size;
            } else {
                break;
            }
        }

        // Add computed fields
        Self::add_computed_fields(metadata);

        Ok(())
    }

    /// Parse EBML element
    fn parse_ebml_element(data: &[u8], offset: usize) -> Option<(u32, usize, &[u8])> {
        if offset >= data.len() {
            return None;
        }

        // Parse element ID (variable length)
        let (element_id, id_size) = Self::parse_vint(data, offset)?;
        
        // Parse element size (variable length)
        let (element_size, size_size) = Self::parse_vint(data, offset + id_size)?;
        
        let total_size = id_size + size_size + element_size as usize;
        if offset + total_size > data.len() {
            return None;
        }

        let element_data = &data[offset + id_size + size_size..offset + total_size];
        Some((element_id, total_size, element_data))
    }

    /// Parse variable-length integer (VINT)
    fn parse_vint(data: &[u8], offset: usize) -> Option<(u32, usize)> {
        if offset >= data.len() {
            return None;
        }

        let first_byte = data[offset];
        let length = first_byte.leading_zeros() as usize + 1;
        
        if offset + length > data.len() || length > 4 {
            return None;
        }

        // Safe mask calculation to avoid overflow
        let mask = if length == 8 { 0xFF } else { (1u8 << (8 - length)) - 1 };
        let mut value = (first_byte & mask) as u32;
        
        for i in 1..length {
            value = (value << 8) | data[offset + i] as u32;
        }

        Some((value, length))
    }

    /// Process MKV element
    fn process_mkv_element(element_id: u32, element_data: &[u8], metadata: &mut HashMap<String, String>) {
        match element_id {
            0x1A45DFA3 => {
                // EBML Header
                Self::parse_ebml_header(element_data, metadata);
            }
            0x18538067 => {
                // Segment
                Self::parse_segment(element_data, metadata);
            }
            _ => {
                // Unknown element, skip
            }
        }
    }

    /// Parse EBML header
    fn parse_ebml_header(data: &[u8], metadata: &mut HashMap<String, String>) {
        let mut offset = 0;
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                match element_id {
                    0x4286 => {
                        // EBMLVersion
                        if let Some((version, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("EBMLVersion".to_string(), version.to_string());
                        }
                    }
                    0x42F7 => {
                        // EBMLReadVersion
                        if let Some((version, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("EBMLReadVersion".to_string(), version.to_string());
                        }
                    }
                    0x42F2 => {
                        // EBMLMaxIDLength
                        if let Some((length, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("EBMLMaxIDLength".to_string(), length.to_string());
                        }
                    }
                    0x42F3 => {
                        // EBMLMaxSizeLength
                        if let Some((length, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("EBMLMaxSizeLength".to_string(), length.to_string());
                        }
                    }
                    0x4282 => {
                        // DocType
                        if let Ok(doc_type) = String::from_utf8(element_data.to_vec()) {
                            metadata.insert("DocType".to_string(), doc_type);
                        }
                    }
                    0x4287 => {
                        // DocTypeVersion
                        if let Some((version, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("DocTypeVersion".to_string(), version.to_string());
                        }
                    }
                    0x4285 => {
                        // DocTypeReadVersion
                        if let Some((version, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("DocTypeReadVersion".to_string(), version.to_string());
                        }
                    }
                    _ => {}
                }
                offset += element_size;
            } else {
                break;
            }
        }
    }

    /// Parse segment
    fn parse_segment(data: &[u8], metadata: &mut HashMap<String, String>) {
        let mut offset = 0;
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                match element_id {
                    0x1549A966 => {
                        // Info
                        Self::parse_info(element_data, metadata);
                    }
                    0x1654AE6B => {
                        // Tracks
                        Self::parse_tracks(element_data, metadata);
                    }
                    0x1F43B675 => {
                        // Cluster
                        // Skip clusters for now as they contain video data
                    }
                    _ => {}
                }
                offset += element_size;
            } else {
                break;
            }
        }
    }

    /// Parse info element
    fn parse_info(data: &[u8], metadata: &mut HashMap<String, String>) {
        let mut offset = 0;
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                match element_id {
                    0x2AD7B1 => {
                        // TimecodeScale
                        if let Some((scale, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("TimecodeScale".to_string(), scale.to_string());
                        }
                    }
                    0x4489 => {
                        // Duration
                        if let Some((duration, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("Duration".to_string(), duration.to_string());
                        }
                    }
                    0x7BA9 => {
                        // Title
                        if let Ok(title) = String::from_utf8(element_data.to_vec()) {
                            metadata.insert("Title".to_string(), title);
                        }
                    }
                    0x4D80 => {
                        // MuxingApp
                        if let Ok(app) = String::from_utf8(element_data.to_vec()) {
                            metadata.insert("MuxingApp".to_string(), app);
                        }
                    }
                    0x5741 => {
                        // WritingApp
                        if let Ok(app) = String::from_utf8(element_data.to_vec()) {
                            metadata.insert("WritingApp".to_string(), app);
                        }
                    }
                    _ => {}
                }
                offset += element_size;
            } else {
                break;
            }
        }
    }

    /// Parse tracks element
    fn parse_tracks(data: &[u8], metadata: &mut HashMap<String, String>) {
        let mut offset = 0;
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                match element_id {
                    0xAE => {
                        // TrackEntry
                        Self::parse_track_entry(element_data, metadata);
                    }
                    _ => {}
                }
                offset += element_size;
            } else {
                break;
            }
        }
    }

    /// Parse track entry
    fn parse_track_entry(data: &[u8], metadata: &mut HashMap<String, String>) {
        let mut offset = 0;
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                match element_id {
                    0xD7 => {
                        // TrackNumber
                        if let Some((number, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("TrackNumber".to_string(), number.to_string());
                        }
                    }
                    0x73C5 => {
                        // TrackUID
                        if let Some((uid, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("TrackUID".to_string(), uid.to_string());
                        }
                    }
                    0x83 => {
                        // TrackType
                        if let Some((track_type, _)) = Self::parse_vint(element_data, 0) {
                            let type_desc = match track_type {
                                1 => "Video",
                                2 => "Audio",
                                3 => "Complex",
                                4 => "Logo",
                                5 => "Subtitle",
                                6 => "Buttons",
                                7 => "Control",
                                _ => "Unknown",
                            };
                            metadata.insert("TrackType".to_string(), type_desc.to_string());
                        }
                    }
                    0x86 => {
                        // CodecID
                        if let Ok(codec_id) = String::from_utf8(element_data.to_vec()) {
                            metadata.insert("CodecID".to_string(), codec_id);
                        }
                    }
                    0xE0 => {
                        // Video
                        Self::parse_video_track(element_data, metadata);
                    }
                    0xE1 => {
                        // Audio
                        Self::parse_audio_track(element_data, metadata);
                    }
                    _ => {}
                }
                offset += element_size;
            } else {
                break;
            }
        }
    }

    /// Parse video track
    fn parse_video_track(data: &[u8], metadata: &mut HashMap<String, String>) {
        let mut offset = 0;
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                match element_id {
                    0xB0 => {
                        // PixelWidth
                        if let Some((width, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("ImageWidth".to_string(), width.to_string());
                        }
                    }
                    0xBA => {
                        // PixelHeight
                        if let Some((height, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("ImageHeight".to_string(), height.to_string());
                        }
                    }
                    _ => {}
                }
                offset += element_size;
            } else {
                break;
            }
        }
    }

    /// Parse audio track
    fn parse_audio_track(data: &[u8], metadata: &mut HashMap<String, String>) {
        let mut offset = 0;
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                match element_id {
                    0xB5 => {
                        // SamplingFrequency
                        if let Some((freq, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("SamplingFrequency".to_string(), freq.to_string());
                        }
                    }
                    0x9F => {
                        // Channels
                        if let Some((channels, _)) = Self::parse_vint(element_data, 0) {
                            metadata.insert("Channels".to_string(), channels.to_string());
                        }
                    }
                    _ => {}
                }
                offset += element_size;
            } else {
                break;
            }
        }
    }

    /// Add computed fields that exiftool provides
    fn add_computed_fields(metadata: &mut HashMap<String, String>) {
        // File information
        metadata.insert("FileTypeExtension".to_string(), "mkv".to_string());
        metadata.insert("MIMEType".to_string(), "video/x-matroska".to_string());
        metadata.insert("ExifByteOrder".to_string(), "Little-endian (Intel, II)".to_string());

        // Add format-specific fields
        if !metadata.contains_key("Format") {
            metadata.insert("Format".to_string(), "MKV".to_string());
        }
    }
}
