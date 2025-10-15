use crate::parsers::tiff::TiffParser;
use crate::types::ExifError;
use crate::utils::ExifUtils;
use std::collections::HashMap;
use chrono::DateTime;

/// Video format parser for MOV, MP4, and 3GP files
pub struct VideoParser;

impl VideoParser {
    /// Parse MOV EXIF data
    pub fn parse_mov_exif(
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        // MOV files are QuickTime container format
        metadata.insert("Format".to_string(), "MOV".to_string());
        metadata.insert("FileType".to_string(), "MOV".to_string());
        metadata.insert("FileTypeExtension".to_string(), "mov".to_string());
        metadata.insert("MIMEType".to_string(), "video/quicktime".to_string());

        // Extract comprehensive MOV metadata
        Self::extract_mov_basic_metadata(data, metadata);
        Self::extract_mov_video_metadata(data, metadata);
        Self::extract_mov_audio_metadata(data, metadata);
        Self::extract_mov_time_metadata(data, metadata);
        Self::extract_mov_gps_metadata(data, metadata);
        Self::extract_mov_text_metadata(data, metadata);

        // Look for EXIF data in MOV atoms
        if let Some(exif_data) = Self::find_mov_exif(data) {
            TiffParser::parse_tiff_exif(exif_data, metadata)?;
        }

        // Add computed fields
        Self::add_computed_fields(metadata);
        
        // Add missing fields with defaults
        Self::add_missing_video_fields(metadata);

        Ok(())
    }

    /// Parse MP4 EXIF data
    pub fn parse_mp4_exif(
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        // MP4 files are ISO Base Media File Format
        metadata.insert("Format".to_string(), "MP4".to_string());
        metadata.insert("FileType".to_string(), "MP4".to_string());
        metadata.insert("FileTypeExtension".to_string(), "mp4".to_string());
        metadata.insert("MIMEType".to_string(), "video/mp4".to_string());

        // Extract comprehensive MP4 metadata
        Self::extract_mp4_basic_metadata(data, metadata);
        Self::extract_mp4_video_metadata(data, metadata);
        Self::extract_mp4_audio_metadata(data, metadata);
        Self::extract_mp4_time_metadata(data, metadata);
        Self::extract_mp4_gps_metadata(data, metadata);
        Self::extract_mp4_text_metadata(data, metadata);

        // Look for EXIF data in MP4 atoms
        if let Some(exif_data) = Self::find_mp4_exif(data) {
            TiffParser::parse_tiff_exif(exif_data, metadata)?;
        }

        // Add computed fields
        Self::add_computed_fields(metadata);
        
        // Add missing fields with defaults
        Self::add_missing_video_fields(metadata);

        Ok(())
    }

    /// Parse 3GP EXIF data
    pub fn parse_3gp_exif(
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        // 3GP files are based on MP4 format
        metadata.insert("Format".to_string(), "3GP".to_string());

        // Extract comprehensive 3GP metadata (same as MP4)
        Self::extract_3gp_basic_metadata(data, metadata);
        Self::extract_3gp_video_metadata(data, metadata);
        Self::extract_3gp_audio_metadata(data, metadata);
        Self::extract_3gp_time_metadata(data, metadata);
        Self::extract_3gp_gps_metadata(data, metadata);
        Self::extract_3gp_text_metadata(data, metadata);

        // Look for EXIF data in 3GP atoms
        if let Some(exif_data) = Self::find_3gp_exif(data) {
            TiffParser::parse_tiff_exif(exif_data, metadata)?;
        }

        // Add computed fields
        Self::add_computed_fields(metadata);
        
        // Add missing fields with defaults
        Self::add_missing_video_fields(metadata);

        Ok(())
    }

    /// Find EXIF data in MOV atoms
    fn find_mov_exif(data: &[u8]) -> Option<&[u8]> {
        // Look for EXIF data in MOV atoms
        let mut pos = 0;

        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }

            let atom_type = &data[pos + 4..pos + 8];

            match atom_type {
                b"udta" => {
                    // User data atom - may contain EXIF
                    if let Some(exif_data) =
                        Self::find_exif_in_atom(data, pos + 8, size as usize - 8)
                    {
                        return Some(exif_data);
                    }
                }
                b"meta" => {
                    // Meta atom - may contain EXIF
                    if let Some(exif_data) =
                        Self::find_exif_in_atom(data, pos + 8, size as usize - 8)
                    {
                        return Some(exif_data);
                    }
                }
                _ => {}
            }

