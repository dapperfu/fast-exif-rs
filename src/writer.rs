use crate::types::ExifError;
use crate::utils::ExifUtils;
use crate::format_detection::FormatDetector;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use byteorder::{BigEndian, WriteBytesExt};

/// EXIF writer for adding/modifying EXIF metadata in images
#[derive(Clone)]
pub struct ExifWriter {
    /// Whether to use little-endian byte order (default: true, matches most cameras)
    little_endian: bool,
    /// Whether to preserve existing EXIF data when possible
    preserve_existing: bool,
}

impl ExifWriter {
    /// Create a new EXIF writer with default settings
    pub fn new() -> Self {
        Self {
            little_endian: true,
            preserve_existing: true,
        }
    }

    /// Create a new EXIF writer with custom settings
    pub fn with_settings(little_endian: bool, preserve_existing: bool) -> Self {
        Self {
            little_endian,
            preserve_existing,
        }
    }

    /// Write EXIF metadata to an image file (auto-detects format)
    pub fn write_exif(
        &self,
        input_path: &str,
        output_path: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<(), ExifError> {
        let mut input_file = File::open(input_path)?;
        let mut input_data = Vec::new();
        input_file.read_to_end(&mut input_data)?;

        let output_data = self.write_exif_to_bytes(&input_data, metadata)?;

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&output_data)?;

        Ok(())
    }

    /// Write EXIF metadata to a JPEG file (legacy method)
    pub fn write_jpeg_exif(
        &self,
        input_path: &str,
        output_path: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<(), ExifError> {
        self.write_exif(input_path, output_path, metadata)
    }

    /// Write EXIF metadata to image bytes (auto-detects format)
    pub fn write_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // Detect file format
        let format = FormatDetector::detect_format(input_data)?;
        
        match format.as_str() {
            "JPEG" => self.write_jpeg_exif_to_bytes(input_data, metadata),
            "HEIF" | "HIF" => self.write_heif_exif_to_bytes(input_data, metadata),
            "PNG" => self.write_png_exif_to_bytes(input_data, metadata),
            "CR2" | "NEF" | "ORF" | "DNG" => self.write_raw_exif_to_bytes(input_data, metadata),
            "MP4" => self.write_mp4_exif_to_bytes(input_data, metadata),
            "MOV" => self.write_mov_exif_to_bytes(input_data, metadata),
            "3GP" => self.write_3gp_exif_to_bytes(input_data, metadata),
            "MKV" => self.write_mkv_exif_to_bytes(input_data, metadata),
            _ => Err(ExifError::UnsupportedFormat(format!(
                "EXIF writing not yet supported for format: {}",
                format
            )))
        }
    }

    /// Write EXIF metadata to JPEG bytes
    pub fn write_jpeg_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // Validate JPEG format
        if input_data.len() < 2 || input_data[0] != 0xFF || input_data[1] != 0xD8 {
            return Err(ExifError::InvalidExif("Invalid JPEG format".to_string()));
        }
        
        // Find existing EXIF segment
        let exif_segment = self.find_jpeg_exif_segment(input_data);
        
        // Create new EXIF data
        let new_exif_data = self.create_exif_segment(metadata)?;
        
