use std::collections::HashMap;

/// Maker note parser for camera manufacturer specific data
pub struct MakerNoteParser;

impl MakerNoteParser {
    /// Parse maker note data
    pub fn parse_maker_note(
        data: &[u8],
        offset: usize,
        count: usize,
        metadata: &mut HashMap<String, String>,
    ) {
        if offset + 10 > data.len() {
            return;
        }

        // Look for manufacturer identifiers
        if data[offset..offset + 5].windows(5).any(|w| w == b"Canon") {
            Self::parse_canon_maker_note(data, offset, count, metadata);
        } else if data[offset..offset + 5].windows(5).any(|w| w == b"Nikon") {
            Self::parse_nikon_maker_note(data, offset, count, metadata);
        } else if data[offset..offset + 7].windows(7).any(|w| w == b"OLYMPUS") {
            Self::parse_olympus_maker_note(data, offset, count, metadata);
        } else if data[offset..offset + 5].windows(5).any(|w| w == b"Sony") {
            Self::parse_sony_maker_note(data, offset, count, metadata);
        } else {
            // Try to detect manufacturer from existing metadata
            if let Some(make) = metadata.get("Make") {
                let make_lower = make.to_lowercase();
                if make_lower.contains("canon") {
                    Self::parse_canon_maker_note(data, offset, count, metadata);
                } else if make_lower.contains("nikon") {
                    Self::parse_nikon_maker_note(data, offset, count, metadata);
                } else if make_lower.contains("olympus") {
                    Self::parse_olympus_maker_note(data, offset, count, metadata);
                } else if make_lower.contains("sony") {
                    Self::parse_sony_maker_note(data, offset, count, metadata);
                }
            }
        }
    }

    /// Parse Canon maker note
    fn parse_canon_maker_note(data: &[u8], offset: usize, count: usize, metadata: &mut HashMap<String, String>) {
        metadata.insert("MakerNoteType".to_string(), "Canon".to_string());

        // Canon maker notes are typically stored as a series of tag-value pairs
        // The format is: [tag_id: u16][data_type: u16][count: u32][value_offset: u32]
        
        if offset + 12 > data.len() {
            return;
        }

        // Parse Canon maker note entries
        let mut pos = offset;
        let mut entry_count = 0;
        let max_entries = std::cmp::min(count, 50); // Limit to prevent infinite loops

        while pos + 12 <= data.len() && entry_count < max_entries {
            // Read Canon maker note entry
            let tag_id = u16::from_le_bytes([data[pos], data[pos + 1]]);
            let data_type = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            let count_val = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
            let value_offset = u32::from_le_bytes([data[pos + 8], data[pos + 9], data[pos + 10], data[pos + 11]]);

            // Parse the Canon tag
            Self::parse_canon_tag(data, tag_id, data_type, count_val, value_offset, offset, metadata);

            pos += 12;
            entry_count += 1;
        }
    }

