use crate::types::ExifError;
use std::collections::HashMap;

/// BMP parser for extracting metadata from BMP files
pub struct BmpParser;

impl BmpParser {
    /// Parse BMP metadata
    pub fn parse_bmp_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        if data.len() < 14 {
            return Err(ExifError::InvalidExif("BMP file too small".to_string()));
        }

        // Check BMP signature
        if data[0] != 0x42 || data[1] != 0x4D {
            return Err(ExifError::InvalidExif("Invalid BMP signature".to_string()));
        }

        // Set format
        metadata.insert("Format".to_string(), "BMP".to_string());

        // Parse BMP header
        let file_size = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]);

        metadata.insert("FileSize".to_string(), file_size.to_string());
        metadata.insert("DataOffset".to_string(), data_offset.to_string());

        // Parse DIB header if present
        if data.len() >= 18 {
            let dib_header_size = u32::from_le_bytes([data[14], data[15], data[16], data[17]]);
            metadata.insert("DIBHeaderSize".to_string(), dib_header_size.to_string());

            match dib_header_size {
                40 => {
                    // BITMAPINFOHEADER
                    Self::parse_bitmapinfoheader(data, metadata)?;
                }
                108 => {
                    // BITMAPV4HEADER
                    Self::parse_bitmapv4header(data, metadata)?;
                }
                124 => {
                    // BITMAPV5HEADER
                    Self::parse_bitmapv5header(data, metadata)?;
                }
                _ => {
                    // Unknown header size
                    metadata.insert("HeaderType".to_string(), "Unknown".to_string());
                }
            }
        }

        // Add computed fields
        Self::add_computed_fields(metadata);

        Ok(())
    }

    /// Parse BITMAPINFOHEADER
    fn parse_bitmapinfoheader(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        if data.len() < 54 {
            return Err(ExifError::InvalidExif("BITMAPINFOHEADER too small".to_string()));
        }

        let width = i32::from_le_bytes([data[18], data[19], data[20], data[21]]);
        let height = i32::from_le_bytes([data[22], data[23], data[24], data[25]]);
        let planes = u16::from_le_bytes([data[26], data[27]]);
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]);
        let compression = u32::from_le_bytes([data[30], data[31], data[32], data[33]]);
        let image_size = u32::from_le_bytes([data[34], data[35], data[36], data[37]]);
        let x_pixels_per_meter = i32::from_le_bytes([data[38], data[39], data[40], data[41]]);
        let y_pixels_per_meter = i32::from_le_bytes([data[42], data[43], data[44], data[45]]);
        let colors_used = u32::from_le_bytes([data[46], data[47], data[48], data[49]]);
        let colors_important = u32::from_le_bytes([data[50], data[51], data[52], data[53]]);

        metadata.insert("ImageWidth".to_string(), width.abs().to_string());
        metadata.insert("ImageHeight".to_string(), height.abs().to_string());
        metadata.insert("ImageSize".to_string(), format!("{}x{}", width.abs(), height.abs()));
        metadata.insert("Planes".to_string(), planes.to_string());
        metadata.insert("BitDepth".to_string(), bits_per_pixel.to_string());
        metadata.insert("Compression".to_string(), Self::get_compression_name(compression));
        metadata.insert("ImageSizeBytes".to_string(), image_size.to_string());
        metadata.insert("XPixelsPerMeter".to_string(), x_pixels_per_meter.to_string());
        metadata.insert("YPixelsPerMeter".to_string(), y_pixels_per_meter.to_string());
        metadata.insert("ColorsUsed".to_string(), colors_used.to_string());
        metadata.insert("ColorsImportant".to_string(), colors_important.to_string());

        // Calculate resolution
        if x_pixels_per_meter > 0 {
            let x_dpi = (x_pixels_per_meter as f64 / 39.3701) as i32;
            metadata.insert("XResolution".to_string(), x_dpi.to_string());
        }
        if y_pixels_per_meter > 0 {
            let y_dpi = (y_pixels_per_meter as f64 / 39.3701) as i32;
            metadata.insert("YResolution".to_string(), y_dpi.to_string());
        }

        // Calculate megapixels
        let megapixels = (width.abs() as f64 * height.abs() as f64) / 1_000_000.0;
        metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));

        Ok(())
    }

    /// Parse BITMAPV4HEADER
    fn parse_bitmapv4header(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // Parse BITMAPINFOHEADER first
        Self::parse_bitmapinfoheader(data, metadata)?;

        if data.len() < 108 {
            return Err(ExifError::InvalidExif("BITMAPV4HEADER too small".to_string()));
        }

        // Parse additional V4 fields
        let red_mask = u32::from_le_bytes([data[54], data[55], data[56], data[57]]);
        let green_mask = u32::from_le_bytes([data[58], data[59], data[60], data[61]]);
        let blue_mask = u32::from_le_bytes([data[62], data[63], data[64], data[65]]);
        let alpha_mask = u32::from_le_bytes([data[66], data[67], data[68], data[69]]);
        let color_space_type = u32::from_le_bytes([data[70], data[71], data[72], data[73]]);

        metadata.insert("RedMask".to_string(), format!("0x{:08X}", red_mask));
        metadata.insert("GreenMask".to_string(), format!("0x{:08X}", green_mask));
        metadata.insert("BlueMask".to_string(), format!("0x{:08X}", blue_mask));
        metadata.insert("AlphaMask".to_string(), format!("0x{:08X}", alpha_mask));
        metadata.insert("ColorSpaceType".to_string(), Self::get_color_space_name(color_space_type));

        Ok(())
    }

    /// Parse BITMAPV5HEADER
    fn parse_bitmapv5header(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // Parse BITMAPV4HEADER first
        Self::parse_bitmapv4header(data, metadata)?;

        if data.len() < 124 {
            return Err(ExifError::InvalidExif("BITMAPV5HEADER too small".to_string()));
        }

        // Parse additional V5 fields
        let gamma_red = u32::from_le_bytes([data[108], data[109], data[110], data[111]]);
        let gamma_green = u32::from_le_bytes([data[112], data[113], data[114], data[115]]);
        let gamma_blue = u32::from_le_bytes([data[116], data[117], data[118], data[119]]);
        let intent = u32::from_le_bytes([data[120], data[121], data[122], data[123]]);

        metadata.insert("GammaRed".to_string(), gamma_red.to_string());
        metadata.insert("GammaGreen".to_string(), gamma_green.to_string());
        metadata.insert("GammaBlue".to_string(), gamma_blue.to_string());
        metadata.insert("Intent".to_string(), Self::get_intent_name(intent));

        Ok(())
    }

    /// Get compression name
    fn get_compression_name(compression: u32) -> String {
        match compression {
            0 => "None".to_string(),
            1 => "RLE8".to_string(),
            2 => "RLE4".to_string(),
            3 => "Bitfields".to_string(),
            4 => "JPEG".to_string(),
            5 => "PNG".to_string(),
            _ => format!("Unknown ({})", compression),
        }
    }

    /// Get color space name
    fn get_color_space_name(color_space: u32) -> String {
        match color_space {
            0x73524742 => "sRGB".to_string(),
            0x57696E20 => "Windows".to_string(),
            0x4C494E4B => "Linked".to_string(),
            0x4D424544 => "Embedded".to_string(),
            _ => format!("Unknown (0x{:08X})", color_space),
        }
    }

    /// Get intent name
    fn get_intent_name(intent: u32) -> String {
        match intent {
            1 => "Saturation".to_string(),
            2 => "Relative Colorimetric".to_string(),
            4 => "Perceptual".to_string(),
            8 => "Absolute Colorimetric".to_string(),
            _ => format!("Unknown ({})", intent),
        }
    }

    /// Add computed fields that exiftool provides
    fn add_computed_fields(metadata: &mut HashMap<String, String>) {
        // File information
        metadata.insert("FileTypeExtension".to_string(), "bmp".to_string());
        metadata.insert("MIMEType".to_string(), "image/bmp".to_string());
        metadata.insert("ExifByteOrder".to_string(), "Little-endian (Intel, II)".to_string());

        // Add format-specific fields
        if !metadata.contains_key("Format") {
            metadata.insert("Format".to_string(), "BMP".to_string());
        }
    }
}
