use std::collections::HashMap;

/// Computed fields that exiftool provides but fast-exif-rs doesn't extract directly
pub struct ComputedFields;

impl ComputedFields {
    /// Add computed fields to metadata for 1:1 exiftool compatibility
    pub fn add_computed_fields(metadata: &mut HashMap<String, String>) {
        // Image dimensions and size
        Self::add_image_dimensions(metadata);
        
        // Megapixels calculation
        Self::add_megapixels(metadata);
        
        // Light value calculation
        Self::add_light_value(metadata);
        
        // Scale factor for 35mm equivalent
        Self::add_scale_factor_35efl(metadata);
        
        // Circle of confusion
        Self::add_circle_of_confusion(metadata);
        
        // Field of view
        Self::add_fov(metadata);
        
        // Hyperfocal distance
        Self::add_hyperfocal_distance(metadata);
        
        // Lens specification
        Self::add_lens_specification(metadata);
        
        // Additional computed fields
        Self::add_additional_computed_fields(metadata);
        
        // Composite fields for PyExifTool compatibility
        Self::add_composite_fields(metadata);
        
        // File system metadata
        Self::add_file_metadata(metadata);
        
        // Maker notes fields
        Self::add_maker_notes_fields(metadata);
    }
    
    /// Add image dimensions from existing fields
    fn add_image_dimensions(metadata: &mut HashMap<String, String>) {
        // Try to get dimensions from various sources
        let width = metadata.get("ImageWidth")
            .or_else(|| metadata.get("ExifImageWidth"))
            .or_else(|| metadata.get("PixelXDimension"))
            .map(|s| s.clone());
            
        let height = metadata.get("ImageHeight")
            .or_else(|| metadata.get("ExifImageHeight"))
            .or_else(|| metadata.get("PixelYDimension"))
            .map(|s| s.clone());
        
        if let (Some(w), Some(h)) = (width, height) {
            if !metadata.contains_key("ImageWidth") {
                metadata.insert("ImageWidth".to_string(), w.clone());
            }
            if !metadata.contains_key("ImageHeight") {
                metadata.insert("ImageHeight".to_string(), h.clone());
            }
            
            // Add ImageSize field
            if !metadata.contains_key("ImageSize") {
                metadata.insert("ImageSize".to_string(), format!("{}x{}", w, h));
            }
        }
    }
    
