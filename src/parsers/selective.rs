use std::collections::HashMap;
use std::collections::HashSet;

/// Selective field extraction for fast-exif-rs 2.0
/// 
/// This module provides efficient field filtering and extraction,
/// allowing users to parse only the EXIF fields they need,
/// significantly improving performance for specific use cases.
pub struct SelectiveFieldExtractor {
    field_groups: HashMap<String, HashSet<String>>,
    custom_fields: HashSet<String>,
}

impl SelectiveFieldExtractor {
    /// Create a new selective field extractor
    pub fn new() -> Self {
        let mut extractor = Self {
            field_groups: HashMap::new(),
            custom_fields: HashSet::new(),
        };
        
        extractor.initialize_field_groups();
        extractor
    }
    
    /// Create extractor with specific field groups
    pub fn with_groups(groups: &[&str]) -> Self {
        let mut extractor = Self::new();
        
        for group in groups {
            if let Some(fields) = extractor.field_groups.get(*group) {
                extractor.custom_fields.extend(fields.iter().cloned());
            }
        }
        
        extractor
    }
    
    /// Create extractor with custom field list
    pub fn with_fields(fields: &[&str]) -> Self {
        let mut extractor = Self::new();
        extractor.custom_fields.extend(fields.iter().map(|s| s.to_string()));
        extractor
    }
    
    /// Filter metadata to only include requested fields
    pub fn filter_metadata(&self, metadata: HashMap<String, String>) -> HashMap<String, String> {
        if self.custom_fields.is_empty() {
            return metadata; // Return all fields if no filter specified
        }
        
        metadata.into_iter()
            .filter(|(key, _)| self.custom_fields.contains(key))
            .collect()
    }
    
    /// Check if a field should be extracted
    pub fn should_extract(&self, field_name: &str) -> bool {
        self.custom_fields.is_empty() || self.custom_fields.contains(field_name)
    }
    
    /// Get list of fields that will be extracted
    pub fn get_extraction_fields(&self) -> Vec<String> {
        self.custom_fields.iter().cloned().collect()
    }
    
    /// Add a field to the extraction list
    pub fn add_field(&mut self, field_name: &str) {
        self.custom_fields.insert(field_name.to_string());
    }
    
    /// Remove a field from the extraction list
    pub fn remove_field(&mut self, field_name: &str) {
        self.custom_fields.remove(field_name);
    }
    
    /// Clear all custom fields (extract all)
    pub fn clear_fields(&mut self) {
        self.custom_fields.clear();
    }
    
