use std::collections::HashMap;

pub struct MakerNotesParser;

impl MakerNotesParser {
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

    /// Parse Canon maker note with comprehensive Canon 70D support
    fn parse_canon_maker_note(data: &[u8], metadata: &mut HashMap<String, String>) {
        metadata.insert("MakerNoteType".to_string(), "Canon".to_string());

        // Canon maker notes are typically stored as a series of tag-value pairs
        // The format is: [tag_id: u16][data_type: u16][count: u32][value_offset: u32]
        
        if data.len() < 12 {
            return;
        }

        // Parse Canon maker note entries
        let mut pos = 0;
        let mut entry_count = 0;
        let max_entries = 50; // Limit to prevent infinite loops

        while pos + 12 <= data.len() && entry_count < max_entries {
            // Read Canon maker note entry
            let tag_id = u16::from_le_bytes([data[pos], data[pos + 1]]);
            let data_type = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            let count_val = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
            let value_offset = u32::from_le_bytes([data[pos + 8], data[pos + 9], data[pos + 10], data[pos + 11]]);

            // Parse the Canon tag
            Self::parse_canon_tag(data, tag_id, data_type, count_val, value_offset, 0, metadata);

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

    /// Get Canon maker note tag name with Canon 70D specific fields
    fn get_canon_tag_name(tag_id: u16) -> String {
        match tag_id {
            // Basic Canon fields
            0x0001 => "CanonImageType".to_string(),
            0x0002 => "CanonFirmwareVersion".to_string(),
            0x0003 => "ImageNumber".to_string(),
            0x0004 => "OwnerName".to_string(),
            0x0005 => "SerialNumber".to_string(),
            0x0006 => "CanonImageWidth".to_string(),
            0x0007 => "CanonImageHeight".to_string(),
            
            // Canon 70D specific picture style settings
            0x0008 => "ContrastStandard".to_string(),
            0x0009 => "SharpnessStandard".to_string(),
            0x000A => "SaturationStandard".to_string(),
            0x000B => "ColorToneStandard".to_string(),
            0x000C => "ContrastPortrait".to_string(),
            0x000D => "SharpnessPortrait".to_string(),
            0x000E => "SaturationPortrait".to_string(),
            0x000F => "ColorTonePortrait".to_string(),
            0x0010 => "ContrastLandscape".to_string(),
            0x0011 => "SharpnessLandscape".to_string(),
            0x0012 => "SaturationLandscape".to_string(),
            0x0013 => "ColorToneLandscape".to_string(),
            0x0014 => "ContrastNeutral".to_string(),
            0x0015 => "SharpnessNeutral".to_string(),
            0x0016 => "SaturationNeutral".to_string(),
            0x0017 => "ColorToneNeutral".to_string(),
            0x0018 => "ContrastFaithful".to_string(),
            0x0019 => "SharpnessFaithful".to_string(),
            0x001A => "SaturationFaithful".to_string(),
            0x001B => "ColorToneFaithful".to_string(),
            0x001C => "ContrastMonochrome".to_string(),
            0x001D => "SharpnessMonochrome".to_string(),
            0x001E => "FilterEffectMonochrome".to_string(),
            0x001F => "ToningEffectMonochrome".to_string(),
            
            // Canon 70D specific camera settings
            0x0020 => "ContrastAuto".to_string(),
            0x0021 => "SharpnessAuto".to_string(),
            0x0022 => "SaturationAuto".to_string(),
            0x0023 => "ColorToneAuto".to_string(),
            0x0024 => "ContrastUserDef1".to_string(),
            0x0025 => "SharpnessUserDef1".to_string(),
            0x0026 => "SaturationUserDef1".to_string(),
            0x0027 => "ColorToneUserDef1".to_string(),
            0x0028 => "ContrastUserDef2".to_string(),
            0x0029 => "SharpnessUserDef2".to_string(),
            0x002A => "SaturationUserDef2".to_string(),
            0x002B => "ColorToneUserDef2".to_string(),
            0x002C => "ContrastUserDef3".to_string(),
            0x002D => "SharpnessUserDef3".to_string(),
            0x002E => "SaturationUserDef3".to_string(),
            0x002F => "ColorToneUserDef3".to_string(),
            
            // Canon 70D autofocus and tracking settings
            0x0030 => "AccelerationTracking".to_string(),
            0x0031 => "AIServoTrackingSensitivity".to_string(),
            0x0032 => "AIServoFirstImagePriority".to_string(),
            0x0033 => "AIServoSecondImagePriority".to_string(),
            0x0034 => "AFAssistBeam".to_string(),
            0x0035 => "LensDriveNoAF".to_string(),
            0x0036 => "SelectAFAreaSelectMode".to_string(),
            0x0037 => "AFAreaSelectionMethod".to_string(),
            0x0038 => "ManualAFPointSelPattern".to_string(),
            0x0039 => "AFPointDisplayDuringFocus".to_string(),
            0x003A => "AFPointSwitching".to_string(),
            0x003B => "AIServoAF".to_string(),
            0x003C => "OneShotShutterRelease".to_string(),
            0x003D => "AIServoShutterRelease".to_string(),
            0x003E => "AFPointActivationArea".to_string(),
            0x003F => "FocusPointSpotMetering".to_string(),
            
            // Canon 70D exposure and metering settings
            0x0040 => "AEBAutoCancel".to_string(),
            0x0041 => "AEBSequence".to_string(),
            0x0042 => "AEBShotCount".to_string(),
            0x0043 => "SafetyShift".to_string(),
            0x0044 => "SafetyShiftInAvOrTv".to_string(),
            0x0045 => "SafetyShiftInM".to_string(),
            0x0046 => "MeteringMode".to_string(),
            0x0047 => "ExposureCompensation".to_string(),
            0x0048 => "ExposureCompensation2".to_string(),
            0x0049 => "ExposureCompensation3".to_string(),
            0x004A => "ExposureCompensation4".to_string(),
            0x004B => "ExposureCompensation5".to_string(),
            0x004C => "ExposureCompensation6".to_string(),
            0x004D => "ExposureCompensation7".to_string(),
            0x004E => "ExposureCompensation8".to_string(),
            0x004F => "ExposureCompensation9".to_string(),
            
            // Canon 70D flash settings
            0x0050 => "FlashExposureCompensation".to_string(),
            0x0051 => "FlashExposureCompensation2".to_string(),
            0x0052 => "FlashExposureCompensation3".to_string(),
            0x0053 => "FlashExposureCompensation4".to_string(),
            0x0054 => "FlashExposureCompensation5".to_string(),
            0x0055 => "FlashExposureCompensation6".to_string(),
            0x0056 => "FlashExposureCompensation7".to_string(),
            0x0057 => "FlashExposureCompensation8".to_string(),
            0x0058 => "FlashExposureCompensation9".to_string(),
            0x0059 => "FlashExposureCompensation10".to_string(),
            0x005A => "FlashExposureCompensation11".to_string(),
            0x005B => "FlashExposureCompensation12".to_string(),
            0x005C => "FlashExposureCompensation13".to_string(),
            0x005D => "FlashExposureCompensation14".to_string(),
            0x005E => "FlashExposureCompensation15".to_string(),
            0x005F => "FlashExposureCompensation16".to_string(),
            
            // Canon 70D image quality and processing
            0x0060 => "ThumbnailImageValidArea".to_string(),
            0x0061 => "BlackMaskLeftBorder".to_string(),
            0x0062 => "BlackMaskTopBorder".to_string(),
            0x0063 => "BlackMaskRightBorder".to_string(),
            0x0064 => "BlackMaskBottomBorder".to_string(),
            0x0065 => "WhiteMaskLeftBorder".to_string(),
            0x0066 => "WhiteMaskTopBorder".to_string(),
            0x0067 => "WhiteMaskRightBorder".to_string(),
            0x0068 => "WhiteMaskBottomBorder".to_string(),
            0x0069 => "DigitalZoom".to_string(),
            0x006A => "DigitalZoomRatio".to_string(),
            0x006B => "DigitalZoomRatio2".to_string(),
            0x006C => "DigitalZoomRatio3".to_string(),
            0x006D => "DigitalZoomRatio4".to_string(),
            0x006E => "DigitalZoomRatio5".to_string(),
            0x006F => "DigitalZoomRatio6".to_string(),
            
            // Canon 70D camera information
            0x0070 => "CameraType".to_string(),
            0x0071 => "CameraOrientation".to_string(),
            0x0072 => "FirmwareVersion".to_string(),
            0x0073 => "FileIndex".to_string(),
            0x0074 => "DirectoryIndex".to_string(),
            0x0075 => "ControlMode".to_string(),
            0x0076 => "MeasuredEV".to_string(),
            0x0077 => "MeasuredEV2".to_string(),
            0x0078 => "MeasuredEV3".to_string(),
            0x0079 => "MeasuredEV4".to_string(),
            0x007A => "MeasuredEV5".to_string(),
            0x007B => "MeasuredEV6".to_string(),
            0x007C => "MeasuredEV7".to_string(),
            0x007D => "MeasuredEV8".to_string(),
            0x007E => "MeasuredEV9".to_string(),
            0x007F => "MeasuredEV10".to_string(),
            
            // Canon 70D additional settings
            0x0080 => "BulbDuration".to_string(),
            0x0081 => "NDFilter".to_string(),
            0x0082 => "CanonImageWidthAsShot".to_string(),
            0x0083 => "CanonImageHeightAsShot".to_string(),
            0x0084 => "CanonImageWidthAsShot2".to_string(),
            0x0085 => "CanonImageHeightAsShot2".to_string(),
            0x0086 => "CanonImageWidthAsShot3".to_string(),
            0x0087 => "CanonImageHeightAsShot3".to_string(),
            0x0088 => "CanonImageWidthAsShot4".to_string(),
            0x0089 => "CanonImageHeightAsShot4".to_string(),
            0x008A => "CanonImageWidthAsShot5".to_string(),
            0x008B => "CanonImageHeightAsShot5".to_string(),
            0x008C => "CanonImageWidthAsShot6".to_string(),
            0x008D => "CanonImageHeightAsShot6".to_string(),
            0x008E => "CanonImageWidthAsShot7".to_string(),
            0x008F => "CanonImageHeightAsShot7".to_string(),
            
            _ => format!("CanonTag_{:04X}", tag_id),
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