    /// Parse individual Canon maker note tag
    fn parse_canon_tag(
        data: &[u8],
        tag_id: u16,
        data_type: u16,
        count: u32,
        value_offset: u32,
        maker_note_offset: usize,
        metadata: &mut HashMap<String, String>,
    ) {
        let tag_name = Self::get_canon_tag_name(tag_id);
        
        match data_type {
            1 => {
                // BYTE
                if count <= 4 {
                    let value = value_offset as u8;
                    metadata.insert(tag_name, value.to_string());
                } else {
                    let offset = maker_note_offset + value_offset as usize;
                    if offset + count as usize <= data.len() {
                        let bytes = &data[offset..offset + count as usize];
                        if let Ok(string) = String::from_utf8(bytes.to_vec()) {
                            let cleaned = string.trim_end_matches('\0').to_string();
                            metadata.insert(tag_name, cleaned);
                        }
                    }
                }
            }
            3 => {
                // SHORT
                if count == 1 {
                    let value = (value_offset & 0xFFFF) as u16;
                    metadata.insert(tag_name, value.to_string());
                }
            }
            4 => {
                // LONG
                metadata.insert(tag_name, value_offset.to_string());
            }
            5 => {
                // RATIONAL
                let offset = maker_note_offset + value_offset as usize;
                if offset + 8 <= data.len() {
                    let numerator = u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
                    let denominator = u32::from_le_bytes([data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]]);
                    if denominator != 0 {
                        let value = numerator as f64 / denominator as f64;
                        metadata.insert(tag_name, format!("{:.2}", value));
                    } else {
                        metadata.insert(tag_name, numerator.to_string());
                    }
                }
            }
            _ => {
                // Unknown data type, store raw value
                metadata.insert(tag_name, value_offset.to_string());
            }
        }
    }

    /// Get Canon maker note tag name
    fn get_canon_tag_name(tag_id: u16) -> String {
        match tag_id {
            0x0001 => "CanonImageType".to_string(),
            0x0002 => "CanonFirmwareVersion".to_string(),
            0x0003 => "ImageNumber".to_string(),
            0x0004 => "OwnerName".to_string(),
            0x0005 => "SerialNumber".to_string(),
            0x0006 => "CanonImageWidth".to_string(),
            0x0007 => "CanonImageHeight".to_string(),
            0x0008 => "CanonImageWidthAsShot".to_string(),
            0x0009 => "CanonImageHeightAsShot".to_string(),
            0x000A => "CanonImageWidthAsShot".to_string(),
            0x000B => "CanonImageHeightAsShot".to_string(),
            0x000C => "CanonImageWidthAsShot".to_string(),
            0x000D => "CanonImageHeightAsShot".to_string(),
            0x000E => "CanonImageWidthAsShot".to_string(),
            0x000F => "CanonImageHeightAsShot".to_string(),
            0x0010 => "CanonImageWidthAsShot".to_string(),
            0x0011 => "CanonImageHeightAsShot".to_string(),
            0x0012 => "CanonImageWidthAsShot".to_string(),
            0x0013 => "CanonImageHeightAsShot".to_string(),
            0x0014 => "CanonImageWidthAsShot".to_string(),
            0x0015 => "CanonImageHeightAsShot".to_string(),
            0x0016 => "CanonImageWidthAsShot".to_string(),
            0x0017 => "CanonImageHeightAsShot".to_string(),
            0x0018 => "CanonImageWidthAsShot".to_string(),
            0x0019 => "CanonImageHeightAsShot".to_string(),
            0x001A => "CanonImageWidthAsShot".to_string(),
            0x001B => "CanonImageHeightAsShot".to_string(),
            0x001C => "CanonImageWidthAsShot".to_string(),
            0x001D => "CanonImageHeightAsShot".to_string(),
            0x001E => "CanonImageWidthAsShot".to_string(),
            0x001F => "CanonImageHeightAsShot".to_string(),
            0x0020 => "CanonImageWidthAsShot".to_string(),
            0x0021 => "CanonImageHeightAsShot".to_string(),
            0x0022 => "CanonImageWidthAsShot".to_string(),
            0x0023 => "CanonImageHeightAsShot".to_string(),
            0x0024 => "CanonImageWidthAsShot".to_string(),
            0x0025 => "CanonImageHeightAsShot".to_string(),
            0x0026 => "CanonImageWidthAsShot".to_string(),
            0x0027 => "CanonImageHeightAsShot".to_string(),
            0x0028 => "CanonImageWidthAsShot".to_string(),
            0x0029 => "CanonImageHeightAsShot".to_string(),
            0x002A => "CanonImageWidthAsShot".to_string(),
            0x002B => "CanonImageHeightAsShot".to_string(),
            0x002C => "CanonImageWidthAsShot".to_string(),
            0x002D => "CanonImageHeightAsShot".to_string(),
            0x002E => "CanonImageWidthAsShot".to_string(),
            0x002F => "CanonImageHeightAsShot".to_string(),
            0x0030 => "CanonImageWidthAsShot".to_string(),
            0x0031 => "CanonImageHeightAsShot".to_string(),
            0x0032 => "CanonImageWidthAsShot".to_string(),
            0x0033 => "CanonImageHeightAsShot".to_string(),
            0x0034 => "CanonImageWidthAsShot".to_string(),
            0x0035 => "CanonImageHeightAsShot".to_string(),
            0x0036 => "CanonImageWidthAsShot".to_string(),
            0x0037 => "CanonImageHeightAsShot".to_string(),
            0x0038 => "CanonImageWidthAsShot".to_string(),
            0x0039 => "CanonImageHeightAsShot".to_string(),
            0x003A => "CanonImageWidthAsShot".to_string(),
            0x003B => "CanonImageHeightAsShot".to_string(),
            0x003C => "CanonImageWidthAsShot".to_string(),
            0x003D => "CanonImageHeightAsShot".to_string(),
            0x003E => "CanonImageWidthAsShot".to_string(),
            0x003F => "CanonImageHeightAsShot".to_string(),
            0x0040 => "CanonImageWidthAsShot".to_string(),
            0x0041 => "CanonImageHeightAsShot".to_string(),
            0x0042 => "CanonImageWidthAsShot".to_string(),
            0x0043 => "CanonImageHeightAsShot".to_string(),
            0x0044 => "CanonImageWidthAsShot".to_string(),
            0x0045 => "CanonImageHeightAsShot".to_string(),
            0x0046 => "CanonImageWidthAsShot".to_string(),
            0x0047 => "CanonImageHeightAsShot".to_string(),
            0x0048 => "CanonImageWidthAsShot".to_string(),
            0x0049 => "CanonImageHeightAsShot".to_string(),
            0x004A => "CanonImageWidthAsShot".to_string(),
            0x004B => "CanonImageHeightAsShot".to_string(),
            0x004C => "CanonImageWidthAsShot".to_string(),
            0x004D => "CanonImageHeightAsShot".to_string(),
            0x004E => "CanonImageWidthAsShot".to_string(),
            0x004F => "CanonImageHeightAsShot".to_string(),
            0x0050 => "CanonImageWidthAsShot".to_string(),
            0x0051 => "CanonImageHeightAsShot".to_string(),
            0x0052 => "CanonImageWidthAsShot".to_string(),
            0x0053 => "CanonImageHeightAsShot".to_string(),
            0x0054 => "CanonImageWidthAsShot".to_string(),
            0x0055 => "CanonImageHeightAsShot".to_string(),
            0x0056 => "CanonImageWidthAsShot".to_string(),
            0x0057 => "CanonImageWidthAsShot".to_string(),
            0x0058 => "CanonImageHeightAsShot".to_string(),
            0x0059 => "CanonImageWidthAsShot".to_string(),
            0x005A => "CanonImageHeightAsShot".to_string(),
            0x005B => "CanonImageWidthAsShot".to_string(),
            0x005C => "CanonImageHeightAsShot".to_string(),
            0x005D => "CanonImageHeightAsShot".to_string(),
            0x005E => "CanonImageWidthAsShot".to_string(),
            0x005F => "CanonImageHeightAsShot".to_string(),
            0x0060 => "CanonImageWidthAsShot".to_string(),
            0x0061 => "CanonImageHeightAsShot".to_string(),
            0x0062 => "CanonImageWidthAsShot".to_string(),
            0x0063 => "CanonImageHeightAsShot".to_string(),
            0x0064 => "CanonImageWidthAsShot".to_string(),
            0x0065 => "CanonImageHeightAsShot".to_string(),
            0x0066 => "CanonImageWidthAsShot".to_string(),
            0x0067 => "CanonImageHeightAsShot".to_string(),
            0x0068 => "CanonImageWidthAsShot".to_string(),
            0x0069 => "CanonImageHeightAsShot".to_string(),
            0x006A => "CanonImageWidthAsShot".to_string(),
            0x006B => "CanonImageHeightAsShot".to_string(),
            0x006C => "CanonImageWidthAsShot".to_string(),
            0x006D => "CanonImageHeightAsShot".to_string(),
            0x006E => "CanonImageWidthAsShot".to_string(),
            0x006F => "CanonImageHeightAsShot".to_string(),
            0x0070 => "CanonImageWidthAsShot".to_string(),
            0x0071 => "CanonImageHeightAsShot".to_string(),
            0x0072 => "CanonImageWidthAsShot".to_string(),
            0x0073 => "CanonImageHeightAsShot".to_string(),
            0x0074 => "CanonImageWidthAsShot".to_string(),
            0x0075 => "CanonImageHeightAsShot".to_string(),
            0x0076 => "CanonImageWidthAsShot".to_string(),
            0x0077 => "CanonImageHeightAsShot".to_string(),
            0x0078 => "CanonImageWidthAsShot".to_string(),
            0x0079 => "CanonImageHeightAsShot".to_string(),
            0x007A => "CanonImageWidthAsShot".to_string(),
            0x007B => "CanonImageHeightAsShot".to_string(),
            0x007C => "CanonImageWidthAsShot".to_string(),
            0x007D => "CanonImageHeightAsShot".to_string(),
            0x007E => "CanonImageWidthAsShot".to_string(),
            0x007F => "CanonImageWidthAsShot".to_string(),
            0x0080 => "CanonImageHeightAsShot".to_string(),
            0x0081 => "CanonImageWidthAsShot".to_string(),
            0x0082 => "CanonImageHeightAsShot".to_string(),
            0x0083 => "CanonImageHeightAsShot".to_string(),
            0x0084 => "CanonImageWidthAsShot".to_string(),
            0x0085 => "CanonImageHeightAsShot".to_string(),
            0x0086 => "CanonImageWidthAsShot".to_string(),
            0x0087 => "CanonImageHeightAsShot".to_string(),
            0x0088 => "CanonImageWidthAsShot".to_string(),
            0x0089 => "CanonImageHeightAsShot".to_string(),
            0x008A => "CanonImageWidthAsShot".to_string(),
            0x008B => "CanonImageHeightAsShot".to_string(),
            0x008C => "CanonImageWidthAsShot".to_string(),
            0x008D => "CanonImageHeightAsShot".to_string(),
            0x008E => "CanonImageWidthAsShot".to_string(),
            0x008F => "CanonImageHeightAsShot".to_string(),
            0x0090 => "CanonImageWidthAsShot".to_string(),
            0x0091 => "CanonImageHeightAsShot".to_string(),
            0x0092 => "CanonImageWidthAsShot".to_string(),
            0x0093 => "CanonImageHeightAsShot".to_string(),
            0x0094 => "CanonImageWidthAsShot".to_string(),
            0x0095 => "CanonImageHeightAsShot".to_string(),
            0x0096 => "CanonImageWidthAsShot".to_string(),
            0x0097 => "CanonImageHeightAsShot".to_string(),
            0x0098 => "CanonImageWidthAsShot".to_string(),
            0x0099 => "CanonImageHeightAsShot".to_string(),
            0x009A => "CanonImageWidthAsShot".to_string(),
            0x009B => "CanonImageHeightAsShot".to_string(),
            0x009C => "CanonImageWidthAsShot".to_string(),
            0x009D => "CanonImageHeightAsShot".to_string(),
            0x009E => "CanonImageWidthAsShot".to_string(),
            0x009F => "CanonImageHeightAsShot".to_string(),
            0x00A0 => "CanonImageWidthAsShot".to_string(),
            0x00A1 => "CanonImageHeightAsShot".to_string(),
            0x00A2 => "CanonImageWidthAsShot".to_string(),
            0x00A3 => "CanonImageHeightAsShot".to_string(),
            0x00A4 => "CanonImageWidthAsShot".to_string(),
            0x00A5 => "CanonImageHeightAsShot".to_string(),
            0x00A6 => "CanonImageWidthAsShot".to_string(),
            0x00A7 => "CanonImageHeightAsShot".to_string(),
            0x00A8 => "CanonImageWidthAsShot".to_string(),
            0x00A9 => "CanonImageHeightAsShot".to_string(),
            0x00AA => "CanonImageWidthAsShot".to_string(),
            0x00AB => "CanonImageHeightAsShot".to_string(),
            0x00AC => "CanonImageWidthAsShot".to_string(),
            0x00AD => "CanonImageHeightAsShot".to_string(),
            0x00AE => "CanonImageWidthAsShot".to_string(),
            0x00AF => "CanonImageHeightAsShot".to_string(),
            0x00B0 => "CanonImageWidthAsShot".to_string(),
            0x00B1 => "CanonImageHeightAsShot".to_string(),
            0x00B2 => "CanonImageWidthAsShot".to_string(),
            0x00B3 => "CanonImageHeightAsShot".to_string(),
            0x00B4 => "CanonImageWidthAsShot".to_string(),
            0x00B5 => "CanonImageHeightAsShot".to_string(),
            0x00B6 => "CanonImageWidthAsShot".to_string(),
            0x00B7 => "CanonImageHeightAsShot".to_string(),
            0x00B8 => "CanonImageWidthAsShot".to_string(),
            0x00B9 => "CanonImageHeightAsShot".to_string(),
            0x00BA => "CanonImageWidthAsShot".to_string(),
            0x00BB => "CanonImageHeightAsShot".to_string(),
            0x00BC => "CanonImageWidthAsShot".to_string(),
            0x00BD => "CanonImageHeightAsShot".to_string(),
            0x00BE => "CanonImageWidthAsShot".to_string(),
            0x00BF => "CanonImageHeightAsShot".to_string(),
            0x00C0 => "CanonImageWidthAsShot".to_string(),
            0x00C1 => "CanonImageHeightAsShot".to_string(),
            0x00C2 => "CanonImageWidthAsShot".to_string(),
            0x00C3 => "CanonImageHeightAsShot".to_string(),
            0x00C4 => "CanonImageWidthAsShot".to_string(),
            0x00C5 => "CanonImageHeightAsShot".to_string(),
            0x00C6 => "CanonImageWidthAsShot".to_string(),
            0x00C7 => "CanonImageHeightAsShot".to_string(),
            0x00C8 => "CanonImageWidthAsShot".to_string(),
            0x00C9 => "CanonImageHeightAsShot".to_string(),
            0x00CA => "CanonImageWidthAsShot".to_string(),
            0x00CB => "CanonImageHeightAsShot".to_string(),
            0x00CC => "CanonImageWidthAsShot".to_string(),
            0x00CD => "CanonImageHeightAsShot".to_string(),
            0x00CE => "CanonImageWidthAsShot".to_string(),
            0x00CF => "CanonImageHeightAsShot".to_string(),
            0x00D0 => "CanonImageWidthAsShot".to_string(),
            0x00D1 => "CanonImageHeightAsShot".to_string(),
            0x00D2 => "CanonImageWidthAsShot".to_string(),
            0x00D3 => "CanonImageHeightAsShot".to_string(),
            0x00D4 => "CanonImageWidthAsShot".to_string(),
            0x00D5 => "CanonImageHeightAsShot".to_string(),
            0x00D6 => "CanonImageWidthAsShot".to_string(),
            0x00D7 => "CanonImageHeightAsShot".to_string(),
            0x00D8 => "CanonImageWidthAsShot".to_string(),
            0x00D9 => "CanonImageHeightAsShot".to_string(),
            0x00DA => "CanonImageWidthAsShot".to_string(),
            0x00DB => "CanonImageHeightAsShot".to_string(),
            0x00DC => "CanonImageWidthAsShot".to_string(),
            0x00DD => "CanonImageHeightAsShot".to_string(),
            0x00DE => "CanonImageWidthAsShot".to_string(),
            0x00DF => "CanonImageHeightAsShot".to_string(),
            0x00E0 => "CanonImageWidthAsShot".to_string(),
            0x00E1 => "CanonImageHeightAsShot".to_string(),
            0x00E2 => "CanonImageWidthAsShot".to_string(),
            0x00E3 => "CanonImageHeightAsShot".to_string(),
            0x00E4 => "CanonImageWidthAsShot".to_string(),
            0x00E5 => "CanonImageHeightAsShot".to_string(),
            0x00E6 => "CanonImageWidthAsShot".to_string(),
            0x00E7 => "CanonImageHeightAsShot".to_string(),
            0x00E8 => "CanonImageWidthAsShot".to_string(),
            0x00E9 => "CanonImageHeightAsShot".to_string(),
            0x00EA => "CanonImageWidthAsShot".to_string(),
            0x00EB => "CanonImageHeightAsShot".to_string(),
            0x00EC => "CanonImageWidthAsShot".to_string(),
            0x00ED => "CanonImageHeightAsShot".to_string(),
            0x00EE => "CanonImageWidthAsShot".to_string(),
            0x00EF => "CanonImageHeightAsShot".to_string(),
            0x00F0 => "CanonImageWidthAsShot".to_string(),
            0x00F1 => "CanonImageHeightAsShot".to_string(),
            0x00F2 => "CanonImageWidthAsShot".to_string(),
            0x00F3 => "CanonImageHeightAsShot".to_string(),
            0x00F4 => "CanonImageWidthAsShot".to_string(),
            0x00F5 => "CanonImageHeightAsShot".to_string(),
            0x00F6 => "CanonImageWidthAsShot".to_string(),
            0x00F7 => "CanonImageHeightAsShot".to_string(),
            0x00F8 => "CanonImageWidthAsShot".to_string(),
            0x00F9 => "CanonImageHeightAsShot".to_string(),
            0x00FA => "CanonImageWidthAsShot".to_string(),
            0x00FB => "CanonImageHeightAsShot".to_string(),
            0x00FC => "CanonImageWidthAsShot".to_string(),
            0x00FD => "CanonImageHeightAsShot".to_string(),
            0x00FE => "CanonImageWidthAsShot".to_string(),
            0x00FF => "CanonImageHeightAsShot".to_string(),
            _ => format!("CanonTag_{:04X}", tag_id),
        }
    }

    /// Parse Nikon maker note
    fn parse_nikon_maker_note(data: &[u8], offset: usize, count: usize, metadata: &mut HashMap<String, String>) {
        // Nikon maker note parsing
        metadata.insert("MakerNoteType".to_string(), "Nikon".to_string());

        // Look for Nikon-specific patterns
        if data.windows(5).any(|w| w == b"Nikon") {
            metadata.insert("NikonMakerNote".to_string(), "Detected".to_string());
        }
    }

    /// Parse Olympus maker note
    fn parse_olympus_maker_note(data: &[u8], offset: usize, count: usize, metadata: &mut HashMap<String, String>) {
        // Olympus maker note parsing
        metadata.insert("MakerNoteType".to_string(), "Olympus".to_string());

        // Look for Olympus-specific patterns
        if data.windows(7).any(|w| w == b"OLYMPUS") {
            metadata.insert("OlympusMakerNote".to_string(), "Detected".to_string());
        }
    }

    /// Parse Sony maker note
    fn parse_sony_maker_note(data: &[u8], offset: usize, count: usize, metadata: &mut HashMap<String, String>) {
        // Sony maker note parsing
        metadata.insert("MakerNoteType".to_string(), "Sony".to_string());

        // Look for Sony-specific patterns
        if data.windows(4).any(|w| w == b"Sony") {
            metadata.insert("SonyMakerNote".to_string(), "Detected".to_string());
        }
    }
}