        if let Some((start, end)) = exif_segment {
            // Replace existing EXIF segment
            let mut result = Vec::new();
            result.extend_from_slice(&input_data[..start]);
            result.extend_from_slice(&new_exif_data);
            result.extend_from_slice(&input_data[end..]);
            Ok(result)
        } else {
            // Insert new EXIF segment after SOI marker
            self.insert_jpeg_exif_segment(input_data, &new_exif_data)
        }
    }

    /// Write EXIF metadata to HEIF bytes
    pub fn write_heif_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // HEIF files use a different structure than JPEG
        // For now, we'll implement a basic approach that preserves the file structure
        // and adds EXIF data in a compatible way
        
        // Validate HEIF format
        if input_data.len() < 12 {
            return Err(ExifError::InvalidExif("Invalid HEIF format".to_string()));
        }
        
        // Check for HEIF signature - HEIF files start with ftyp box
        let is_heif = input_data.len() >= 8 && &input_data[4..8] == b"ftyp";
        
        if !is_heif {
            return Err(ExifError::InvalidExif("Not a valid HEIF file".to_string()));
        }
        
        // Check for HEIF brand identifiers
        let mut is_heif_brand = false;
        if input_data.len() >= 12 {
            // Check major brand (bytes 8-12)
            let major_brand = &input_data[8..12];
            is_heif_brand = major_brand == b"heic" || 
                           major_brand == b"heix" || 
                           major_brand == b"heim" || 
                           major_brand == b"heis" ||
                           major_brand == b"hevc" || 
                           major_brand == b"hevx" || 
                           major_brand == b"hevm" || 
                           major_brand == b"hevs";
        }
        
        if !is_heif_brand {
            return Err(ExifError::InvalidExif("Not a valid HEIF brand".to_string()));
        }
        
        // For HEIF files, we need to preserve the container structure
        // and add metadata in a HEIF-compliant way
        self.add_heif_metadata_atoms(input_data, metadata)
    }

    /// Write EXIF metadata to PNG bytes (placeholder - not yet implemented)
    pub fn write_png_exif_to_bytes(
        &self,
        _input_data: &[u8],
        _metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        Err(ExifError::UnsupportedFormat(
            "PNG EXIF writing not yet implemented".to_string()
        ))
    }

    /// Write EXIF metadata to RAW bytes
    pub fn write_raw_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // RAW files (CR2, NEF, ORF, DNG) use TIFF-based structure
        // Most RAW files have EXIF data embedded in TIFF format
        
        if input_data.len() < 8 {
            return Err(ExifError::InvalidExif("RAW file too small".to_string()));
        }
        
        // Detect RAW format
        let is_cr2 = input_data.starts_with(b"II*\0") || input_data.starts_with(b"MM\0*");
        let is_nef = input_data.starts_with(b"II*\0") || input_data.starts_with(b"MM\0*");
        let is_orf = input_data.starts_with(b"II*\0") || input_data.starts_with(b"MM\0*");
        let is_dng = input_data.starts_with(b"II*\0") || input_data.starts_with(b"MM\0*");
        
        if !(is_cr2 || is_nef || is_orf || is_dng) {
            return Err(ExifError::InvalidExif("Not a supported RAW format".to_string()));
        }
        
        // For RAW files, we need to find and replace the existing EXIF data
        // This is more complex as RAW files have multiple IFDs
        
        // Create new EXIF data
        let new_exif_data = self.create_exif_segment(metadata)?;
        
        // Find existing EXIF data in the RAW file
        // RAW files typically have EXIF data starting at offset 8 (after TIFF header)
        let mut result = Vec::new();
        
        if input_data.len() >= 8 {
            // Copy TIFF header (first 8 bytes)
            result.extend_from_slice(&input_data[..8]);
            
            // For now, we'll append the new EXIF data
            // In a full implementation, we'd need to properly parse and replace
            // the existing EXIF structure
            result.extend_from_slice(&new_exif_data);
            
            // Copy the rest of the file
            if input_data.len() > 8 {
                result.extend_from_slice(&input_data[8..]);
            }
        } else {
            return Err(ExifError::InvalidExif("RAW file too small".to_string()));
        }
        
        Ok(result)
    }

    /// Copy high-priority EXIF fields from source to target image
    pub fn copy_high_priority_exif(
        &self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> Result<(), ExifError> {
        // Read source image EXIF data using existing parser
        let source_metadata = self.read_exif_metadata(source_path)?;
        
        // Filter to high-priority fields only
        let high_priority_metadata = ExifUtils::filter_high_priority_fields(&source_metadata);
        
        if high_priority_metadata.is_empty() {
            return Err(ExifError::InvalidExif("No high-priority EXIF fields found in source".to_string()));
        }
        
        // Write filtered EXIF data to target image
        self.write_exif(target_path, output_path, &high_priority_metadata)
    }

    /// Read EXIF metadata from file using existing parser infrastructure
    fn read_exif_metadata(&self, _file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        // This is a simplified implementation that would use the existing parser
        // For now, return empty metadata - this would be replaced with actual parsing
        let metadata = HashMap::new();
        
        // TODO: Integrate with existing EXIF reading infrastructure
        // This would use the same parsers as FastExifReader
        
        Ok(metadata)
    }

    /// Copy high-priority EXIF fields from source bytes to target bytes
    pub fn copy_high_priority_exif_to_bytes(
        &self,
        source_data: &[u8],
        target_data: &[u8],
    ) -> Result<Vec<u8>, ExifError> {
        // Parse source EXIF data
        let source_metadata = self.parse_exif_from_bytes(source_data)?;
        
        // Filter to high-priority fields only
        let high_priority_metadata = ExifUtils::filter_high_priority_fields(&source_metadata);
        
        if high_priority_metadata.is_empty() {
            return Err(ExifError::InvalidExif("No high-priority EXIF fields found in source".to_string()));
        }
        
        // Write filtered EXIF data to target bytes
        self.write_jpeg_exif_to_bytes(target_data, &high_priority_metadata)
    }

    /// Parse EXIF data from bytes (simplified implementation)
    fn parse_exif_from_bytes(&self, _data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        // This is a simplified implementation for demonstration
        // In practice, you would use the existing EXIF parsing infrastructure
        let metadata = HashMap::new();
        
        // For now, return empty metadata - this would be replaced with actual parsing
        // using the existing TiffParser or other parsers in the codebase
        Ok(metadata)
    }

    /// Find JPEG EXIF segment (APP1 marker with EXIF)
    fn find_jpeg_exif_segment(&self, data: &[u8]) -> Option<(usize, usize)> {
        let mut pos = 0;
        
        while pos + 4 < data.len() {
            if data[pos] == 0xFF && data[pos + 1] == 0xE1 {
                // APP1 marker found
                let segment_length = ((data[pos + 2] as u16) << 8) | (data[pos + 3] as u16);
                
                if pos + 4 + segment_length as usize <= data.len() {
                    let segment_data = &data[pos + 4..pos + 4 + segment_length as usize];
                    
                    // Check if this is an EXIF segment
                    if segment_data.len() >= 6 && &segment_data[0..6] == b"Exif\0\0" {
                        return Some((pos, pos + 4 + segment_length as usize));
                    }
                }
            }
            
            // Move to next marker
            if data[pos] == 0xFF {
                pos += 1;
                if pos < data.len() && data[pos] != 0x00 {
                    // Skip marker data
                    if pos + 2 < data.len() {
                        let length = ((data[pos + 1] as u16) << 8) | (data[pos + 2] as u16);
                        pos += 2 + length as usize;
                    } else {
                        break;
                    }
                } else {
                    pos += 1;
                }
            } else {
                pos += 1;
            }
        }
        
        None
    }

    /// Insert EXIF segment into JPEG data
    fn insert_jpeg_exif_segment(
        &self,
        input_data: &[u8],
        exif_data: &[u8],
    ) -> Result<Vec<u8>, ExifError> {
        // Find SOI marker (0xFF 0xD8)
        let soi_pos = input_data.windows(2)
            .position(|w| w == [0xFF, 0xD8])
            .ok_or_else(|| ExifError::InvalidExif("SOI marker not found".to_string()))?;

        let mut result = Vec::new();
        
        // Copy SOI marker
        result.extend_from_slice(&input_data[soi_pos..soi_pos + 2]);
        
        // Insert EXIF segment
        result.extend_from_slice(exif_data);
        
        // Copy rest of the data
        result.extend_from_slice(&input_data[soi_pos + 2..]);
        
        Ok(result)
    }

    /// Create EXIF segment with metadata
    fn create_exif_segment(&self, metadata: &HashMap<String, String>) -> Result<Vec<u8>, ExifError> {
        crate::exif_encode::encode_jpeg_app1(self.little_endian, metadata)
    }

    /// Parse rational value from string (e.g., "1/60", "4.0", "50")
    fn parse_rational(&self, value: &str) -> Result<(u32, u32), ExifError> {
        if value.contains('/') {
            // Fraction format (e.g., "1/60")
            let parts: Vec<&str> = value.split('/').collect();
            if parts.len() == 2 {
                let numerator = parts[0].parse::<u32>()
                    .map_err(|_| ExifError::InvalidExif("Invalid numerator".to_string()))?;
                let denominator = parts[1].parse::<u32>()
                    .map_err(|_| ExifError::InvalidExif("Invalid denominator".to_string()))?;
                return Ok((numerator, denominator));
            }
        } else if let Ok(float_value) = value.parse::<f64>() {
            // Decimal format (e.g., "4.0", "50")
            if float_value.fract() == 0.0 {
                // Whole number
                return Ok((float_value as u32, 1));
            } else {
                // Convert to fraction
                let precision = 1000000; // 6 decimal places
                let numerator = (float_value * precision as f64) as u32;
                return Ok((numerator, precision));
            }
        }
        
        Err(ExifError::InvalidExif(format!("Invalid rational value: {}", value)))
    }

    /// Parse signed rational value from string (e.g., "-1/60", "4.0", "-50")
    fn parse_srational(&self, value: &str) -> Result<(u32, u32), ExifError> {
        if value.contains('/') {
            // Fraction format (e.g., "-1/60")
            let parts: Vec<&str> = value.split('/').collect();
            if parts.len() == 2 {
                let numerator = parts[0].parse::<i32>()
                    .map_err(|_| ExifError::InvalidExif("Invalid numerator".to_string()))?;
                let denominator = parts[1].parse::<u32>()
                    .map_err(|_| ExifError::InvalidExif("Invalid denominator".to_string()))?;
                // Convert signed to unsigned (two's complement)
                return Ok((numerator as u32, denominator));
            }
        } else if let Ok(float_value) = value.parse::<f64>() {
            // Decimal format (e.g., "4.0", "-50")
            if float_value.fract() == 0.0 {
                // Whole number
                return Ok((float_value as i32 as u32, 1));
            } else {
                // Convert to fraction
                let precision = 1000000; // 6 decimal places
                let numerator = (float_value * precision as f64) as i32 as u32;
                return Ok((numerator, precision));
            }
        }
        
        Err(ExifError::InvalidExif(format!("Invalid signed rational value: {}", value)))
    }
    
    /// Write EXIF metadata to MP4 bytes
    pub fn write_mp4_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // MP4 files use ISO Base Media File Format
        // We need to add metadata atoms to the MP4 structure
        
        // Validate MP4 format
        if input_data.len() < 8 {
            return Err(ExifError::InvalidExif("Invalid MP4 format".to_string()));
        }
        
        // Check for MP4 signature (ftyp atom)
        if input_data.len() < 8 || &input_data[4..8] != b"ftyp" {
            return Err(ExifError::InvalidExif("Not a valid MP4 file".to_string()));
        }
        
        // For MP4, we'll add metadata atoms (udta, meta, etc.)
        // This is a simplified implementation that preserves the file structure
        self.add_mp4_metadata_atoms(input_data, metadata)
    }
    
    /// Write EXIF metadata to MOV bytes
    pub fn write_mov_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // MOV files use QuickTime container format
        // Similar to MP4 but with some differences in atom structure
        
        // Validate MOV format
        if input_data.len() < 8 {
            return Err(ExifError::InvalidExif("Invalid MOV format".to_string()));
        }
        
        // Check for QuickTime signature
        if input_data.len() < 8 || &input_data[4..8] != b"ftyp" {
            return Err(ExifError::InvalidExif("Not a valid MOV file".to_string()));
        }
        
        // For MOV, we'll add metadata atoms similar to MP4
        self.add_mov_metadata_atoms(input_data, metadata)
    }
    
    /// Write EXIF metadata to 3GP bytes
    pub fn write_3gp_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // 3GP files use the same structure as MP4
        self.write_mp4_exif_to_bytes(input_data, metadata)
    }
    
    /// Write EXIF metadata to MKV bytes
    pub fn write_mkv_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // MKV files use Matroska container format (EBML)
        // This is more complex than MP4/MOV
        
        // Validate MKV format
        if input_data.len() < 4 {
            return Err(ExifError::InvalidExif("Invalid MKV format".to_string()));
        }
        
        // Check for Matroska signature (EBML header)
        if input_data.len() < 4 || &input_data[0..4] != b"\x1A\x45\xDF\xA3" {
            return Err(ExifError::InvalidExif("Not a valid MKV file".to_string()));
        }
        
        // For MKV, we'll add metadata elements to the EBML structure
        self.add_mkv_metadata_elements(input_data, metadata)
    }
    
    /// Add MP4 metadata atoms
    fn add_mp4_metadata_atoms(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // Create metadata atoms for MP4
        let mut result = Vec::new();
        
        // For now, we'll implement a basic approach that preserves the file
        // and adds a simple metadata atom
        result.extend_from_slice(input_data);
        
        // Add udta (user data) atom with metadata
        let udta_atom = self.create_mp4_udta_atom(metadata)?;
        result.extend_from_slice(&udta_atom);
        
        Ok(result)
    }
    
    /// Add MOV metadata atoms
    fn add_mov_metadata_atoms(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // Similar to MP4 but with QuickTime-specific atoms
        let mut result = Vec::new();
        result.extend_from_slice(input_data);
        
        // Add udta atom with metadata
        let udta_atom = self.create_mov_udta_atom(metadata)?;
        result.extend_from_slice(&udta_atom);
        
        Ok(result)
    }
    
    /// Add MKV metadata elements
    fn add_mkv_metadata_elements(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // MKV uses EBML format, which is more complex
        // For now, we'll implement a basic approach
        let mut result = Vec::new();
        result.extend_from_slice(input_data);
        
        // Add metadata elements to MKV
        let metadata_elements = self.create_mkv_metadata_elements(metadata)?;
        result.extend_from_slice(&metadata_elements);
        
        Ok(result)
    }
    
    /// Create MP4 udta (user data) atom
    fn create_mp4_udta_atom(&self, metadata: &HashMap<String, String>) -> Result<Vec<u8>, ExifError> {
        let mut atom = Vec::new();
        
        // udta atom header (size + type)
        atom.write_u32::<BigEndian>(0)?; // Size (will be calculated)
        atom.extend_from_slice(b"udta");
        
        // Add metadata atoms within udta
        if let Some(title) = metadata.get("Title") {
            let title_atom = self.create_mp4_text_atom(b"\xa9nam", title)?;
            atom.extend_from_slice(&title_atom);
        }
        
        if let Some(artist) = metadata.get("Artist") {
            let artist_atom = self.create_mp4_text_atom(b"\xa9ART", artist)?;
            atom.extend_from_slice(&artist_atom);
        }
        
        if let Some(description) = metadata.get("Description") {
            let desc_atom = self.create_mp4_text_atom(b"\xa9des", description)?;
            atom.extend_from_slice(&desc_atom);
        }
        
        if let Some(comment) = metadata.get("Comment") {
            let comment_atom = self.create_mp4_text_atom(b"\xa9cmt", comment)?;
            atom.extend_from_slice(&comment_atom);
        }
        
        if let Some(copyright) = metadata.get("Copyright") {
            let copyright_atom = self.create_mp4_text_atom(b"\xa9cpy", copyright)?;
            atom.extend_from_slice(&copyright_atom);
        }
        
        // Update size field
        let size = atom.len() as u32;
        atom[0..4].copy_from_slice(&size.to_be_bytes());
        
        Ok(atom)
    }
    
    /// Create MOV udta atom
    fn create_mov_udta_atom(&self, metadata: &HashMap<String, String>) -> Result<Vec<u8>, ExifError> {
        // Similar to MP4 but with QuickTime-specific text atoms
        self.create_mp4_udta_atom(metadata)
    }
    
    /// Create MKV metadata elements
    fn create_mkv_metadata_elements(&self, metadata: &HashMap<String, String>) -> Result<Vec<u8>, ExifError> {
        // MKV uses EBML format
        let mut elements = Vec::new();
        
        // Add metadata elements (simplified implementation)
        if let Some(title) = metadata.get("Title") {
            let title_element = self.create_mkv_text_element(0x7BA9, title)?;
            elements.extend_from_slice(&title_element);
        }
        
        if let Some(artist) = metadata.get("Artist") {
            let artist_element = self.create_mkv_text_element(0x5F91, artist)?;
            elements.extend_from_slice(&artist_element);
        }
        
        Ok(elements)
    }
    
    /// Create MP4 text atom
    fn create_mp4_text_atom(&self, atom_type: &[u8; 4], text: &str) -> Result<Vec<u8>, ExifError> {
        let mut atom = Vec::new();
        
        // Atom header
        let text_bytes = text.as_bytes();
        let size = 8 + text_bytes.len() as u32;
        atom.write_u32::<BigEndian>(size)?;
        atom.extend_from_slice(atom_type);
        
        // Text data
        atom.extend_from_slice(text_bytes);
        
        Ok(atom)
    }
    
    /// Create MKV text element
    fn create_mkv_text_element(&self, element_id: u32, text: &str) -> Result<Vec<u8>, ExifError> {
        let mut element = Vec::new();
        
        // EBML element header (simplified)
        let text_bytes = text.as_bytes();
        let size = text_bytes.len() as u32;
        
        // Element ID (variable length)
        element.write_u32::<BigEndian>(element_id)?;
        
        // Element size (variable length)
        element.write_u32::<BigEndian>(size)?;
        
        // Text data
        element.extend_from_slice(text_bytes);
        
        Ok(element)
    }
    
    /// Add HEIF metadata atoms
    fn add_heif_metadata_atoms(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // HEIF files use ISO Base Media File Format
        // We need to properly integrate metadata into the HEIF structure
        
        // For now, we'll implement a simplified approach that preserves the file
        // and adds metadata in a way that can be read back
        let mut result = Vec::new();
        result.extend_from_slice(input_data);
        
        // Add metadata as a custom atom at the end
        // This is a simplified approach - in a full implementation,
        // we would need to properly parse and modify the HEIF structure
        let metadata_atom = self.create_heif_metadata_atom(metadata)?;
        result.extend_from_slice(&metadata_atom);
        
        Ok(result)
    }
    
    /// Check if HEIF file has meta box
    fn has_meta_box(&self, data: &[u8]) -> bool {
        let mut pos = 0;
        while pos + 8 < data.len() {
            let size = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap_or([0; 4])) as usize;
            if size == 0 || size > data.len() {
                break;
            }
            
            let box_type = &data[pos + 4..pos + 8];
            if box_type == b"meta" {
                return true;
            }
            
            pos += size;
        }
        false
    }
    
    /// Create HEIF meta box with metadata
    fn create_heif_meta_box(&self, metadata: &HashMap<String, String>) -> Result<Vec<u8>, ExifError> {
        let mut meta_box = Vec::new();
        
        // Meta box header
        meta_box.write_u32::<BigEndian>(0)?; // Size (will be calculated)
        meta_box.extend_from_slice(b"meta");
        
        // Meta box version and flags
        meta_box.write_u32::<BigEndian>(0)?; // Version and flags
        
        // Add metadata atoms within meta box
        if let Some(title) = metadata.get("Title") {
            let title_atom = self.create_heif_text_atom(b"titl", title)?;
            meta_box.extend_from_slice(&title_atom);
        }
        
        if let Some(artist) = metadata.get("Artist") {
            let artist_atom = self.create_heif_text_atom(b"auth", artist)?;
            meta_box.extend_from_slice(&artist_atom);
        }
        
        if let Some(description) = metadata.get("Description") {
            let desc_atom = self.create_heif_text_atom(b"desc", description)?;
            meta_box.extend_from_slice(&desc_atom);
        }
        
        if let Some(comment) = metadata.get("Comment") {
            let comment_atom = self.create_heif_text_atom(b"cmnt", comment)?;
            meta_box.extend_from_slice(&comment_atom);
        }
        
        if let Some(copyright) = metadata.get("Copyright") {
            let copyright_atom = self.create_heif_text_atom(b"cprt", copyright)?;
            meta_box.extend_from_slice(&copyright_atom);
        }
        
        // Update size field
        let size = meta_box.len() as u32;
        meta_box[0..4].copy_from_slice(&size.to_be_bytes());
        
        Ok(meta_box)
    }
    
    /// Update existing HEIF meta box
    fn update_heif_meta_box(
        &self,
        input_data: &[u8],
        metadata: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ExifError> {
        // For now, we'll implement a simple approach that preserves the file
        // and adds metadata atoms at the end
        let mut result = Vec::new();
        result.extend_from_slice(input_data);
        
        // Add metadata atoms
        let meta_box = self.create_heif_meta_box(metadata)?;
        result.extend_from_slice(&meta_box);
        
        Ok(result)
    }
    
    /// Create HEIF text atom
    fn create_heif_text_atom(&self, atom_type: &[u8; 4], text: &str) -> Result<Vec<u8>, ExifError> {
        let mut atom = Vec::new();
        
        // Atom header
        let text_bytes = text.as_bytes();
        let size = 8 + text_bytes.len() as u32;
        atom.write_u32::<BigEndian>(size)?;
        atom.extend_from_slice(atom_type);
        
        // Text data
        atom.extend_from_slice(text_bytes);
        
        Ok(atom)
    }
    
    /// Create HEIF metadata atom
    fn create_heif_metadata_atom(&self, metadata: &HashMap<String, String>) -> Result<Vec<u8>, ExifError> {
        let mut atom = Vec::new();
        
        // Create a custom metadata atom with our data
        // This is a simplified approach for demonstration
        let mut metadata_bytes = Vec::new();
        
        for (key, value) in metadata {
            metadata_bytes.extend_from_slice(key.as_bytes());
            metadata_bytes.push(0); // null separator
            metadata_bytes.extend_from_slice(value.as_bytes());
            metadata_bytes.push(0); // null separator
        }
        
        // Atom header
        let size = 8 + metadata_bytes.len() as u32;
        atom.write_u32::<BigEndian>(size)?;
        atom.extend_from_slice(b"meta"); // Custom metadata atom type
        
        // Metadata data
        atom.extend_from_slice(&metadata_bytes);
        
        Ok(atom)
    }
}

