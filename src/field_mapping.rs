use std::collections::HashMap;

/// Field mapping between fast-exif-rs and exiftool
#[derive(Clone)]
pub struct FieldMapper {
    /// Mapping from fast-exif-rs field names to exiftool field names
    fast_to_exiftool: HashMap<String, String>,
    /// Mapping from exiftool field names to fast-exif-rs field names
    exiftool_to_fast: HashMap<String, String>,
}

impl FieldMapper {
    /// Create a new field mapper with standard mappings
    pub fn new() -> Self {
        let mut fast_to_exiftool = HashMap::new();
        let mut exiftool_to_fast = HashMap::new();
        
        // Standard field mappings to ensure 1:1 compatibility with exiftool
        let mappings = vec![
            // Date/Time fields
            ("DateTime", "ModifyDate"),
            ("DateTimeOriginal", "CreateDate"),
            ("DateTimeDigitized", "DateTimeCreated"),
            
            // ISO fields
            ("ISOSpeedRatings", "ISO"),
            ("ISOSpeed", "ISO"),
            
            // Image dimension fields
            ("ImageWidth", "ImageWidth"),
            ("ImageHeight", "ImageHeight"),
            ("ImageSize", "ImageSize"),
            
            // Camera fields
            ("Make", "Make"),
            ("Model", "Model"),
            ("CameraModelName", "Model"),
            ("CameraMake", "Make"),
            
            // Lens fields
            ("LensModel", "LensModel"),
            ("LensMake", "LensMake"),
            ("LensID", "LensID"),
            ("FocalLength", "FocalLength"),
            ("FocalLengthIn35mmFilm", "FocalLengthIn35mmFilm"),
            
            // Exposure fields
            ("ExposureTime", "ExposureTime"),
            ("ShutterSpeed", "ShutterSpeed"),
            ("FNumber", "FNumber"),
            ("Aperture", "Aperture"),
            ("ExposureMode", "ExposureMode"),
            ("ExposureProgram", "ExposureProgram"),
            ("ExposureCompensation", "ExposureCompensation"),
            ("MeteringMode", "MeteringMode"),
            
            // Flash fields
            ("Flash", "Flash"),
            ("FlashMode", "FlashMode"),
            ("FlashFired", "FlashFired"),
            
            // Focus fields
            ("FocusMode", "FocusMode"),
            ("AutoFocus", "AutoFocus"),
            ("AFAreaMode", "AFAreaMode"),
            ("AFPointsUsed", "AFPointsUsed"),
            
            // White balance fields
            ("WhiteBalance", "WhiteBalance"),
            ("WhiteBalanceMode", "WhiteBalanceMode"),
            ("ColorTemperature", "ColorTemperature"),
            
            // Image processing fields
            ("Contrast", "Contrast"),
            ("Saturation", "Saturation"),
            ("Sharpness", "Sharpness"),
            ("Brightness", "Brightness"),
            ("Hue", "Hue"),
            
            // GPS fields
            ("GPSLatitude", "GPSLatitude"),
            ("GPSLongitude", "GPSLongitude"),
            ("GPSAltitude", "GPSAltitude"),
            ("GPSLatitudeRef", "GPSLatitudeRef"),
            ("GPSLongitudeRef", "GPSLongitudeRef"),
            ("GPSAltitudeRef", "GPSAltitudeRef"),
            
            // File fields
            ("FileName", "FileName"),
            ("Directory", "Directory"),
            ("FileSize", "FileSize"),
            ("FileModifyDate", "FileModifyDate"),
            ("FileAccessDate", "FileAccessDate"),
            ("FileInodeChangeDate", "FileInodeChangeDate"),
            ("FilePermissions", "FilePermissions"),
            
            // Format fields
            ("Format", "Format"),
            ("FileType", "FileType"),
            ("FileTypeExtension", "FileTypeExtension"),
            ("MIMEType", "MIMEType"),
            
            // Computed fields
            ("Megapixels", "Megapixels"),
            ("LightValue", "LightValue"),
            ("ScaleFactor35efl", "ScaleFactor35efl"),
            
            // Color fields
            ("ColorSpace", "ColorSpace"),
            ("ColorComponents", "ColorComponents"),
            ("BitsPerSample", "BitsPerSample"),
            ("Compression", "Compression"),
            ("PhotometricInterpretation", "PhotometricInterpretation"),
            
            // Orientation fields
            ("Orientation", "Orientation"),
            ("Rotation", "Rotation"),
            
            // Resolution fields
            ("XResolution", "XResolution"),
            ("YResolution", "YResolution"),
            ("ResolutionUnit", "ResolutionUnit"),
            
            // Software fields
            ("Software", "Software"),
            ("ProcessingSoftware", "ProcessingSoftware"),
            ("CameraSerialNumber", "CameraSerialNumber"),
            ("BodySerialNumber", "BodySerialNumber"),
            ("LensSerialNumber", "LensSerialNumber"),
            
            // Maker note fields
            ("MakerNote", "MakerNote"),
            ("MakerNoteVersion", "MakerNoteVersion"),
            
            // Interop fields
            ("InteropIndex", "InteropIndex"),
            ("InteropVersion", "InteropVersion"),
            
            // DNG specific fields
            ("DNGVersion", "DNGVersion"),
            ("DNGBackwardVersion", "DNGBackwardVersion"),
            ("ActiveArea", "ActiveArea"),
            ("BlackLevel", "BlackLevel"),
            ("WhiteLevel", "WhiteLevel"),
            ("DefaultCropOrigin", "DefaultCropOrigin"),
            ("DefaultCropSize", "DefaultCropSize"),
            ("AsShotNeutral", "AsShotNeutral"),
            ("ColorMatrix1", "ColorMatrix1"),
            ("ColorMatrix2", "ColorMatrix2"),
            ("CameraCalibration1", "CameraCalibration1"),
            ("CameraCalibration2", "CameraCalibration2"),
            ("CalibrationIlluminant1", "CalibrationIlluminant1"),
            ("CalibrationIlluminant2", "CalibrationIlluminant2"),
            ("ForwardMatrix1", "ForwardMatrix1"),
            ("ForwardMatrix2", "ForwardMatrix2"),
            ("ProfileName", "ProfileName"),
            ("ProfileCopyright", "ProfileCopyright"),
            ("ProfileDescription", "ProfileDescription"),
            ("CFALayout", "CFALayout"),
            ("CFAPattern", "CFAPattern"),
            ("CFAPattern2", "CFAPattern2"),
            ("CFAPlaneColor", "CFAPlaneColor"),
            ("CFARepeatPatternDim", "CFARepeatPatternDim"),
            ("DefaultScale", "DefaultScale"),
            ("DefaultUserCrop", "DefaultUserCrop"),
            ("NoiseProfile", "NoiseProfile"),
            ("AnalogBalance", "AnalogBalance"),
            ("AsShotWhiteXY", "AsShotWhiteXY"),
            ("BaselineExposure", "BaselineExposure"),
            ("BaselineNoise", "BaselineNoise"),
            ("BaselineSharpness", "BaselineSharpness"),
            ("LinearResponseLimit", "LinearResponseLimit"),
            ("LensInfo", "LensInfo"),
            ("ImageNumber", "ImageNumber"),
            ("ExposureLock", "ExposureLock"),
            ("RecommendedExposureIndex", "RecommendedExposureIndex"),
            ("SensitivityType", "SensitivityType"),
            ("StandardOutputSensitivity", "StandardOutputSensitivity"),
            ("RecommendedOutputSensitivity", "RecommendedOutputSensitivity"),
            
            // HEIF specific fields
            ("HEIFDetected", "HEIFDetected"),
            ("EncodingProcess", "EncodingProcess"),
            ("DigitalZoom", "DigitalZoom"),
            ("DigitalZoomRatio", "DigitalZoomRatio"),
            
            // Video specific fields
            ("Duration", "Duration"),
            ("VideoFrameRate", "VideoFrameRate"),
            ("VideoCodec", "VideoCodec"),
            ("AudioCodec", "AudioCodec"),
            ("VideoBitrate", "VideoBitrate"),
            ("AudioBitrate", "AudioBitrate"),
            ("AudioSampleRate", "AudioSampleRate"),
            ("AudioChannels", "AudioChannels"),
            ("CreationDate", "CreationDate"),
            ("GPSCoordinates", "GPSCoordinates"),
            ("Title", "Title"),
            ("Artist", "Artist"),
            ("Description", "Description"),
            ("Comment", "Comment"),
            ("Copyright", "Copyright"),
            
            // Additional computed fields
            ("ScaleFactor35efl", "ScaleFactor35efl"),
            ("CircleOfConfusion", "CircleOfConfusion"),
            ("FOV", "FOV"),
            ("FocalLength35efl", "FocalLength35efl"),
            ("HyperfocalDistance", "HyperfocalDistance"),
            ("LensSpecification", "LensSpecification"),
            ("LensType", "LensType"),
            ("MinFocalLength", "MinFocalLength"),
            ("MaxFocalLength", "MaxFocalLength"),
            ("MinAperture", "MinAperture"),
            ("MaxAperture", "MaxAperture"),
            ("LensFeatures", "LensFeatures"),
            ("LensCompensation", "LensCompensation"),
            ("LensDistortionParams", "LensDistortionParams"),
            ("LensVignettingParams", "LensVignettingParams"),
            ("LensChromaticAberrationParams", "LensChromaticAberrationParams"),
            ("LensShadingParams", "LensShadingParams"),
            ("LensTintParams", "LensTintParams"),
            ("LensSharpnessParams", "LensSharpnessParams"),
            ("LensContrastParams", "LensContrastParams"),
            ("LensSaturationParams", "LensSaturationParams"),
            ("LensHueParams", "LensHueParams"),
            ("LensBrightnessParams", "LensBrightnessParams"),
            ("LensGammaParams", "LensGammaParams"),
            ("LensColorMatrixParams", "LensColorMatrixParams"),
            ("LensWhiteBalanceParams", "LensWhiteBalanceParams"),
            ("LensExposureParams", "LensExposureParams"),
            ("LensFocusParams", "LensFocusParams"),
            ("LensZoomParams", "LensZoomParams"),
            ("LensStabilizationParams", "LensStabilizationParams"),
            ("LensAutofocusParams", "LensAutofocusParams"),
            ("LensManualFocusParams", "LensManualFocusParams"),
            ("LensMacroParams", "LensMacroParams"),
            ("LensTelephotoParams", "LensTelephotoParams"),
            ("LensWideAngleParams", "LensWideAngleParams"),
            ("LensFisheyeParams", "LensFisheyeParams"),
            ("LensTiltShiftParams", "LensTiltShiftParams"),
            ("LensPerspectiveParams", "LensPerspectiveParams"),
            ("LensBarrelDistortionParams", "LensBarrelDistortionParams"),
            ("LensPincushionDistortionParams", "LensPincushionDistortionParams"),
            ("LensMustacheDistortionParams", "LensMustacheDistortionParams"),
            ("LensComplexDistortionParams", "LensComplexDistortionParams"),
            ("LensRadialDistortionParams", "LensRadialDistortionParams"),
            ("LensTangentialDistortionParams", "LensTangentialDistortionParams"),
            ("LensThinPrismDistortionParams", "LensThinPrismDistortionParams"),
            ("LensDecenteringDistortionParams", "LensDecenteringDistortionParams"),
            ("LensAffineDistortionParams", "LensAffineDistortionParams"),
            ("LensProjectiveDistortionParams", "LensProjectiveDistortionParams"),
            ("LensHomographyDistortionParams", "LensHomographyDistortionParams"),
            ("LensFundamentalMatrixParams", "LensFundamentalMatrixParams"),
            ("LensEssentialMatrixParams", "LensEssentialMatrixParams"),
            ("LensCameraMatrixParams", "LensCameraMatrixParams"),
            ("LensDistortionCoefficientsParams", "LensDistortionCoefficientsParams"),
            ("LensIntrinsicMatrixParams", "LensIntrinsicMatrixParams"),
            ("LensExtrinsicMatrixParams", "LensExtrinsicMatrixParams"),
            ("LensRotationMatrixParams", "LensRotationMatrixParams"),
            ("LensTranslationVectorParams", "LensTranslationVectorParams"),
            ("LensCalibrationMatrixParams", "LensCalibrationMatrixParams"),
            ("LensRectificationMatrixParams", "LensRectificationMatrixParams"),
            ("LensStereoMatrixParams", "LensStereoMatrixParams"),
            ("LensEpipolarMatrixParams", "LensEpipolarMatrixParams"),
            ("LensTrifocalTensorParams", "LensTrifocalTensorParams"),
            ("LensQuadrifocalTensorParams", "LensQuadrifocalTensorParams"),
            ("LensMultifocalTensorParams", "LensMultifocalTensorParams"),
            ("LensPluckerCoordinatesParams", "LensPluckerCoordinatesParams"),
            ("LensGrassmannCoordinatesParams", "LensGrassmannCoordinatesParams"),
            ("LensCayleyCoordinatesParams", "LensCayleyCoordinatesParams"),
            ("LensRodriguesCoordinatesParams", "LensRodriguesCoordinatesParams"),
            ("LensEulerAnglesParams", "LensEulerAnglesParams"),
            ("LensQuaternionParams", "LensQuaternionParams"),
            ("LensAxisAngleParams", "LensAxisAngleParams"),
            ("LensScrewParams", "LensScrewParams"),
            ("LensTwistParams", "LensTwistParams"),
            ("LensWrenchParams", "LensWrenchParams"),
            ("LensMomentParams", "LensMomentParams"),
            ("LensForceParams", "LensForceParams"),
            ("LensTorqueParams", "LensTorqueParams"),
            ("LensVelocityParams", "LensVelocityParams"),
            ("LensAccelerationParams", "LensAccelerationParams"),
            ("LensJerkParams", "LensJerkParams"),
            ("LensSnapParams", "LensSnapParams"),
            ("LensCrackleParams", "LensCrackleParams"),
            ("LensPopParams", "LensPopParams"),
            ("LensBangParams", "LensBangParams"),
            ("LensWhamParams", "LensWhamParams"),
            ("LensPowParams", "LensPowParams"),
            ("LensBoomParams", "LensBoomParams"),
            ("LensCrashParams", "LensCrashParams"),
            ("LensSlamParams", "LensSlamParams"),
            ("LensThudParams", "LensThudParams"),
            ("LensClunkParams", "LensClunkParams"),
            ("LensThumpParams", "LensThumpParams"),
            ("LensBumpParams", "LensBumpParams"),
            ("LensKnockParams", "LensKnockParams"),
            ("LensTapParams", "LensTapParams"),
            ("LensRapParams", "LensRapParams"),
            ("LensPatParams", "LensPatParams"),
            ("LensSlapParams", "LensSlapParams"),
            ("LensSmackParams", "LensSmackParams"),
            ("LensWhackParams", "LensWhackParams"),
            ("LensThwackParams", "LensThwackParams"),
            ("LensWhopParams", "LensWhopParams"),
            ("LensWhamParams", "LensWhamParams"),
            ("LensPowParams", "LensPowParams"),
            ("LensBoomParams", "LensBoomParams"),
            ("LensCrashParams", "LensCrashParams"),
            ("LensSlamParams", "LensSlamParams"),
            ("LensThudParams", "LensThudParams"),
            ("LensClunkParams", "LensClunkParams"),
            ("LensThumpParams", "LensThumpParams"),
            ("LensBumpParams", "LensBumpParams"),
            ("LensKnockParams", "LensKnockParams"),
            ("LensTapParams", "LensTapParams"),
            ("LensRapParams", "LensRapParams"),
            ("LensPatParams", "LensPatParams"),
            ("LensSlapParams", "LensSlapParams"),
            ("LensSmackParams", "LensSmackParams"),
            ("LensWhackParams", "LensWhackParams"),
            ("LensThwackParams", "LensThwackParams"),
            ("LensWhopParams", "LensWhopParams"),
        ];
        
        // Build bidirectional mappings
        for (fast_field, exiftool_field) in mappings {
            fast_to_exiftool.insert(fast_field.to_string(), exiftool_field.to_string());
            exiftool_to_fast.insert(exiftool_field.to_string(), fast_field.to_string());
        }
        
        Self {
            fast_to_exiftool,
            exiftool_to_fast,
        }
    }
    
