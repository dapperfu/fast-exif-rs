use crate::types::ExifError;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;

/// Common utility functions for EXIF processing
pub struct ExifUtils;

impl ExifUtils {
    /// Read a 32-bit big-endian integer from data at position
    pub fn read_u32_be(data: &[u8], pos: usize) -> Result<u32, ExifError> {
        if pos + 4 > data.len() {
            return Err(ExifError::InvalidExif(
                "Insufficient data for u32 read".to_string(),
            ));
        }
        Ok(((data[pos] as u32) << 24)
            | ((data[pos + 1] as u32) << 16)
            | ((data[pos + 2] as u32) << 8)
            | (data[pos + 3] as u32))
    }

    /// Read a 64-bit big-endian integer from data at position
    pub fn read_u64_be(data: &[u8], pos: usize) -> Result<u64, ExifError> {
        if pos + 8 > data.len() {
            return Err(ExifError::InvalidExif(
                "Insufficient data for u64 read".to_string(),
            ));
        }
        Ok(((data[pos] as u64) << 56)
            | ((data[pos + 1] as u64) << 48)
            | ((data[pos + 2] as u64) << 40)
            | ((data[pos + 3] as u64) << 32)
            | ((data[pos + 4] as u64) << 24)
            | ((data[pos + 5] as u64) << 16)
            | ((data[pos + 6] as u64) << 8)
            | (data[pos + 7] as u64))
    }

    /// Find a pattern in data and return its position
    pub fn find_pattern_in_data(data: &[u8], pattern: &[u8]) -> Option<usize> {
        data.windows(pattern.len())
            .position(|window| window == pattern)
    }

    /// Format Unix timestamp to EXIF datetime format
    pub fn format_timestamp(timestamp: i64) -> Option<String> {
        // Format Unix timestamp to EXIF datetime format
        let datetime = UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64);
        let _system_time = datetime;

