use crate::types::ExifError;
use crate::utils::ExifUtils;
use crate::format_detection::FormatDetector;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use byteorder::{LittleEndian, BigEndian, WriteBytesExt};

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
        let mut exif_data = Vec::new();
        
        // APP1 marker (0xFF 0xE1)
        exif_data.write_u8(0xFF)?;
        exif_data.write_u8(0xE1)?;
        
        // Calculate segment length (will be updated later)
        let length_pos = exif_data.len();
        exif_data.write_u16::<BigEndian>(0)?; // Placeholder for length
        
        // EXIF signature
        exif_data.extend_from_slice(b"Exif\0\0");
        
        // TIFF header
        let _tiff_header_pos = exif_data.len();
        if self.little_endian {
            exif_data.extend_from_slice(b"II"); // Little-endian
        } else {
            exif_data.extend_from_slice(b"MM"); // Big-endian
        }
        
        // TIFF version (42)
        if self.little_endian {
            exif_data.write_u16::<LittleEndian>(42)?;
        } else {
            exif_data.write_u16::<BigEndian>(42)?;
        }
        
        // IFD offset (will be updated later)
        let ifd_offset_pos = exif_data.len();
        if self.little_endian {
            exif_data.write_u32::<LittleEndian>(0)?; // Placeholder
        } else {
            exif_data.write_u32::<BigEndian>(0)?; // Placeholder
        }
        
        // Create IFD entries
        let (ifd_data, value_data) = self.create_ifd_entries(metadata)?;
        
        // Update IFD offset (relative to TIFF header start)
        let tiff_header_start = 8; // After "Exif\0\0"
        let ifd_offset = (exif_data.len() - tiff_header_start) as u32;
        if self.little_endian {
            exif_data[ifd_offset_pos..ifd_offset_pos + 4].copy_from_slice(&ifd_offset.to_le_bytes());
        } else {
            exif_data[ifd_offset_pos..ifd_offset_pos + 4].copy_from_slice(&ifd_offset.to_be_bytes());
        }
        
        // Add IFD data
        exif_data.extend_from_slice(&ifd_data);
        
        // Add value data
        exif_data.extend_from_slice(&value_data);
        
        // Update segment length
        let segment_length = (exif_data.len() - 2) as u16; // Exclude APP1 marker
        exif_data[length_pos..length_pos + 2].copy_from_slice(&segment_length.to_be_bytes());
        
        Ok(exif_data)
    }

    /// Create IFD entries for comprehensive EXIF fields
    fn create_ifd_entries(&self, metadata: &HashMap<String, String>) -> Result<(Vec<u8>, Vec<u8>), ExifError> {
        let mut ifd_data = Vec::new();
        let mut value_data = Vec::new();
        
        // Count of directory entries
        let mut entries = Vec::new();
        
        // Comprehensive EXIF field mapping based on exiftool compatibility
        let exif_fields = [
            // Basic image information (IFD0)
            ("ImageDescription", 0x010E, 2), // ASCII
            ("Make", 0x010F, 2), // ASCII
            ("Model", 0x0110, 2), // ASCII
            ("Orientation", 0x0112, 3), // SHORT
            ("XResolution", 0x011A, 5), // RATIONAL
            ("YResolution", 0x011B, 5), // RATIONAL
            ("ResolutionUnit", 0x0128, 3), // SHORT
            ("Software", 0x0131, 2), // ASCII
            ("DateTime", 0x0132, 2), // ASCII
            ("Artist", 0x013B, 2), // ASCII
            ("WhitePoint", 0x013E, 5), // RATIONAL
            ("PrimaryChromaticities", 0x013F, 5), // RATIONAL
            ("YCbCrCoefficients", 0x0211, 5), // RATIONAL
            ("YCbCrSubSampling", 0x0212, 3), // SHORT
            ("YCbCrPositioning", 0x0213, 3), // SHORT
            ("ReferenceBlackWhite", 0x0214, 5), // RATIONAL
            ("Copyright", 0x8298, 2), // ASCII
            
            // EXIF-specific fields (ExifIFD)
            ("ExposureTime", 0x829A, 5), // RATIONAL
            ("FNumber", 0x829D, 5), // RATIONAL
            ("ExposureProgram", 0x8822, 3), // SHORT
            ("SpectralSensitivity", 0x8824, 2), // ASCII
            ("ISOSpeedRatings", 0x8827, 3), // SHORT
            ("OECF", 0x8828, 7), // UNDEFINED
            ("ExifVersion", 0x9000, 7), // UNDEFINED
            ("DateTimeOriginal", 0x9003, 2), // ASCII
            ("DateTimeDigitized", 0x9004, 2), // ASCII
            ("ComponentsConfiguration", 0x9101, 7), // UNDEFINED
            ("CompressedBitsPerPixel", 0x9102, 5), // RATIONAL
            ("BrightnessValue", 0x9203, 10), // SRATIONAL
            ("ExposureBiasValue", 0x9204, 10), // SRATIONAL
            ("MaxApertureValue", 0x9205, 5), // RATIONAL
            ("SubjectDistance", 0x9206, 5), // RATIONAL
            ("MeteringMode", 0x9207, 3), // SHORT
            ("LightSource", 0x9208, 3), // SHORT
            ("Flash", 0x9209, 3), // SHORT
            ("FocalLength", 0x920A, 5), // RATIONAL
            ("SubjectArea", 0x9214, 3), // SHORT
            ("MakerNote", 0x927C, 7), // UNDEFINED
            ("UserComment", 0x9286, 7), // UNDEFINED
            ("SubSecTime", 0x9290, 2), // ASCII
            ("SubSecTimeOriginal", 0x9291, 2), // ASCII
            ("SubSecTimeDigitized", 0x9292, 2), // ASCII
            ("FlashpixVersion", 0xA000, 7), // UNDEFINED
            ("ColorSpace", 0xA001, 3), // SHORT
            ("PixelXDimension", 0xA002, 4), // LONG
            ("PixelYDimension", 0xA003, 4), // LONG
            ("RelatedSoundFile", 0xA004, 2), // ASCII
            ("InteropIndex", 0xA005, 2), // ASCII
            ("InteropVersion", 0xA006, 7), // UNDEFINED
            ("RelatedImageFileFormat", 0xA100, 2), // ASCII
            ("RelatedImageWidth", 0xA101, 3), // SHORT
            ("RelatedImageLength", 0xA102, 3), // SHORT
            ("ExposureIndex", 0xA215, 5), // RATIONAL
            ("SensingMethod", 0xA217, 3), // SHORT
            ("FileSource", 0xA300, 7), // UNDEFINED
            ("SceneType", 0xA301, 7), // UNDEFINED
            ("CFAPattern", 0xA302, 7), // UNDEFINED
            ("CustomRendered", 0xA401, 3), // SHORT
            ("ExposureMode", 0xA402, 3), // SHORT
            ("WhiteBalance", 0xA403, 3), // SHORT
            ("DigitalZoomRatio", 0xA404, 5), // RATIONAL
            ("FocalLengthIn35mmFilm", 0xA405, 3), // SHORT
            ("SceneCaptureType", 0xA406, 3), // SHORT
            ("GainControl", 0xA407, 3), // SHORT
            ("Contrast", 0xA408, 3), // SHORT
            ("Saturation", 0xA409, 3), // SHORT
            ("Sharpness", 0xA40A, 3), // SHORT
            ("DeviceSettingDescription", 0xA40B, 7), // UNDEFINED
            ("SubjectDistanceRange", 0xA40C, 3), // SHORT
            ("ImageUniqueID", 0xA420, 2), // ASCII
            ("CameraOwnerName", 0xA430, 2), // ASCII
            ("BodySerialNumber", 0xA431, 2), // ASCII
            ("LensSpecification", 0xA432, 5), // RATIONAL
            ("LensMake", 0xA433, 2), // ASCII
            ("LensModel", 0xA434, 2), // ASCII
            ("LensSerialNumber", 0xA435, 2), // ASCII
            
            // GPS fields (GPS IFD)
            ("GPSVersionID", 0x0000, 1), // BYTE
            ("GPSLatitudeRef", 0x0001, 2), // ASCII
            ("GPSLatitude", 0x0002, 5), // RATIONAL
            ("GPSLongitudeRef", 0x0003, 2), // ASCII
            ("GPSLongitude", 0x0004, 5), // RATIONAL
            ("GPSAltitudeRef", 0x0005, 1), // BYTE
            ("GPSAltitude", 0x0006, 5), // RATIONAL
            ("GPSTimeStamp", 0x0007, 5), // RATIONAL
            ("GPSSatellites", 0x0008, 2), // ASCII
            ("GPSStatus", 0x0009, 2), // ASCII
            ("GPSMeasureMode", 0x000A, 2), // ASCII
            ("GPSDOP", 0x000B, 5), // RATIONAL
            ("GPSSpeedRef", 0x000C, 2), // ASCII
            ("GPSSpeed", 0x000D, 5), // RATIONAL
            ("GPSTrackRef", 0x000E, 2), // ASCII
            ("GPSTrack", 0x000F, 5), // RATIONAL
            ("GPSImgDirectionRef", 0x0010, 2), // ASCII
            ("GPSImgDirection", 0x0011, 5), // RATIONAL
            ("GPSMapDatum", 0x0012, 2), // ASCII
            ("GPSDestLatitudeRef", 0x0013, 2), // ASCII
            ("GPSDestLatitude", 0x0014, 5), // RATIONAL
            ("GPSDestLongitudeRef", 0x0015, 2), // ASCII
            ("GPSDestLongitude", 0x0016, 5), // RATIONAL
            ("GPSDestBearingRef", 0x0017, 2), // ASCII
            ("GPSDestBearing", 0x0018, 5), // RATIONAL
            ("GPSDestDistanceRef", 0x0019, 2), // ASCII
            ("GPSDestDistance", 0x001A, 5), // RATIONAL
            ("GPSProcessingMethod", 0x001B, 7), // UNDEFINED
            ("GPSAreaInformation", 0x001C, 7), // UNDEFINED
            ("GPSDateStamp", 0x001D, 2), // ASCII
            ("GPSDifferential", 0x001E, 3), // SHORT
            
            // Additional common fields
            ("OffsetTime", 0x9010, 2), // ASCII
            ("OffsetTimeOriginal", 0x9011, 2), // ASCII
            ("OffsetTimeDigitized", 0x9012, 2), // ASCII
            ("ShutterSpeedValue", 0x9201, 10), // SRATIONAL
            ("ApertureValue", 0x9202, 5), // RATIONAL
        ];
        
        for (field_name, tag_id, data_type) in exif_fields.iter() {
            if let Some(value) = metadata.get(*field_name) {
                if let Some(entry) = self.create_ifd_entry(
                    *tag_id,
                    *data_type,
                    value,
                    &mut value_data,
                )? {
                    entries.push(entry);
                }
            }
        }
        
        // Write entry count
        let entry_count = entries.len();
        if self.little_endian {
            ifd_data.write_u16::<LittleEndian>(entry_count as u16)?;
        } else {
            ifd_data.write_u16::<BigEndian>(entry_count as u16)?;
        }
        
        // Write entries
        for entry in entries {
            ifd_data.extend_from_slice(&entry);
        }
        
        // Next IFD offset (0 for last IFD)
        if self.little_endian {
            ifd_data.write_u32::<LittleEndian>(0)?;
        } else {
            ifd_data.write_u32::<BigEndian>(0)?;
        }
        
        Ok((ifd_data, value_data))
    }

    /// Create a single IFD entry
    fn create_ifd_entry(
        &self,
        tag_id: u16,
        data_type: u16,
        value: &str,
        value_data: &mut Vec<u8>,
    ) -> Result<Option<Vec<u8>>, ExifError> {
        let mut entry = Vec::new();
        
        // Tag ID
        if self.little_endian {
            entry.write_u16::<LittleEndian>(tag_id)?;
        } else {
            entry.write_u16::<BigEndian>(tag_id)?;
        }
        
        // Data type
        if self.little_endian {
            entry.write_u16::<LittleEndian>(data_type)?;
        } else {
            entry.write_u16::<BigEndian>(data_type)?;
        }
        
        // Count and value/offset
        match data_type {
            1 => {
                // BYTE
                if let Ok(byte_value) = value.parse::<u8>() {
                    if self.little_endian {
                        entry.write_u32::<LittleEndian>(1)?; // Count
                        entry.write_u32::<LittleEndian>(byte_value as u32)?; // Value
                    } else {
                        entry.write_u32::<BigEndian>(1)?; // Count
                        entry.write_u32::<BigEndian>(byte_value as u32)?; // Value
                    }
                } else {
                    return Ok(None); // Skip invalid values
                }
            }
            2 => {
                // ASCII
                let value_bytes = value.as_bytes();
                let count = value_bytes.len() + 1; // +1 for null terminator
                
                if self.little_endian {
                    entry.write_u32::<LittleEndian>(count as u32)?;
                } else {
                    entry.write_u32::<BigEndian>(count as u32)?;
                }
                
                if count <= 4 {
                    // Value fits in 4 bytes
                    let mut value_bytes_padded = [0u8; 4];
                    value_bytes_padded[..value_bytes.len()].copy_from_slice(value_bytes);
                    entry.extend_from_slice(&value_bytes_padded);
                } else {
                    // Value stored at offset
                    let offset = value_data.len() as u32;
                    if self.little_endian {
                        entry.write_u32::<LittleEndian>(offset)?;
                    } else {
                        entry.write_u32::<BigEndian>(offset)?;
                    }
                    
                    // Add value to value data
                    value_data.extend_from_slice(value_bytes);
                    value_data.push(0); // Null terminator
                }
            }
            3 => {
                // SHORT
                if let Ok(short_value) = value.parse::<u16>() {
                    if self.little_endian {
                        entry.write_u32::<LittleEndian>(1)?; // Count
                        entry.write_u32::<LittleEndian>(short_value as u32)?; // Value
                    } else {
                        entry.write_u32::<BigEndian>(1)?; // Count
                        entry.write_u32::<BigEndian>(short_value as u32)?; // Value
                    }
                } else {
                    return Ok(None); // Skip invalid values
                }
            }
            4 => {
                // LONG
                if let Ok(long_value) = value.parse::<u32>() {
                    if self.little_endian {
                        entry.write_u32::<LittleEndian>(1)?; // Count
                        entry.write_u32::<LittleEndian>(long_value)?; // Value
                    } else {
                        entry.write_u32::<BigEndian>(1)?; // Count
                        entry.write_u32::<BigEndian>(long_value)?; // Value
                    }
                } else {
                    return Ok(None); // Skip invalid values
                }
            }
            5 => {
                // RATIONAL
                if let Ok(rational_value) = self.parse_rational(value) {
                    if self.little_endian {
                        entry.write_u32::<LittleEndian>(1)?; // Count
                        entry.write_u32::<LittleEndian>(value_data.len() as u32)?; // Offset
                    } else {
                        entry.write_u32::<BigEndian>(1)?; // Count
                        entry.write_u32::<BigEndian>(value_data.len() as u32)?; // Offset
                    }
                    
                    // Add rational value to value data
                    if self.little_endian {
                        value_data.write_u32::<LittleEndian>(rational_value.0)?;
                        value_data.write_u32::<LittleEndian>(rational_value.1)?;
                    } else {
                        value_data.write_u32::<BigEndian>(rational_value.0)?;
                        value_data.write_u32::<BigEndian>(rational_value.1)?;
                    }
                } else {
                    return Ok(None); // Skip invalid values
                }
            }
            7 => {
                // UNDEFINED
                let value_bytes = value.as_bytes();
                let count = value_bytes.len();
                
                if self.little_endian {
                    entry.write_u32::<LittleEndian>(count as u32)?;
                } else {
                    entry.write_u32::<BigEndian>(count as u32)?;
                }
                
                if count <= 4 {
                    // Value fits in 4 bytes
                    let mut value_bytes_padded = [0u8; 4];
                    value_bytes_padded[..value_bytes.len()].copy_from_slice(value_bytes);
                    entry.extend_from_slice(&value_bytes_padded);
                } else {
                    // Value stored at offset
                    let offset = value_data.len() as u32;
                    if self.little_endian {
                        entry.write_u32::<LittleEndian>(offset)?;
                    } else {
                        entry.write_u32::<BigEndian>(offset)?;
                    }
                    
                    // Add value to value data
                    value_data.extend_from_slice(value_bytes);
                }
            }
            10 => {
                // SRATIONAL (signed rational)
                if let Ok(rational_value) = self.parse_srational(value) {
                    if self.little_endian {
                        entry.write_u32::<LittleEndian>(1)?; // Count
                        entry.write_u32::<LittleEndian>(value_data.len() as u32)?; // Offset
                    } else {
                        entry.write_u32::<BigEndian>(1)?; // Count
                        entry.write_u32::<BigEndian>(value_data.len() as u32)?; // Offset
                    }
                    
                    // Add signed rational value to value data
                    if self.little_endian {
                        value_data.write_u32::<LittleEndian>(rational_value.0)?;
                        value_data.write_u32::<LittleEndian>(rational_value.1)?;
                    } else {
                        value_data.write_u32::<BigEndian>(rational_value.0)?;
                        value_data.write_u32::<BigEndian>(rational_value.1)?;
                    }
                } else {
                    return Ok(None); // Skip invalid values
                }
            }
            _ => {
                return Ok(None); // Unsupported data type
            }
        }
        
        Ok(Some(entry))
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
        metadata.insert("DateTime".to_string(), "2023:12:25 12:00:00".to_string());
        
        let exif_data = writer.create_exif_segment(&metadata).unwrap();
        
        // Check basic structure
        assert!(exif_data.len() > 100);
        assert_eq!(&exif_data[0..2], [0xFF, 0xE1]); // APP1 marker
        assert_eq!(&exif_data[4..10], b"Exif\0\0"); // EXIF signature
    }
}