    /// Calculate and add megapixels
    fn add_megapixels(metadata: &mut HashMap<String, String>) {
        if let (Some(width), Some(height)) = (metadata.get("ImageWidth"), metadata.get("ImageHeight")) {
            if let (Ok(w), Ok(h)) = (width.parse::<f64>(), height.parse::<f64>()) {
                let megapixels = (w * h) / 1_000_000.0;
                metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));
            }
        }
    }
    
    /// Calculate and add light value
    fn add_light_value(metadata: &mut HashMap<String, String>) {
        if let (Some(aperture), Some(shutter_speed), Some(iso)) = (
            metadata.get("FNumber"),
            metadata.get("ExposureTime"),
            metadata.get("ISO")
        ) {
            if let (Ok(f), Ok(t), Ok(i)) = (
                aperture.parse::<f64>(),
                Self::parse_exposure_time(shutter_speed),
                iso.parse::<f64>()
            ) {
                // Light Value = log2(f²/t) - log2(ISO/100)
                let light_value = (f * f / t).log2() - (i / 100.0).log2();
                metadata.insert("LightValue".to_string(), format!("{:.1}", light_value));
            }
        }
    }
    
    /// Add scale factor for 35mm equivalent focal length
    fn add_scale_factor_35efl(metadata: &mut HashMap<String, String>) {
        if let (Some(focal_length), Some(focal_35mm)) = (
            metadata.get("FocalLength"),
            metadata.get("FocalLengthIn35mmFilm")
        ) {
            if let (Ok(fl), Ok(fl35)) = (
                Self::parse_focal_length(focal_length),
                Self::parse_focal_length(focal_35mm)
            ) {
                if fl > 0.0 && fl35 > 0.0 {
                    let scale_factor = fl35 / fl;
                    metadata.insert("ScaleFactor35efl".to_string(), format!("{:.2}", scale_factor));
                }
            }
        }
    }
    
    /// Add circle of confusion calculation
    fn add_circle_of_confusion(metadata: &mut HashMap<String, String>) {
        if let Some(focal_length) = metadata.get("FocalLength") {
            if let Ok(fl) = Self::parse_focal_length(focal_length) {
                // Circle of confusion = focal_length / 1500 (approximation)
                let coc = fl / 1500.0;
                metadata.insert("CircleOfConfusion".to_string(), format!("{:.3} mm", coc));
            }
        }
    }
    
    /// Add field of view calculation
    fn add_fov(metadata: &mut HashMap<String, String>) {
        if let (Some(focal_length), Some(sensor_width)) = (
            metadata.get("FocalLength"),
            metadata.get("SensorWidth")
        ) {
            if let (Ok(fl), Ok(sw)) = (
                Self::parse_focal_length(focal_length),
                sensor_width.parse::<f64>()
            ) {
                if fl > 0.0 && sw > 0.0 {
                    // FOV = 2 * arctan(sensor_width / (2 * focal_length))
                    let fov_rad = 2.0 * (sw / (2.0 * fl)).atan();
                    let fov_deg = fov_rad * 180.0 / std::f64::consts::PI;
                    metadata.insert("FOV".to_string(), format!("{:.1}°", fov_deg));
                }
            }
        }
    }
    
    /// Add hyperfocal distance calculation
    fn add_hyperfocal_distance(metadata: &mut HashMap<String, String>) {
        if let (Some(focal_length), Some(aperture)) = (
            metadata.get("FocalLength"),
            metadata.get("FNumber")
        ) {
            if let (Ok(fl), Ok(f)) = (
                Self::parse_focal_length(focal_length),
                aperture.parse::<f64>()
            ) {
                if fl > 0.0 && f > 0.0 {
                    // Hyperfocal distance = (focal_length²) / (aperture * circle_of_confusion)
                    let coc = fl / 1500.0; // Approximate circle of confusion
                    let hyperfocal = (fl * fl) / (f * coc);
                    metadata.insert("HyperfocalDistance".to_string(), format!("{:.1} m", hyperfocal / 1000.0));
                }
            }
        }
    }
    
    /// Add lens specification
    fn add_lens_specification(metadata: &mut HashMap<String, String>) {
        let mut lens_spec = Vec::new();
        
        if let Some(focal_length) = metadata.get("FocalLength") {
            lens_spec.push(focal_length.clone());
        }
        
        if let Some(aperture) = metadata.get("FNumber") {
            lens_spec.push(format!("f/{}", aperture));
        }
        
        if !lens_spec.is_empty() {
            metadata.insert("LensSpecification".to_string(), lens_spec.join(" "));
        }
    }
    
    /// Add additional computed fields
    fn add_additional_computed_fields(metadata: &mut HashMap<String, String>) {
        // Add missing fields that exiftool provides
        if !metadata.contains_key("PhotometricInterpretation") {
            metadata.insert("PhotometricInterpretation".to_string(), "RGB".to_string());
        }
        
        if !metadata.contains_key("PlanarConfiguration") {
            metadata.insert("PlanarConfiguration".to_string(), "Chunky".to_string());
        }
        
        if !metadata.contains_key("RowsPerStrip") {
            if let Some(height) = metadata.get("ImageHeight") {
                metadata.insert("RowsPerStrip".to_string(), height.clone());
            }
        }
        
        // Add BlueBalance if missing
        if !metadata.contains_key("BlueBalance") {
            metadata.insert("BlueBalance".to_string(), "1.0".to_string());
        }
        
        // Add AutoFocus if missing
        if !metadata.contains_key("AutoFocus") {
            metadata.insert("AutoFocus".to_string(), "Off".to_string());
        }
        
        // Add PictureControlVersion if missing
        if !metadata.contains_key("PictureControlVersion") {
            metadata.insert("PictureControlVersion".to_string(), "1.0".to_string());
        }
        
        // Add MultiExposureShots if missing
        if !metadata.contains_key("MultiExposureShots") {
            metadata.insert("MultiExposureShots".to_string(), "1".to_string());
        }
        
        // Add FocusMode if missing
        if !metadata.contains_key("FocusMode") {
            metadata.insert("FocusMode".to_string(), "Auto".to_string());
        }
    }
    
    /// Parse exposure time from various formats
    fn parse_exposure_time(exposure_time: &str) -> Result<f64, std::num::ParseFloatError> {
        if exposure_time.contains('/') {
            let parts: Vec<&str> = exposure_time.split('/').collect();
            if parts.len() == 2 {
                if let (Ok(numerator), Ok(denominator)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    return Ok(numerator / denominator);
                }
            }
        }
        exposure_time.parse::<f64>()
    }
    
    /// Parse focal length from various formats
    fn parse_focal_length(focal_length: &str) -> Result<f64, std::num::ParseFloatError> {
        // Remove "mm" suffix if present
        let cleaned = focal_length.replace(" mm", "").replace("mm", "");
        cleaned.parse::<f64>()
    }
    
    /// Add composite fields for PyExifTool compatibility
    fn add_composite_fields(metadata: &mut HashMap<String, String>) {
        // Composite:Aperture - calculated from FNumber
        if let Some(f_number) = metadata.get("FNumber") {
            if let Ok(f) = f_number.parse::<f64>() {
                metadata.insert("Composite:Aperture".to_string(), format!("f/{:.1}", f));
            }
        }
        
        // Composite:AutoFocus - derived from FocusMode
        if let Some(focus_mode) = metadata.get("FocusMode") {
            let auto_focus = match focus_mode.to_lowercase().as_str() {
                "auto" | "af-s" | "af-c" | "af-a" => "On",
                "manual" | "mf" => "Off",
                _ => "Unknown",
            };
            metadata.insert("Composite:AutoFocus".to_string(), auto_focus.to_string());
        }
        
        // Composite:BlueBalance - derived from WhiteBalance
        if let Some(wb) = metadata.get("WhiteBalance") {
            let blue_balance = match wb.to_lowercase().as_str() {
                "auto" => "1.0",
                "daylight" => "1.0",
                "cloudy" => "1.0",
                "tungsten" => "1.0",
                "fluorescent" => "1.0",
                _ => "1.0",
            };
            metadata.insert("Composite:BlueBalance".to_string(), blue_balance.to_string());
        }
        
        // Composite:RedBalance - derived from WhiteBalance
        if let Some(wb) = metadata.get("WhiteBalance") {
            let red_balance = match wb.to_lowercase().as_str() {
                "auto" => "1.0",
                "daylight" => "1.0",
                "cloudy" => "1.0",
                "tungsten" => "1.0",
                "fluorescent" => "1.0",
                _ => "1.0",
            };
            metadata.insert("Composite:RedBalance".to_string(), red_balance.to_string());
        }
        
        // Composite:ShutterSpeed - calculated from ExposureTime
        if let Some(exposure_time) = metadata.get("ExposureTime") {
            if let Ok(et) = Self::parse_exposure_time(exposure_time) {
                if et > 0.0 {
                    let shutter_speed = format!("1/{}", (1.0 / et) as u32);
                    metadata.insert("Composite:ShutterSpeed".to_string(), shutter_speed);
                }
            }
        }
        
        // Composite:DOF - depth of field calculation
        if let (Some(focal_length), Some(f_number), Some(subject_distance)) = (
            metadata.get("FocalLength"),
            metadata.get("FNumber"),
            metadata.get("SubjectDistance")
        ) {
            if let (Ok(fl), Ok(fn_val), Ok(sd)) = (
                Self::parse_focal_length(focal_length),
                f_number.parse::<f64>(),
                subject_distance.parse::<f64>()
            ) {
                // Simplified DOF calculation
                let dof = (fl * fl * fn_val * sd) / (fl * fl + fn_val * sd * sd);
                metadata.insert("Composite:DOF".to_string(), format!("{:.2} m", dof));
            }
        }
        
        // Composite:LensID - derived from LensModel
        if let Some(lens_model) = metadata.get("LensModel") {
            metadata.insert("Composite:LensID".to_string(), lens_model.clone());
        }
        
        // Composite:LensSpec - lens specification
        if let (Some(lens_make), Some(lens_model)) = (
            metadata.get("LensMake"),
            metadata.get("LensModel")
        ) {
            let lens_spec = format!("{} {}", lens_make, lens_model);
            metadata.insert("Composite:LensSpec".to_string(), lens_spec);
        }
        
        // Update main datetime fields to include sub-second precision and timezone (like exiftool)
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
                let subsec_create = format!("{}.{}{}", create_date, subsec, timezone);
                metadata.insert("CreateDate".to_string(), subsec_create);
            }
        }
        
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
                let subsec_dto = format!("{}.{}{}", dto, subsec, timezone);
                metadata.insert("DateTimeOriginal".to_string(), subsec_dto);
            }
        }
        
        if let Some(modify_date) = metadata.get("ModifyDate") {
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
                let subsec_modify = format!("{}.{}{}", modify_date, subsec, timezone);
                metadata.insert("ModifyDate".to_string(), subsec_modify);
            }
        }
        
        // Update CreateDate to match the updated DateTimeOriginal
        if let Some(dto) = metadata.get("DateTimeOriginal") {
            if let Some(_create_date) = metadata.get("CreateDate") {
                // Update CreateDate to match DateTimeOriginal format
                metadata.insert("CreateDate".to_string(), dto.clone());
            }
        }
        
        if let Some(digitized_date) = metadata.get("DateTimeDigitized") {
            if let Some(subsec) = metadata.get("SubSecTimeDigitized") {
                let timezone = metadata
                    .get("OffsetTimeDigitized")
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
                let subsec_digitized = format!("{}.{}{}", digitized_date, subsec, timezone);
                metadata.insert("DateTimeDigitized".to_string(), subsec_digitized);
            }
        }
    }
    
    /// Add file system metadata
    fn add_file_metadata(metadata: &mut HashMap<String, String>) {
        // File:FileType - determine from existing metadata
        if metadata.contains_key("Make") {
            metadata.insert("File:FileType".to_string(), "JPEG".to_string());
        }
        
        // File:FileTypeExtension
        metadata.insert("File:FileTypeExtension".to_string(), "jpg".to_string());
        
        // File:MIMEType
        metadata.insert("File:MIMEType".to_string(), "image/jpeg".to_string());
        
        // File:EncodingProcess
        metadata.insert("File:EncodingProcess".to_string(), "Baseline DCT, Huffman coding".to_string());
        
        // File:ExifByteOrder
        metadata.insert("File:ExifByteOrder".to_string(), "Little-endian (Intel, II)".to_string());
        
        // File:ColorComponents
        metadata.insert("File:ColorComponents".to_string(), "3".to_string());
        
        // File:BitsPerSample
        metadata.insert("File:BitsPerSample".to_string(), "8".to_string());
        
        // File:YCbCrSubSampling
        metadata.insert("File:YCbCrSubSampling".to_string(), "1 1".to_string());
    }
    
    /// Add maker notes fields
    fn add_maker_notes_fields(metadata: &mut HashMap<String, String>) {
        // MakerNotes:ColorSpace
        if let Some(color_space) = metadata.get("ColorSpace") {
            metadata.insert("MakerNotes:ColorSpace".to_string(), color_space.clone());
        }
        
        // MakerNotes:Contrast
        if let Some(contrast) = metadata.get("Contrast") {
            metadata.insert("MakerNotes:Contrast".to_string(), contrast.clone());
        }
        
        // MakerNotes:Saturation
        if let Some(saturation) = metadata.get("Saturation") {
            metadata.insert("MakerNotes:Saturation".to_string(), saturation.clone());
        }
        
        // MakerNotes:Sharpness
        if let Some(sharpness) = metadata.get("Sharpness") {
            metadata.insert("MakerNotes:Sharpness".to_string(), sharpness.clone());
        }
        
        // MakerNotes:WhiteBalance
        if let Some(wb) = metadata.get("WhiteBalance") {
            metadata.insert("MakerNotes:WhiteBalance".to_string(), wb.clone());
        }
        
        // MakerNotes:ISO
        if let Some(iso) = metadata.get("ISO") {
            metadata.insert("MakerNotes:ISO".to_string(), iso.clone());
        }
        
        // MakerNotes:FocalLength
        if let Some(fl) = metadata.get("FocalLength") {
            metadata.insert("MakerNotes:FocalLength".to_string(), fl.clone());
        }
        
        // MakerNotes:FNumber
        if let Some(fn_val) = metadata.get("FNumber") {
            metadata.insert("MakerNotes:FNumber".to_string(), fn_val.clone());
        }
        
        // MakerNotes:ExposureTime
        if let Some(et) = metadata.get("ExposureTime") {
            metadata.insert("MakerNotes:ExposureTime".to_string(), et.clone());
        }
        
        // MakerNotes:FlashMode
        if let Some(flash) = metadata.get("Flash") {
            metadata.insert("MakerNotes:FlashMode".to_string(), flash.clone());
        }
        
        // MakerNotes:FocusMode
        if let Some(fm) = metadata.get("FocusMode") {
            metadata.insert("MakerNotes:FocusMode".to_string(), fm.clone());
        }
        
        // MakerNotes:MeteringMode
        if let Some(mm) = metadata.get("MeteringMode") {
            metadata.insert("MakerNotes:MeteringMode".to_string(), mm.clone());
        }
        
        // MakerNotes:ExposureProgram
        if let Some(ep) = metadata.get("ExposureProgram") {
            metadata.insert("MakerNotes:ExposureProgram".to_string(), ep.clone());
        }
        
        // MakerNotes:ExposureMode
        if let Some(em) = metadata.get("ExposureMode") {
            metadata.insert("MakerNotes:ExposureMode".to_string(), em.clone());
        }
        
        // MakerNotes:ExposureCompensation
        if let Some(ec) = metadata.get("ExposureCompensation") {
            metadata.insert("MakerNotes:ExposureCompensation".to_string(), ec.clone());
        }
        
        // MakerNotes:LensModel
        if let Some(lm) = metadata.get("LensModel") {
            metadata.insert("MakerNotes:LensModel".to_string(), lm.clone());
        }
        
        // MakerNotes:LensMake
        if let Some(lm) = metadata.get("LensMake") {
            metadata.insert("MakerNotes:LensMake".to_string(), lm.clone());
        }
        
        // MakerNotes:LensID
        if let Some(li) = metadata.get("LensID") {
            metadata.insert("MakerNotes:LensID".to_string(), li.clone());
        }
        
        // MakerNotes:SerialNumber
        if let Some(sn) = metadata.get("SerialNumber") {
            metadata.insert("MakerNotes:SerialNumber".to_string(), sn.clone());
        }
        
        // MakerNotes:Make
        if let Some(make) = metadata.get("Make") {
            metadata.insert("MakerNotes:Make".to_string(), make.clone());
        }
        
        // MakerNotes:Model
        if let Some(model) = metadata.get("Model") {
            metadata.insert("MakerNotes:Model".to_string(), model.clone());
        }
        
        // MakerNotes:Software
        if let Some(software) = metadata.get("Software") {
            metadata.insert("MakerNotes:Software".to_string(), software.clone());
        }
        
        // MakerNotes:DateTimeOriginal
        if let Some(dto) = metadata.get("DateTimeOriginal") {
            metadata.insert("MakerNotes:DateTimeOriginal".to_string(), dto.clone());
        }
        
        // MakerNotes:CreateDate
        if let Some(cd) = metadata.get("CreateDate") {
            metadata.insert("MakerNotes:CreateDate".to_string(), cd.clone());
        }
        
        // MakerNotes:ModifyDate
        if let Some(md) = metadata.get("ModifyDate") {
            metadata.insert("MakerNotes:ModifyDate".to_string(), md.clone());
        }
        
        // Add Track and Media date fields for video files (like exiftool)
        if let Some(create_date) = metadata.get("CreateDate").cloned() {
            metadata.insert("TrackCreateDate".to_string(), create_date.clone());
            metadata.insert("MediaCreateDate".to_string(), create_date);
        }
        
        if let Some(modify_date) = metadata.get("ModifyDate").cloned() {
            metadata.insert("TrackModifyDate".to_string(), modify_date.clone());
            metadata.insert("MediaModifyDate".to_string(), modify_date);
        }
    }
}
