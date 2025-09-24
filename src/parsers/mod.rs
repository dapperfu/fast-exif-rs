//! Parser modules for different image and video formats
//! 
//! This module contains specialized parsers for various formats:
//! - JPEG: Standard JPEG EXIF parsing
//! - RAW: Canon CR2, Nikon NEF, Olympus ORF, Adobe DNG
//! - HEIF: HEIF/HEIC format parsing
//! - Video: MOV, MP4, 3GP video format parsing
//! - TIFF: TIFF-based EXIF parsing
//! - Maker Notes: Camera manufacturer specific data

pub mod jpeg;
pub mod raw;
pub mod heif;
pub mod video;
pub mod tiff;
pub mod maker_notes;

// Re-export commonly used parsers
pub use jpeg::JpegParser;
pub use raw::RawParser;
pub use heif::HeifParser;
pub use video::VideoParser;
