use crate::types::ExifError;
use std::collections::HashMap;

/// PNG parser for extracting EXIF data from PNG files
pub struct PngParser;

impl PngParser {
    /// Parse PNG EXIF data
    pub fn parse_png_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        if data.len() < 8 {
            return Err(ExifError::InvalidExif("PNG file too small".to_string()));
        }

        // Check PNG signature
        if data[0] != 0x89 || data[1] != 0x50 || data[2] != 0x4E || data[3] != 0x47
            || data[4] != 0x0D || data[5] != 0x0A || data[6] != 0x1A || data[7] != 0x0A
        {
            return Err(ExifError::InvalidExif("Invalid PNG signature".to_string()));
        }

        // Set format
        metadata.insert("Format".to_string(), "PNG".to_string());

        // Parse PNG chunks
        let mut offset = 8; // Skip PNG signature
        while offset + 8 <= data.len() {
            // Read chunk length
            let length = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;

            if offset + 8 + length > data.len() {
                break;
            }

            // Read chunk type
            let chunk_type = &data[offset + 4..offset + 8];

            // Check for EXIF chunk (eXIf)
            if chunk_type == b"eXIf" {
                // Extract EXIF data from PNG chunk
                let exif_data = &data[offset + 8..offset + 8 + length];
                Self::parse_png_exif_chunk(exif_data, metadata)?;
            }
            // Check for tEXt chunks (textual data)
            else if chunk_type == b"tEXt" {
                Self::parse_png_text_chunk(&data[offset + 8..offset + 8 + length], metadata);
            }
            // Check for iTXt chunks (international textual data)
            else if chunk_type == b"iTXt" {
                Self::parse_png_itxt_chunk(&data[offset + 8..offset + 8 + length], metadata);
            }
            // Check for IHDR chunk (image header)
            else if chunk_type == b"IHDR" {
                Self::parse_png_ihdr_chunk(&data[offset + 8..offset + 8 + length], metadata);
            }

            // Move to next chunk (length + type + data + CRC)
            offset += 8 + length + 4;
        }

        // Add computed fields
        Self::add_computed_fields(metadata);

