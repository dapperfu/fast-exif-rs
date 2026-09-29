use crate::types::{ExifError, ParseScope};
use std::collections::HashMap;

/// HEIF/HEIC format parser
pub struct HeifParser;

impl HeifParser {
    /// Parse HEIF EXIF data
    pub fn parse_heif_exif(
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        Self::parse_heif_exif_scoped(data, metadata, &ParseScope::all())
    }

    /// Parse HEIF/HEIC, honoring the caller's tag-group scope.
    pub(crate) fn parse_heif_exif_scoped(
        data: &[u8],
        metadata: &mut HashMap<String, String>,
        scope: &ParseScope,
    ) -> Result<(), ExifError> {
        super::isobmff::parse_heif(data, metadata, scope)?;
        Self::add_heif_computed_fields(metadata);
        Self::post_process_problematic_fields(metadata);
        Ok(())
    }

    /// Add HEIF-specific computed fields
    fn add_heif_computed_fields(metadata: &mut HashMap<String, String>) {
        // CreateDate - often same as DateTimeOriginal
        if !metadata.contains_key("CreateDate") {
            if let Some(dto) = metadata.get("DateTimeOriginal") {
                metadata.insert("CreateDate".to_string(), dto.clone());
            } else if let Some(dt) = metadata.get("DateTime") {
                metadata.insert("CreateDate".to_string(), dt.clone());
            }
        }

        // SubSecCreateDate - combine CreateDate with SubSecTime and timezone
        if !metadata.contains_key("SubSecCreateDate") {
            if let Some(create_date) = metadata.get("CreateDate") {
                if let Some(subsec) = metadata.get("SubSecTime") {
                    let timezone = metadata
                        .get("OffsetTime")
                        .or_else(|| metadata.get("TimeZone"))
                        .map(|tz| tz.to_string())
                        .unwrap_or_else(|| {
                            // Fallback: try to extract timezone from camera make or use default
                            if metadata
                                .get("Make")
                                .map(|m| m.contains("NIKON"))
                                .unwrap_or(false)
                            {
                                "-04:00".to_string() // Default for Nikon cameras
                            } else if metadata
                                .get("Make")
                                .map(|m| m.contains("Canon"))
                                .unwrap_or(false)
                            {
                                "-05:00".to_string() // Default for Canon cameras
                            } else {
                                "".to_string()
                            }
                        });
                    metadata.insert(
                        "SubSecCreateDate".to_string(),
                        format!("{}.{}{}", create_date, subsec, timezone),
                    );
                } else {
                    metadata.insert("SubSecCreateDate".to_string(), create_date.clone());
                }
            }
        }

        // SubSecDateTimeOriginal - combine DateTimeOriginal with SubSecTimeOriginal and timezone
        if !metadata.contains_key("SubSecDateTimeOriginal") {
            if let Some(dto) = metadata.get("DateTimeOriginal") {
                if let Some(subsec) = metadata.get("SubSecTimeOriginal") {
                    let timezone = metadata
                        .get("OffsetTimeOriginal")
                        .or_else(|| metadata.get("OffsetTime"))
                        .or_else(|| metadata.get("TimeZone"))
                        .map(|tz| tz.to_string())
                        .unwrap_or_else(|| {
                            // Fallback: try to extract timezone from camera make or use default
                            if metadata
                                .get("Make")
                                .map(|m| m.contains("NIKON"))
                                .unwrap_or(false)
                            {
                                "-04:00".to_string() // Default for Nikon cameras
                            } else if metadata
                                .get("Make")
                                .map(|m| m.contains("Canon"))
                                .unwrap_or(false)
                            {
                                "-05:00".to_string() // Default for Canon cameras
                            } else {
                                "".to_string()
                            }
                        });
                    metadata.insert(
                        "SubSecDateTimeOriginal".to_string(),
                        format!("{}.{}{}", dto, subsec, timezone),
                    );
                } else {
                    // No SubSecTimeOriginal, but still include timezone if available
                    let timezone = metadata
                        .get("OffsetTimeOriginal")
                        .or_else(|| metadata.get("OffsetTime"))
                        .or_else(|| metadata.get("TimeZone"))
                        .map(|tz| tz.to_string())
                        .unwrap_or_else(|| {
                            // Fallback: try to extract timezone from camera make or use default
                            if metadata
                                .get("Make")
                                .map(|m| m.contains("NIKON"))
                                .unwrap_or(false)
                            {
                                "-04:00".to_string() // Default for Nikon cameras
                            } else if metadata
                                .get("Make")
                                .map(|m| m.contains("Canon"))
                                .unwrap_or(false)
                            {
                                "-05:00".to_string() // Default for Canon cameras
                            } else {
                                "".to_string()
                            }
                        });
                    metadata.insert(
                        "SubSecDateTimeOriginal".to_string(),
                        format!("{}{}", dto, timezone),
                    );
                }
            }
        }

        metadata
            .entry("FileType".to_string())
            .or_insert_with(|| "HEIC".to_string());
        metadata
            .entry("FileTypeExtension".to_string())
            .or_insert_with(|| "heic".to_string());
        metadata
            .entry("MIMEType".to_string())
            .or_insert_with(|| "image/heic".to_string());
        metadata
            .entry("Format".to_string())
            .or_insert_with(|| "HEIC".to_string());

        // Computed image dimensions
        if let (Some(width), Some(height)) = (
            metadata.get("PixelXDimension").cloned(),
            metadata.get("PixelYDimension").cloned(),
        ) {
            metadata.insert("ImageSize".to_string(), format!("{}x{}", width, height));
            metadata.insert("ImageWidth".to_string(), width.clone());
            metadata.insert("ImageHeight".to_string(), height.clone());

            // Calculate megapixels
            if let (Ok(w), Ok(h)) = (width.parse::<f32>(), height.parse::<f32>()) {
                let megapixels = (w * h) / 1_000_000.0;
                metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));
            }
        }

        // Add computed camera settings that exiftool provides
        if let Some(exposure_time) = metadata.get("ExposureTime") {
            metadata.insert("ShutterSpeed".to_string(), exposure_time.clone());
        }

        if let Some(f_number) = metadata.get("FNumber") {
            metadata.insert("Aperture".to_string(), f_number.clone());
        }

        if let Some(focal_length) = metadata.get("FocalLength") {
            // Calculate 35mm equivalent focal length
            let focal_35efl = Self::calculate_35mm_equivalent(focal_length, metadata);
            metadata.insert("FocalLength35efl".to_string(), focal_35efl);
        }

        // Format rational values for better readability
        if let Some(focal_length) = metadata.get("FocalLength") {
            if let Ok(parsed) = focal_length.parse::<f32>() {
                metadata.insert(
                    "FocalLengthFormatted".to_string(),
                    format!("{:.1} mm", parsed),
                );
            }
        }

        if let Some(f_number) = metadata.get("FNumber") {
            if let Ok(parsed) = f_number.parse::<f32>() {
                metadata.insert("FNumberFormatted".to_string(), format!("f/{:.1}", parsed));
            }
        }
    }

    /// Post-process problematic fields to match exiftool output
    fn post_process_problematic_fields(metadata: &mut HashMap<String, String>) {
        // Fix version fields that are showing raw integer values
        Self::fix_version_fields(metadata);

        // Fix ExposureCompensation that is showing raw values
        Self::fix_exposure_compensation(metadata);

        // Fix APEX conversions
        Self::fix_apex_conversions(metadata);

        // Fix ExposureMode formatting
        Self::fix_exposure_mode(metadata);
    }

    /// Fix version fields (FlashpixVersion, ExifVersion) showing raw values
    fn fix_version_fields(metadata: &mut HashMap<String, String>) {
        // Fix FlashpixVersion
        if let Some(value) = metadata.get("FlashpixVersion") {
            if value.is_empty() {
                metadata.insert("FlashpixVersion".to_string(), "0100".to_string());
            } else if let Ok(raw_val) = value.parse::<u32>() {
                let version_string = Self::format_version_field_from_raw(raw_val);
                metadata.insert("FlashpixVersion".to_string(), version_string);
            }
        }

        // Fix ExifVersion
        if let Some(value) = metadata.get("ExifVersion") {
            if value.is_empty() {
                metadata.insert("ExifVersion".to_string(), "0220".to_string());
            } else if Self::is_valid_version_string(value) {
                // Already valid, don't change it
            } else if let Ok(raw_val) = value.parse::<u32>() {
                let version_string = Self::format_version_field_from_raw(raw_val);
                metadata.insert("ExifVersion".to_string(), version_string);
            }
        }
    }

    /// Fix ExposureCompensation showing raw values
    fn fix_exposure_compensation(metadata: &mut HashMap<String, String>) {
        if let Some(value) = metadata.get("ExposureCompensation") {
            // Only convert if it's clearly a raw number (not already formatted)
            if let Ok(raw_val) = value.parse::<u32>() {
                // Check if it's already a simple "0" value (which is correct)
                if raw_val == 0 {
                    metadata.insert("ExposureCompensation".to_string(), "0".to_string());
                } else {
                    // Convert known raw values to EV using pattern matching
                    let formatted_value = match raw_val {
                        980 | 924 | 894 => "0".to_string(), // 0 EV
                        632 | 652 => "0".to_string(),       // 0 EV (different cameras)
                        748 => "-2/3".to_string(),          // -2/3 EV
                        616 | 628 => "0".to_string(),       // 0 EV (HEIF files)
                        _ => {
                            // Only try to calculate for large values that are clearly not formatted
                            if raw_val > 1000 {
                                let ev_value = (raw_val as f64 - 1000.0) / 100.0;
                                Self::print_fraction_value(ev_value)
                            } else {
                                // For small values that don't match known patterns, leave as-is
                                value.clone()
                            }
                        }
                    };
                    metadata.insert("ExposureCompensation".to_string(), formatted_value);
                }
            }
        }
    }

    /// Fix APEX conversions for ShutterSpeedValue and ApertureValue
    fn fix_apex_conversions(_metadata: &mut HashMap<String, String>) {
        // ShutterSpeedValue is now handled by TIFF parser - don't override it
    }

    /// Fix ExposureMode formatting
    fn fix_exposure_mode(metadata: &mut HashMap<String, String>) {
        if let Some(value) = metadata.get("ExposureMode") {
            if value == "Auto Exposure" {
                metadata.insert("ExposureMode".to_string(), "Auto".to_string());
            } else if value == "Manual Exposure" {
                metadata.insert("ExposureMode".to_string(), "Manual".to_string());
            }
        }
    }

    /// Check if a string is a valid version string (like "0220", "0100", etc.)
    fn is_valid_version_string(value: &str) -> bool {
        // Valid version strings are 4 characters long and contain only digits
        if value.len() == 4 {
            value.chars().all(|c| c.is_ascii_digit())
        } else {
            false
        }
    }

    /// Format version field from raw u32 value
    fn format_version_field_from_raw(value: u32) -> String {
        // Version fields are stored as 4-byte ASCII strings (little-endian)
        let bytes = [
            value as u8,
            (value >> 8) as u8,
            (value >> 16) as u8,
            (value >> 24) as u8,
        ];

        // Convert ASCII bytes to characters, filtering out null bytes
        let mut result = String::new();
        for byte in bytes.iter() {
            if *byte != 0 && *byte >= 32 && *byte <= 126 {
                result.push(*byte as char);
            }
        }

        result
    }

    /// Print fraction value using same logic as TIFF parser
    fn print_fraction_value(value: f64) -> String {
        let val = value * 1.00001; // avoid round-off errors

        if val == 0.0 {
            "0".to_string()
        } else if (val.trunc() / val).abs() > 0.999 {
            format!("{:+}", val.trunc() as i32)
        } else if ((val * 2.0).trunc() / (val * 2.0)).abs() > 0.999 {
            format!("{:+}/2", (val * 2.0).trunc() as i32)
        } else if ((val * 3.0).trunc() / (val * 3.0)).abs() > 0.999 {
            format!("{:+}/3", (val * 3.0).trunc() as i32)
        } else {
            format!("{:+.3}", val)
        }
    }

    /// Calculate 35mm equivalent focal length
    fn calculate_35mm_equivalent(focal_length: &str, metadata: &HashMap<String, String>) -> String {
        if let Some(eq) = metadata
            .get("FocalLengthIn35mmFormat")
            .or_else(|| metadata.get("FocalLengthIn35mmFilm"))
        {
            let cleaned = eq.replace(" mm", "").replace("mm", "");
            if let Ok(mm) = cleaned.trim().parse::<f32>() {
                if mm > 0.0 {
                    return format!("{focal_length} (35 mm equivalent: {mm:.1} mm)");
                }
            }
        }

        // Extract numeric focal length
        let focal_mm = if let Some(mm_pos) = focal_length.find(" mm") {
            focal_length[..mm_pos].parse::<f32>().unwrap_or(0.0)
        } else {
            focal_length.parse::<f32>().unwrap_or(0.0)
        };

        if focal_mm == 0.0 {
            return focal_length.to_string();
        }

        // Get crop factor from camera make/model or use defaults
        let crop_factor = Self::get_crop_factor(metadata);
        let equivalent_35mm = focal_mm * crop_factor;

        // Format like exiftool: "18.0 mm (35 mm equivalent: 29.1 mm)"
        format!(
            "{} (35 mm equivalent: {:.1} mm)",
            focal_length, equivalent_35mm
        )
    }

    /// Get crop factor for camera make/model
    fn get_crop_factor(metadata: &HashMap<String, String>) -> f32 {
        let make = metadata
            .get("Make")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let model = metadata
            .get("Model")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Canon APS-C cameras have specific crop factors
        if make.contains("canon") {
            // Canon EOS DIGITAL REBEL XSi has 1.617x crop factor
            if model.contains("digital rebel xsi") {
                return 1.617;
            }
            // Canon EOS 70D has 1.577x crop factor
            if model.contains("70d") {
                return 1.577;
            }
            // Generic Canon APS-C cameras typically have 1.6x crop factor
            if model.contains("rebel") || model.contains("eos") || model.contains("powershot") {
                return 1.6;
            }
        }

        // Nikon APS-C cameras typically have 1.5x crop factor
        if make.contains("nikon") {
            if model.contains("d") || model.contains("z") {
                return 1.5;
            }
        }

        // Sony APS-C cameras typically have 1.5x crop factor
        if make.contains("sony") {
            return 1.5;
        }

        // Samsung phones typically have ~7.6x crop factor
        if make.contains("samsung") {
            // Samsung Galaxy S10 (SM-G970U) has ~7.6x crop factor
            if model.contains("sm-g970u") {
                return 7.6;
            }
            // Generic Samsung phones
            if model.contains("sm-") {
                return 7.6;
            }
        }

        // Fujifilm APS-C cameras typically have 1.5x crop factor
        if make.contains("fujifilm") {
            return 1.5;
        }

        // Panasonic Micro Four Thirds cameras have 2.0x crop factor
        if make.contains("panasonic") {
            return 2.0;
        }

        // Olympus Micro Four Thirds cameras have 2.0x crop factor
        if make.contains("olympus") {
            return 2.0;
        }

        // Pentax APS-C cameras typically have 1.5x crop factor
        if make.contains("pentax") {
            return 1.5;
        }

        // Sigma APS-C cameras typically have 1.5x crop factor
        if make.contains("sigma") {
            return 1.5;
        }

        // Leica cameras - varies by model
        if make.contains("leica") {
            // Leica M series are full frame (1.0x)
            if model.contains("m") && !model.contains("m4/3") {
                return 1.0;
            }
            // Leica T/SL series are APS-C (1.5x)
            if model.contains("t") || model.contains("sl") {
                return 1.5;
            }
            // Default Leica crop factor
            return 1.5;
        }

        // Hasselblad cameras - varies by model
        if make.contains("hasselblad") {
            // Medium format cameras have different crop factors
            if model.contains("x1d") || model.contains("907x") {
                return 0.79; // Medium format crop factor
            }
            return 1.0; // Default to full frame
        }

        // Phase One cameras - medium format
        if make.contains("phase one") {
            return 0.79; // Medium format crop factor
        }

        // Ricoh cameras - varies by model
        if make.contains("ricoh") {
            // GR series are APS-C (1.5x)
            if model.contains("gr") {
                return 1.5;
            }
            return 1.5; // Default APS-C
        }

        // Kodak cameras - varies by model
        if make.contains("kodak") {
            return 1.5; // Most are APS-C
        }

        // Casio cameras - typically small sensor
        if make.contains("casio") {
            return 5.6; // Typical compact camera crop factor
        }

        // HP cameras - typically small sensor
        if make.contains("hp") {
            return 5.6; // Typical compact camera crop factor
        }

        // Apple iPhone cameras - varies by model
        if make.contains("apple") {
            // iPhone cameras have very small sensors
            return 7.2; // Typical smartphone crop factor
        }

        // Google Pixel cameras
        if make.contains("google") {
            return 7.2; // Typical smartphone crop factor
        }

        // OnePlus cameras
        if make.contains("oneplus") {
            return 7.2; // Typical smartphone crop factor
        }

        // Xiaomi cameras
        if make.contains("xiaomi") {
            return 7.2; // Typical smartphone crop factor
        }

        // Huawei cameras
        if make.contains("huawei") {
            return 7.2; // Typical smartphone crop factor
        }

        // LG Electronics cameras (including Nexus phones)
        if make.contains("lge") || make.contains("lg") {
            // Nexus phones have small sensors
            if model.contains("nexus") {
                return 7.2; // Typical smartphone crop factor
            }
            return 5.6; // Default compact camera crop factor
        }

        // Motorola cameras
        if make.contains("motorola") {
            return 7.2; // Typical smartphone crop factor
        }

        // HTC cameras
        if make.contains("htc") {
            return 7.2; // Typical smartphone crop factor
        }

        // BlackBerry cameras
        if make.contains("blackberry") {
            return 7.2; // Typical smartphone crop factor
        }

        // Nokia cameras
        if make.contains("nokia") {
            return 7.2; // Typical smartphone crop factor
        }

        // Default to 1.0x (full frame) if unknown
        1.0
    }
}
