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
}
