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
            Self::parse_canon_maker_note(data, metadata);
        } else if data[offset..offset + 5].windows(5).any(|w| w == b"Nikon") {
            Self::parse_nikon_maker_note(data, metadata);
        } else if data[offset..offset + 7].windows(7).any(|w| w == b"OLYMPUS") {
            Self::parse_olympus_maker_note(data, metadata);
        } else if data[offset..offset + 5].windows(5).any(|w| w == b"Sony") {
            Self::parse_sony_maker_note(data, metadata);
        } else {
            // Try to detect manufacturer from existing metadata
            if let Some(make) = metadata.get("Make") {
                let make_lower = make.to_lowercase();
                if make_lower.contains("canon") {
                    Self::parse_canon_maker_note(data, metadata);
                } else if make_lower.contains("nikon") {
                    Self::parse_nikon_maker_note(data, metadata);
                } else if make_lower.contains("olympus") {
                    Self::parse_olympus_maker_note(data, metadata);
                } else if make_lower.contains("sony") {
                    Self::parse_sony_maker_note(data, metadata);
                }
            }
        }
    }

    /// Parse Canon maker note
    fn parse_canon_maker_note(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Canon maker note parsing
        metadata.insert("MakerNoteType".to_string(), "Canon".to_string());

        // Look for Canon-specific patterns
        if data.windows(5).any(|w| w == b"Canon") {
            metadata.insert("CanonMakerNote".to_string(), "Detected".to_string());
        }
    }

    /// Parse Nikon maker note
    fn parse_nikon_maker_note(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Nikon maker note parsing
        metadata.insert("MakerNoteType".to_string(), "Nikon".to_string());

        // Look for Nikon-specific patterns
        if data.windows(5).any(|w| w == b"Nikon") {
            metadata.insert("NikonMakerNote".to_string(), "Detected".to_string());
        }
    }

    /// Parse Olympus maker note
    fn parse_olympus_maker_note(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Olympus maker note parsing
        metadata.insert("MakerNoteType".to_string(), "Olympus".to_string());

        // Look for Olympus-specific patterns
        if data.windows(7).any(|w| w == b"OLYMPUS") {
            metadata.insert("OlympusMakerNote".to_string(), "Detected".to_string());
        }
    }

    /// Parse Sony maker note
    fn parse_sony_maker_note(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Sony maker note parsing
        metadata.insert("MakerNoteType".to_string(), "Sony".to_string());

        // Look for Sony-specific patterns
        if data.windows(4).any(|w| w == b"Sony") {
            metadata.insert("SonyMakerNote".to_string(), "Detected".to_string());
        }
    }
}
