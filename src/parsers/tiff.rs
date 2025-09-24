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
                    // Value is inline
                    let bytes = value_offset.to_le_bytes();
                    if let Ok(string) = String::from_utf8(bytes.to_vec()) {
                        metadata.insert(tag_name, string.trim_end_matches('\0').to_string());
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
                    
                    // Format special fields
                    let formatted_value = Self::format_special_field(tag_id, value);
                    metadata.insert(tag_name, formatted_value);
                }
            },
            4 => { // LONG
                if count == 1 {
                    metadata.insert(tag_name, value_offset.to_string());
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
                // For other types, just store the raw value
                metadata.insert(tag_name, value_offset.to_string());
            }
        }
        
        Ok(())
    }
    
    /// Calculate greatest common divisor
    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }
    
    /// Format exposure time to match exiftool's algorithm
    fn format_exposure_time(value: f64) -> String {
        // Handle very long exposures (> 1 second)
        if value >= 1.0 {
            if value == value.floor() {
                return format!("{}", value as u32);
            } else {
                return format!("{:.1}", value);
            }
        }
        
        // Handle fractional exposures (< 1 second)
        // Try to find the best fraction representation
        let common_denominators = vec![
            1, 2, 3, 4, 5, 6, 8, 10, 12, 15, 16, 20, 25, 30, 40, 50, 60, 80, 100, 125, 160, 200, 250, 320, 400, 500, 640, 800, 1000, 1250, 1600, 2000, 2500, 3200, 4000, 5000, 6400, 8000, 10000, 12500, 16000, 20000, 25000, 32000, 40000, 50000, 64000, 80000, 100000, 125000, 160000, 200000, 250000, 320000, 400000, 500000, 640000, 800000, 1000000
        ];
        
        let mut best_fraction = (1, 1);
        let mut best_error = f64::MAX;
        
        for &denom in &common_denominators {
            let num = (value * denom as f64).round() as u32;
            if num > 0 {
                let fraction_value = num as f64 / denom as f64;
                let error = (fraction_value - value).abs();
                
                if error < best_error {
                    best_error = error;
                    best_fraction = (num, denom);
                }
                
                // If we're very close, use this fraction
                if error < 0.0001 {
                    break;
                }
            }
        }
        
        let (num, denom) = best_fraction;
        
        // Simplify the fraction
        let gcd = Self::gcd(num, denom);
        let simplified_num = num / gcd;
        let simplified_den = denom / gcd;
        
        if simplified_den == 1 {
            format!("{}", simplified_num)
        } else {
            format!("{}/{}", simplified_num, simplified_den)
        }
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
            0x0213 => { // YCbCrPositioning
                match value {
                    1 => "Centered".to_string(),
                    2 => "Co-sited".to_string(),
                    _ => value.to_string(),
                }
            },
            _ => value.to_string(),
        }
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
            0xA20E => "FocalPlaneXResolution".to_string(),
            0xA20F => "FocalPlaneYResolution".to_string(),
            0xA210 => "FocalPlaneResolutionUnit".to_string(),
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
