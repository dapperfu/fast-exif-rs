use crate::types::ExifError;
use crate::utils::ExifUtils;
use crate::FastExifReader;
use std::collections::HashMap;

/// EXIF copier for copying metadata between images
#[derive(Clone)]
pub struct ExifCopier {
    reader: FastExifReader,
    writer: crate::writer::ExifWriter,
}

impl ExifCopier {
    pub fn new() -> Self {
        Self {
            reader: FastExifReader::new(),
            writer: crate::writer::ExifWriter::new(),
        }
    }

    /// Copy high-priority EXIF fields from source to target image
    pub fn copy_high_priority_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> Result<(), ExifError> {
        // Read source image EXIF data
        let source_metadata = self.reader.read_file(source_path)
            .map_err(|e| ExifError::InvalidExif(format!("Failed to read source EXIF: {}", e)))?;
        
        // Filter to high-priority fields only
        let high_priority_metadata = ExifUtils::filter_high_priority_fields(&source_metadata);
        
        if high_priority_metadata.is_empty() {
            return Err(ExifError::InvalidExif("No high-priority EXIF fields found in source".to_string()));
        }
        
        // Write filtered EXIF data to target image
        self.writer.write_exif(target_path, output_path, &high_priority_metadata)
    }

    /// Copy high-priority EXIF fields from source bytes to target bytes
    pub fn copy_high_priority_exif_to_bytes(
        &mut self,
        source_data: &[u8],
        target_data: &[u8],
    ) -> Result<Vec<u8>, ExifError> {
        // Read source image EXIF data
        let source_metadata = self.reader.read_bytes(source_data)
            .map_err(|e| ExifError::InvalidExif(format!("Failed to read source EXIF: {}", e)))?;
        
        // Filter to high-priority fields only
        let high_priority_metadata = ExifUtils::filter_high_priority_fields(&source_metadata);
        
        if high_priority_metadata.is_empty() {
            return Err(ExifError::InvalidExif("No high-priority EXIF fields found in source".to_string()));
        }
        
        // Write filtered EXIF data to target image
        self.writer.write_exif_to_bytes(target_data, &high_priority_metadata)
    }

    /// Copy all EXIF fields from source to target image
    pub fn copy_all_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> Result<(), ExifError> {
        // Read source image EXIF data
        let source_metadata = self.reader.read_file(source_path)
            .map_err(|e| ExifError::InvalidExif(format!("Failed to read source EXIF: {}", e)))?;
        
        if source_metadata.is_empty() {
            return Err(ExifError::InvalidExif("No EXIF fields found in source".to_string()));
        }
        
        // Write all EXIF data to target image
        self.writer.write_exif(target_path, output_path, &source_metadata)
    }

    /// Copy specific EXIF fields from source to target image
    pub fn copy_specific_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
        field_names: &[&str],
    ) -> Result<(), ExifError> {
        // Read source image EXIF data
        let source_metadata = self.reader.read_file(source_path)
            .map_err(|e| ExifError::InvalidExif(format!("Failed to read source EXIF: {}", e)))?;
        
        // Filter to specific fields only
        let mut filtered_metadata = HashMap::new();
        for field_name in field_names {
            if let Some(value) = source_metadata.get(*field_name) {
                filtered_metadata.insert(field_name.to_string(), value.clone());
            }
        }
        
        if filtered_metadata.is_empty() {
            return Err(ExifError::InvalidExif("No specified EXIF fields found in source".to_string()));
        }
        
        // Write filtered EXIF data to target image
        self.writer.write_exif(target_path, output_path, &filtered_metadata)
    }

    /// Get available EXIF fields from source image
    pub fn get_available_fields(&mut self, source_path: &str) -> Result<Vec<String>, ExifError> {
        let source_metadata = self.reader.read_file(source_path)
            .map_err(|e| ExifError::InvalidExif(format!("Failed to read source EXIF: {}", e)))?;
        
        Ok(source_metadata.keys().cloned().collect())
    }

    /// Get high-priority EXIF fields from source image
    pub fn get_high_priority_fields(&mut self, source_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let source_metadata = self.reader.read_file(source_path)
            .map_err(|e| ExifError::InvalidExif(format!("Failed to read source EXIF: {}", e)))?;
        
        Ok(ExifUtils::filter_high_priority_fields(&source_metadata))
    }
}

impl Default for ExifCopier {
    fn default() -> Self {
        Self::new()
    }
}
