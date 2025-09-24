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
                    metadata.insert(tag_name, value.to_string());
                }
            },
            4 => { // LONG
                if count == 1 {
                    metadata.insert(tag_name, value_offset.to_string());
                }
            },
            _ => {
                // For other types, just store the raw value
                metadata.insert(tag_name, value_offset.to_string());
            }
        }
        
        Ok(())
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
