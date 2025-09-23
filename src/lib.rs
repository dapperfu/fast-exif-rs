use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use memmap2::Mmap;
use thiserror::Error;
use pyo3::types::PyBytes;

#[derive(Error, Debug)]
pub enum ExifError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Invalid EXIF data: {0}")]
    InvalidExif(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<ExifError> for PyErr {
    fn from(err: ExifError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string())
    }
}

/// Fast EXIF reader optimized for Canon 70D and Nikon Z50 II
#[pyclass]
#[derive(Clone)]
pub struct FastExifReader {
    // Pre-allocated buffers for performance
    buffer: Vec<u8>,
}

#[pymethods]
impl FastExifReader {
    #[new]
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(1024 * 1024), // 1MB buffer
        }
    }

    /// Read EXIF data from file path
    fn read_file(&mut self, file_path: &str) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let metadata = self.read_exif_fast(file_path)?;
            Ok(metadata.into_py(py))
        })
    }

    /// Read EXIF data from bytes
    fn read_bytes(&mut self, data: &[u8]) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let metadata = self.read_exif_from_bytes(data)?;
            Ok(metadata.into_py(py))
        })
    }

    /// Support for pickle protocol
    fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        // Serialize the buffer as bytes
        let buffer_bytes = PyBytes::new(py, &self.buffer);
        Ok(buffer_bytes.into())
    }

    /// Support for pickle protocol
    fn __setstate__(&mut self, py: Python, state: PyObject) -> PyResult<()> {
        // Deserialize the buffer from bytes
        let buffer_bytes: &PyBytes = state.extract(py)?;
        self.buffer = buffer_bytes.as_bytes().to_vec();
        Ok(())
    }
}

impl FastExifReader {
    fn read_exif_fast(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        self.read_exif_from_bytes(&mmap)
    }

    fn read_exif_from_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();
        
        // Detect file format
        let format = self.detect_format(data)?;
        metadata.insert("Format".to_string(), format.clone());
        
        // Parse EXIF based on format
        match format.as_str() {
            "JPEG" => self.parse_jpeg_exif(data, &mut metadata)?,
            "CR2" => self.parse_cr2_exif(data, &mut metadata)?,
            "NEF" => self.parse_nef_exif(data, &mut metadata)?,
            "HEIF" | "HIF" => self.parse_heif_exif(data, &mut metadata)?,
            _ => return Err(ExifError::UnsupportedFormat(format)),
        }
        