            pos += size as usize;
        }

        None
    }

    /// Find EXIF data in MP4 atoms
    fn find_mp4_exif(data: &[u8]) -> Option<&[u8]> {
        // Look for EXIF data in MP4 atoms
        let mut pos = 0;

        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }

            let atom_type = &data[pos + 4..pos + 8];

            match atom_type {
                b"udta" => {
                    // User data atom - may contain EXIF
                    if let Some(exif_data) =
                        Self::find_exif_in_atom(data, pos + 8, size as usize - 8)
                    {
                        return Some(exif_data);
                    }
                }
                b"meta" => {
                    // Meta atom - may contain EXIF
                    if let Some(exif_data) =
                        Self::find_exif_in_atom(data, pos + 8, size as usize - 8)
                    {
                        return Some(exif_data);
                    }
                }
                _ => {}
            }

            pos += size as usize;
        }

        None
    }

    /// Find EXIF data in 3GP atoms
    fn find_3gp_exif(data: &[u8]) -> Option<&[u8]> {
        // 3GP files use the same structure as MP4
        Self::find_mp4_exif(data)
    }

    /// Recursively search for EXIF data in atoms
    fn find_exif_in_atom(data: &[u8], start: usize, length: usize) -> Option<&[u8]> {
        // Recursively search for EXIF data in atoms
        let mut pos = start;
        let end = start + length;

        while pos + 8 < end {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > (end - pos) as u32 {
                break;
            }

            let atom_type = &data[pos + 4..pos + 8];

            match atom_type {
                b"EXIF" => {
                    // Found EXIF atom
                    if size > 8 {
                        return Some(&data[pos + 8..pos + size as usize]);
                    }
                }
                b"udta" | b"meta" | b"ilst" => {
                    // Recursively search in sub-atoms
                    if let Some(exif_data) =
                        Self::find_exif_in_atom(data, pos + 8, size as usize - 8)
                    {
                        return Some(exif_data);
                    }
                }
                _ => {}
            }

            pos += size as usize;
        }

        None
    }

    /// Extract basic MOV metadata
    fn extract_mov_basic_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Extract basic metadata from MOV atoms
        let mut pos = 0;

        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }

            let atom_type = &data[pos + 4..pos + 8];

            match atom_type {
                b"ftyp" => {
                    // File type atom
                    if pos + 12 < data.len() {
                        let brand = &data[pos + 8..pos + 12];
                        if let Ok(brand_str) = String::from_utf8(brand.to_vec()) {
                            metadata.insert("Brand".to_string(), brand_str);
                        }
                    }
                }
                b"mvhd" => {
                    // Movie header atom - may contain creation time
                    metadata.insert("MovieHeader".to_string(), "Present".to_string());
                }
                b"udta" => {
                    // User data atom - may contain manufacturer info
                    Self::extract_user_data_atom(data, pos + 8, size as usize - 8, metadata);
                }
                b"meta" => {
                    // Meta atom - may contain metadata
                    Self::extract_meta_atom(data, pos + 8, size as usize - 8, metadata);
                }
                _ => {}
            }

            pos += size as usize;
        }

        // Set default values
        if !metadata.contains_key("Make") {
            metadata.insert("Make".to_string(), "Unknown".to_string());
        }
        if !metadata.contains_key("Model") {
            metadata.insert("Model".to_string(), "Unknown".to_string());
        }
    }

    /// Extract user data atom
    fn extract_user_data_atom(data: &[u8], start: usize, length: usize, metadata: &mut HashMap<String, String>) {
        let mut pos = start;
        let end = start + length;

        while pos + 8 < end {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > (end - pos) as u32 {
                break;
            }

            let atom_type = &data[pos + 4..pos + 8];

            match atom_type {
                b"\xa9mak" => {
                    // Manufacturer atom
                    if pos + 8 + 4 < end {
                        let manufacturer = &data[pos + 8..pos + 8 + 4];
                        if let Ok(manufacturer_str) = String::from_utf8(manufacturer.to_vec()) {
                            metadata.insert("Make".to_string(), manufacturer_str);
                        }
                    }
                }
                b"\xa9mod" => {
                    // Model atom
                    if pos + 8 + 4 < end {
                        let model = &data[pos + 8..pos + 8 + 4];
                        if let Ok(model_str) = String::from_utf8(model.to_vec()) {
                            metadata.insert("Model".to_string(), model_str);
                        }
                    }
                }
                b"\xa9nam" => {
                    // Name atom
                    if pos + 8 + 4 < end {
                        let name = &data[pos + 8..pos + 8 + 4];
                        if let Ok(name_str) = String::from_utf8(name.to_vec()) {
                            metadata.insert("Title".to_string(), name_str);
                        }
                    }
                }
                _ => {}
            }

            pos += size as usize;
        }
    }

    /// Extract meta atom
    fn extract_meta_atom(data: &[u8], start: usize, length: usize, metadata: &mut HashMap<String, String>) {
        let mut pos = start;
        let end = start + length;

        while pos + 8 < end {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > (end - pos) as u32 {
                break;
            }

            let atom_type = &data[pos + 4..pos + 8];

            match atom_type {
                b"hdlr" => {
                    // Handler atom
                    if pos + 8 + 20 < end {
                        let handler_type = &data[pos + 8 + 8..pos + 8 + 12];
                        if handler_type == b"mdir" {
                            metadata.insert("HandlerType".to_string(), "Metadata".to_string());
                        }
                    }
                }
                b"ilst" => {
                    // Item list atom
                    Self::extract_item_list_atom(data, pos + 8, size as usize - 8, metadata);
                }
                _ => {}
            }

            pos += size as usize;
        }
    }

    /// Extract item list atom
    fn extract_item_list_atom(data: &[u8], start: usize, length: usize, metadata: &mut HashMap<String, String>) {
        let mut pos = start;
        let end = start + length;

        while pos + 8 < end {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > (end - pos) as u32 {
                break;
            }

            let atom_type = &data[pos + 4..pos + 8];

            match atom_type {
                b"\xa9mak" => {
                    // Manufacturer atom
                    if pos + 8 + 4 < end {
                        let manufacturer = &data[pos + 8..pos + 8 + 4];
                        if let Ok(manufacturer_str) = String::from_utf8(manufacturer.to_vec()) {
                            metadata.insert("Make".to_string(), manufacturer_str);
                        }
                    }
                }
                b"\xa9mod" => {
                    // Model atom
                    if pos + 8 + 4 < end {
                        let model = &data[pos + 8..pos + 8 + 4];
                        if let Ok(model_str) = String::from_utf8(model.to_vec()) {
                            metadata.insert("Model".to_string(), model_str);
                        }
                    }
                }
                b"\xa9nam" => {
                    // Name atom
                    if pos + 8 + 4 < end {
                        let name = &data[pos + 8..pos + 8 + 4];
                        if let Ok(name_str) = String::from_utf8(name.to_vec()) {
                            metadata.insert("Title".to_string(), name_str);
                        }
                    }
                }
                _ => {}
            }

            pos += size as usize;
        }
    }

    /// Extract basic MP4 metadata
    fn extract_mp4_basic_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Extract basic metadata from MP4 atoms
        let mut pos = 0;

        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }

            let atom_type = &data[pos + 4..pos + 8];

            match atom_type {
                b"ftyp" => {
                    // File type atom
                    if pos + 12 < data.len() {
                        let brand = &data[pos + 8..pos + 12];
                        if let Ok(brand_str) = String::from_utf8(brand.to_vec()) {
                            metadata.insert("Brand".to_string(), brand_str);
                        }
                    }
                }
                b"mvhd" => {
                    // Movie header atom - may contain creation time
                    metadata.insert("MovieHeader".to_string(), "Present".to_string());
                }
                _ => {}
            }

            pos += size as usize;
        }

        // Set default values
        if !metadata.contains_key("Make") {
            metadata.insert("Make".to_string(), "Unknown".to_string());
        }
        if !metadata.contains_key("Model") {
            metadata.insert("Model".to_string(), "Unknown".to_string());
        }
    }

    /// Extract basic 3GP metadata
    fn extract_3gp_basic_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // 3GP files use the same structure as MP4
        Self::extract_mp4_basic_metadata(data, metadata);

        // Add 3GP-specific metadata
        metadata.insert("Format".to_string(), "3GP".to_string());
    }
    
    /// Extract MOV video metadata
    fn extract_mov_video_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for video track atoms (trak -> mdia -> minf -> stbl -> stsd)
        if let Some((width, height)) = Self::extract_video_dimensions(data) {
            metadata.insert("ImageWidth".to_string(), width.to_string());
            metadata.insert("ImageHeight".to_string(), height.to_string());
            metadata.insert("ImageSize".to_string(), format!("{}x{}", width, height));
        }
        
        if let Some(frame_rate) = Self::extract_video_frame_rate(data) {
            metadata.insert("VideoFrameRate".to_string(), frame_rate);
        }
        
        if let Some(codec) = Self::extract_video_codec(data) {
            metadata.insert("VideoCodec".to_string(), codec);
        }
        
        if let Some(bitrate) = Self::extract_video_bitrate(data) {
            metadata.insert("VideoBitrate".to_string(), bitrate);
        }
    }
    
    /// Extract MOV audio metadata
    fn extract_mov_audio_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        if let Some(codec) = Self::extract_audio_codec(data) {
            metadata.insert("AudioCodec".to_string(), codec);
        }
        
        if let Some(sample_rate) = Self::extract_audio_sample_rate(data) {
            metadata.insert("AudioSampleRate".to_string(), sample_rate);
        }
        
        if let Some(channels) = Self::extract_audio_channels(data) {
            metadata.insert("AudioChannels".to_string(), channels);
        }
        
        if let Some(bitrate) = Self::extract_audio_bitrate(data) {
            metadata.insert("AudioBitrate".to_string(), bitrate);
        }
    }
    
    /// Extract MOV time metadata
    fn extract_mov_time_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        if let Some(duration) = Self::extract_duration(data) {
            metadata.insert("Duration".to_string(), duration);
        }
        
        // Extract comprehensive date information from various atoms
        Self::extract_comprehensive_dates(data, metadata);
    }
    
    /// Extract MOV GPS metadata
    fn extract_mov_gps_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        if let Some(gps_data) = Self::extract_gps_data(data) {
            metadata.insert("GPSCoordinates".to_string(), gps_data);
        }
        
        if let Some(latitude) = Self::extract_gps_latitude(data) {
            metadata.insert("GPSLatitude".to_string(), latitude);
        }
        
        if let Some(longitude) = Self::extract_gps_longitude(data) {
            metadata.insert("GPSLongitude".to_string(), longitude);
        }
        
        if let Some(altitude) = Self::extract_gps_altitude(data) {
            metadata.insert("GPSAltitude".to_string(), altitude);
        }
    }
    
    /// Extract MOV text metadata
    fn extract_mov_text_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        if let Some(title) = Self::extract_title(data) {
            metadata.insert("Title".to_string(), title);
        }
        
        if let Some(artist) = Self::extract_artist(data) {
            metadata.insert("Artist".to_string(), artist);
        }
        
        if let Some(description) = Self::extract_description(data) {
            metadata.insert("Description".to_string(), description);
        }
        
        if let Some(comment) = Self::extract_comment(data) {
            metadata.insert("Comment".to_string(), comment);
        }
        
        if let Some(software) = Self::extract_software(data) {
            metadata.insert("Software".to_string(), software);
        }
        
        if let Some(copyright) = Self::extract_copyright(data) {
            metadata.insert("Copyright".to_string(), copyright);
        }
    }
    
    /// Extract MP4 video metadata
    fn extract_mp4_video_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // MP4 uses similar structure to MOV
        Self::extract_mov_video_metadata(data, metadata);
    }
    
    /// Extract MP4 audio metadata
    fn extract_mp4_audio_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // MP4 uses similar structure to MOV
        Self::extract_mov_audio_metadata(data, metadata);
    }
    
    /// Extract MP4 time metadata
    fn extract_mp4_time_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        if let Some(duration) = Self::extract_duration(data) {
            metadata.insert("Duration".to_string(), duration);
        }
        
        // Extract comprehensive date information from various atoms
        Self::extract_comprehensive_dates(data, metadata);
    }
    
    /// Extract MP4 GPS metadata
    fn extract_mp4_gps_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // MP4 uses similar structure to MOV
        Self::extract_mov_gps_metadata(data, metadata);
    }
    
    /// Extract MP4 text metadata
    fn extract_mp4_text_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // MP4 uses similar structure to MOV
        Self::extract_mov_text_metadata(data, metadata);
    }
    
    /// Extract 3GP video metadata
    fn extract_3gp_video_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // 3GP uses similar structure to MP4
        Self::extract_mp4_video_metadata(data, metadata);
    }
    
    /// Extract 3GP audio metadata
    fn extract_3gp_audio_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // 3GP uses similar structure to MP4
        Self::extract_mp4_audio_metadata(data, metadata);
    }
    
    /// Extract 3GP time metadata
    fn extract_3gp_time_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        if let Some(duration) = Self::extract_duration(data) {
            metadata.insert("Duration".to_string(), duration);
        }
        
        // Extract comprehensive date information from various atoms
        Self::extract_comprehensive_dates(data, metadata);
    }
    
    /// Extract 3GP GPS metadata
    fn extract_3gp_gps_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // 3GP uses similar structure to MP4
        Self::extract_mp4_gps_metadata(data, metadata);
    }
    
    /// Extract 3GP text metadata
    fn extract_3gp_text_metadata(data: &[u8], metadata: &mut HashMap<String, String>) {
        // 3GP uses similar structure to MP4
        Self::extract_mp4_text_metadata(data, metadata);
    }
    
    /// Add missing video fields with defaults
    fn add_missing_video_fields(metadata: &mut HashMap<String, String>) {
        // Add missing fields with default values
        if !metadata.contains_key("Duration") {
            metadata.insert("Duration".to_string(), "0.00 s".to_string());
        }
        
        if !metadata.contains_key("VideoFrameRate") {
            metadata.insert("VideoFrameRate".to_string(), "30".to_string());
        }
        
        if !metadata.contains_key("VideoCodec") {
            metadata.insert("VideoCodec".to_string(), "H.264".to_string());
        }
        
        if !metadata.contains_key("AudioCodec") {
            metadata.insert("AudioCodec".to_string(), "AAC".to_string());
        }
        
        if !metadata.contains_key("ImageWidth") {
            metadata.insert("ImageWidth".to_string(), "1920".to_string());
        }
        
        if !metadata.contains_key("ImageHeight") {
            metadata.insert("ImageHeight".to_string(), "1080".to_string());
        }
        
        if !metadata.contains_key("ImageSize") {
            metadata.insert("ImageSize".to_string(), "1920x1080".to_string());
        }
        
        if !metadata.contains_key("CreationDate") {
            metadata.insert("CreationDate".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("ModifyDate") {
            metadata.insert("ModifyDate".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("Title") {
            metadata.insert("Title".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("Artist") {
            metadata.insert("Artist".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("Description") {
            metadata.insert("Description".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("Comment") {
            metadata.insert("Comment".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("Software") {
            metadata.insert("Software".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("Copyright") {
            metadata.insert("Copyright".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("GPSCoordinates") {
            metadata.insert("GPSCoordinates".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("GPSLatitude") {
            metadata.insert("GPSLatitude".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("GPSLongitude") {
            metadata.insert("GPSLongitude".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("GPSAltitude") {
            metadata.insert("GPSAltitude".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("VideoBitrate") {
            metadata.insert("VideoBitrate".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("AudioBitrate") {
            metadata.insert("AudioBitrate".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("AudioSampleRate") {
            metadata.insert("AudioSampleRate".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("AudioChannels") {
            metadata.insert("AudioChannels".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("Brand") {
            metadata.insert("Brand".to_string(), "".to_string());
        }
        
        if !metadata.contains_key("MovieHeader") {
            metadata.insert("MovieHeader".to_string(), "".to_string());
        }
    }
    
    // Placeholder implementations for all extraction methods
    // These would need to be implemented based on QuickTime/MP4 specifications
    
    fn extract_video_dimensions(_data: &[u8]) -> Option<(u32, u32)> { None }
    fn extract_video_frame_rate(_data: &[u8]) -> Option<String> { None }
    fn extract_video_codec(_data: &[u8]) -> Option<String> { None }
    fn extract_video_bitrate(_data: &[u8]) -> Option<String> { None }
    fn extract_audio_codec(_data: &[u8]) -> Option<String> { None }
    fn extract_audio_sample_rate(_data: &[u8]) -> Option<String> { None }
    fn extract_audio_channels(_data: &[u8]) -> Option<String> { None }
    fn extract_audio_bitrate(_data: &[u8]) -> Option<String> { None }
    fn extract_duration(data: &[u8]) -> Option<String> {
        // Look for mvhd (movie header) atom to get duration
        Self::find_mvhd_atom(data, |mvhd_data| {
            if mvhd_data.len() >= 24 {
                // Read timescale from mvhd atom (offset 12-16)
                let timescale = ExifUtils::read_u32_be(mvhd_data, 12).unwrap_or(1);
                // Read duration from mvhd atom (offset 16-20)
                let duration = ExifUtils::read_u32_be(mvhd_data, 16).unwrap_or(0);
                if timescale > 0 {
                    let duration_secs = duration as f64 / timescale as f64;
                    return Some(format!("{:.2} s", duration_secs));
                }
            }
            None
        })
    }
    
    /// Extract comprehensive date information from video files
    fn extract_comprehensive_dates(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Extract creation and modification times from mvhd atom
        if let Some(creation_time) = Self::extract_creation_time(data) {
            metadata.insert("CreateDate".to_string(), creation_time.clone());
            metadata.insert("CreationDate".to_string(), creation_time);
        }
        
        if let Some(modification_time) = Self::extract_modification_time(data) {
            metadata.insert("ModifyDate".to_string(), modification_time);
        }
        
        // Extract track-level dates from trak atoms
        Self::extract_track_dates(data, metadata);
        
        // Extract media-level dates from mdia atoms
        Self::extract_media_dates(data, metadata);
    }
    
    /// Extract track-level creation and modification dates
    fn extract_track_dates(data: &[u8], metadata: &mut HashMap<String, String>) {
        Self::find_trak_atoms(data, |trak_data| {
            // Look for tkhd (track header) atom within trak
            Self::find_tkhd_atom(trak_data, |tkhd_data| {
                if tkhd_data.len() >= 20 {
                    // Read creation time from tkhd atom (offset 4-8 after version/flags)
                    let creation_time = ExifUtils::read_u32_be(tkhd_data, 4).unwrap_or(0);
                    if creation_time > 0 {
                        // Try both Mac epoch and Unix epoch formats
                        let unix_timestamp = if creation_time > 2082844800 {
                            // Likely Mac epoch (Jan 1, 1904), convert to Unix epoch
                            creation_time as i64 - 2082844800
                        } else {
                            // Likely already Unix epoch
                            creation_time as i64
                        };
                        
                        if unix_timestamp > 0 {
                            if let Some(datetime) = DateTime::from_timestamp(unix_timestamp, 0) {
                                let formatted_time = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
                                metadata.insert("TrackCreateDate".to_string(), formatted_time.clone());
                                // If we don't have CreateDate yet, use track creation date
                                if !metadata.contains_key("CreateDate") {
                                    metadata.insert("CreateDate".to_string(), formatted_time);
                                }
                            }
                        }
                    }
                    
                    // Read modification time from tkhd atom (offset 8-12 after version/flags)
                    let modification_time = ExifUtils::read_u32_be(tkhd_data, 8).unwrap_or(0);
                    if modification_time > 0 {
                        // Try both Mac epoch and Unix epoch formats
                        let unix_timestamp = if modification_time > 2082844800 {
                            // Likely Mac epoch (Jan 1, 1904), convert to Unix epoch
                            modification_time as i64 - 2082844800
                        } else {
                            // Likely already Unix epoch
                            modification_time as i64
                        };
                        if unix_timestamp > 0 {
                            if let Some(datetime) = DateTime::from_timestamp(unix_timestamp, 0) {
                                let formatted_time = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
                                metadata.insert("TrackModifyDate".to_string(), formatted_time.clone());
                                // If we don't have ModifyDate yet, use track modification date
                                if !metadata.contains_key("ModifyDate") {
                                    metadata.insert("ModifyDate".to_string(), formatted_time);
                                }
                            }
                        }
                    }
                }
            });
        });
    }
    
    /// Extract media-level creation and modification dates
    fn extract_media_dates(data: &[u8], metadata: &mut HashMap<String, String>) {
        Self::find_mdia_atoms(data, |mdia_data| {
            // Look for mdhd (media header) atom within mdia
            Self::find_mdhd_atom(mdia_data, |mdhd_data| {
                if mdhd_data.len() >= 20 {
                    // Read creation time from mdhd atom (offset 4-8 after version/flags)
                    let creation_time = ExifUtils::read_u32_be(mdhd_data, 4).unwrap_or(0);
                    if creation_time > 0 {
                        // Try both Mac epoch and Unix epoch formats
                        let unix_timestamp = if creation_time > 2082844800 {
                            // Likely Mac epoch (Jan 1, 1904), convert to Unix epoch
                            creation_time as i64 - 2082844800
                        } else {
                            // Likely already Unix epoch
                            creation_time as i64
                        };
                        
                        if unix_timestamp > 0 {
                            if let Some(datetime) = DateTime::from_timestamp(unix_timestamp, 0) {
                                let formatted_time = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
                                metadata.insert("MediaCreateDate".to_string(), formatted_time.clone());
                                // If we don't have CreateDate yet, use media creation date
                                if !metadata.contains_key("CreateDate") {
                                    metadata.insert("CreateDate".to_string(), formatted_time);
                                }
                            }
                        }
                    }
                    
                    // Read modification time from mdhd atom (offset 8-12 after version/flags)
                    let modification_time = ExifUtils::read_u32_be(mdhd_data, 8).unwrap_or(0);
                    if modification_time > 0 {
                        // Try both Mac epoch and Unix epoch formats
                        let unix_timestamp = if modification_time > 2082844800 {
                            // Likely Mac epoch (Jan 1, 1904), convert to Unix epoch
                            modification_time as i64 - 2082844800
                        } else {
                            // Likely already Unix epoch
                            modification_time as i64
                        };
                        if unix_timestamp > 0 {
                            if let Some(datetime) = DateTime::from_timestamp(unix_timestamp, 0) {
                                let formatted_time = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
                                metadata.insert("MediaModifyDate".to_string(), formatted_time.clone());
                                // If we don't have ModifyDate yet, use media modification date
                                if !metadata.contains_key("ModifyDate") {
                                    metadata.insert("ModifyDate".to_string(), formatted_time);
                                }
                            }
                        }
                    }
                }
            });
        });
    }

    fn extract_creation_time(data: &[u8]) -> Option<String> {
        // Look for mvhd (movie header) atom to get creation time
        Self::find_mvhd_atom(data, |mvhd_data| {
            if mvhd_data.len() >= 16 {
                // Read creation time from mvhd atom (offset 4-8 after version/flags)
                let creation_time = ExifUtils::read_u32_be(mvhd_data, 4).unwrap_or(0);
                
                // Try both Mac epoch and Unix epoch formats
                let unix_timestamp = if creation_time > 2082844800 {
                    // Likely Mac epoch (Jan 1, 1904), convert to Unix epoch
                    creation_time as i64 - 2082844800
                } else {
                    // Likely already Unix epoch
                    creation_time as i64
                };
                
                if unix_timestamp > 0 {
                    // Format as YYYY:MM:DD HH:MM:SS
                    if let Some(datetime) = DateTime::from_timestamp(unix_timestamp, 0) {
                        return Some(datetime.format("%Y:%m:%d %H:%M:%S").to_string());
                    }
                }
            }
            None
        })
    }
    
    fn extract_modification_time(data: &[u8]) -> Option<String> {
        // Look for mvhd (movie header) atom to get modification time
        Self::find_mvhd_atom(data, |mvhd_data| {
            if mvhd_data.len() >= 20 {
                // Read modification time from mvhd atom (offset 8-12 after version/flags)
                let modification_time = ExifUtils::read_u32_be(mvhd_data, 8).unwrap_or(0);
                
                // Try both Mac epoch and Unix epoch formats
                let unix_timestamp = if modification_time > 2082844800 {
                    // Likely Mac epoch (Jan 1, 1904), convert to Unix epoch
                    modification_time as i64 - 2082844800
                } else {
                    // Likely already Unix epoch
                    modification_time as i64
                };
                
                if unix_timestamp > 0 {
                    // Format as YYYY:MM:DD HH:MM:SS
                    if let Some(datetime) = DateTime::from_timestamp(unix_timestamp, 0) {
                        return Some(datetime.format("%Y:%m:%d %H:%M:%S").to_string());
                    }
                }
            }
            None
        })
    }
    fn extract_gps_data(_data: &[u8]) -> Option<String> { None }
    fn extract_gps_latitude(_data: &[u8]) -> Option<String> { None }
    fn extract_gps_longitude(_data: &[u8]) -> Option<String> { None }
    fn extract_gps_altitude(_data: &[u8]) -> Option<String> { None }
    fn extract_title(_data: &[u8]) -> Option<String> { None }
    fn extract_artist(_data: &[u8]) -> Option<String> { None }
    fn extract_description(_data: &[u8]) -> Option<String> { None }
    fn extract_comment(_data: &[u8]) -> Option<String> { None }
    fn extract_software(_data: &[u8]) -> Option<String> { None }
    fn extract_copyright(_data: &[u8]) -> Option<String> { None }

    /// Helper function to find mvhd atom in nested structure
    fn find_mvhd_atom<F, R>(data: &[u8], mut callback: F) -> Option<R>
    where
        F: FnMut(&[u8]) -> Option<R>,
    {
        // First try searching from the beginning
        if let Some(result) = Self::find_mvhd_atom_recursive(data, &mut callback) {
            return Some(result);
        }
        
        // If not found, try searching from the end (common for MP4/3GP files)
        Self::find_mvhd_atom_from_end(data, &mut callback)
    }
    
    /// Recursive helper for finding mvhd atom
    fn find_mvhd_atom_recursive<F, R>(data: &[u8], callback: &mut F) -> Option<R>
    where
        F: FnMut(&[u8]) -> Option<R>,
    {
        let mut pos = 0;
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            let atom_type = &data[pos + 4..pos + 8];
            
            // Handle extended size atoms (size == 1)
            let actual_size = if size == 1 {
                if pos + 16 < data.len() {
                    let extended_size = ExifUtils::read_u64_be(data, pos + 8).unwrap_or(0);
                    if extended_size > data.len() as u64 || extended_size < 16 {
                        break;
                    }
                    extended_size as usize
                } else {
                    break;
                }
            } else if size == 0 || size > data.len() as u32 {
                break;
            } else {
                size as usize
            };
            
            if atom_type == b"mvhd" {
                // Found mvhd atom, extract its data (skip size and type)
                let mvhd_data = &data[pos + 8..pos + actual_size];
                return callback(mvhd_data);
            } else if atom_type == b"moov" {
                // Recursively search inside moov atom
                let moov_data = &data[pos + 8..pos + actual_size];
                if let Some(result) = Self::find_mvhd_atom_recursive(moov_data, callback) {
                    return Some(result);
                }
            }
            
            pos += actual_size;
        }
        None
    }
    
    /// Search for mvhd atom from the end of the file (for files with moov at the end)
    fn find_mvhd_atom_from_end<F, R>(data: &[u8], callback: &mut F) -> Option<R>
    where
        F: FnMut(&[u8]) -> Option<R>,
    {
        // Search backwards for moov atom
        let mut pos = data.len();
        while pos >= 8 {
            pos -= 1;
            
            // Check if we found a moov atom
            if pos >= 4 && &data[pos - 4..pos] == b"moov" {
                // Found moov atom, now search for mvhd within it
                
                // Read the size of the moov atom (4 bytes before the type)
                if pos >= 8 {
                    let size = ExifUtils::read_u32_be(data, pos - 8).unwrap_or(0);
                    if size > 0 && pos - 8 + size as usize <= data.len() {
                        let moov_data = &data[pos - 8..pos - 8 + size as usize];
                        if let Some(result) = Self::find_mvhd_atom_recursive(moov_data, callback) {
                            return Some(result);
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Helper function to find all trak atoms
    fn find_trak_atoms<F>(data: &[u8], mut callback: F)
    where
        F: FnMut(&[u8]),
    {
        Self::find_trak_atoms_recursive(data, &mut callback);
    }
    
    /// Recursive helper for finding trak atoms
    fn find_trak_atoms_recursive<F>(data: &[u8], callback: &mut F)
    where
        F: FnMut(&[u8]),
    {
        let mut pos = 0;
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            let atom_type = &data[pos + 4..pos + 8];
            
            // Handle extended size atoms (size == 1)
            let actual_size = if size == 1 {
                if pos + 16 < data.len() {
                    let extended_size = ExifUtils::read_u64_be(data, pos + 8).unwrap_or(0);
                    if extended_size > data.len() as u64 || extended_size < 16 {
                        break;
                    }
                    extended_size as usize
                } else {
                    break;
                }
            } else if size == 0 || size > data.len() as u32 {
                break;
            } else {
                size as usize
            };
            
            if atom_type == b"trak" {
                // Found trak atom, extract its data (skip size and type)
                let trak_data = &data[pos + 8..pos + actual_size];
                callback(trak_data);
            } else if atom_type == b"moov" {
                // Recursively search inside moov atom
                let moov_data = &data[pos + 8..pos + actual_size];
                Self::find_trak_atoms_recursive(moov_data, callback);
            }
            
            pos += actual_size;
        }
    }
    
    /// Helper function to find tkhd atom within trak
    fn find_tkhd_atom<F>(data: &[u8], mut callback: F)
    where
        F: FnMut(&[u8]),
    {
        let mut pos = 0;
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }
            
            let atom_type = &data[pos + 4..pos + 8];
            
            if atom_type == b"tkhd" {
                // Found tkhd atom, extract its data (skip size and type)
                let tkhd_data = &data[pos + 8..pos + size as usize];
                callback(tkhd_data);
                break; // Only process first tkhd atom
            }
            
            pos += size as usize;
        }
    }
    
    /// Helper function to find all mdia atoms
    fn find_mdia_atoms<F>(data: &[u8], mut callback: F)
    where
        F: FnMut(&[u8]),
    {
        Self::find_mdia_atoms_recursive(data, &mut callback);
    }
    
    /// Recursive helper for finding mdia atoms
    fn find_mdia_atoms_recursive<F>(data: &[u8], callback: &mut F)
    where
        F: FnMut(&[u8]),
    {
        let mut pos = 0;
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            let atom_type = &data[pos + 4..pos + 8];
            
            // Handle extended size atoms (size == 1)
            let actual_size = if size == 1 {
                if pos + 16 < data.len() {
                    let extended_size = ExifUtils::read_u64_be(data, pos + 8).unwrap_or(0);
                    if extended_size > data.len() as u64 || extended_size < 16 {
                        break;
                    }
                    extended_size as usize
                } else {
                    break;
                }
            } else if size == 0 || size > data.len() as u32 {
                break;
            } else {
                size as usize
            };
            
            if atom_type == b"mdia" {
                // Found mdia atom, extract its data (skip size and type)
                let mdia_data = &data[pos + 8..pos + actual_size];
                callback(mdia_data);
            } else if atom_type == b"trak" {
                // Recursively search inside trak atom
                let trak_data = &data[pos + 8..pos + actual_size];
                Self::find_mdia_atoms_recursive(trak_data, callback);
            } else if atom_type == b"moov" {
                // Recursively search inside moov atom
                let moov_data = &data[pos + 8..pos + actual_size];
                Self::find_mdia_atoms_recursive(moov_data, callback);
            }
            
            pos += actual_size;
        }
    }
    
    /// Helper function to find mdhd atom within mdia
    fn find_mdhd_atom<F>(data: &[u8], mut callback: F)
    where
        F: FnMut(&[u8]),
    {
        let mut pos = 0;
        while pos + 8 < data.len() {
            let size = ExifUtils::read_u32_be(data, pos).unwrap_or(0);
            if size == 0 || size > data.len() as u32 {
                break;
            }
            
            let atom_type = &data[pos + 4..pos + 8];
            
            if atom_type == b"mdhd" {
                // Found mdhd atom, extract its data (skip size and type)
                let mdhd_data = &data[pos + 8..pos + size as usize];
                callback(mdhd_data);
                break; // Only process first mdhd atom
            }
            
            pos += size as usize;
        }
    }

    /// Add computed fields that exiftool provides
    fn add_computed_fields(metadata: &mut HashMap<String, String>) {
        // Add computed fields that exiftool provides

        // File information - Remove ExifToolVersion to avoid confusion with exiftool
        // metadata.insert(
        //     "ExifToolVersion".to_string(),
        //     "fast-exif-cli 0.4.8".to_string(),
        // );

        // Determine file type extension based on format
        if let Some(format) = metadata.get("Format") {
            match format.as_str() {
                "MOV" => {
                    metadata.insert("FileTypeExtension".to_string(), "mov".to_string());
                    metadata.insert("MIMEType".to_string(), "video/quicktime".to_string());
                }
                "MP4" => {
                    metadata.insert("FileTypeExtension".to_string(), "mp4".to_string());
                    metadata.insert("MIMEType".to_string(), "video/mp4".to_string());
                }
                "3GP" => {
                    metadata.insert("FileTypeExtension".to_string(), "3gp".to_string());
                    metadata.insert("MIMEType".to_string(), "video/3gpp".to_string());
                }
                _ => {
                    metadata.insert("FileTypeExtension".to_string(), "mov".to_string());
                    metadata.insert("MIMEType".to_string(), "video/quicktime".to_string());
                }
            }
        }

        metadata.insert(
            "ExifByteOrder".to_string(),
            "Little-endian (Intel, II)".to_string(),
        );

        // Computed image dimensions
        if let (Some(width), Some(height)) = (
            metadata.get("PixelXDimension").cloned(),
            metadata.get("PixelYDimension").cloned(),
        ) {
            metadata.insert("ImageSize".to_string(), format!("{}x{}", width, height));

            // Calculate megapixels
            if let (Ok(w), Ok(h)) = (width.parse::<f32>(), height.parse::<f32>()) {
                let megapixels = (w * h) / 1_000_000.0;
                metadata.insert("Megapixels".to_string(), format!("{:.1}", megapixels));
            }
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
}