impl Default for ExifWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_rational() {
        let writer = ExifWriter::new();
        
        // Test fraction format
        assert_eq!(writer.parse_rational("1/60").unwrap(), (1, 60));
        assert_eq!(writer.parse_rational("4/1").unwrap(), (4, 1));
        
        // Test decimal format
        assert_eq!(writer.parse_rational("4.0").unwrap(), (4, 1));
        assert_eq!(writer.parse_rational("50").unwrap(), (50, 1));
        
        // Test decimal with fraction
        let (num, den) = writer.parse_rational("1.5").unwrap();
        assert_eq!(num, 1500000);
        assert_eq!(den, 1000000);
    }

    #[test]
    fn test_create_exif_segment() {
        let writer = ExifWriter::new();
        let mut metadata = HashMap::new();
        metadata.insert("Make".to_string(), "Canon".to_string());
        metadata.insert("Model".to_string(), "EOS 70D".to_string());
        metadata.insert("FocalLength".to_string(), "77.0 mm".to_string());
        
        let exif_data = writer.create_exif_segment(&metadata).unwrap();
        
        // Check basic structure
        assert!(exif_data.len() > 64);
        assert_eq!(&exif_data[0..2], [0xFF, 0xE1]); // APP1 marker
        assert_eq!(&exif_data[4..10], b"Exif\0\0"); // EXIF signature
    }

    #[test]
    fn write_then_read_roundtrip_jpeg() {
        let jpeg = [0xFF, 0xD8, 0xFF, 0xD9];
        let mut metadata = HashMap::new();
        metadata.insert("Make".to_string(), "NIKON CORPORATION".to_string());
        metadata.insert("Model".to_string(), "NIKON Z50_2".to_string());
        metadata.insert("ModifyDate".to_string(), "2026:05:10 15:03:55".to_string());
        metadata.insert("DateTimeOriginal".to_string(), "2026:05:10 15:03:55".to_string());
        metadata.insert("CreateDate".to_string(), "2026:05:10 15:03:55".to_string());
        metadata.insert("ISO".to_string(), "500".to_string());
        metadata.insert("ExposureTime".to_string(), "1/1250".to_string());
        metadata.insert("FNumber".to_string(), "9.0".to_string());
        metadata.insert("FocalLength".to_string(), "77.0 mm".to_string());
        metadata.insert("Artist".to_string(), "Jedediah Frey".to_string());
        metadata.insert("Copyright".to_string(), "Jedediah Frey".to_string());
        metadata.insert("OffsetTimeOriginal".to_string(), "-05:00".to_string());
        metadata.insert("SubSecTimeOriginal".to_string(), "95".to_string());
        metadata.insert("LensModel".to_string(), "NIKKOR Z DX 50-250mm f/4.5-6.3 VR".to_string());
        metadata.insert("SerialNumber".to_string(), "3016339".to_string());
        metadata.insert("Orientation".to_string(), "Horizontal (normal)".to_string());
        metadata.insert("Flash".to_string(), "Off, Did not fire".to_string());

        let writer = ExifWriter::new();
        let written = writer.write_jpeg_exif_to_bytes(&jpeg, &metadata).unwrap();
        let mut reader = crate::FastExifReader::new();
        let back = reader.read_bytes(&written).unwrap();

        assert_eq!(back.get("Make").unwrap(), "NIKON CORPORATION");
        assert_eq!(back.get("Model").unwrap(), "NIKON Z50_2");
        assert_eq!(back.get("DateTimeOriginal").unwrap(), "2026:05:10 15:03:55");
        assert_eq!(back.get("CreateDate").unwrap(), "2026:05:10 15:03:55");
        assert_eq!(back.get("ISO").unwrap(), "500");
        assert_eq!(back.get("Artist").unwrap(), "Jedediah Frey");
        assert_eq!(back.get("LensModel").unwrap(), "NIKKOR Z DX 50-250mm f/4.5-6.3 VR");
        assert_eq!(back.get("SerialNumber").unwrap(), "3016339");
        assert_eq!(
            back.get("SubSecDateTimeOriginal").unwrap(),
            "2026:05:10 15:03:55.95-05:00"
        );
    }
}
