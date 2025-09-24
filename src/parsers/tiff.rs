use crate::types::ExifError;
use crate::parsers::maker_notes::MakerNoteParser;
use std::collections::HashMap;

/// TIFF-based EXIF parser
pub struct TiffParser;

impl TiffParser {
    /// Parse TIFF-based EXIF data
    pub fn parse_tiff_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        if data.len() < 8 {
            return Err(ExifError::InvalidExif("TIFF header too small".to_string()));
        }
        
        // Find the actual TIFF header (skip any padding/null bytes)
        let mut tiff_start = 0;
        for i in 0..data.len().saturating_sub(8) {
            if &data[i..i+2] == b"II" || &data[i..i+2] == b"MM" {
                tiff_start = i;
                break;
            }
        }
        
        if tiff_start + 8 > data.len() {
            return Err(ExifError::InvalidExif("TIFF header not found".to_string()));
        }
        
        // Determine byte order
        let is_little_endian = &data[tiff_start..tiff_start+2] == b"II";
        let is_big_endian = &data[tiff_start..tiff_start+2] == b"MM";
        
        if !is_little_endian && !is_big_endian {
            return Err(ExifError::InvalidExif("Invalid TIFF byte order".to_string()));
        }
        
        // Validate TIFF version (should be 42)
        if tiff_start + 8 > data.len() {
            return Err(ExifError::InvalidExif("TIFF header incomplete".to_string()));
        }
        
        let version = if is_little_endian {
            u16::from_le_bytes([data[tiff_start + 2], data[tiff_start + 3]])
        } else {
            u16::from_be_bytes([data[tiff_start + 2], data[tiff_start + 3]])
        };
        
        if version != 42 {
            return Err(ExifError::InvalidExif("Invalid TIFF version".to_string()));
        }
        
        // Get IFD offset
        let ifd_offset = if is_little_endian {
            u32::from_le_bytes([
                data[tiff_start + 4], data[tiff_start + 5], 
                data[tiff_start + 6], data[tiff_start + 7]
            ])
        } else {
            u32::from_be_bytes([
                data[tiff_start + 4], data[tiff_start + 5], 
                data[tiff_start + 6], data[tiff_start + 7]
            ])
        };
        
        // Parse the first IFD
        Self::parse_ifd(data, tiff_start + ifd_offset as usize, is_little_endian, tiff_start, metadata)?;
        
        // Parse EXIF IFD if present (contains DateTimeOriginal, ExposureTime, etc.)
        if let Some(exif_ifd_offset) = Self::find_sub_ifd_offset(data, tiff_start + ifd_offset as usize, 0x8769, is_little_endian, tiff_start) {
            Self::parse_ifd(data, tiff_start + exif_ifd_offset as usize, is_little_endian, tiff_start, metadata)?;
        }
        
        // Parse GPS IFD if present (contains GPS metadata)
        if let Some(gps_ifd_offset) = Self::find_sub_ifd_offset(data, tiff_start + ifd_offset as usize, 0x8825, is_little_endian, tiff_start) {
            Self::parse_ifd(data, tiff_start + gps_ifd_offset as usize, is_little_endian, tiff_start, metadata)?;
        }
        
        // Parse Interoperability IFD if present (contains InteropIndex, InteropVersion, etc.)
        if let Some(interop_ifd_offset) = Self::find_sub_ifd_offset(data, tiff_start + ifd_offset as usize, 0xA005, is_little_endian, tiff_start) {
            Self::parse_ifd(data, tiff_start + interop_ifd_offset as usize, is_little_endian, tiff_start, metadata)?;
        }
        