        Ok(())
    }

    /// Parse PNG EXIF chunk
    fn parse_png_exif_chunk(exif_data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // PNG EXIF data is stored in TIFF format
        // Skip the first 4 bytes which contain the TIFF header offset
        if exif_data.len() < 4 {
            return Ok(());
        }

        let tiff_offset = u32::from_le_bytes([
            exif_data[0],
            exif_data[1],
            exif_data[2],
            exif_data[3],
        ]) as usize;

        if tiff_offset >= exif_data.len() {
            return Ok(());
        }

        // Parse TIFF data starting from the offset
        let tiff_data = &exif_data[tiff_offset..];
        
        // Use the existing TIFF parser to parse the EXIF data
        crate::parsers::tiff::TiffParser::parse_tiff_exif(tiff_data, metadata)
    }

    /// Parse PNG text chunk
    fn parse_png_text_chunk(data: &[u8], metadata: &mut HashMap<String, String>) {
        if data.is_empty() {
            return;
        }

        // Find null separator
        if let Some(null_pos) = data.iter().position(|&b| b == 0) {
            if null_pos > 0 && null_pos < data.len() - 1 {
                let keyword = String::from_utf8_lossy(&data[..null_pos]);
                let text = String::from_utf8_lossy(&data[null_pos + 1..]);
                
                // Map common PNG text keywords to EXIF fields
                match keyword.as_ref() {
                    "Software" => {
                        metadata.insert("Software".to_string(), text.to_string());
                    }
                    "Author" => {
                        metadata.insert("Artist".to_string(), text.to_string());
                    }
                    "Title" => {
                        metadata.insert("ImageDescription".to_string(), text.to_string());
                    }
                    "Description" => {
                        metadata.insert("ImageDescription".to_string(), text.to_string());
                    }
                    "Copyright" => {
                        metadata.insert("Copyright".to_string(), text.to_string());
                    }
                    "Creation Time" => {
                        metadata.insert("DateTime".to_string(), text.to_string());
                    }
                    _ => {
                        // Store as custom field
                        metadata.insert(format!("PNG_{}", keyword), text.to_string());
                    }
                }
            }
        }
    }

    /// Parse PNG iTXt chunk
    fn parse_png_itxt_chunk(data: &[u8], metadata: &mut HashMap<String, String>) {
        if data.len() < 5 {
            return;
        }

        // iTXt format: keyword\0compression_flag\0compression_method\0language_tag\0translated_keyword\0text
        if let Some(null_pos) = data.iter().position(|&b| b == 0) {
            if null_pos > 0 && null_pos < data.len() - 1 {
                let keyword = String::from_utf8_lossy(&data[..null_pos]);
                let remaining = &data[null_pos + 1..];
                
                // Skip compression flag, compression method, language tag, translated keyword
                let mut text_start = 0;
                let mut null_count = 0;
                for (i, &byte) in remaining.iter().enumerate() {
                    if byte == 0 {
                        null_count += 1;
                        if null_count == 4 { // After 4 nulls, we have the text
                            text_start = i + 1;
                            break;
                        }
                    }
                }
                
                if text_start < remaining.len() {
                    let text = String::from_utf8_lossy(&remaining[text_start..]);
                    
                    // Map common PNG text keywords to EXIF fields
                    match keyword.as_ref() {
                        "Software" => {
                            metadata.insert("Software".to_string(), text.to_string());
                        }
                        "Author" => {
                            metadata.insert("Artist".to_string(), text.to_string());
                        }
                        "Title" => {
                            metadata.insert("ImageDescription".to_string(), text.to_string());
                        }
                        "Description" => {
                            metadata.insert("ImageDescription".to_string(), text.to_string());
                        }
                        "Copyright" => {
                            metadata.insert("Copyright".to_string(), text.to_string());
                        }
                        "Creation Time" => {
                            metadata.insert("DateTime".to_string(), text.to_string());
                        }
                        _ => {
                            // Store as custom field
                            metadata.insert(format!("PNG_{}", keyword), text.to_string());
                        }
                    }
                }
            }
        }
    }

    /// Parse PNG IHDR chunk
    fn parse_png_ihdr_chunk(data: &[u8], metadata: &mut HashMap<String, String>) {
        if data.len() < 13 {
            return;
        }

        // IHDR format: width(4) height(4) bit_depth(1) color_type(1) compression(1) filter(1) interlace(1)
        let width = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let height = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let bit_depth = data[8];
        let color_type = data[9];

        metadata.insert("ImageWidth".to_string(), width.to_string());
        metadata.insert("ImageHeight".to_string(), height.to_string());
        metadata.insert("ImageSize".to_string(), format!("{}x{}", width, height));
        metadata.insert("BitDepth".to_string(), bit_depth.to_string());

        // Calculate megapixels
        let megapixels = (width as f64 * height as f64) / 1_000_000.0;
        metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));

        // Color type description
        let color_type_desc = match color_type {
            0 => "Grayscale",
            2 => "RGB",
            3 => "Palette",
            4 => "Grayscale with Alpha",
            6 => "RGB with Alpha",
            _ => "Unknown",
        };
        metadata.insert("ColorType".to_string(), color_type_desc.to_string());
    }

    /// Add computed fields that exiftool provides
    fn add_computed_fields(metadata: &mut HashMap<String, String>) {
        // File information
        metadata.insert("FileTypeExtension".to_string(), "png".to_string());
        metadata.insert("MIMEType".to_string(), "image/png".to_string());
        metadata.insert("ExifByteOrder".to_string(), "Little-endian (Intel, II)".to_string());

        // Add format-specific fields
        if !metadata.contains_key("Format") {
            metadata.insert("Format".to_string(), "PNG".to_string());
        }
    }
}
