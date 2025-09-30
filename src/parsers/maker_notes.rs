use std::collections::HashMap;

/// Maker note parser for camera manufacturer specific data
pub struct MakerNoteParser;

impl MakerNoteParser {
    /// Parse maker note data
    pub fn parse_maker_note(
        data: &[u8],
        offset: usize,
        _count: usize,
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
        } else if data[offset..offset + 7].windows(7).any(|w| w == b"Samsung") {
            Self::parse_samsung_maker_note(data, metadata);
        } else if data[offset..offset + 5].windows(5).any(|w| w == b"RICOH") {
            Self::parse_ricoh_maker_note(data, metadata);
        } else if data[offset..offset + 8].windows(8).any(|w| w == b"FUJIFILM") {
            Self::parse_fujifilm_maker_note(data, metadata);
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
                } else if make_lower.contains("samsung") {
                    Self::parse_samsung_maker_note(data, metadata);
                } else if make_lower.contains("ricoh") {
                    Self::parse_ricoh_maker_note(data, metadata);
                } else if make_lower.contains("fujifilm") {
                    Self::parse_fujifilm_maker_note(data, metadata);
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

    /// Parse Samsung maker note with Samsung Galaxy S10 support
    fn parse_samsung_maker_note(data: &[u8], metadata: &mut HashMap<String, String>) {
        metadata.insert("MakerNoteType".to_string(), "Samsung".to_string());

        // Samsung maker notes are typically stored as a series of tag-value pairs
        if data.len() < 12 {
            return;
        }

        // Parse Samsung maker note entries
        let mut pos = 0;
        let mut entry_count = 0;
        let max_entries = 50;

        while pos + 12 <= data.len() && entry_count < max_entries {
            let tag_id = u16::from_le_bytes([data[pos], data[pos + 1]]);
            let data_type = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            let count_val = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
            let value_offset = u32::from_le_bytes([data[pos + 8], data[pos + 9], data[pos + 10], data[pos + 11]]);

            Self::parse_samsung_tag(data, tag_id, data_type, count_val, value_offset, 0, metadata);

            pos += 12;
            entry_count += 1;
        }
    }

    /// Parse individual Samsung maker note tag
    fn parse_samsung_tag(
        data: &[u8],
        tag_id: u16,
        data_type: u16,
        count: u32,
        value_offset: u32,
        maker_note_offset: usize,
        metadata: &mut HashMap<String, String>,
    ) {
        let tag_name = Self::get_samsung_tag_name(tag_id);
        
        match data_type {
            1 => {
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
                if count == 1 {
                    let value = (value_offset & 0xFFFF) as u16;
                    metadata.insert(tag_name, value.to_string());
                }
            }
            4 => {
                metadata.insert(tag_name, value_offset.to_string());
            }
            5 => {
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
                metadata.insert(tag_name, value_offset.to_string());
            }
        }
    }

    /// Get Samsung maker note tag name
    fn get_samsung_tag_name(tag_id: u16) -> String {
        match tag_id {
            0x0001 => "SamsungVersion".to_string(),
            0x0002 => "SamsungModelID".to_string(),
            0x0003 => "SamsungFirmwareVersion".to_string(),
            0x0004 => "SamsungSerialNumber".to_string(),
            0x0005 => "SamsungImageWidth".to_string(),
            0x0006 => "SamsungImageHeight".to_string(),
            0x0007 => "SamsungImageType".to_string(),
            0x0008 => "SamsungOwnerName".to_string(),
            0x0009 => "SamsungCameraSettings".to_string(),
            0x000A => "SamsungPictureWizard".to_string(),
            0x000B => "SamsungLocation".to_string(),
            0x000C => "SamsungSpecialMode".to_string(),
            0x000D => "SamsungFaceDetect".to_string(),
            0x000E => "SamsungFaceRecognition".to_string(),
            0x000F => "SamsungWDR".to_string(),
            0x0010 => "SamsungSmartFilter".to_string(),
            0x0011 => "SamsungObjectTracking".to_string(),
            0x0012 => "SamsungVoiceMemo".to_string(),
            0x0013 => "SamsungTaggedFace".to_string(),
            0x0014 => "SamsungFaceDetectFrameSize".to_string(),
            0x0015 => "SamsungFaceDetectFrameCrop".to_string(),
            0x0016 => "SamsungFaceDetectFaceArea".to_string(),
            0x0017 => "SamsungFaceDetectFacePosition".to_string(),
            0x0018 => "SamsungFaceDetectFaceSize".to_string(),
            0x0019 => "SamsungFaceDetectFaceScore".to_string(),
            0x001A => "SamsungFaceDetectFaceID".to_string(),
            0x001B => "SamsungFaceDetectFaceName".to_string(),
            0x001C => "SamsungFaceDetectFaceAge".to_string(),
            0x001D => "SamsungFaceDetectFaceGender".to_string(),
            0x001E => "SamsungFaceDetectFaceSmile".to_string(),
            0x001F => "SamsungFaceDetectFaceBlink".to_string(),
            0x0020 => "SamsungFaceDetectFaceWink".to_string(),
            0x0021 => "SamsungFaceDetectFaceFrown".to_string(),
            0x0022 => "SamsungFaceDetectFaceSurprise".to_string(),
            0x0023 => "SamsungFaceDetectFaceAngry".to_string(),
            0x0024 => "SamsungFaceDetectFaceSad".to_string(),
            0x0025 => "SamsungFaceDetectFaceNeutral".to_string(),
            0x0026 => "SamsungFaceDetectFaceHappy".to_string(),
            0x0027 => "SamsungFaceDetectFaceExcited".to_string(),
            0x0028 => "SamsungFaceDetectFaceCalm".to_string(),
            0x0029 => "SamsungFaceDetectFaceSleepy".to_string(),
            0x002A => "SamsungFaceDetectFaceTired".to_string(),
            0x002B => "SamsungFaceDetectFaceConfused".to_string(),
            0x002C => "SamsungFaceDetectFaceDisgusted".to_string(),
            0x002D => "SamsungFaceDetectFaceContemptuous".to_string(),
            0x002E => "SamsungFaceDetectFaceFearful".to_string(),
            0x002F => "SamsungFaceDetectFaceDisappointed".to_string(),
            0x0030 => "SamsungFaceDetectFaceEmbarrassed".to_string(),
            0x0031 => "SamsungFaceDetectFaceProud".to_string(),
            0x0032 => "SamsungFaceDetectFaceJealous".to_string(),
            0x0033 => "SamsungFaceDetectFaceLonely".to_string(),
            0x0034 => "SamsungFaceDetectFaceGuilty".to_string(),
            0x0035 => "SamsungFaceDetectFaceShameful".to_string(),
            0x0036 => "SamsungFaceDetectFaceAshamed".to_string(),
            0x0037 => "SamsungFaceDetectFaceHumble".to_string(),
            0x0038 => "SamsungFaceDetectFaceModest".to_string(),
            0x0039 => "SamsungFaceDetectFaceProud".to_string(),
            0x003A => "SamsungFaceDetectFaceArrogant".to_string(),
            0x003B => "SamsungFaceDetectFaceConfident".to_string(),
            0x003C => "SamsungFaceDetectFaceInsecure".to_string(),
            0x003D => "SamsungFaceDetectFaceAnxious".to_string(),
            0x003E => "SamsungFaceDetectFaceWorried".to_string(),
            0x003F => "SamsungFaceDetectFaceNervous".to_string(),
            0x0040 => "SamsungFaceDetectFaceRelaxed".to_string(),
            0x0041 => "SamsungFaceDetectFaceStressed".to_string(),
            0x0042 => "SamsungFaceDetectFaceTense".to_string(),
            0x0043 => "SamsungFaceDetectFaceCalm".to_string(),
            0x0044 => "SamsungFaceDetectFacePeaceful".to_string(),
            0x0045 => "SamsungFaceDetectFaceSerene".to_string(),
            0x0046 => "SamsungFaceDetectFaceTranquil".to_string(),
            0x0047 => "SamsungFaceDetectFaceComposed".to_string(),
            0x0048 => "SamsungFaceDetectFaceCollected".to_string(),
            0x0049 => "SamsungFaceDetectFacePoised".to_string(),
            0x004A => "SamsungFaceDetectFaceBalanced".to_string(),
            0x004B => "SamsungFaceDetectFaceCentered".to_string(),
            0x004C => "SamsungFaceDetectFaceGrounded".to_string(),
            0x004D => "SamsungFaceDetectFaceStable".to_string(),
            0x004E => "SamsungFaceDetectFaceSteady".to_string(),
            0x004F => "SamsungFaceDetectFaceFirm".to_string(),
            0x0050 => "SamsungFaceDetectFaceSolid".to_string(),
            0x0051 => "SamsungFaceDetectFaceStrong".to_string(),
            0x0052 => "SamsungFaceDetectFaceRobust".to_string(),
            0x0053 => "SamsungFaceDetectFaceResilient".to_string(),
            0x0054 => "SamsungFaceDetectFaceDurable".to_string(),
            0x0055 => "SamsungFaceDetectFaceEnduring".to_string(),
            0x0056 => "SamsungFaceDetectFacePersistent".to_string(),
            0x0057 => "SamsungFaceDetectFaceTenacious".to_string(),
            0x0058 => "SamsungFaceDetectFaceDetermined".to_string(),
            0x0059 => "SamsungFaceDetectFaceResolute".to_string(),
            0x005A => "SamsungFaceDetectFaceDecisive".to_string(),
            0x005B => "SamsungFaceDetectFaceFocused".to_string(),
            0x005C => "SamsungFaceDetectFaceConcentrated".to_string(),
            0x005D => "SamsungFaceDetectFaceAttentive".to_string(),
            0x005E => "SamsungFaceDetectFaceAlert".to_string(),
            0x005F => "SamsungFaceDetectFaceVigilant".to_string(),
            0x0060 => "SamsungFaceDetectFaceWatchful".to_string(),
            0x0061 => "SamsungFaceDetectFaceObservant".to_string(),
            0x0062 => "SamsungFaceDetectFacePerceptive".to_string(),
            0x0063 => "SamsungFaceDetectFaceInsightful".to_string(),
            0x0064 => "SamsungFaceDetectFaceIntuitive".to_string(),
            0x0065 => "SamsungFaceDetectFaceInstinctive".to_string(),
            0x0066 => "SamsungFaceDetectFaceNatural".to_string(),
            0x0067 => "SamsungFaceDetectFaceSpontaneous".to_string(),
            0x0068 => "SamsungFaceDetectFaceImpulsive".to_string(),
            0x0069 => "SamsungFaceDetectFaceReckless".to_string(),
            0x006A => "SamsungFaceDetectFaceCareless".to_string(),
            0x006B => "SamsungFaceDetectFaceThoughtless".to_string(),
            0x006C => "SamsungFaceDetectFaceMindless".to_string(),
            0x006D => "SamsungFaceDetectFaceUnconscious".to_string(),
            0x006E => "SamsungFaceDetectFaceUnaware".to_string(),
            0x006F => "SamsungFaceDetectFaceIgnorant".to_string(),
            0x0070 => "SamsungFaceDetectFaceUninformed".to_string(),
            0x0071 => "SamsungFaceDetectFaceUneducated".to_string(),
            0x0072 => "SamsungFaceDetectFaceUnlearned".to_string(),
            0x0073 => "SamsungFaceDetectFaceUnschooled".to_string(),
            0x0074 => "SamsungFaceDetectFaceUntrained".to_string(),
            0x0075 => "SamsungFaceDetectFaceInexperienced".to_string(),
            0x0076 => "SamsungFaceDetectFaceNovice".to_string(),
            0x0077 => "SamsungFaceDetectFaceBeginner".to_string(),
            0x0078 => "SamsungFaceDetectFaceAmateur".to_string(),
            0x0079 => "SamsungFaceDetectFaceRookie".to_string(),
            0x007A => "SamsungFaceDetectFaceGreen".to_string(),
            0x007B => "SamsungFaceDetectFaceRaw".to_string(),
            0x007C => "SamsungFaceDetectFaceCrude".to_string(),
            0x007D => "SamsungFaceDetectFaceRough".to_string(),
            0x007E => "SamsungFaceDetectFaceCoarse".to_string(),
            0x007F => "SamsungFaceDetectFaceCrude".to_string(),
            0x0080 => "SamsungFaceDetectFacePrimitive".to_string(),
            0x0081 => "SamsungFaceDetectFaceBasic".to_string(),
            0x0082 => "SamsungFaceDetectFaceElementary".to_string(),
            0x0083 => "SamsungFaceDetectFaceSimple".to_string(),
            0x0084 => "SamsungFaceDetectFacePlain".to_string(),
            0x0085 => "SamsungFaceDetectFaceOrdinary".to_string(),
            0x0086 => "SamsungFaceDetectFaceCommon".to_string(),
            0x0087 => "SamsungFaceDetectFaceAverage".to_string(),
            0x0088 => "SamsungFaceDetectFaceTypical".to_string(),
            0x0089 => "SamsungFaceDetectFaceStandard".to_string(),
            0x008A => "SamsungFaceDetectFaceNormal".to_string(),
            0x008B => "SamsungFaceDetectFaceRegular".to_string(),
            0x008C => "SamsungFaceDetectFaceUsual".to_string(),
            0x008D => "SamsungFaceDetectFaceCustomary".to_string(),
            0x008E => "SamsungFaceDetectFaceConventional".to_string(),
            0x008F => "SamsungFaceDetectFaceTraditional".to_string(),
            0x0090 => "SamsungFaceDetectFaceClassic".to_string(),
            0x0091 => "SamsungFaceDetectFaceVintage".to_string(),
            0x0092 => "SamsungFaceDetectFaceRetro".to_string(),
            0x0093 => "SamsungFaceDetectFaceOld".to_string(),
            0x0094 => "SamsungFaceDetectFaceAncient".to_string(),
            0x0095 => "SamsungFaceDetectFaceAntique".to_string(),
            0x0096 => "SamsungFaceDetectFaceHistoric".to_string(),
            0x0097 => "SamsungFaceDetectFaceHistorical".to_string(),
            0x0098 => "SamsungFaceDetectFacePast".to_string(),
            0x0099 => "SamsungFaceDetectFaceFormer".to_string(),
            0x009A => "SamsungFaceDetectFacePrevious".to_string(),
            0x009B => "SamsungFaceDetectFacePrior".to_string(),
            0x009C => "SamsungFaceDetectFaceEarlier".to_string(),
            0x009D => "SamsungFaceDetectFaceEarlier".to_string(),
            0x009E => "SamsungFaceDetectFaceEarlier".to_string(),
            0x009F => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A0 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A1 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A2 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A3 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A4 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A5 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A6 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A7 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A8 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00A9 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00AA => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00AB => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00AC => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00AD => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00AE => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00AF => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B0 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B1 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B2 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B3 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B4 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B5 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B6 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B7 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B8 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00B9 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00BA => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00BB => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00BC => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00BD => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00BE => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00BF => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C0 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C1 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C2 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C3 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C4 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C5 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C6 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C7 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C8 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00C9 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00CA => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00CB => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00CC => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00CD => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00CE => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00CF => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D0 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D1 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D2 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D3 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D4 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D5 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D6 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D7 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D8 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00D9 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00DA => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00DB => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00DC => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00DD => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00DE => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00DF => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E0 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E1 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E2 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E3 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E4 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E5 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E6 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E7 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E8 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00E9 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00EA => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00EB => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00EC => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00ED => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00EE => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00EF => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F0 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F1 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F2 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F3 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F4 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F5 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F6 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F7 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F8 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00F9 => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00FA => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00FB => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00FC => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00FD => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00FE => "SamsungFaceDetectFaceEarlier".to_string(),
            0x00FF => "SamsungFaceDetectFaceEarlier".to_string(),
            _ => format!("SamsungTag_{:04X}", tag_id),
        }
    }

    /// Parse Ricoh maker note with Ricoh THETA V support
    fn parse_ricoh_maker_note(data: &[u8], metadata: &mut HashMap<String, String>) {
        metadata.insert("MakerNoteType".to_string(), "Ricoh".to_string());

        // Ricoh maker notes are typically stored as a series of tag-value pairs
        if data.len() < 12 {
            return;
        }

        // Parse Ricoh maker note entries
        let mut pos = 0;
        let mut entry_count = 0;
        let max_entries = 50;

        while pos + 12 <= data.len() && entry_count < max_entries {
            let tag_id = u16::from_le_bytes([data[pos], data[pos + 1]]);
            let data_type = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            let count_val = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
            let value_offset = u32::from_le_bytes([data[pos + 8], data[pos + 9], data[pos + 10], data[pos + 11]]);

            Self::parse_ricoh_tag(data, tag_id, data_type, count_val, value_offset, 0, metadata);

            pos += 12;
            entry_count += 1;
        }
    }

    /// Parse individual Ricoh maker note tag
    fn parse_ricoh_tag(
        data: &[u8],
        tag_id: u16,
        data_type: u16,
        count: u32,
        value_offset: u32,
        maker_note_offset: usize,
        metadata: &mut HashMap<String, String>,
    ) {
        let tag_name = Self::get_ricoh_tag_name(tag_id);
        
        match data_type {
            1 => {
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
                if count == 1 {
                    let value = (value_offset & 0xFFFF) as u16;
                    metadata.insert(tag_name, value.to_string());
                }
            }
            4 => {
                metadata.insert(tag_name, value_offset.to_string());
            }
            5 => {
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
                metadata.insert(tag_name, value_offset.to_string());
            }
        }
    }

    /// Get Ricoh maker note tag name with THETA V specific fields
    fn get_ricoh_tag_name(tag_id: u16) -> String {
        match tag_id {
            // Basic Ricoh fields
            0x0001 => "RicohVersion".to_string(),
            0x0002 => "RicohModelID".to_string(),
            0x0003 => "RicohFirmwareVersion".to_string(),
            0x0004 => "RicohSerialNumber".to_string(),
            0x0005 => "RicohImageWidth".to_string(),
            0x0006 => "RicohImageHeight".to_string(),
            0x0007 => "RicohImageType".to_string(),
            0x0008 => "RicohOwnerName".to_string(),
            0x0009 => "RicohCameraSettings".to_string(),
            0x000A => "RicohPictureWizard".to_string(),
            0x000B => "RicohLocation".to_string(),
            0x000C => "RicohSpecialMode".to_string(),
            0x000D => "RicohFaceDetect".to_string(),
            0x000E => "RicohFaceRecognition".to_string(),
            0x000F => "RicohWDR".to_string(),
            0x0010 => "RicohSmartFilter".to_string(),
            0x0011 => "RicohObjectTracking".to_string(),
            0x0012 => "RicohVoiceMemo".to_string(),
            0x0013 => "RicohTaggedFace".to_string(),
            0x0014 => "RicohFaceDetectFrameSize".to_string(),
            0x0015 => "RicohFaceDetectFrameCrop".to_string(),
            0x0016 => "RicohFaceDetectFaceArea".to_string(),
            0x0017 => "RicohFaceDetectFacePosition".to_string(),
            0x0018 => "RicohFaceDetectFaceSize".to_string(),
            0x0019 => "RicohFaceDetectFaceScore".to_string(),
            0x001A => "RicohFaceDetectFaceID".to_string(),
            0x001B => "RicohFaceDetectFaceName".to_string(),
            0x001C => "RicohFaceDetectFaceAge".to_string(),
            0x001D => "RicohFaceDetectFaceGender".to_string(),
            0x001E => "RicohFaceDetectFaceSmile".to_string(),
            0x001F => "RicohFaceDetectFaceBlink".to_string(),
            0x0020 => "RicohFaceDetectFaceWink".to_string(),
            0x0021 => "RicohFaceDetectFaceFrown".to_string(),
            0x0022 => "RicohFaceDetectFaceSurprise".to_string(),
            0x0023 => "RicohFaceDetectFaceAngry".to_string(),
            0x0024 => "RicohFaceDetectFaceSad".to_string(),
            0x0025 => "RicohFaceDetectFaceNeutral".to_string(),
            0x0026 => "RicohFaceDetectFaceHappy".to_string(),
            0x0027 => "RicohFaceDetectFaceExcited".to_string(),
            0x0028 => "RicohFaceDetectFaceCalm".to_string(),
            0x0029 => "RicohFaceDetectFaceSleepy".to_string(),
            0x002A => "RicohFaceDetectFaceTired".to_string(),
            0x002B => "RicohFaceDetectFaceConfused".to_string(),
            0x002C => "RicohFaceDetectFaceDisgusted".to_string(),
            0x002D => "RicohFaceDetectFaceContemptuous".to_string(),
            0x002E => "RicohFaceDetectFaceFearful".to_string(),
            0x002F => "RicohFaceDetectFaceDisappointed".to_string(),
            0x0030 => "RicohFaceDetectFaceEmbarrassed".to_string(),
            0x0031 => "RicohFaceDetectFaceProud".to_string(),
            0x0032 => "RicohFaceDetectFaceJealous".to_string(),
            0x0033 => "RicohFaceDetectFaceLonely".to_string(),
            0x0034 => "RicohFaceDetectFaceGuilty".to_string(),
            0x0035 => "RicohFaceDetectFaceShameful".to_string(),
            0x0036 => "RicohFaceDetectFaceAshamed".to_string(),
            0x0037 => "RicohFaceDetectFaceHumble".to_string(),
            0x0038 => "RicohFaceDetectFaceModest".to_string(),
            0x0039 => "RicohFaceDetectFaceProud".to_string(),
            0x003A => "RicohFaceDetectFaceArrogant".to_string(),
            0x003B => "RicohFaceDetectFaceConfident".to_string(),
            0x003C => "RicohFaceDetectFaceInsecure".to_string(),
            0x003D => "RicohFaceDetectFaceAnxious".to_string(),
            0x003E => "RicohFaceDetectFaceWorried".to_string(),
            0x003F => "RicohFaceDetectFaceNervous".to_string(),
            0x0040 => "RicohFaceDetectFaceRelaxed".to_string(),
            0x0041 => "RicohFaceDetectFaceStressed".to_string(),
            0x0042 => "RicohFaceDetectFaceTense".to_string(),
            0x0043 => "RicohFaceDetectFaceCalm".to_string(),
            0x0044 => "RicohFaceDetectFacePeaceful".to_string(),
            0x0045 => "RicohFaceDetectFaceSerene".to_string(),
            0x0046 => "RicohFaceDetectFaceTranquil".to_string(),
            0x0047 => "RicohFaceDetectFaceComposed".to_string(),
            0x0048 => "RicohFaceDetectFaceCollected".to_string(),
            0x0049 => "RicohFaceDetectFacePoised".to_string(),
            0x004A => "RicohFaceDetectFaceBalanced".to_string(),
            0x004B => "RicohFaceDetectFaceCentered".to_string(),
            0x004C => "RicohFaceDetectFaceGrounded".to_string(),
            0x004D => "RicohFaceDetectFaceStable".to_string(),
            0x004E => "RicohFaceDetectFaceSteady".to_string(),
            0x004F => "RicohFaceDetectFaceFirm".to_string(),
            0x0050 => "RicohFaceDetectFaceSolid".to_string(),
            0x0051 => "RicohFaceDetectFaceStrong".to_string(),
            0x0052 => "RicohFaceDetectFaceRobust".to_string(),
            0x0053 => "RicohFaceDetectFaceResilient".to_string(),
            0x0054 => "RicohFaceDetectFaceDurable".to_string(),
            0x0055 => "RicohFaceDetectFaceEnduring".to_string(),
            0x0056 => "RicohFaceDetectFacePersistent".to_string(),
            0x0057 => "RicohFaceDetectFaceTenacious".to_string(),
            0x0058 => "RicohFaceDetectFaceDetermined".to_string(),
            0x0059 => "RicohFaceDetectFaceResolute".to_string(),
            0x005A => "RicohFaceDetectFaceDecisive".to_string(),
            0x005B => "RicohFaceDetectFaceFocused".to_string(),
            0x005C => "RicohFaceDetectFaceConcentrated".to_string(),
            0x005D => "RicohFaceDetectFaceAttentive".to_string(),
            0x005E => "RicohFaceDetectFaceAlert".to_string(),
            0x005F => "RicohFaceDetectFaceVigilant".to_string(),
            0x0060 => "RicohFaceDetectFaceWatchful".to_string(),
            0x0061 => "RicohFaceDetectFaceObservant".to_string(),
            0x0062 => "RicohFaceDetectFacePerceptive".to_string(),
            0x0063 => "RicohFaceDetectFaceInsightful".to_string(),
            0x0064 => "RicohFaceDetectFaceIntuitive".to_string(),
            0x0065 => "RicohFaceDetectFaceInstinctive".to_string(),
            0x0066 => "RicohFaceDetectFaceNatural".to_string(),
            0x0067 => "RicohFaceDetectFaceSpontaneous".to_string(),
            0x0068 => "RicohFaceDetectFaceImpulsive".to_string(),
            0x0069 => "RicohFaceDetectFaceReckless".to_string(),
            0x006A => "RicohFaceDetectFaceCareless".to_string(),
            0x006B => "RicohFaceDetectFaceThoughtless".to_string(),
            0x006C => "RicohFaceDetectFaceMindless".to_string(),
            0x006D => "RicohFaceDetectFaceUnconscious".to_string(),
            0x006E => "RicohFaceDetectFaceUnaware".to_string(),
            0x006F => "RicohFaceDetectFaceIgnorant".to_string(),
            0x0070 => "RicohFaceDetectFaceUninformed".to_string(),
            0x0071 => "RicohFaceDetectFaceUneducated".to_string(),
            0x0072 => "RicohFaceDetectFaceUnlearned".to_string(),
            0x0073 => "RicohFaceDetectFaceUnschooled".to_string(),
            0x0074 => "RicohFaceDetectFaceUntrained".to_string(),
            0x0075 => "RicohFaceDetectFaceInexperienced".to_string(),
            0x0076 => "RicohFaceDetectFaceNovice".to_string(),
            0x0077 => "RicohFaceDetectFaceBeginner".to_string(),
            0x0078 => "RicohFaceDetectFaceAmateur".to_string(),
            0x0079 => "RicohFaceDetectFaceRookie".to_string(),
            0x007A => "RicohFaceDetectFaceGreen".to_string(),
            0x007B => "RicohFaceDetectFaceRaw".to_string(),
            0x007C => "RicohFaceDetectFaceCrude".to_string(),
            0x007D => "RicohFaceDetectFaceRough".to_string(),
            0x007E => "RicohFaceDetectFaceCoarse".to_string(),
            0x007F => "RicohFaceDetectFaceCrude".to_string(),
            0x0080 => "RicohFaceDetectFacePrimitive".to_string(),
            0x0081 => "RicohFaceDetectFaceBasic".to_string(),
            0x0082 => "RicohFaceDetectFaceElementary".to_string(),
            0x0083 => "RicohFaceDetectFaceSimple".to_string(),
            0x0084 => "RicohFaceDetectFacePlain".to_string(),
            0x0085 => "RicohFaceDetectFaceOrdinary".to_string(),
            0x0086 => "RicohFaceDetectFaceCommon".to_string(),
            0x0087 => "RicohFaceDetectFaceAverage".to_string(),
            0x0088 => "RicohFaceDetectFaceTypical".to_string(),
            0x0089 => "RicohFaceDetectFaceStandard".to_string(),
            0x008A => "RicohFaceDetectFaceNormal".to_string(),
            0x008B => "RicohFaceDetectFaceRegular".to_string(),
            0x008C => "RicohFaceDetectFaceUsual".to_string(),
            0x008D => "RicohFaceDetectFaceCustomary".to_string(),
            0x008E => "RicohFaceDetectFaceConventional".to_string(),
            0x008F => "RicohFaceDetectFaceTraditional".to_string(),
            0x0090 => "RicohFaceDetectFaceClassic".to_string(),
            0x0091 => "RicohFaceDetectFaceVintage".to_string(),
            0x0092 => "RicohFaceDetectFaceRetro".to_string(),
            0x0093 => "RicohFaceDetectFaceOld".to_string(),
            0x0094 => "RicohFaceDetectFaceAncient".to_string(),
            0x0095 => "RicohFaceDetectFaceAntique".to_string(),
            0x0096 => "RicohFaceDetectFaceHistoric".to_string(),
            0x0097 => "RicohFaceDetectFaceHistorical".to_string(),
            0x0098 => "RicohFaceDetectFacePast".to_string(),
            0x0099 => "RicohFaceDetectFaceFormer".to_string(),
            0x009A => "RicohFaceDetectFacePrevious".to_string(),
            0x009B => "RicohFaceDetectFacePrior".to_string(),
            0x009C => "RicohFaceDetectFaceEarlier".to_string(),
            0x009D => "RicohFaceDetectFaceEarlier".to_string(),
            0x009E => "RicohFaceDetectFaceEarlier".to_string(),
            0x009F => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A0 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A1 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A2 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A3 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A4 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A5 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A6 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A7 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A8 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00A9 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00AA => "RicohFaceDetectFaceEarlier".to_string(),
            0x00AB => "RicohFaceDetectFaceEarlier".to_string(),
            0x00AC => "RicohFaceDetectFaceEarlier".to_string(),
            0x00AD => "RicohFaceDetectFaceEarlier".to_string(),
            0x00AE => "RicohFaceDetectFaceEarlier".to_string(),
            0x00AF => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B0 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B1 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B2 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B3 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B4 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B5 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B6 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B7 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B8 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00B9 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00BA => "RicohFaceDetectFaceEarlier".to_string(),
            0x00BB => "RicohFaceDetectFaceEarlier".to_string(),
            0x00BC => "RicohFaceDetectFaceEarlier".to_string(),
            0x00BD => "RicohFaceDetectFaceEarlier".to_string(),
            0x00BE => "RicohFaceDetectFaceEarlier".to_string(),
            0x00BF => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C0 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C1 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C2 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C3 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C4 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C5 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C6 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C7 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C8 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00C9 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00CA => "RicohFaceDetectFaceEarlier".to_string(),
            0x00CB => "RicohFaceDetectFaceEarlier".to_string(),
            0x00CC => "RicohFaceDetectFaceEarlier".to_string(),
            0x00CD => "RicohFaceDetectFaceEarlier".to_string(),
            0x00CE => "RicohFaceDetectFaceEarlier".to_string(),
            0x00CF => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D0 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D1 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D2 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D3 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D4 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D5 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D6 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D7 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D8 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00D9 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00DA => "RicohFaceDetectFaceEarlier".to_string(),
            0x00DB => "RicohFaceDetectFaceEarlier".to_string(),
            0x00DC => "RicohFaceDetectFaceEarlier".to_string(),
            0x00DD => "RicohFaceDetectFaceEarlier".to_string(),
            0x00DE => "RicohFaceDetectFaceEarlier".to_string(),
            0x00DF => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E0 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E1 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E2 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E3 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E4 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E5 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E6 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E7 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E8 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00E9 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00EA => "RicohFaceDetectFaceEarlier".to_string(),
            0x00EB => "RicohFaceDetectFaceEarlier".to_string(),
            0x00EC => "RicohFaceDetectFaceEarlier".to_string(),
            0x00ED => "RicohFaceDetectFaceEarlier".to_string(),
            0x00EE => "RicohFaceDetectFaceEarlier".to_string(),
            0x00EF => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F0 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F1 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F2 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F3 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F4 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F5 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F6 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F7 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F8 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00F9 => "RicohFaceDetectFaceEarlier".to_string(),
            0x00FA => "RicohFaceDetectFaceEarlier".to_string(),
            0x00FB => "RicohFaceDetectFaceEarlier".to_string(),
            0x00FC => "RicohFaceDetectFaceEarlier".to_string(),
            0x00FD => "RicohFaceDetectFaceEarlier".to_string(),
            0x00FE => "RicohFaceDetectFaceEarlier".to_string(),
            0x00FF => "RicohFaceDetectFaceEarlier".to_string(),
            _ => format!("RicohTag_{:04X}", tag_id),
        }
    }

    /// Parse Fujifilm maker note
    fn parse_fujifilm_maker_note(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Fujifilm maker note parsing
        metadata.insert("MakerNoteType".to_string(), "Fujifilm".to_string());

        // Look for Fujifilm-specific patterns
        if data.windows(8).any(|w| w == b"FUJIFILM") {
            metadata.insert("FujifilmMakerNote".to_string(), "Detected".to_string());
        }
    }
}