    /// Initialize predefined field groups
    fn initialize_field_groups(&mut self) {
        // Basic camera information
        let basic_fields = HashSet::from([
            "Make".to_string(),
            "Model".to_string(),
            "DateTime".to_string(),
            "DateTimeOriginal".to_string(),
            "DateTimeDigitized".to_string(),
            "Software".to_string(),
            "Artist".to_string(),
            "Copyright".to_string(),
        ]);
        self.field_groups.insert("basic".to_string(), basic_fields);
        
        // Camera settings
        let camera_fields = HashSet::from([
            "FocalLength".to_string(),
            "FocalLength35efl".to_string(),
            "FNumber".to_string(),
            "ExposureTime".to_string(),
            "ShutterSpeedValue".to_string(),
            "ApertureValue".to_string(),
            "ISO".to_string(),
            "ExposureCompensation".to_string(),
            "ExposureMode".to_string(),
            "ExposureProgram".to_string(),
            "MeteringMode".to_string(),
            "Flash".to_string(),
            "WhiteBalance".to_string(),
            "ColorSpace".to_string(),
        ]);
        self.field_groups.insert("camera".to_string(), camera_fields);
        
        // GPS information
        let gps_fields = HashSet::from([
            "GPSLatitude".to_string(),
            "GPSLongitude".to_string(),
            "GPSAltitude".to_string(),
            "GPSPosition".to_string(),
            "GPSDateTime".to_string(),
            "GPSProcessingMethod".to_string(),
            "GPSImgDirection".to_string(),
            "GPSImgDirectionRef".to_string(),
            "GPSSpeed".to_string(),
            "GPSSpeedRef".to_string(),
        ]);
        self.field_groups.insert("gps".to_string(), gps_fields);
        
        // Image properties
        let image_fields = HashSet::from([
            "ImageWidth".to_string(),
            "ImageHeight".to_string(),
            "Orientation".to_string(),
            "XResolution".to_string(),
            "YResolution".to_string(),
            "ResolutionUnit".to_string(),
            "YCbCrSubSampling".to_string(),
            "YCbCrPositioning".to_string(),
            "Compression".to_string(),
            "PhotometricInterpretation".to_string(),
        ]);
        self.field_groups.insert("image".to_string(), image_fields);
        
        // Maker notes
        let maker_fields = HashSet::from([
            "MakerNote".to_string(),
            "MakerNoteType".to_string(),
            "CanonFlashMode".to_string(),
            "CanonFlashExposureCompensation".to_string(),
            "CanonWhiteBalance".to_string(),
            "CanonPictureStyle".to_string(),
            "NikonFlashMode".to_string(),
            "NikonWhiteBalance".to_string(),
            "SonyFlashMode".to_string(),
            "SonyWhiteBalance".to_string(),
        ]);
        self.field_groups.insert("maker".to_string(), maker_fields);
        
        // Video specific
        let video_fields = HashSet::from([
            "VideoCodec".to_string(),
            "VideoFrameRate".to_string(),
            "VideoDuration".to_string(),
            "VideoBitrate".to_string(),
            "AudioCodec".to_string(),
            "AudioSampleRate".to_string(),
            "AudioChannels".to_string(),
        ]);
        self.field_groups.insert("video".to_string(), video_fields);
        
        // Thumbnail generation (minimal fields for fast processing)
        let thumbnail_fields = HashSet::from([
            "Make".to_string(),
            "Model".to_string(),
            "DateTime".to_string(),
            "ImageWidth".to_string(),
            "ImageHeight".to_string(),
            "Orientation".to_string(),
        ]);
        self.field_groups.insert("thumbnail".to_string(), thumbnail_fields);
        
        // File management
        let file_fields = HashSet::from([
            "Make".to_string(),
            "Model".to_string(),
            "DateTime".to_string(),
            "DateTimeOriginal".to_string(),
            "Software".to_string(),
            "Artist".to_string(),
            "Copyright".to_string(),
            "GPSPosition".to_string(),
        ]);
        self.field_groups.insert("file".to_string(), file_fields);
    }
    
    /// Get available field groups
    pub fn get_available_groups(&self) -> Vec<String> {
        self.field_groups.keys().cloned().collect()
    }
    
    /// Get fields in a specific group
    pub fn get_group_fields(&self, group_name: &str) -> Option<Vec<String>> {
        self.field_groups.get(group_name).map(|fields| fields.iter().cloned().collect())
    }
    
    /// Add a custom field group
    pub fn add_field_group(&mut self, group_name: &str, fields: Vec<&str>) {
        let field_set: HashSet<String> = fields.iter().map(|s| s.to_string()).collect();
        self.field_groups.insert(group_name.to_string(), field_set);
    }
}

impl Default for SelectiveFieldExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined field extractors for common use cases
pub struct FieldExtractors;

impl FieldExtractors {
    /// Create extractor for thumbnail generation (minimal fields)
    pub fn thumbnail() -> SelectiveFieldExtractor {
        SelectiveFieldExtractor::with_groups(&["thumbnail"])
    }
    
    /// Create extractor for file management
    pub fn file_management() -> SelectiveFieldExtractor {
        SelectiveFieldExtractor::with_groups(&["file"])
    }
    
    /// Create extractor for camera settings only
    pub fn camera_settings() -> SelectiveFieldExtractor {
        SelectiveFieldExtractor::with_groups(&["camera"])
    }
    
    /// Create extractor for GPS data only
    pub fn gps_only() -> SelectiveFieldExtractor {
        SelectiveFieldExtractor::with_groups(&["gps"])
    }
    
    /// Create extractor for basic information
    pub fn basic_info() -> SelectiveFieldExtractor {
        SelectiveFieldExtractor::with_groups(&["basic"])
    }
    
    /// Create extractor for video metadata
    pub fn video_metadata() -> SelectiveFieldExtractor {
        SelectiveFieldExtractor::with_groups(&["video"])
    }
    
    /// Create extractor for maker notes
    pub fn maker_notes() -> SelectiveFieldExtractor {
        SelectiveFieldExtractor::with_groups(&["maker"])
    }
    
    /// Create extractor for image properties
    pub fn image_properties() -> SelectiveFieldExtractor {
        SelectiveFieldExtractor::with_groups(&["image"])
    }
}
