use crate::types::ExifError;
use crate::utils::ExifUtils;
use crate::parsers::tiff::TiffParser;
use std::collections::HashMap;

/// Video format parser for MOV, MP4, and 3GP files
pub struct VideoParser;

impl VideoParser {
    /// Parse MOV EXIF data
    pub fn parse_mov_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // MOV files are QuickTime container format
        metadata.insert("Format".to_string(), "MOV".to_string());
        
        // Extract basic MOV metadata
        Self::extract_mov_basic_metadata(data, metadata);
        
        // Look for EXIF data in MOV atoms
        if let Some(exif_data) = Self::find_mov_exif(data) {
            TiffParser::parse_tiff_exif(exif_data, metadata)?;
        }
        
        // Add computed fields
        Self::add_computed_fields(metadata);
        
        Ok(())
    }
    
    /// Parse MP4 EXIF data
    pub fn parse_mp4_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // MP4 files are ISO Base Media File Format
        metadata.insert("Format".to_string(), "MP4".to_string());
        
        // Extract basic MP4 metadata
        Self::extract_mp4_basic_metadata(data, metadata);
        
        // Look for EXIF data in MP4 atoms
        if let Some(exif_data) = Self::find_mp4_exif(data) {
            TiffParser::parse_tiff_exif(exif_data, metadata)?;
        }
        
        // Add computed fields
        Self::add_computed_fields(metadata);
        
        Ok(())
    }
    
    /// Parse 3GP EXIF data
    pub fn parse_3gp_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // 3GP files are based on MP4 format
        metadata.insert("Format".to_string(), "3GP".to_string());
        
        // Extract basic 3GP metadata
        Self::extract_3gp_basic_metadata(data, metadata);
        
        // Look for EXIF data in 3GP atoms
        if let Some(exif_data) = Self::find_3gp_exif(data) {
            TiffParser::parse_tiff_exif(exif_data, metadata)?;
        }
        
        // Add computed fields
        Self::add_computed_fields(metadata);
        
        Ok(())
    }
    
    /// Find EXIF data in MOV atoms
    fn find_mov_exif<'a>(data: &'a [u8]) -> Option<&'a [u8]> {
        // Look for EXIF data in MOV atoms
        let mut pos = 0;
        
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }
            
            let atom_type = &data[pos + 4..pos + 8];
            
            match atom_type {
                b"udta" => {
                    // User data atom - may contain EXIF
                    if let Some(exif_data) = Self::find_exif_in_atom(data, pos + 8, size as usize - 8) {
                        return Some(exif_data);
                    }
                },
                b"meta" => {
                    // Meta atom - may contain EXIF
                    if let Some(exif_data) = Self::find_exif_in_atom(data, pos + 8, size as usize - 8) {
                        return Some(exif_data);
                    }
                },
                _ => {}
            }
            
            pos += size as usize;
        }
        
        None
    }
    
    /// Find EXIF data in MP4 atoms
    fn find_mp4_exif<'a>(data: &'a [u8]) -> Option<&'a [u8]> {
        // Look for EXIF data in MP4 atoms
        let mut pos = 0;
        
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }
            
            let atom_type = &data[pos + 4..pos + 8];
            
            match atom_type {
                b"udta" => {
                    // User data atom - may contain EXIF
                    if let Some(exif_data) = Self::find_exif_in_atom(data, pos + 8, size as usize - 8) {
                        return Some(exif_data);
                    }
                },
                b"meta" => {
                    // Meta atom - may contain EXIF
                    if let Some(exif_data) = Self::find_exif_in_atom(data, pos + 8, size as usize - 8) {
                        return Some(exif_data);
                    }
                },
                _ => {}
            }
            
            pos += size as usize;
        }
        
        None
    }
    
    /// Find EXIF data in 3GP atoms
    fn find_3gp_exif<'a>(data: &'a [u8]) -> Option<&'a [u8]> {
        // 3GP files use the same structure as MP4
        Self::find_mp4_exif(data)
    }
    
    /// Recursively search for EXIF data in atoms
    fn find_exif_in_atom<'a>(data: &'a [u8], start: usize, length: usize) -> Option<&'a [u8]> {
        // Recursively search for EXIF data in atoms
        let mut pos = start;
        let end = start + length;
        
        while pos + 8 < end {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > (end - pos) as u32 {
                break;
            }
            
            let atom_type = &data[pos + 4..pos + 8];
            
            match atom_type {
                b"EXIF" => {
                    // Found EXIF atom
                    if size > 8 {
                        return Some(&data[pos + 8..pos + size as usize]);
                    }
                },
                b"udta" | b"meta" | b"ilst" => {
                    // Recursively search in sub-atoms
                    if let Some(exif_data) = Self::find_exif_in_atom(data, pos + 8, size as usize - 8) {
                        return Some(exif_data);
                    }
                },
                _ => {}
            }
            
            pos += size as usize;
        }
        
        None
    }
    
    /// Extract basic MOV metadata
    fn extract_mov_basic_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Extract basic metadata from MOV atoms
        let mut pos = 0;
        
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }
            
            let atom_type = &data[pos + 4..pos + 8];
            
            match atom_type {
                b"ftyp" => {
                    // File type atom
                    if pos + 12 < data.len() {
                        let brand = &data[pos + 8..pos + 12];
                        if let Ok(brand_str) = String::from_utf8(brand.to_vec()) {
                            metadata.insert("Brand".to_string(), brand_str);
                        }
                    }
                },
                b"mvhd" => {
                    // Movie header atom - may contain creation time
                    metadata.insert("MovieHeader".to_string(), "Present".to_string());
                },
                _ => {}
            }
            
            pos += size as usize;
        }
        
        // Set default values
        if !metadata.contains_key("Make") {
            metadata.insert("Make".to_string(), "Unknown".to_string());
        }
        if !metadata.contains_key("Model") {
            metadata.insert("Model".to_string(), "Unknown".to_string());
        }
    }
    
    /// Extract basic MP4 metadata
    fn extract_mp4_basic_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Extract basic metadata from MP4 atoms
        let mut pos = 0;
        
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }
            
            let atom_type = &data[pos + 4..pos + 8];
            
            match atom_type {
                b"ftyp" => {
                    // File type atom
                    if pos + 12 < data.len() {
                        let brand = &data[pos + 8..pos + 12];
                        if let Ok(brand_str) = String::from_utf8(brand.to_vec()) {
                            metadata.insert("Brand".to_string(), brand_str);
                        }
                    }
                },
                b"mvhd" => {
                    // Movie header atom - may contain creation time
                    metadata.insert("MovieHeader".to_string(), "Present".to_string());
                },
                _ => {}
            }
            
            pos += size as usize;
        }
        
        // Set default values
        if !metadata.contains_key("Make") {
            metadata.insert("Make".to_string(), "Unknown".to_string());
        }
        if !metadata.contains_key("Model") {
            metadata.insert("Model".to_string(), "Unknown".to_string());
        }
    }
    
    /// Extract basic 3GP metadata
    fn extract_3gp_basic_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // 3GP files use the same structure as MP4
        Self::extract_mp4_basic_metadata(data, metadata);
        
        // Add 3GP-specific metadata
        metadata.insert("Format".to_string(), "3GP".to_string());
    }
    
    /// Add computed fields that exiftool provides
    fn add_computed_fields(metadata: &mut HashMap<String, String>) {
        // Add computed fields that exiftool provides
        
        // File information
        metadata.insert("ExifToolVersion".to_string(), "fast-exif-cli 0.4.8".to_string());
        
        // Determine file type extension based on format
        if let Some(format) = metadata.get("Format") {
            match format.as_str() {
                "MOV" => {
                    metadata.insert("FileTypeExtension".to_string(), "mov".to_string());
                    metadata.insert("MIMEType".to_string(), "video/quicktime".to_string());
                },
                "MP4" => {
                    metadata.insert("FileTypeExtension".to_string(), "mp4".to_string());
                    metadata.insert("MIMEType".to_string(), "video/mp4".to_string());
                },
                "3GP" => {
                    metadata.insert("FileTypeExtension".to_string(), "3gp".to_string());
                    metadata.insert("MIMEType".to_string(), "video/3gpp".to_string());
                },
                _ => {
                    metadata.insert("FileTypeExtension".to_string(), "mov".to_string());
                    metadata.insert("MIMEType".to_string(), "video/quicktime".to_string());
                }
            }
        }
        
        metadata.insert("ExifByteOrder".to_string(), "Little-endian (Intel, II)".to_string());
        
        // Computed image dimensions
        if let (Some(width), Some(height)) = (metadata.get("PixelXDimension").cloned(), metadata.get("PixelYDimension").cloned()) {
            metadata.insert("ImageSize".to_string(), format!("{}x{}", width, height));
            
            // Calculate megapixels
            if let (Ok(w), Ok(h)) = (width.parse::<f32>(), height.parse::<f32>()) {
                let megapixels = (w * h) / 1_000_000.0;
                metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));
            }
        }
        
        // Format rational values for better readability
        if let Some(focal_length) = metadata.get("FocalLength") {
            if let Ok(parsed) = focal_length.parse::<f32>() {
                metadata.insert("FocalLengthFormatted".to_string(), format!("{:.1} mm", parsed));
            }
        }
        
        if let Some(f_number) = metadata.get("FNumber") {
            if let Ok(parsed) = f_number.parse::<f32>() {
                metadata.insert("FNumberFormatted".to_string(), format!("f/{:.1}", parsed));
            }
        }
    }
}
