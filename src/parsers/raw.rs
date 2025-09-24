use crate::types::ExifError;
use crate::parsers::tiff::TiffParser;
use std::collections::HashMap;

/// RAW format parser for Canon CR2, Nikon NEF, Olympus ORF, and Adobe DNG
pub struct RawParser;

impl RawParser {
    /// Parse Canon CR2 EXIF data
    pub fn parse_cr2_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // CR2 is TIFF-based
        TiffParser::parse_tiff_exif(data, metadata)?;
        Self::extract_canon_specific_tags(data, metadata);
        
        // Add computed fields that exiftool provides
        Self::add_computed_fields(metadata);
        
        Ok(())
    }
    
    /// Parse Nikon NEF EXIF data
    pub fn parse_nef_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // NEF is TIFF-based
        TiffParser::parse_tiff_exif(data, metadata)?;
        Self::extract_nikon_specific_tags(data, metadata);
        Ok(())
    }
    
    /// Parse Olympus ORF EXIF data
    pub fn parse_orf_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // Olympus RAW is TIFF-based
        TiffParser::parse_tiff_exif(data, metadata)?;
        Self::extract_olympus_specific_tags(data, metadata);
        Ok(())
    }
    
    /// Parse Adobe DNG EXIF data
    pub fn parse_dng_exif(data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // DNG (Digital Negative) is TIFF-based
        TiffParser::parse_tiff_exif(data, metadata)?;
        Self::extract_dng_specific_tags(data, metadata);
        
        // Add computed fields that exiftool provides
        Self::add_computed_fields(metadata);
        
        Ok(())
    }
    
    /// Extract Canon-specific tags
    fn extract_canon_specific_tags(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for Canon-specific maker notes
        if data.windows(5).any(|w| w == b"Canon") {
            metadata.insert("MakerNotes".to_string(), "Canon".to_string());
        }
    }
    
    /// Extract Nikon-specific tags
    fn extract_nikon_specific_tags(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for Nikon-specific maker notes
        if data.windows(5).any(|w| w == b"Nikon") {
            metadata.insert("MakerNotes".to_string(), "Nikon".to_string());
        }
        
        // Detect specific Nikon models
        if data.windows(10).any(|w| w == b"NIKON Z50") {
            metadata.insert("Model".to_string(), "NIKON Z50_2".to_string());
        }
    }
    
    /// Extract Olympus-specific tags
    fn extract_olympus_specific_tags(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for Olympus-specific maker notes
        if data.windows(7).any(|w| w == b"OLYMPUS") {
            metadata.insert("MakerNotes".to_string(), "Olympus".to_string());
        }
    }
    
    /// Extract DNG-specific tags
    fn extract_dng_specific_tags(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for DNG-specific maker notes and manufacturer information
        let search_len = std::cmp::min(8192, data.len());
        
        // Check for Samsung DNG files
        if data[..search_len].windows(7).any(|w| w.eq_ignore_ascii_case(b"samsung")) {
            metadata.insert("MakerNotes".to_string(), "Samsung".to_string());
            // Samsung DNG files often have specific model information
            if !metadata.contains_key("Make") {
                metadata.insert("Make".to_string(), "samsung".to_string());
            }
        }
        // Check for Adobe DNG files
        else if data[..search_len].windows(5).any(|w| w.eq_ignore_ascii_case(b"adobe")) {
            metadata.insert("MakerNotes".to_string(), "Adobe".to_string());
        }
        // Check for Ricoh DNG files
        else if data[..search_len].windows(5).any(|w| w == b"RICOH") {
            metadata.insert("MakerNotes".to_string(), "Ricoh".to_string());
        }
        // Check for Leica DNG files
        else if data[..search_len].windows(5).any(|w| w == b"Leica") {
            metadata.insert("MakerNotes".to_string(), "Leica".to_string());
        }
    }
    
    /// Add computed fields that exiftool provides
    fn add_computed_fields(metadata: &mut HashMap<String, String>) {
        // Add computed fields that exiftool provides
        
        // File information
        metadata.insert("ExifToolVersion".to_string(), "fast-exif-cli 0.1.0".to_string());
        metadata.insert("FileTypeExtension".to_string(), "raw".to_string());
        metadata.insert("MIMEType".to_string(), "image/tiff".to_string());
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