        // Convert to EXIF format (YYYY:MM:DD HH:MM:SS)
        // This is a simplified implementation
        Some("2024:01:01 00:00:00".to_string()) // Placeholder
    }

    /// Extract timestamp at a specific position in data
    pub fn extract_timestamp_at_position(data: &[u8], pos: usize) -> Option<String> {
        // Try to extract a timestamp starting at the given position
        // Look for pattern: YYYY:MM:DD HH:MM:SS

        if pos + 19 > data.len() {
            return None;
        }

        let timestamp_bytes = &data[pos..pos + 19];

        // Check if it looks like a timestamp (YYYY:MM:DD HH:MM:SS)
        if timestamp_bytes.len() >= 19 {
            let year = &timestamp_bytes[0..4];
            let month = &timestamp_bytes[5..7];
            let day = &timestamp_bytes[8..10];
            let hour = &timestamp_bytes[11..13];
            let minute = &timestamp_bytes[14..16];
            let second = &timestamp_bytes[17..19];

            // Basic validation
            if Self::is_digit_string(year)
                && Self::is_digit_string(month)
                && Self::is_digit_string(day)
                && Self::is_digit_string(hour)
                && Self::is_digit_string(minute)
                && Self::is_digit_string(second)
            {
                if let Ok(timestamp) = String::from_utf8(timestamp_bytes.to_vec()) {
                    return Some(timestamp);
                }
            }
        }

        None
    }

    /// Check if a byte slice contains only ASCII digits
    pub fn is_digit_string(bytes: &[u8]) -> bool {
        bytes.iter().all(|&b| b.is_ascii_digit())
    }

    /// Extract Unix timestamps from data and add to metadata
    pub fn extract_unix_timestamps(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for Unix timestamps (32-bit integers representing seconds since epoch)
        // Common ranges: 2020-2030 (1577836800 - 1893456000)

        let min_timestamp: u32 = 1577836800; // 2020-01-01
        let max_timestamp: u32 = 1893456000; // 2030-01-01

        // Search for 4-byte sequences that could be timestamps
        for i in 0..data.len().saturating_sub(3) {
            let timestamp_bytes = &data[i..i + 4];

            // Try both little-endian and big-endian interpretations
            let timestamp_le = u32::from_le_bytes([
                timestamp_bytes[0],
                timestamp_bytes[1],
                timestamp_bytes[2],
                timestamp_bytes[3],
            ]);
            let timestamp_be = u32::from_be_bytes([
                timestamp_bytes[0],
                timestamp_bytes[1],
                timestamp_bytes[2],
                timestamp_bytes[3],
            ]);

            // Check if either interpretation falls within our expected range
            if timestamp_le >= min_timestamp && timestamp_le <= max_timestamp {
                if let Some(formatted) = Self::format_timestamp(timestamp_le as i64) {
                    metadata.insert("UnixTimestamp_LE".to_string(), formatted);
                }
            }

            if timestamp_be >= min_timestamp && timestamp_be <= max_timestamp {
                if let Some(formatted) = Self::format_timestamp(timestamp_be as i64) {
                    metadata.insert("UnixTimestamp_BE".to_string(), formatted);
                }
            }
        }
    }

    /// Get high-priority EXIF fields for copying between images
    pub fn get_high_priority_fields() -> Vec<&'static str> {
        vec![
            // DateTime fields (most important for photography)
            "DateTime",
            "DateTimeOriginal",
            "DateTimeDigitized",
            "SubSecTime",
            "SubSecTimeOriginal",
            "SubSecTimeDigitized",
            "OffsetTime",
            "OffsetTimeOriginal",
            "OffsetTimeDigitized",
            
            // Camera information (essential for identification)
            "Make",
            "Model",
            "Software",
            "BodySerialNumber",
            "LensMake",
            "LensModel",
            "LensSerialNumber",
            
            // Exposure settings (core photography data)
            "ExposureTime",
            "FNumber",
            "ISOSpeedRatings",
            "FocalLength",
            "ExposureProgram",
            "ExposureMode",
            "ExposureBiasValue",
            "MeteringMode",
            "Flash",
            "WhiteBalance",
            
            // Image properties (technical details)
            "Orientation",
            "XResolution",
            "YResolution",
            "ResolutionUnit",
            "PixelXDimension",
            "PixelYDimension",
            "ColorSpace",
            
            // Advanced camera settings
            "ShutterSpeedValue",
            "ApertureValue",
            "MaxApertureValue",
            "LightSource",
            "SubjectDistance",
            "SubjectDistanceRange",
            "DigitalZoomRatio",
            "FocalLengthIn35mmFilm",
            "SceneCaptureType",
            "GainControl",
            "Contrast",
            "Saturation",
            "Sharpness",
            
            // Metadata
            "Artist",
            "Copyright",
            "ImageDescription",
        ]
    }

    /// Get comprehensive EXIF field list for 1:1 exiftool compatibility
    pub fn get_comprehensive_fields() -> Vec<&'static str> {
        vec![
            // Basic image information (IFD0)
            "ImageDescription",
            "Make",
            "Model",
            "Orientation",
            "XResolution",
            "YResolution",
            "ResolutionUnit",
            "Software",
            "DateTime",
            "Artist",
            "WhitePoint",
            "PrimaryChromaticities",
            "YCbCrCoefficients",
            "YCbCrSubSampling",
            "YCbCrPositioning",
            "ReferenceBlackWhite",
            "Copyright",
            
            // EXIF-specific fields (ExifIFD)
            "ExposureTime",
            "FNumber",
            "ExposureProgram",
            "SpectralSensitivity",
            "ISOSpeedRatings",
            "OECF",
            "ExifVersion",
            "DateTimeOriginal",
            "DateTimeDigitized",
            "ComponentsConfiguration",
            "CompressedBitsPerPixel",
            "BrightnessValue",
            "ExposureBiasValue",
            "MaxApertureValue",
            "SubjectDistance",
            "MeteringMode",
            "LightSource",
            "Flash",
            "FocalLength",
            "SubjectArea",
            "MakerNote",
            "UserComment",
            "SubSecTime",
            "SubSecTimeOriginal",
            "SubSecTimeDigitized",
            "FlashpixVersion",
            "ColorSpace",
            "PixelXDimension",
            "PixelYDimension",
            "RelatedSoundFile",
            "InteropIndex",
            "InteropVersion",
            "RelatedImageFileFormat",
            "RelatedImageWidth",
            "RelatedImageLength",
            "ExposureIndex",
            "SensingMethod",
            "FileSource",
            "SceneType",
            "CFAPattern",
            "CustomRendered",
            "ExposureMode",
            "WhiteBalance",
            "DigitalZoomRatio",
            "FocalLengthIn35mmFilm",
            "SceneCaptureType",
            "GainControl",
            "Contrast",
            "Saturation",
            "Sharpness",
            "DeviceSettingDescription",
            "SubjectDistanceRange",
            "ImageUniqueID",
            "CameraOwnerName",
            "BodySerialNumber",
            "LensSpecification",
            "LensMake",
            "LensModel",
            "LensSerialNumber",
            
            // GPS fields (GPS IFD)
            "GPSVersionID",
            "GPSLatitudeRef",
            "GPSLatitude",
            "GPSLongitudeRef",
            "GPSLongitude",
            "GPSAltitudeRef",
            "GPSAltitude",
            "GPSTimeStamp",
            "GPSSatellites",
            "GPSStatus",
            "GPSMeasureMode",
            "GPSDOP",
            "GPSSpeedRef",
            "GPSSpeed",
            "GPSTrackRef",
            "GPSTrack",
            "GPSImgDirectionRef",
            "GPSImgDirection",
            "GPSMapDatum",
            "GPSDestLatitudeRef",
            "GPSDestLatitude",
            "GPSDestLongitudeRef",
            "GPSDestLongitude",
            "GPSDestBearingRef",
            "GPSDestBearing",
            "GPSDestDistanceRef",
            "GPSDestDistance",
            "GPSProcessingMethod",
            "GPSAreaInformation",
            "GPSDateStamp",
            "GPSDifferential",
            
            // Additional common fields
            "OffsetTime",
            "OffsetTimeOriginal",
            "OffsetTimeDigitized",
            "ShutterSpeedValue",
            "ApertureValue",
        ]
    }

    /// Filter metadata to only include high-priority fields
    pub fn filter_high_priority_fields(metadata: &HashMap<String, String>) -> HashMap<String, String> {
        let high_priority = Self::get_high_priority_fields();
        let mut filtered = HashMap::new();
        
        for field in high_priority {
            if let Some(value) = metadata.get(field) {
                filtered.insert(field.to_string(), value.clone());
            }
        }
        
        filtered
    }

    /// Check if a field is high-priority
    pub fn is_high_priority_field(field_name: &str) -> bool {
        Self::get_high_priority_fields().contains(&field_name)
    }

    /// Validate EXIF field value format with comprehensive checks
    pub fn validate_field_value(field_name: &str, value: &str) -> Result<(), ExifError> {
        let cleaned_value = value.trim();
        
        if cleaned_value.is_empty() {
            return Err(ExifError::InvalidExif(
                format!("Empty value for field: {}", field_name)
            ));
        }
        
        match field_name {
            // DateTime fields - strict format validation
            "DateTime" | "DateTimeOriginal" | "DateTimeDigitized" => {
                Self::validate_datetime_format(cleaned_value)?;
            }
            
            // Sub-second time fields
            "SubSecTime" | "SubSecTimeOriginal" | "SubSecTimeDigitized" => {
                Self::validate_subsec_time_format(cleaned_value)?;
            }
            
            // Offset time fields
            "OffsetTime" | "OffsetTimeOriginal" | "OffsetTimeDigitized" => {
                Self::validate_offset_time_format(cleaned_value)?;
            }
            
            // Exposure time - supports fractions and decimals
            "ExposureTime" => {
                Self::validate_exposure_time_format(cleaned_value)?;
            }
            
            // F-number - must be positive decimal
            "FNumber" => {
                Self::validate_fnumber_format(cleaned_value)?;
            }
            
            // ISO - must be positive integer
            "ISOSpeedRatings" => {
                Self::validate_iso_format(cleaned_value)?;
            }
            
            // Focal length - supports decimals and units
            "FocalLength" => {
                Self::validate_focal_length_format(cleaned_value)?;
            }
            
            // Orientation - must be 1-8
            "Orientation" => {
                Self::validate_orientation_format(cleaned_value)?;
            }
            
            // Resolution values - must be positive
            "XResolution" | "YResolution" => {
                Self::validate_resolution_format(cleaned_value)?;
            }
            
            // Resolution unit - must be 1, 2, or 3
            "ResolutionUnit" => {
                Self::validate_resolution_unit_format(cleaned_value)?;
            }
            
            // GPS coordinates
            "GPSLatitude" | "GPSLongitude" => {
                Self::validate_gps_coordinate_format(cleaned_value)?;
            }
            
            // GPS references
            "GPSLatitudeRef" | "GPSLongitudeRef" => {
                Self::validate_gps_ref_format(cleaned_value)?;
            }
            
            // GPS altitude reference
            "GPSAltitudeRef" => {
                Self::validate_gps_altitude_ref_format(cleaned_value)?;
            }
            
            // Version fields
            "ExifVersion" | "FlashpixVersion" => {
                Self::validate_version_format(cleaned_value)?;
            }
            
            // Color space
            "ColorSpace" => {
                Self::validate_color_space_format(cleaned_value)?;
            }
            
            // Flash
            "Flash" => {
                Self::validate_flash_format(cleaned_value)?;
            }
            
            // White balance
            "WhiteBalance" => {
                Self::validate_white_balance_format(cleaned_value)?;
            }
            
            // Scene capture type
            "SceneCaptureType" => {
                Self::validate_scene_capture_type_format(cleaned_value)?;
            }
            
            // Metering mode
            "MeteringMode" => {
                Self::validate_metering_mode_format(cleaned_value)?;
            }
            
            // Light source
            "LightSource" => {
                Self::validate_light_source_format(cleaned_value)?;
            }
            
            // Exposure program
            "ExposureProgram" => {
                Self::validate_exposure_program_format(cleaned_value)?;
            }
            
            // Exposure mode
            "ExposureMode" => {
                Self::validate_exposure_mode_format(cleaned_value)?;
            }
            
            // Contrast, Saturation, Sharpness
            "Contrast" | "Saturation" | "Sharpness" => {
                Self::validate_image_adjustment_format(cleaned_value)?;
            }
            
            // Gain control
            "GainControl" => {
                Self::validate_gain_control_format(cleaned_value)?;
            }
            
            // Subject distance range
            "SubjectDistanceRange" => {
                Self::validate_subject_distance_range_format(cleaned_value)?;
            }
            
            // Sensing method
            "SensingMethod" => {
                Self::validate_sensing_method_format(cleaned_value)?;
            }
            
            // File source
            "FileSource" => {
                Self::validate_file_source_format(cleaned_value)?;
            }
            
            // Scene type
            "SceneType" => {
                Self::validate_scene_type_format(cleaned_value)?;
            }
            
            // Custom rendered
            "CustomRendered" => {
                Self::validate_custom_rendered_format(cleaned_value)?;
            }
            
            // Digital zoom ratio
            "DigitalZoomRatio" => {
                Self::validate_digital_zoom_ratio_format(cleaned_value)?;
            }
            
            // Focal length in 35mm film
            "FocalLengthIn35mmFilm" => {
                Self::validate_focal_length_35mm_format(cleaned_value)?;
            }
            
            // Pixel dimensions
            "PixelXDimension" | "PixelYDimension" => {
                Self::validate_pixel_dimension_format(cleaned_value)?;
            }
            
            // ASCII fields - basic validation
            "Make" | "Model" | "Software" | "Artist" | "Copyright" | "ImageDescription" |
            "BodySerialNumber" | "LensMake" | "LensModel" | "LensSerialNumber" |
            "CameraOwnerName" | "ImageUniqueID" | "RelatedSoundFile" |
            "GPSMapDatum" | "GPSProcessingMethod" | "GPSAreaInformation" |
            "GPSDateStamp" | "GPSStatus" | "GPSMeasureMode" | "GPSSatellites" |
            "GPSDestLatitudeRef" | "GPSDestLongitudeRef" | "GPSDestBearingRef" |
            "GPSDestDistanceRef" | "GPSImgDirectionRef" | "GPSTrackRef" |
            "GPSSpeedRef" | "InteropIndex" | "RelatedImageFileFormat" => {
                Self::validate_ascii_field_format(cleaned_value)?;
            }
            
            // Rational fields - validate as decimal or fraction
            "MaxApertureValue" | "SubjectDistance" | "BrightnessValue" | "ExposureBiasValue" |
            "ShutterSpeedValue" | "ApertureValue" | "CompressedBitsPerPixel" |
            "GPSTimeStamp" | "GPSDOP" | "GPSSpeed" | "GPSTrack" | "GPSImgDirection" |
            "GPSDestLatitude" | "GPSDestLongitude" | "GPSDestBearing" | "GPSDestDistance" |
            "GPSAltitude" | "WhitePoint" | "PrimaryChromaticities" | "YCbCrCoefficients" |
            "ReferenceBlackWhite" | "LensSpecification" | "ExposureIndex" => {
                Self::validate_rational_field_format(cleaned_value)?;
            }
            
            // Undefined fields - basic length check
            "MakerNote" | "UserComment" | "OECF" | "ComponentsConfiguration" |
            "CFAPattern" | "DeviceSettingDescription" | "InteropVersion" => {
                Self::validate_undefined_field_format(cleaned_value)?;
            }
            
            // Default validation for unknown fields
            _ => {
                Self::validate_generic_field_format(cleaned_value)?;
            }
        }
        
        Ok(())
    }

    /// Clean and normalize EXIF field values
    pub fn normalize_field_value(field_name: &str, value: &str) -> String {
        let cleaned = value.trim().to_string();
        
        match field_name {
            "DateTime" | "DateTimeOriginal" | "DateTimeDigitized" => {
                // Ensure datetime format is correct
                if cleaned.len() == 19 && cleaned.chars().nth(4) == Some(':') {
                    cleaned
                } else {
                    // Try to parse and reformat
                    // This is a simplified approach - in practice you'd use a proper datetime parser
                    cleaned
                }
            }
            "ExposureTime" => {
                // Normalize exposure time format
                if let Ok(float_val) = cleaned.parse::<f64>() {
                    if float_val < 1.0 && float_val > 0.0 {
                        // Convert to fraction format
                        let denominator = (1.0 / float_val).round() as u32;
                        format!("1/{}", denominator)
                    } else {
                        cleaned
                    }
                } else {
                    cleaned
                }
            }
            "FNumber" => {
                // Normalize f-number format
                if let Ok(float_val) = cleaned.parse::<f64>() {
                    format!("{:.1}", float_val)
                } else {
                    cleaned
                }
            }
            _ => cleaned,
        }
    }

    // Validation helper functions
    fn validate_datetime_format(value: &str) -> Result<(), ExifError> {
        if value.len() != 19 || value.chars().nth(4) != Some(':') || 
           value.chars().nth(7) != Some(':') || value.chars().nth(10) != Some(' ') ||
           value.chars().nth(13) != Some(':') || value.chars().nth(16) != Some(':') {
            return Err(ExifError::InvalidExif(
                format!("Invalid datetime format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_subsec_time_format(value: &str) -> Result<(), ExifError> {
        if value.len() > 6 || !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(ExifError::InvalidExif(
                format!("Invalid subsec time format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_offset_time_format(value: &str) -> Result<(), ExifError> {
        if !value.starts_with('+') && !value.starts_with('-') {
            return Err(ExifError::InvalidExif(
                format!("Invalid offset time format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_exposure_time_format(value: &str) -> Result<(), ExifError> {
        if value.contains('/') {
            let parts: Vec<&str> = value.split('/').collect();
            if parts.len() != 2 || parts[0].parse::<f64>().is_err() || parts[1].parse::<f64>().is_err() {
                return Err(ExifError::InvalidExif(
                    format!("Invalid exposure time fraction format: {}", value)
                ));
            }
        } else if value.parse::<f64>().is_err() {
            return Err(ExifError::InvalidExif(
                format!("Invalid exposure time format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_fnumber_format(value: &str) -> Result<(), ExifError> {
        if let Ok(f) = value.parse::<f64>() {
            if f <= 0.0 {
                return Err(ExifError::InvalidExif(
                    format!("F-number must be positive: {}", value)
                ));
            }
        } else {
            return Err(ExifError::InvalidExif(
                format!("Invalid f-number format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_iso_format(value: &str) -> Result<(), ExifError> {
        if let Ok(iso) = value.parse::<u32>() {
            if iso == 0 {
                return Err(ExifError::InvalidExif("ISO cannot be zero".to_string()));
            }
        } else {
            return Err(ExifError::InvalidExif(
                format!("Invalid ISO value: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_focal_length_format(value: &str) -> Result<(), ExifError> {
        let numeric_part = value.replace(" mm", "").replace("mm", "");
        if numeric_part.parse::<f64>().is_err() && !numeric_part.contains('-') {
            return Err(ExifError::InvalidExif(
                format!("Invalid focal length format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_orientation_format(value: &str) -> Result<(), ExifError> {
        if let Ok(orientation) = value.parse::<u8>() {
            if orientation < 1 || orientation > 8 {
                return Err(ExifError::InvalidExif(
                    format!("Invalid orientation value: {}", value)
                ));
            }
        } else {
            return Err(ExifError::InvalidExif(
                format!("Invalid orientation format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_resolution_format(value: &str) -> Result<(), ExifError> {
        if let Ok(res) = value.parse::<f64>() {
            if res <= 0.0 {
                return Err(ExifError::InvalidExif(
                    format!("Resolution must be positive: {}", value)
                ));
            }
        } else {
            return Err(ExifError::InvalidExif(
                format!("Invalid resolution format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_resolution_unit_format(value: &str) -> Result<(), ExifError> {
        if let Ok(unit) = value.parse::<u8>() {
            if unit < 1 || unit > 3 {
                return Err(ExifError::InvalidExif(
                    format!("Invalid resolution unit: {}", value)
                ));
            }
        } else {
            return Err(ExifError::InvalidExif(
                format!("Invalid resolution unit format: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_gps_coordinate_format(value: &str) -> Result<(), ExifError> {
        // Basic GPS coordinate validation - should be in degrees format
        if value.contains("deg") && value.contains("'") && value.contains("\"") {
            Ok(())
        } else if value.parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid GPS coordinate format: {}", value)
            ))
        }
    }

    fn validate_gps_ref_format(value: &str) -> Result<(), ExifError> {
        if value == "North" || value == "South" || value == "East" || value == "West" ||
           value == "N" || value == "S" || value == "E" || value == "W" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid GPS reference: {}", value)
            ))
        }
    }

    fn validate_gps_altitude_ref_format(value: &str) -> Result<(), ExifError> {
        if value == "0" || value == "1" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid GPS altitude reference: {}", value)
            ))
        }
    }

    fn validate_version_format(value: &str) -> Result<(), ExifError> {
        if value.len() == 4 && value.chars().all(|c| c.is_ascii_digit()) {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid version format: {}", value)
            ))
        }
    }

    fn validate_color_space_format(value: &str) -> Result<(), ExifError> {
        if value == "1" || value == "65535" || value == "sRGB" || value == "Uncalibrated" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid color space: {}", value)
            ))
        }
    }

    fn validate_flash_format(value: &str) -> Result<(), ExifError> {
        if value.parse::<u16>().is_ok() {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid flash value: {}", value)
            ))
        }
    }

    fn validate_white_balance_format(value: &str) -> Result<(), ExifError> {
        if value == "0" || value == "1" || value == "Auto" || value == "Manual" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid white balance: {}", value)
            ))
        }
    }

    fn validate_scene_capture_type_format(value: &str) -> Result<(), ExifError> {
        if value == "0" || value == "1" || value == "2" || value == "3" ||
           value == "Standard" || value == "Landscape" || value == "Portrait" || value == "Night" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid scene capture type: {}", value)
            ))
        }
    }

    fn validate_metering_mode_format(value: &str) -> Result<(), ExifError> {
        if value.parse::<u16>().is_ok() {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid metering mode: {}", value)
            ))
        }
    }

    fn validate_light_source_format(value: &str) -> Result<(), ExifError> {
        if value.parse::<u16>().is_ok() {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid light source: {}", value)
            ))
        }
    }

    fn validate_exposure_program_format(value: &str) -> Result<(), ExifError> {
        if value.parse::<u8>().is_ok() {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid exposure program: {}", value)
            ))
        }
    }

    fn validate_exposure_mode_format(value: &str) -> Result<(), ExifError> {
        if value == "0" || value == "1" || value == "2" || value == "Auto" || value == "Manual" || value == "Auto bracket" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid exposure mode: {}", value)
            ))
        }
    }

    fn validate_image_adjustment_format(value: &str) -> Result<(), ExifError> {
        if value == "0" || value == "1" || value == "2" || value == "Normal" || value == "Low" || value == "High" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid image adjustment value: {}", value)
            ))
        }
    }

    fn validate_gain_control_format(value: &str) -> Result<(), ExifError> {
        if value == "0" || value == "1" || value == "2" || value == "3" || value == "4" ||
           value == "None" || value == "Low gain up" || value == "High gain up" || 
           value == "Low gain down" || value == "High gain down" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid gain control: {}", value)
            ))
        }
    }

    fn validate_subject_distance_range_format(value: &str) -> Result<(), ExifError> {
        if value == "0" || value == "1" || value == "2" || value == "3" ||
           value == "Unknown" || value == "Macro" || value == "Close" || value == "Distant" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid subject distance range: {}", value)
            ))
        }
    }

    fn validate_sensing_method_format(value: &str) -> Result<(), ExifError> {
        if value.parse::<u8>().is_ok() {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid sensing method: {}", value)
            ))
        }
    }

    fn validate_file_source_format(value: &str) -> Result<(), ExifError> {
        if value == "3" || value == "Digital Camera" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid file source: {}", value)
            ))
        }
    }

    fn validate_scene_type_format(value: &str) -> Result<(), ExifError> {
        if value == "1" || value == "Directly photographed" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid scene type: {}", value)
            ))
        }
    }

    fn validate_custom_rendered_format(value: &str) -> Result<(), ExifError> {
        if value == "0" || value == "1" || value == "Normal" || value == "Custom" {
            Ok(())
        } else {
            Err(ExifError::InvalidExif(
                format!("Invalid custom rendered: {}", value)
            ))
        }
    }

    fn validate_digital_zoom_ratio_format(value: &str) -> Result<(), ExifError> {
        if let Ok(ratio) = value.parse::<f64>() {
            if ratio < 0.0 {
                return Err(ExifError::InvalidExif(
                    format!("Digital zoom ratio must be non-negative: {}", value)
                ));
            }
        } else {
            return Err(ExifError::InvalidExif(
                format!("Invalid digital zoom ratio: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_focal_length_35mm_format(value: &str) -> Result<(), ExifError> {
        if let Ok(focal) = value.parse::<u16>() {
            if focal == 0 {
                return Err(ExifError::InvalidExif(
                    format!("35mm focal length cannot be zero: {}", value)
                ));
            }
        } else {
            return Err(ExifError::InvalidExif(
                format!("Invalid 35mm focal length: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_pixel_dimension_format(value: &str) -> Result<(), ExifError> {
        if let Ok(dim) = value.parse::<u32>() {
            if dim == 0 {
                return Err(ExifError::InvalidExif(
                    format!("Pixel dimension cannot be zero: {}", value)
                ));
            }
        } else {
            return Err(ExifError::InvalidExif(
                format!("Invalid pixel dimension: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_ascii_field_format(value: &str) -> Result<(), ExifError> {
        if value.len() > 255 {
            return Err(ExifError::InvalidExif(
                format!("ASCII field too long: {}", value.len())
            ));
        }
        Ok(())
    }

    fn validate_rational_field_format(value: &str) -> Result<(), ExifError> {
        if value.contains('/') {
            let parts: Vec<&str> = value.split('/').collect();
            if parts.len() != 2 || parts[0].parse::<f64>().is_err() || parts[1].parse::<f64>().is_err() {
                return Err(ExifError::InvalidExif(
                    format!("Invalid rational format: {}", value)
                ));
            }
        } else if value.parse::<f64>().is_err() {
            return Err(ExifError::InvalidExif(
                format!("Invalid rational value: {}", value)
            ));
        }
        Ok(())
    }

    fn validate_undefined_field_format(value: &str) -> Result<(), ExifError> {
        if value.len() > 65535 {
            return Err(ExifError::InvalidExif(
                format!("Undefined field too long: {}", value.len())
            ));
        }
        Ok(())
    }

    fn validate_generic_field_format(value: &str) -> Result<(), ExifError> {
        if value.len() > 1000 {
            return Err(ExifError::InvalidExif(
                format!("Field value too long: {}", value.len())
            ));
        }
        Ok(())
    }
}