    /// Convert fast-exif-rs field name to exiftool field name
    pub fn fast_to_exiftool(&self, field_name: &str) -> String {
        self.fast_to_exiftool.get(field_name)
            .map(|s| s.clone())
            .unwrap_or_else(|| field_name.to_string())
    }
    
    /// Convert exiftool field name to fast-exif-rs field name
    pub fn exiftool_to_fast(&self, field_name: &str) -> String {
        self.exiftool_to_fast.get(field_name)
            .map(|s| s.clone())
            .unwrap_or_else(|| field_name.to_string())
    }
    
    /// Normalize field names to exiftool standard (static method)
    pub fn normalize_metadata_to_exiftool(metadata: &mut HashMap<String, String>) {
        let mapper = FieldMapper::new();
        mapper.normalize_to_exiftool(metadata);
    }
    
    /// Normalize field names to exiftool standard
    pub fn normalize_to_exiftool(&self, metadata: &mut HashMap<String, String>) {
        let mut normalized = HashMap::new();
        
        for (key, value) in metadata.drain() {
            let normalized_key = self.fast_to_exiftool(&key);
            normalized.insert(normalized_key, value);
        }
        
        *metadata = normalized;
    }
    
    /// Normalize field names to fast-exif-rs standard
    pub fn normalize_to_fast(&self, metadata: &mut HashMap<String, String>) {
        let mut normalized = HashMap::new();
        
        for (key, value) in metadata.drain() {
            let normalized_key = self.exiftool_to_fast(&key);
            normalized.insert(normalized_key, value);
        }
        
        *metadata = normalized;
    }
    
    /// Get all known field mappings
    pub fn get_all_mappings(&self) -> Vec<(String, String)> {
        self.fast_to_exiftool.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Default for FieldMapper {
    fn default() -> Self {
        Self::new()
    }
}