        Ok(())
    }
    
    /// Parse Image File Directory (IFD)
    fn parse_ifd(data: &[u8], offset: usize, is_little_endian: bool, tiff_start: usize, metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        if offset + 2 > data.len() {
            return Err(ExifError::InvalidExif("IFD header incomplete".to_string()));
        }
        
        // Read number of directory entries
        let entry_count = if is_little_endian {
            u16::from_le_bytes([data[offset], data[offset + 1]])
        } else {
            u16::from_be_bytes([data[offset], data[offset + 1]])
        };
        
        if entry_count == 0 || entry_count > 1000 {
            return Err(ExifError::InvalidExif("Invalid IFD entry count".to_string()));
        }
        
        // Parse each directory entry
        for i in 0..entry_count {
            let entry_offset = offset + 2 + (i as usize * 12);
            if entry_offset + 12 > data.len() {
                continue;
            }
            
            Self::parse_ifd_entry(data, entry_offset, is_little_endian, tiff_start, metadata)?;
        }
        
        // Parse maker notes if present
        if let Some(maker_note_offset) = Self::find_sub_ifd_offset(data, offset, 0x927C, is_little_endian, tiff_start) {
            MakerNoteParser::parse_maker_note(data, tiff_start + maker_note_offset as usize, 0, metadata);
        }
        
        Ok(())
    }
    
    /// Parse a single IFD entry
    fn parse_ifd_entry(data: &[u8], offset: usize, is_little_endian: bool, tiff_start: usize, metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // Read tag ID
        let tag_id = if is_little_endian {
            u16::from_le_bytes([data[offset], data[offset + 1]])
        } else {
            u16::from_be_bytes([data[offset], data[offset + 1]])
        };
        
        // Read data type
        let data_type = if is_little_endian {
            u16::from_le_bytes([data[offset + 2], data[offset + 3]])
        } else {
            u16::from_be_bytes([data[offset + 2], data[offset + 3]])
        };
        
        // Read count
        let count = if is_little_endian {
            u32::from_le_bytes([
                data[offset + 4], data[offset + 5], 
                data[offset + 6], data[offset + 7]
            ])
        } else {
            u32::from_be_bytes([
                data[offset + 4], data[offset + 5], 
                data[offset + 6], data[offset + 7]
            ])
        };
        
        // Read value/offset
        let value_offset = if is_little_endian {
            u32::from_le_bytes([
                data[offset + 8], data[offset + 9], 
                data[offset + 10], data[offset + 11]
            ])
        } else {
            u32::from_be_bytes([
                data[offset + 8], data[offset + 9], 
                data[offset + 10], data[offset + 11]
            ])
        };
        
        // Parse the tag value
        Self::parse_tag_value(data, tag_id, data_type, count, value_offset, is_little_endian, tiff_start, metadata)?;
        
        Ok(())
    }
    
    /// Parse tag value based on type and count
    #[allow(clippy::too_many_arguments)]
    fn parse_tag_value(data: &[u8], tag_id: u16, data_type: u16, count: u32, value_offset: u32, is_little_endian: bool, tiff_start: usize, metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        let tag_name = Self::get_tag_name(tag_id);
        
        match data_type {
            1 => { // BYTE
                if count <= 4 {
                    // Value is inline
                    let value = if is_little_endian {
                        value_offset as u8
                    } else {
                        (value_offset >> 24) as u8
                    };
                    metadata.insert(tag_name, value.to_string());
                } else {
                    // Value is at offset
                    let offset = tiff_start + value_offset as usize;
                    if offset + count as usize <= data.len() {
                        let bytes = &data[offset..offset + count as usize];
                        if let Ok(string) = String::from_utf8(bytes.to_vec()) {
                            metadata.insert(tag_name, string.trim_end_matches('\0').to_string());
                        }
                    }
                }
            },
            2 => { // ASCII
                if count <= 4 {
                    // Special handling for version fields that might be processed as ASCII
                    if tag_id == 0xA000 || tag_id == 0x9000 { // FlashpixVersion or ExifVersion
                        // Convert the 32-bit value to ASCII characters
                        let version_string = Self::format_version_field(value_offset, is_little_endian);
                        metadata.insert(tag_name, version_string);
                    } else {
                        // Value is inline
                        let bytes = value_offset.to_le_bytes();
                        if let Ok(string) = String::from_utf8(bytes.to_vec()) {
                            metadata.insert(tag_name, string.trim_end_matches('\0').to_string());
                        }
                    }
                } else {
                    // Value is at offset
                    let offset = tiff_start + value_offset as usize;
                    if offset + count as usize <= data.len() {
                        let bytes = &data[offset..offset + count as usize];
                        if let Ok(string) = String::from_utf8(bytes.to_vec()) {
                            metadata.insert(tag_name, string.trim_end_matches('\0').to_string());
                        }
                    }
                }
            },
                    3 => { // SHORT
                        if count == 1 {
                            let value = if is_little_endian {
                                (value_offset & 0xFFFF) as u16
                            } else {
                                (value_offset >> 16) as u16
                            };
                            
                            // Special handling for ExposureCompensation as SHORT
                            if tag_id == 0x9204 { // ExposureCompensation
                                // Convert SHORT value to EV using APEX conversion
                                // The value is stored as a signed 16-bit integer in 1/1000 EV units
                                // So 0 = 0 EV, 1000 = +1 EV, -1000 = -1 EV
                                eprintln!("DEBUG: ExposureCompensation SHORT value: {}", value);
                                let ev_value = value as i16 as f64 / 1000.0;
                                eprintln!("DEBUG: ExposureCompensation EV value: {}", ev_value);
                                let formatted_value = Self::print_fraction(ev_value);
                                eprintln!("DEBUG: ExposureCompensation formatted: '{}'", formatted_value);
                                metadata.insert(tag_name, formatted_value);
                            } else {
                                // Format special fields
                                let formatted_value = Self::format_special_field(tag_id, value);
                                metadata.insert(tag_name, formatted_value);
                            }
                        }
                    },
            4 => { // LONG
                if count == 1 {
                    // Special handling for version fields
                    if tag_id == 0xA000 || tag_id == 0x9000 { // FlashpixVersion or ExifVersion
                        // Convert 4-byte version field to hex string
                        eprintln!("DEBUG: Version field {} value_offset: 0x{:08X}", tag_name, value_offset);
                        let version_string = Self::format_version_field(value_offset, is_little_endian);
                        eprintln!("DEBUG: Version field {} result: '{}'", tag_name, version_string);
                        metadata.insert(tag_name, version_string);
                    } else if tag_id == 0xA402 { // ExposureMode as LONG
                        let formatted_value = Self::format_special_field(tag_id, value_offset as u16);
                        metadata.insert(tag_name, formatted_value);
                    } else {
                        metadata.insert(tag_name, value_offset.to_string());
                    }
                }
            },
            5 => { // RATIONAL
                if count == 1 {
                    // For rational values, we need to read the actual value from the offset
                    let offset = tiff_start + value_offset as usize;
                    if offset + 8 <= data.len() {
                        let numerator = if is_little_endian {
                            ((data[offset] as u32) | 
                             ((data[offset + 1] as u32) << 8) |
                             ((data[offset + 2] as u32) << 16) |
                             ((data[offset + 3] as u32) << 24))
                        } else {
                            (((data[offset] as u32) << 24) |
                             ((data[offset + 1] as u32) << 16) |
                             ((data[offset + 2] as u32) << 8) |
                             (data[offset + 3] as u32))
                        };
                        
                        let denominator = if is_little_endian {
                            ((data[offset + 4] as u32) | 
                             ((data[offset + 5] as u32) << 8) |
                             ((data[offset + 6] as u32) << 16) |
                             ((data[offset + 7] as u32) << 24))
                        } else {
                            (((data[offset + 4] as u32) << 24) |
                             ((data[offset + 5] as u32) << 16) |
                             ((data[offset + 6] as u32) << 8) |
                             (data[offset + 7] as u32))
                        };
                        
                        // Format rational values based on field type
                        if tag_id == 0x011A || tag_id == 0x011B { // XResolution or YResolution
                            metadata.insert(tag_name, numerator.to_string());
                        } else if tag_id == 0xA20C || tag_id == 0xA20F { // FocalPlaneXResolution or FocalPlaneYResolution
                            if denominator != 0 {
                                let value = numerator as f64 / denominator as f64;
                                metadata.insert(tag_name, format!("{:.5}", value));
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0xA20E { // FocalPlaneResolutionUnit
                            // This should be treated as a SHORT value, not rational
                            // Convert the numerator to the appropriate string
                            let unit_string = match numerator {
                                1 => "None".to_string(),
                                2 => "inches".to_string(),
                                3 => "cm".to_string(),
                                _ => numerator.to_string(),
                            };
                            metadata.insert(tag_name, unit_string);
                        } else if tag_id == 0x829A { // ExposureTime
                            // Format exposure time to match exiftool's algorithm
                            if denominator != 0 {
                                let value = numerator as f64 / denominator as f64;
                                let formatted = Self::format_exposure_time(value);
                                metadata.insert(tag_name, formatted);
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0x829D { // FNumber
                            // Format f-number (e.g., "4.0")
                            if denominator != 0 {
                                let value = numerator as f64 / denominator as f64;
                                metadata.insert(tag_name, format!("{:.1}", value));
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0x9201 { // ShutterSpeedValue
                            // Convert APEX value to shutter speed: 2^(-apex_value) like exiftool
                            if denominator != 0 {
                                let apex_value = numerator as f64 / denominator as f64;
                                let shutter_speed = if apex_value.abs() < 100.0 {
                                    2.0_f64.powf(-apex_value)
                                } else {
                                    0.0
                                };
                                let formatted = Self::format_exposure_time(shutter_speed);
                                metadata.insert(tag_name, formatted);
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0x9202 { // ApertureValue
                            // Convert APEX value to f-number: 2^(apex_value/2)
                            if denominator != 0 {
                                let apex_value = numerator as f64 / denominator as f64;
                                let f_number = 2.0_f64.powf(apex_value / 2.0);
                                metadata.insert(tag_name, format!("{:.1}", f_number));
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0x9203 { // BrightnessValue
                            // Format brightness value
                            if denominator != 0 {
                                let value = numerator as f64 / denominator as f64;
                                metadata.insert(tag_name, format!("{:.2}", value));
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0x9204 { // ExposureCompensation
                            // Format exposure compensation for RATIONAL type using PrintFraction logic
                            if denominator != 0 {
                                let value = numerator as f64 / denominator as f64;
                                let formatted_value = Self::print_fraction(value);
                                metadata.insert(tag_name, formatted_value);
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0x9205 { // MaxApertureValue
                            // Convert APEX value to f-number: 2^(apex_value/2)
                            if denominator != 0 {
                                let apex_value = numerator as f64 / denominator as f64;
                                let f_number = 2.0_f64.powf(apex_value / 2.0);
                                metadata.insert(tag_name, format!("{:.1}", f_number));
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0x9206 { // SubjectDistance
                            // Format subject distance
                            if denominator != 0 {
                                let value = numerator as f64 / denominator as f64;
                                if value >= 1000.0 {
                                    metadata.insert(tag_name, format!("{:.0} m", value / 1000.0));
                                } else {
                                    metadata.insert(tag_name, format!("{:.2} m", value));
                                }
                            } else {
                                metadata.insert(tag_name, numerator.to_string());
                            }
                        } else if tag_id == 0x920A { // FocalLength
                            // Format focal length
                            if denominator != 0 {
                                let value = numerator as f64 / denominator as f64;
                                metadata.insert(tag_name, format!("{:.1} mm", value));
                            } else {
                                metadata.insert(tag_name, format!("{} mm", numerator));
                            }
                        } else if tag_id == 0xA405 { // FocalLengthIn35mmFilm
                            // Format focal length in 35mm film
                            if denominator != 0 {
                                let value = numerator as f64 / denominator as f64;
                                metadata.insert(tag_name, format!("{:.0} mm", value));
                            } else {
                                metadata.insert(tag_name, format!("{} mm", numerator));
                            }
                                } else if tag_id == 0xA404 { // DigitalZoomRatio
                                    // Format digital zoom ratio
                                    if denominator != 0 {
                                        let value = numerator as f64 / denominator as f64;
                                        if value == 1.0 {
                                            metadata.insert(tag_name, "1".to_string());
                                        } else {
                                            metadata.insert(tag_name, format!("{:.6}", value));
                                        }
                                    } else {
                                        metadata.insert(tag_name, numerator.to_string());
                                    }
                                } else {
                                    // For other rational fields, format as decimal
                                    if denominator != 0 {
                                        let value = numerator as f64 / denominator as f64;
                                        metadata.insert(tag_name, format!("{:.6}", value));
                                    } else {
                                        metadata.insert(tag_name, numerator.to_string());
                                    }
                                }
                    }
                }
            },
            _ => {
                // Special handling for version fields (stored as UNDEFINED type)
                if tag_id == 0xA000 || tag_id == 0x9000 { // FlashpixVersion or ExifVersion
                    eprintln!("DEBUG TIFF: Version field {} (UNDEFINED) value_offset: 0x{:08X}", tag_name, value_offset);
                    // Version fields are stored as 4-byte ASCII strings
                    // Convert the 32-bit value to ASCII characters
                    let version_string = Self::format_version_field(value_offset, is_little_endian);
                    eprintln!("DEBUG TIFF: Version field {} result: '{}'", tag_name, version_string);
                    metadata.insert(tag_name, version_string);
                } else if tag_id == 0xA402 { // ExposureMode as other types
                    let formatted_value = Self::format_special_field(tag_id, value_offset as u16);
                    metadata.insert(tag_name, formatted_value);
                } else {
                    // For other types, just store the raw value
                    metadata.insert(tag_name, value_offset.to_string());
                }
            }
        }
        
        Ok(())
    }
    
    /// Calculate greatest common divisor
    #[allow(dead_code)]
    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }
    
    /// Format special field values to match exiftool output
    fn format_special_field(tag_id: u16, value: u16) -> String {
        match tag_id {
            0x0112 => { // Orientation
                match value {
                    1 => "Horizontal (normal)".to_string(),
                    2 => "Mirror horizontal".to_string(),
                    3 => "Rotate 180".to_string(),
                    4 => "Mirror vertical".to_string(),
                    5 => "Mirror horizontal and rotate 270 CW".to_string(),
                    6 => "Rotate 90 CW".to_string(),
                    7 => "Mirror horizontal and rotate 90 CW".to_string(),
                    8 => "Rotate 270 CW".to_string(),
                    _ => value.to_string(),
                }
            },
            0x0128 => { // ResolutionUnit
                match value {
                    1 => "None".to_string(),
                    2 => "inches".to_string(),
                    3 => "cm".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA20E => { // FocalPlaneResolutionUnit
                match value {
                    1 => "None".to_string(),
                    2 => "inches".to_string(),
                    3 => "cm".to_string(),
                    _ => value.to_string(),
                }
            },
            0x0213 => { // YCbCrPositioning
                match value {
                    1 => "Centered".to_string(),
                    2 => "Co-sited".to_string(),
                    _ => value.to_string(),
                }
            },
            0x8822 => { // ExposureProgram
                match value {
                    0 => "Not Defined".to_string(),
                    1 => "Manual".to_string(),
                    2 => "Program AE".to_string(),
                    3 => "Aperture-priority AE".to_string(),
                    4 => "Shutter-priority AE".to_string(),
                    5 => "Creative Program (biased toward depth of field)".to_string(),
                    6 => "Action Program (biased toward fast shutter speed)".to_string(),
                    7 => "Portrait Mode (for closeup photos with background blur)".to_string(),
                    8 => "Landscape Mode (for landscape photos with background sharp)".to_string(),
                    _ => value.to_string(),
                }
            },
            0x8827 => { // ISO
                format!("{}", value)
            },
            0x9207 => { // MeteringMode
                match value {
                    0 => "Unknown".to_string(),
                    1 => "Average".to_string(),
                    2 => "Center-weighted average".to_string(),
                    3 => "Spot".to_string(),
                    4 => "Multi-segment".to_string(),
                    5 => "Multi-segment".to_string(),
                    6 => "Partial".to_string(),
                    255 => "Other".to_string(),
                    _ => value.to_string(),
                }
            },
            0x9208 => { // LightSource
                match value {
                    0 => "Unknown".to_string(),
                    1 => "Daylight".to_string(),
                    2 => "Fluorescent".to_string(),
                    3 => "Tungsten (Incandescent)".to_string(),
                    4 => "Flash".to_string(),
                    9 => "Fine Weather".to_string(),
                    10 => "Cloudy Weather".to_string(),
                    11 => "Shade".to_string(),
                    12 => "Daylight Fluorescent (D 5700-7100K)".to_string(),
                    13 => "Day White Fluorescent (N 4600-5400K)".to_string(),
                    14 => "Cool White Fluorescent (W 3900-4500K)".to_string(),
                    15 => "White Fluorescent (WW 3200-3700K)".to_string(),
                    17 => "Standard Light A".to_string(),
                    18 => "Standard Light B".to_string(),
                    19 => "Standard Light C".to_string(),
                    20 => "D55".to_string(),
                    21 => "D65".to_string(),
                    22 => "D75".to_string(),
                    23 => "D50".to_string(),
                    24 => "ISO Studio Tungsten".to_string(),
                    255 => "Other".to_string(),
                    _ => value.to_string(),
                }
            },
            0x9209 => { // Flash
                match value {
                    0 => "No Flash".to_string(),
                    1 => "Fired".to_string(),
                    5 => "Fired, Return not detected".to_string(),
                    7 => "Fired, Return detected".to_string(),
                    8 => "On, Did not fire".to_string(),
                    9 => "On, Fired".to_string(),
                    13 => "On, Return not detected".to_string(),
                    15 => "On, Return detected".to_string(),
                    16 => "Off, Did not fire".to_string(),
                    24 => "Off, Did not fire".to_string(),
                    25 => "Off, Fired".to_string(),
                    29 => "Off, Return not detected".to_string(),
                    31 => "Off, Return detected".to_string(),
                    32 => "No Flash Function".to_string(),
                    65 => "Fired, Red-eye reduction".to_string(),
                    69 => "Fired, Red-eye reduction, Return not detected".to_string(),
                    71 => "Fired, Red-eye reduction, Return detected".to_string(),
                    73 => "On, Red-eye reduction".to_string(),
                    77 => "On, Red-eye reduction, Return not detected".to_string(),
                    79 => "On, Red-eye reduction, Return detected".to_string(),
                    89 => "On, Red-eye reduction".to_string(),
                    93 => "On, Red-eye reduction, Return not detected".to_string(),
                    95 => "On, Red-eye reduction, Return detected".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA001 => { // ColorSpace
                match value {
                    1 => "sRGB".to_string(),
                    65535 => "Uncalibrated".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA300 => { // FileSource
                match value {
                    1 => "Film Scanner".to_string(),
                    2 => "Reflection Print Scanner".to_string(),
                    3 => "Digital Camera".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA301 => { // SceneType
                match value {
                    1 => "Directly photographed".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA401 => { // CustomRendered
                match value {
                    0 => "Normal".to_string(),
                    1 => "Custom".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA402 => { // ExposureMode
                match value {
                    0 => "Auto".to_string(),
                    1 => "Manual".to_string(),
                    2 => "Auto Bracket".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA403 => { // WhiteBalance
                match value {
                    0 => "Auto".to_string(),
                    1 => "Manual".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA406 => { // SceneCaptureType
                match value {
                    0 => "Standard".to_string(),
                    1 => "Landscape".to_string(),
                    2 => "Portrait".to_string(),
                    3 => "Night Scene".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA407 => { // GainControl
                match value {
                    0 => "None".to_string(),
                    1 => "Low gain up".to_string(),
                    2 => "High gain up".to_string(),
                    3 => "Low gain down".to_string(),
                    4 => "High gain down".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA408 => { // Contrast
                match value {
                    0 => "Normal".to_string(),
                    1 => "Soft".to_string(),
                    2 => "Hard".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA409 => { // Saturation
                match value {
                    0 => "Normal".to_string(),
                    1 => "Low".to_string(),
                    2 => "High".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA40A => { // Sharpness
                match value {
                    0 => "Normal".to_string(),
                    1 => "Soft".to_string(),
                    2 => "Hard".to_string(),
                    3 => "3".to_string(),
                    4 => "4".to_string(),
                    5 => "5".to_string(),
                    6 => "6".to_string(),
                    7 => "7".to_string(),
                    8 => "8".to_string(),
                    9 => "9".to_string(),
                    10 => "10".to_string(),
                    25 => "25".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA40C => { // SubjectDistanceRange
                match value {
                    0 => "Unknown".to_string(),
                    1 => "Macro".to_string(),
                    2 => "Close View".to_string(),
                    3 => "Distant View".to_string(),
                    _ => value.to_string(),
                }
            },
            0xA217 => { // SensingMethod
                match value {
                    1 => "Not defined".to_string(),
                    2 => "One-chip color area".to_string(),
                    3 => "Two-chip color area sensor".to_string(),
                    4 => "Three-chip color area sensor".to_string(),
                    5 => "Color sequential area sensor".to_string(),
                    7 => "Trilinear sensor".to_string(),
                    8 => "Color sequential linear sensor".to_string(),
                    _ => value.to_string(),
                }
            },
            _ => value.to_string(),
        }
    }
    
    /// Print fraction - converts decimal values to fractions like exiftool
    fn print_fraction(value: f64) -> String {
        let val = value * 1.00001; // avoid round-off errors
        
        if val == 0.0 {
            "0".to_string()
        } else if (val.trunc() / val).abs() > 0.999 {
            format!("{:+}", val.trunc() as i32)
        } else if ((val * 2.0).trunc() / (val * 2.0)).abs() > 0.999 {
            format!("{:+}/2", (val * 2.0).trunc() as i32)
        } else if ((val * 3.0).trunc() / (val * 3.0)).abs() > 0.999 {
            format!("{:+}/3", (val * 3.0).trunc() as i32)
        } else {
            format!("{:+.3}", val)
        }
    }
    
    /// Format exposure time like exiftool's PrintExposureTime
    fn format_exposure_time(secs: f64) -> String {
        if secs < 0.25001 && secs > 0.0 {
            format!("1/{}", (0.5 + 1.0 / secs) as i32)
        } else {
            let formatted = format!("{:.1}", secs);
            if formatted.ends_with(".0") {
                formatted.trim_end_matches(".0").to_string()
            } else {
                formatted
            }
        }
    }
    
    /// Format version field (4 bytes) to ASCII string
    fn format_version_field(value: u32, is_little_endian: bool) -> String {
        // Version fields are stored as 4-byte ASCII strings
        // Extract bytes and convert to ASCII characters
        let bytes = if is_little_endian {
            [
                value as u8,
                (value >> 8) as u8,
                (value >> 16) as u8,
                (value >> 24) as u8,
            ]
        } else {
            [
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ]
        };
        
        // Convert ASCII bytes to characters, filtering out null bytes
        let mut result = String::new();
        for byte in bytes.iter() {
            if *byte != 0 && *byte >= 32 && *byte <= 126 {
                result.push(*byte as char);
            }
        }
        
        result
    }
    
    /// Get human-readable tag name
    fn get_tag_name(tag_id: u16) -> String {
        match tag_id {
            0x010E => "ImageDescription".to_string(),
            0x010F => "Make".to_string(),
            0x0110 => "Model".to_string(),
            0x0112 => "Orientation".to_string(),
            0x011A => "XResolution".to_string(),
            0x011B => "YResolution".to_string(),
            0x0128 => "ResolutionUnit".to_string(),
            0x0131 => "Software".to_string(),
            0x0132 => "DateTime".to_string(),
            0x9003 => "DateTimeOriginal".to_string(),
            0x9004 => "DateTimeDigitized".to_string(),
            0x829A => "ExposureTime".to_string(),
            0x829D => "FNumber".to_string(),
            0x8822 => "ExposureProgram".to_string(),
            0x8827 => "ISO".to_string(),
            0x9201 => "ShutterSpeedValue".to_string(),
            0x9202 => "ApertureValue".to_string(),
            0x9203 => "BrightnessValue".to_string(),
            0x9204 => "ExposureCompensation".to_string(),
            0x9205 => "MaxApertureValue".to_string(),
            0x9206 => "SubjectDistance".to_string(),
            0x9207 => "MeteringMode".to_string(),
            0x9208 => "LightSource".to_string(),
            0x9209 => "Flash".to_string(),
            0x920A => "FocalLength".to_string(),
            0x9290 => "SubSecTime".to_string(),
            0x9291 => "SubSecTimeOriginal".to_string(),
            0x9292 => "SubSecTimeDigitized".to_string(),
            0x013E => "WhitePoint".to_string(),
            0x013F => "PrimaryChromaticities".to_string(),
            0x0211 => "YCbCrCoefficients".to_string(),
            0x0213 => "YCbCrPositioning".to_string(),
            0x0214 => "ReferenceBlackWhite".to_string(),
            0x8298 => "Copyright".to_string(),
            0x8769 => "ExifIFD".to_string(),
            0x8825 => "GPSInfo".to_string(),
            0xA000 => "FlashpixVersion".to_string(),
            0xA001 => "ColorSpace".to_string(),
            0xA002 => "PixelXDimension".to_string(),
            0xA003 => "PixelYDimension".to_string(),
            0xA004 => "RelatedSoundFile".to_string(),
            0xA005 => "InteroperabilityIFD".to_string(),
            0x9000 => "ExifVersion".to_string(),
            0xA20C => "FocalPlaneXResolution".to_string(),
            0xA20E => "FocalPlaneResolutionUnit".to_string(),
            0xA20F => "FocalPlaneYResolution".to_string(),
            0xA210 => "CompressedBitsPerPixel".to_string(),
            0xA217 => "SensingMethod".to_string(),
            0xA300 => "FileSource".to_string(),
            0xA301 => "SceneType".to_string(),
            0xA302 => "CFAPattern".to_string(),
            0xA401 => "CustomRendered".to_string(),
            0xA402 => "ExposureMode".to_string(),
            0xA403 => "WhiteBalance".to_string(),
            0xA404 => "DigitalZoomRatio".to_string(),
            0xA405 => "FocalLengthIn35mmFilm".to_string(),
            0xA406 => "SceneCaptureType".to_string(),
            0xA407 => "GainControl".to_string(),
            0xA408 => "Contrast".to_string(),
            0xA409 => "Saturation".to_string(),
            0xA40A => "Sharpness".to_string(),
            0xA40B => "DeviceSettingDescription".to_string(),
            0xA40C => "SubjectDistanceRange".to_string(),
            0xA420 => "ImageUniqueID".to_string(),
            0xA430 => "CameraOwnerName".to_string(),
            0xA431 => "BodySerialNumber".to_string(),
            0xA432 => "LensSpecification".to_string(),
            0xA433 => "LensMake".to_string(),
            0xA434 => "LensModel".to_string(),
            0xA435 => "LensSerialNumber".to_string(),
            0x927C => "MakerNote".to_string(),
            _ => format!("UnknownTag_{:04X}", tag_id)
        }
    }
    
    /// Find sub-IFD offset for a specific tag
    fn find_sub_ifd_offset(data: &[u8], ifd_offset: usize, target_tag: u16, is_little_endian: bool, _tiff_start: usize) -> Option<u32> {
        if ifd_offset + 2 > data.len() {
            return None;
        }
        
        let entry_count = if is_little_endian {
            u16::from_le_bytes([data[ifd_offset], data[ifd_offset + 1]])
        } else {
            u16::from_be_bytes([data[ifd_offset], data[ifd_offset + 1]])
        };
        
        for i in 0..entry_count {
            let entry_offset = ifd_offset + 2 + (i as usize * 12);
            if entry_offset + 12 > data.len() {
                continue;
            }
            
            let tag_id = if is_little_endian {
                u16::from_le_bytes([data[entry_offset], data[entry_offset + 1]])
            } else {
                u16::from_be_bytes([data[entry_offset], data[entry_offset + 1]])
            };
            
            if tag_id == target_tag {
                let value_offset = if is_little_endian {
                    u32::from_le_bytes([
                        data[entry_offset + 8], data[entry_offset + 9], 
                        data[entry_offset + 10], data[entry_offset + 11]
                    ])
                } else {
                    u32::from_be_bytes([
                        data[entry_offset + 8], data[entry_offset + 9], 
                        data[entry_offset + 10], data[entry_offset + 11]
                    ])
                };
                return Some(value_offset);
            }
        }
        
        None
    }
}