        Ok(metadata)
    }
    
    fn detect_format(&self, data: &[u8]) -> Result<String, ExifError> {
        if data.len() < 4 {
            return Err(ExifError::InvalidExif("File too small".to_string()));
        }
        
        // Check for JPEG
        if data[0] == 0xFF && data[1] == 0xD8 {
            return Ok("JPEG".to_string());
        }
        
        // Check for TIFF/CR2 (Canon RAW)
        if (data[0] == 0x49 && data[1] == 0x49) || (data[0] == 0x4D && data[1] == 0x4D) {
            // Check if it's CR2 by looking for Canon-specific markers
            if self.is_canon_cr2(data) {
                return Ok("CR2".to_string());
            } else if self.is_nikon_nef(data) {
                return Ok("NEF".to_string());
            } else {
                return Ok("TIFF".to_string());
            }
        }
        
        // Check for HEIF/HIF
        if data.len() >= 12 {
            let header = &data[4..12];
            if header == b"ftypheic" || header == b"ftypheix" || header == b"ftypmif1" {
                return Ok("HEIF".to_string());
            }
        }
        
        Err(ExifError::UnsupportedFormat("Unknown format".to_string()))
    }
    
    fn is_canon_cr2(&self, data: &[u8]) -> bool {
        // CR2 files have Canon-specific markers
        // Look for "Canon" in the first 1KB
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(5).any(|w| w == b"Canon")
    }
    
    fn is_nikon_nef(&self, data: &[u8]) -> bool {
        // NEF files have Nikon-specific markers
        // Look for "Nikon" in the first 1KB
        let search_len = std::cmp::min(1024, data.len());
        data[..search_len].windows(5).any(|w| w == b"Nikon")
    }
    
    fn parse_jpeg_exif(&self, data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // Find EXIF segment in JPEG
        if let Some(exif_data) = self.find_jpeg_exif_segment(data) {
            self.parse_tiff_exif(exif_data, metadata)?;
        } else {
            return Err(ExifError::InvalidExif("No EXIF segment found".to_string()));
        }
        Ok(())
    }
    
    fn parse_cr2_exif(&self, data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // CR2 is TIFF-based
        self.parse_tiff_exif(data, metadata)?;
        self.extract_canon_specific_tags(data, metadata);
        Ok(())
    }
    
    fn parse_nef_exif(&self, data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // NEF is TIFF-based
        self.parse_tiff_exif(data, metadata)?;
        self.extract_nikon_specific_tags(data, metadata);
        Ok(())
    }
    
    fn parse_heif_exif(&self, data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        // HEIF/HIF files are based on QuickTime/MOV container format
        // They use ISO Base Media File Format (ISO 14496-12)
        metadata.insert("Format".to_string(), "HEIF".to_string());
        
        // Parse HEIF atoms to find EXIF data
        if let Some(exif_data) = self.find_heif_exif_atom(data) {
            self.parse_tiff_exif(exif_data, metadata)?;
        } else {
            // If no EXIF atom found, try to extract basic HEIF metadata
            self.extract_heif_basic_metadata(data, metadata);
        }
        
        Ok(())
    }
    
    fn find_jpeg_exif_segment<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        // Look for APP1 segment (0xFFE1) containing EXIF
        let mut pos = 2;
        
        while pos < data.len().saturating_sub(6) {
            if data[pos] == 0xFF && data[pos + 1] == 0xE1 {
                // Read segment length (big-endian)
                let length = ((data[pos + 2] as u16) << 8) | (data[pos + 3] as u16);
                let segment_end = pos + 2 + length as usize;
                
                if segment_end > data.len() {
                    break;
                }
                
                // Look for "Exif" identifier anywhere in the segment
                let segment_start = pos + 4;
                for exif_start in segment_start..segment_end.saturating_sub(4) {
                    if &data[exif_start..exif_start + 4] == b"Exif" {
                        // Found EXIF identifier, return the data after it
                        let exif_data_start = exif_start + 4;
                        if exif_data_start < segment_end {
                            return Some(&data[exif_data_start..segment_end]);
                        }
                    }
                }
                
                // Move to next segment
                pos = segment_end;
            } else {
                pos += 1;
            }
        }
        
        None
    }
    
    fn parse_tiff_exif(&self, data: &[u8], metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
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
        
        // Skip TIFF version (should be 42)
        if tiff_start + 8 > data.len() {
            return Err(ExifError::InvalidExif("TIFF header incomplete".to_string()));
        }
        
        // Read IFD0 offset
        let ifd0_offset = if is_little_endian {
            ((data[tiff_start + 4] as u32) | ((data[tiff_start + 5] as u32) << 8) | ((data[tiff_start + 6] as u32) << 16) | ((data[tiff_start + 7] as u32) << 24)) as usize
        } else {
            (((data[tiff_start + 4] as u32) << 24) | ((data[tiff_start + 5] as u32) << 16) | ((data[tiff_start + 6] as u32) << 8) | (data[tiff_start + 7] as u32)) as usize
        };
        
        // Parse IFD0 - offset is relative to TIFF header start
        self.parse_ifd(data, tiff_start + ifd0_offset, is_little_endian, tiff_start, metadata)?;
        
        Ok(())
    }
    
    fn parse_ifd(&self, data: &[u8], offset: usize, is_little_endian: bool, tiff_start: usize, metadata: &mut HashMap<String, String>) -> Result<(), ExifError> {
        if offset >= data.len() {
            return Ok(());
        }
        
        let ifd_data = &data[offset..];
        if ifd_data.len() < 2 {
            return Ok(());
        }
        
        // Read entry count
        let entry_count = if is_little_endian {
            ((ifd_data[0] as u16) | ((ifd_data[1] as u16) << 8)) as usize
        } else {
            (((ifd_data[0] as u16) << 8) | (ifd_data[1] as u16)) as usize
        };
        
        let mut current = 2;
        
        for _ in 0..entry_count {
            if current + 12 > ifd_data.len() {
                break;
            }
            
            let entry_data = &ifd_data[current..current + 12];
            
            let tag = if is_little_endian {
                ((entry_data[0] as u16) | ((entry_data[1] as u16) << 8))
            } else {
                (((entry_data[0] as u16) << 8) | (entry_data[1] as u16))
            };
            
            let format = if is_little_endian {
                ((entry_data[2] as u16) | ((entry_data[3] as u16) << 8))
            } else {
                (((entry_data[2] as u16) << 8) | (entry_data[3] as u16))
            };
            
            let count = if is_little_endian {
                ((entry_data[4] as u32) | ((entry_data[5] as u32) << 8) | ((entry_data[6] as u32) << 16) | ((entry_data[7] as u32) << 24))
            } else {
                (((entry_data[4] as u32) << 24) | ((entry_data[5] as u32) << 16) | ((entry_data[6] as u32) << 8) | (entry_data[7] as u32))
            };
            
            let value_offset = if is_little_endian {
                ((entry_data[8] as u32) | ((entry_data[9] as u32) << 8) | ((entry_data[10] as u32) << 16) | ((entry_data[11] as u32) << 24))
            } else {
                (((entry_data[8] as u32) << 24) | ((entry_data[9] as u32) << 16) | ((entry_data[10] as u32) << 8) | (entry_data[11] as u32))
            };
            
            // Extract common tags - value_offset is relative to TIFF header start
            self.extract_tag_value(tag, format, count, value_offset, data, is_little_endian, tiff_start, metadata);
            
            current += 12;
        }
        
        Ok(())
    }
    
    fn extract_tag_value(&self, tag: u16, format: u16, count: u32, value_offset: u32, data: &[u8], is_little_endian: bool, tiff_start: usize, metadata: &mut HashMap<String, String>) {
        match tag {
            // Basic camera info
            0x010F => { // Make
                if let Some(value) = self.read_string_value(data, tiff_start + value_offset as usize, count as usize) {
                    metadata.insert("Make".to_string(), value);
                }
            },
            0x0110 => { // Model
                if let Some(value) = self.read_string_value(data, tiff_start + value_offset as usize, count as usize) {
                    metadata.insert("Model".to_string(), value);
                }
            },
            0x0132 => { // DateTime
                if let Some(value) = self.read_string_value(data, tiff_start + value_offset as usize, count as usize) {
                    metadata.insert("DateTime".to_string(), value);
                }
            },
            0x0131 => { // Software
                if let Some(value) = self.read_string_value(data, tiff_start + value_offset as usize, count as usize) {
                    metadata.insert("Software".to_string(), value);
                }
            },
            0x010E => { // ImageDescription
                if let Some(value) = self.read_string_value(data, tiff_start + value_offset as usize, count as usize) {
                    metadata.insert("ImageDescription".to_string(), value);
                }
            },
            0x9003 => { // DateTimeOriginal
                if let Some(value) = self.read_string_value(data, tiff_start + value_offset as usize, count as usize) {
                    metadata.insert("DateTimeOriginal".to_string(), value);
                }
            },
            0x9004 => { // DateTimeDigitized
                if let Some(value) = self.read_string_value(data, tiff_start + value_offset as usize, count as usize) {
                    metadata.insert("DateTimeDigitized".to_string(), value);
                }
            },
            
            // Camera settings
            0x829A => { // ExposureTime
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ExposureTime".to_string(), value);
                }
            },
            0x829D => { // FNumber
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("FNumber".to_string(), value);
                }
            },
            0x8827 => { // ISOSpeedRatings
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ISOSpeedRatings".to_string(), value.to_string());
                }
            },
            0x920A => { // FocalLength
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("FocalLength".to_string(), value);
                }
            },
            0x9201 => { // ShutterSpeedValue
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ShutterSpeedValue".to_string(), value);
                }
            },
            0x9202 => { // ApertureValue
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ApertureValue".to_string(), value);
                }
            },
            0x9204 => { // ExposureBiasValue
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ExposureBiasValue".to_string(), value);
                }
            },
            0x9205 => { // MaxApertureValue
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("MaxApertureValue".to_string(), value);
                }
            },
            0x9207 => { // MeteringMode
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("MeteringMode".to_string(), value.to_string());
                }
            },
            0x9208 => { // LightSource
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("LightSource".to_string(), value.to_string());
                }
            },
            0x9209 => { // Flash
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("Flash".to_string(), value.to_string());
                }
            },
            0x9206 => { // SubjectDistance
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("SubjectDistance".to_string(), value);
                }
            },
            
            // Image characteristics
            0x0100 => { // ImageWidth
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ImageWidth".to_string(), value.to_string());
                }
            },
            0x0101 => { // ImageLength
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ImageLength".to_string(), value.to_string());
                }
            },
            0x0112 => { // Orientation
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("Orientation".to_string(), value.to_string());
                }
            },
            0x011A => { // XResolution
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("XResolution".to_string(), value);
                }
            },
            0x011B => { // YResolution
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("YResolution".to_string(), value);
                }
            },
            0x0128 => { // ResolutionUnit
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ResolutionUnit".to_string(), value.to_string());
                }
            },
            
            // GPS
            0x0001 => { // GPSLatitudeRef
                if let Some(value) = self.read_string_value(data, value_offset as usize, count as usize) {
                    metadata.insert("GPSLatitudeRef".to_string(), value);
                }
            },
            0x0002 => { // GPSLatitude
                if let Some(value) = self.read_gps_coordinate(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("GPSLatitude".to_string(), value);
                }
            },
            0x0003 => { // GPSLongitudeRef
                if let Some(value) = self.read_string_value(data, value_offset as usize, count as usize) {
                    metadata.insert("GPSLongitudeRef".to_string(), value);
                }
            },
            0x0004 => { // GPSLongitude
                if let Some(value) = self.read_gps_coordinate(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("GPSLongitude".to_string(), value);
                }
            },
            0x0005 => { // GPSAltitudeRef
                if let Some(value) = self.read_u8_value(data, tiff_start + value_offset as usize) {
                    metadata.insert("GPSAltitudeRef".to_string(), value.to_string());
                }
            },
            0x0006 => { // GPSAltitude
                if let Some(value) = self.read_rational_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("GPSAltitude".to_string(), value);
                }
            },
            
            // Maker notes and other
            0x927C => { // MakerNote
                if let Some(value) = self.read_string_value(data, value_offset as usize, count as usize) {
                    metadata.insert("MakerNote".to_string(), value);
                }
            },
            0x9286 => { // UserComment
                if let Some(value) = self.read_string_value(data, value_offset as usize, count as usize) {
                    metadata.insert("UserComment".to_string(), value);
                }
            },
            0xA000 => { // FlashPixVersion
                if let Some(value) = self.read_string_value(data, value_offset as usize, count as usize) {
                    metadata.insert("FlashPixVersion".to_string(), value);
                }
            },
            0xA001 => { // ColorSpace
                if let Some(value) = self.read_u16_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("ColorSpace".to_string(), value.to_string());
                }
            },
            0xA002 => { // PixelXDimension
                if let Some(value) = self.read_u32_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("PixelXDimension".to_string(), value.to_string());
                }
            },
            0xA003 => { // PixelYDimension
                if let Some(value) = self.read_u32_value(data, tiff_start + value_offset as usize, is_little_endian) {
                    metadata.insert("PixelYDimension".to_string(), value.to_string());
                }
            },
            _ => {}
        }
    }
    
    fn read_string_value(&self, data: &[u8], offset: usize, count: usize) -> Option<String> {
        if offset + count > data.len() {
            return None;
        }
        
        let string_data = &data[offset..offset + count];
        // Remove null terminator
        let end = string_data.iter().position(|&b| b == 0).unwrap_or(count);
        Some(String::from_utf8_lossy(&string_data[..end]).to_string())
    }
    
    fn read_rational_value(&self, data: &[u8], offset: usize, is_little_endian: bool) -> Option<String> {
        if offset + 8 > data.len() {
            return None;
        }
        
        let numerator = if is_little_endian {
            ((data[offset] as u32) | ((data[offset + 1] as u32) << 8) | ((data[offset + 2] as u32) << 16) | ((data[offset + 3] as u32) << 24))
        } else {
            (((data[offset] as u32) << 24) | ((data[offset + 1] as u32) << 16) | ((data[offset + 2] as u32) << 8) | (data[offset + 3] as u32))
        };
        
        let denominator = if is_little_endian {
            ((data[offset + 4] as u32) | ((data[offset + 5] as u32) << 8) | ((data[offset + 6] as u32) << 16) | ((data[offset + 7] as u32) << 24))
        } else {
            (((data[offset + 4] as u32) << 24) | ((data[offset + 5] as u32) << 16) | ((data[offset + 6] as u32) << 8) | (data[offset + 7] as u32))
        };
        
        if denominator != 0 {
            Some(format!("{}/{}", numerator, denominator))
        } else {
            Some("0".to_string())
        }
    }
    
    fn read_u16_value(&self, data: &[u8], offset: usize, is_little_endian: bool) -> Option<u16> {
        if offset + 2 > data.len() {
            return None;
        }
        
        Some(if is_little_endian {
            ((data[offset] as u16) | ((data[offset + 1] as u16) << 8))
        } else {
            (((data[offset] as u16) << 8) | (data[offset + 1] as u16))
        })
    }
    
    fn extract_canon_specific_tags(&self, data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for Canon-specific maker notes
        if data.windows(5).any(|w| w == b"Canon") {
            metadata.insert("MakerNotes".to_string(), "Canon".to_string());
        }
    }
    
    fn extract_nikon_specific_tags(&self, data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for Nikon-specific maker notes
        if data.windows(5).any(|w| w == b"Nikon") {
            metadata.insert("MakerNotes".to_string(), "Nikon".to_string());
        }
    }
    
    fn find_heif_exif_atom<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        // HEIF files use atom-based structure
        // Look for 'exif' atom containing EXIF data
        let mut pos = 0;
        
        while pos + 8 < data.len() {
            // Read atom size (4 bytes, big-endian)
            let size = ((data[pos] as u32) << 24) | 
                      ((data[pos + 1] as u32) << 16) | 
                      ((data[pos + 2] as u32) << 8) | 
                      (data[pos + 3] as u32);
            
            if size == 0 || size > data.len() as u32 {
                break;
            }
            
            // Read atom type (4 bytes)
            let atom_type = &data[pos + 4..pos + 8];
            
            if atom_type == b"exif" {
                // Found EXIF atom, return the data (skip the 8-byte header)
                let start = pos + 8;
                let end = (pos + size as usize).min(data.len());
                if start < end {
                    return Some(&data[start..end]);
                }
            }
            
            // Move to next atom
            pos += size as usize;
        }
        
        None
    }
    
    fn extract_heif_basic_metadata(&self, data: &[u8], metadata: &mut HashMap<String, String>) {
        // Extract basic HEIF metadata from ftyp atom and other atoms
        let mut pos = 0;
        
        while pos + 8 < data.len() {
            // Read atom size (4 bytes, big-endian)
            let size = ((data[pos] as u32) << 24) | 
                      ((data[pos + 1] as u32) << 16) | 
                      ((data[pos + 2] as u32) << 8) | 
                      (data[pos + 3] as u32);
            
            if size == 0 || size > data.len() as u32 {
                break;
            }
            
            // Read atom type (4 bytes)
            let atom_type = &data[pos + 4..pos + 8];
            
            match atom_type {
                b"ftyp" => {
                    // File type atom - contains brand information
                    if pos + 12 < data.len() {
                        let brand = &data[pos + 8..pos + 12];
                        match brand {
                            b"heic" => { metadata.insert("Brand".to_string(), "HEIC".to_string()); },
                            b"heix" => { metadata.insert("Brand".to_string(), "HEIX".to_string()); },
                            b"mif1" => { metadata.insert("Brand".to_string(), "MIF1".to_string()); },
                            b"msf1" => { metadata.insert("Brand".to_string(), "MSF1".to_string()); },
                            _ => {}
                        }
                    }
                },
                b"meta" => {
                    // Metadata atom - may contain camera information
                    self.extract_heif_meta_atom(&data[pos + 8..pos + size as usize], metadata);
                },
                _ => {}
            }
            
            // Move to next atom
            pos += size as usize;
        }
        
        // Set default values if no specific metadata found
        if !metadata.contains_key("Make") {
            metadata.insert("Make".to_string(), "Unknown".to_string());
        }
        if !metadata.contains_key("Model") {
            metadata.insert("Model".to_string(), "Unknown".to_string());
        }
    }
    
    fn extract_heif_meta_atom(&self, meta_data: &[u8], metadata: &mut HashMap<String, String>) {
        // Parse metadata atom for camera information
        // This is a simplified version - real HEIF metadata parsing is more complex
        
        // Look for common camera manufacturer strings
        if meta_data.windows(5).any(|w| w == b"Canon") {
            metadata.insert("Make".to_string(), "Canon".to_string());
        } else if meta_data.windows(5).any(|w| w == b"Nikon") {
            metadata.insert("Make".to_string(), "Nikon".to_string());
        } else if meta_data.windows(5).any(|w| w == b"Sony") {
            metadata.insert("Make".to_string(), "Sony".to_string());
        } else if meta_data.windows(5).any(|w| w == b"Apple") {
            metadata.insert("Make".to_string(), "Apple".to_string());
        }
    }
    
    fn read_u8_value(&self, data: &[u8], offset: usize) -> Option<u8> {
        if offset >= data.len() {
            return None;
        }
        Some(data[offset])
    }
    
    fn read_u32_value(&self, data: &[u8], offset: usize, is_little_endian: bool) -> Option<u32> {
        if offset + 4 > data.len() {
            return None;
        }
        let value = if is_little_endian {
            (data[offset] as u32) | ((data[offset + 1] as u32) << 8) | ((data[offset + 2] as u32) << 16) | ((data[offset + 3] as u32) << 24)
        } else {
            ((data[offset] as u32) << 24) | ((data[offset + 1] as u32) << 16) | ((data[offset + 2] as u32) << 8) | (data[offset + 3] as u32)
        };
        Some(value)
    }
    
    fn read_gps_coordinate(&self, data: &[u8], offset: usize, is_little_endian: bool) -> Option<String> {
        // GPS coordinates are typically stored as 3 rational values (degrees, minutes, seconds)
        if offset + 24 > data.len() {
            return None;
        }
        
        let degrees = self.read_rational_value(data, offset, is_little_endian)?;
        let minutes = self.read_rational_value(data, offset + 8, is_little_endian)?;
        let seconds = self.read_rational_value(data, offset + 16, is_little_endian)?;
        
        Some(format!("{}° {}' {}''", degrees, minutes, seconds))
    }
}

/// Python module definition
#[pymodule]
fn fast_exif_reader(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<FastExifReader>()?;
    Ok(())
}