use crate::types::ExifError;
use std::collections::HashMap;
use nom::IResult;

/// Enhanced image format parser supporting additional image formats
pub struct EnhancedImageParser;

impl EnhancedImageParser {
    /// Parse GIF files
    pub fn parse_gif(data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();
        
        metadata.insert("FileType".to_string(), "GIF".to_string());
        metadata.insert("MIMEType".to_string(), "image/gif".to_string());
        
        // Parse GIF header
        if data.len() >= 6 {
            if data[0..6] == *b"GIF87a" || data[0..6] == *b"GIF89a" {
                metadata.insert("Format".to_string(), "GIF".to_string());
                metadata.insert("Version".to_string(), 
                    if data[0..6] == *b"GIF87a" { "87a".to_string() } 
                    else { "89a".to_string() });
                
                // Parse GIF-specific metadata
                if let Ok(gif_metadata) = Self::parse_gif_header(data) {
                    metadata.extend(gif_metadata);
                }
            }
        }
        
        Ok(metadata)
    }

    /// Parse WEBP files
    pub fn parse_webp(data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();
        
        metadata.insert("FileType".to_string(), "WEBP".to_string());
        metadata.insert("MIMEType".to_string(), "image/webp".to_string());
        
        // Parse WEBP header
        if data.len() >= 12 {
            if data[0..4] == *b"RIFF" && data[8..12] == *b"WEBP" {
                metadata.insert("Format".to_string(), "WEBP".to_string());
                
                // Parse WEBP-specific metadata
                if let Ok(webp_metadata) = Self::parse_webp_header(data) {
                    metadata.extend(webp_metadata);
                }
            }
        }
        
        Ok(metadata)
    }

    /// Parse GIF header
    fn parse_gif_header(data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();
        
        if data.len() < 13 {
            return Ok(metadata);
        }
        
        // Parse GIF logical screen descriptor
        let width = u16::from_le_bytes([data[6], data[7]]);
        let height = u16::from_le_bytes([data[8], data[9]]);
        
        metadata.insert("ImageWidth".to_string(), width.to_string());
        metadata.insert("ImageHeight".to_string(), height.to_string());
        
        // Parse color information
        let packed = data[10];
        let global_color_table = (packed & 0x80) != 0;
        let color_resolution = ((packed & 0x70) >> 4) + 1;
        let sort_flag = (packed & 0x08) != 0;
        let global_color_table_size = 2 << (packed & 0x07);
        
        metadata.insert("GlobalColorTable".to_string(), global_color_table.to_string());
        metadata.insert("ColorResolution".to_string(), color_resolution.to_string());
        metadata.insert("SortFlag".to_string(), sort_flag.to_string());
        metadata.insert("GlobalColorTableSize".to_string(), global_color_table_size.to_string());
        
        Ok(metadata)
    }

    /// Parse WEBP header
    fn parse_webp_header(data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();
        
        if data.len() < 16 {
            return Ok(metadata);
        }
        
        // Parse WEBP chunk header
        let chunk_type = &data[12..16];
        
        match chunk_type {
            b"VP8 " => {
                metadata.insert("Codec".to_string(), "VP8".to_string());
                metadata.insert("Format".to_string(), "Lossy".to_string());
            },
            b"VP8L" => {
                metadata.insert("Codec".to_string(), "VP8L".to_string());
                metadata.insert("Format".to_string(), "Lossless".to_string());
            },
            b"VP8X" => {
                metadata.insert("Codec".to_string(), "VP8X".to_string());
                metadata.insert("Format".to_string(), "Extended".to_string());
            },
            _ => {
                metadata.insert("Codec".to_string(), "Unknown".to_string());
            }
        }
        
        Ok(metadata)
    }
}
