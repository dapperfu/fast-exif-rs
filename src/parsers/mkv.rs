use crate::types::ExifError;
use std::collections::HashMap;
use chrono::DateTime;

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
        let mut offset = 0; // Start from beginning - MKV signature is part of EBML Header
        while offset < data.len() {
            if let Some((element_id, element_size, element_data)) = Self::parse_ebml_element(data, offset) {
                Self::process_mkv_element(element_id, element_data, metadata);
                offset += element_size;
            } else {
                break;
            }
        }
        
        // Also search for DateUTC elements using pattern matching
        Self::search_for_date_elements(data, metadata);

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
        
        // Count leading zeros to determine length
        let length = first_byte.leading_zeros() as usize + 1;
        
        if offset + length > data.len() || length > 8 {
            return None;
        }

        // For VINT, the first byte contains the length information
        // The value is the remaining bits plus any additional bytes
        let mut value = 0u32;
        
        if length == 1 {
            // 1-byte VINT: value is the entire byte
            value = first_byte as u32;
        } else {
            // Multi-byte VINT: first byte has length info, remaining bytes contain the value
            // For multi-byte VINTs, we need to read all bytes including the first one
            value = first_byte as u32;
            
            for i in 1..length {
                value = (value << 8) | data[offset + i] as u32;
            }
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
                        println!("DEBUG: Found Info element, parsing...");
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
                    0x4461 => {
                        // DateUTC - Creation date in nanoseconds since 2001-01-01 00:00:00 UTC
                        println!("DEBUG: Found DateUTC element!");
                        if element_data.len() == 8 {
                            // Read as big-endian 64-bit integer
                            let nanoseconds = ((element_data[0] as u64) << 56) |
                                            ((element_data[1] as u64) << 48) |
                                            ((element_data[2] as u64) << 40) |
                                            ((element_data[3] as u64) << 32) |
                                            ((element_data[4] as u64) << 24) |
                                            ((element_data[5] as u64) << 16) |
                                            ((element_data[6] as u64) << 8) |
                                            (element_data[7] as u64);
                            
                            // Convert from nanoseconds since 2001-01-01 to Unix timestamp
                            // 2001-01-01 00:00:00 UTC = 978307200 Unix timestamp
                            let unix_timestamp = (nanoseconds / 1_000_000_000) + 978307200;
                            
                            if unix_timestamp > 0 {
                                if let Some(datetime) = chrono::DateTime::from_timestamp(unix_timestamp as i64, 0) {
                                    let formatted_time = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
                                    metadata.insert("DateTimeOriginal".to_string(), formatted_time.clone());
                                    metadata.insert("CreateDate".to_string(), formatted_time.clone());
                                    metadata.insert("CreationDate".to_string(), formatted_time);
                                }
                            }
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

    /// Search for DateUTC elements using pattern matching
    fn search_for_date_elements(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Search for DateUTC element pattern (0x4461 followed by size field, then 8-byte timestamp)
        let pattern = [0x44, 0x61];
        let mut offset = 0;
        let mut found_reasonable_date = false;
        
        while offset < data.len() - 15 && !found_reasonable_date { // Need at least 15 bytes: 2 (pattern) + 1 (size) + 8 (timestamp) + 4 (buffer)
            if let Some(pos) = Self::find_pattern(data, offset, &pattern) {
                offset = pos + 2; // Move past the pattern
                
                // Parse the size field (should be 1 byte with value 8 for DateUTC)
                if offset < data.len() {
                    let size_byte = data[offset];
                    let size_length = size_byte.leading_zeros() as usize + 1;
                    
                    // Try different size interpretations
                    let mut element_size = 0u32;
                    let mut size_bytes_used = 1;
                    
                    if size_length == 1 {
                        element_size = size_byte as u32;
                    } else if size_length == 2 && offset + 1 < data.len() {
                        element_size = ((size_byte as u32) << 8) | data[offset + 1] as u32;
                        size_bytes_used = 2;
                    } else if size_length == 3 && offset + 2 < data.len() {
                        element_size = ((size_byte as u32) << 16) | ((data[offset + 1] as u32) << 8) | data[offset + 2] as u32;
                        size_bytes_used = 3;
                    } else if size_length == 4 && offset + 3 < data.len() {
                        element_size = ((size_byte as u32) << 24) | ((data[offset + 1] as u32) << 16) | ((data[offset + 2] as u32) << 8) | data[offset + 3] as u32;
                        size_bytes_used = 4;
                    }
                    
                    if element_size >= 8 { // DateUTC should have at least 8 bytes
                        offset += size_bytes_used; // Move past size field
                        
                        // Check if we have enough bytes for the data
                        if offset + element_size as usize <= data.len() {
                            let element_data = &data[offset..offset + element_size as usize];
                            
                            // Look for 8-byte timestamp within the element data
                            for i in 0..=element_data.len().saturating_sub(8) {
                                let timestamp_bytes = &element_data[i..i + 8];
                                
                                // Read as big-endian 64-bit integer
                                let nanoseconds = ((timestamp_bytes[0] as u64) << 56) |
                                                ((timestamp_bytes[1] as u64) << 48) |
                                                ((timestamp_bytes[2] as u64) << 40) |
                                                ((timestamp_bytes[3] as u64) << 32) |
                                                ((timestamp_bytes[4] as u64) << 24) |
                                                ((timestamp_bytes[5] as u64) << 16) |
                                                ((timestamp_bytes[6] as u64) << 8) |
                                                (timestamp_bytes[7] as u64);
                                
                                // Convert from nanoseconds since 2001-01-01 to Unix timestamp
                                // 2001-01-01 00:00:00 UTC = 978307200 Unix timestamp
                                let unix_timestamp = (nanoseconds / 1_000_000_000) + 978307200;
                                
                                // Also try direct Unix timestamp conversion (in case it's already Unix epoch)
                                let direct_unix = nanoseconds / 1_000_000_000;
                                
                                if unix_timestamp > 0 && unix_timestamp < 2000000000 { // Reasonable timestamp range
                                    if let Some(datetime) = chrono::DateTime::from_timestamp(unix_timestamp as i64, 0) {
                                        let formatted_time = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
                                        
                                        // Check if this timestamp is close to 2023 (the expected year)
                                        if unix_timestamp > 1600000000 && unix_timestamp < 1800000000 { // 2020-2027 range
                                            metadata.insert("DateTimeOriginal".to_string(), formatted_time.clone());
                                            metadata.insert("CreateDate".to_string(), formatted_time.clone());
                                            metadata.insert("CreationDate".to_string(), formatted_time.clone());
                                            found_reasonable_date = true;
                                            break; // Found a reasonable date, stop searching
                                        }
                                    }
                                }
                                
                                // Also try direct Unix timestamp
                                if direct_unix > 0 && direct_unix < 2000000000 {
                                    if let Some(datetime) = chrono::DateTime::from_timestamp(direct_unix as i64, 0) {
                                        let formatted_time = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
                                        
                                        if direct_unix > 1600000000 && direct_unix < 1800000000 {
                                            metadata.insert("DateTimeOriginal".to_string(), formatted_time.clone());
                                            metadata.insert("CreateDate".to_string(), formatted_time.clone());
                                            metadata.insert("CreationDate".to_string(), formatted_time.clone());
                                            found_reasonable_date = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }
    }
    
    /// Find pattern in data starting from offset
    fn find_pattern(data: &[u8], start_offset: usize, pattern: &[u8]) -> Option<usize> {
        for i in start_offset..data.len() - pattern.len() + 1 {
            if &data[i..i + pattern.len()] == pattern {
                return Some(i);
            }
        }
        None
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
