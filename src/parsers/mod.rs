//! Parser modules for different image and video formats
//!
//! This module contains specialized parsers for various formats:
//! - JPEG: Standard JPEG EXIF parsing
//! - RAW: Canon CR2, Nikon NEF, Olympus ORF, Adobe DNG
//! - HEIF: HEIF/HEIC format parsing
//! - Video: MOV, MP4, 3GP video format parsing
//! - TIFF: TIFF-based EXIF parsing
//! - Maker Notes: Camera manufacturer specific data

pub mod heif;
pub mod jpeg;
pub mod maker_notes;
pub mod raw;
pub mod tiff;
pub mod video;

// Re-export commonly used parsers
pub use heif::HeifParser;
pub use jpeg::JpegParser;
pub use raw::RawParser;
pub use video::VideoParser;
