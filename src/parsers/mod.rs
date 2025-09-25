//! Parser modules for different image and video formats
//!
//! This module contains specialized parsers for various formats:
//! - JPEG: Standard JPEG EXIF parsing
//! - RAW: Canon CR2, Nikon NEF, Olympus ORF, Adobe DNG
//! - HEIF: HEIF/HEIC format parsing
//! - Video: MOV, MP4, 3GP video format parsing
//! - TIFF: TIFF-based EXIF parsing
//! - PNG: PNG format parsing with EXIF support
//! - BMP: BMP format parsing
//! - MKV: Matroska video format parsing
//! - Maker Notes: Camera manufacturer specific data

pub mod bmp;
pub mod heif;
pub mod jpeg;
pub mod maker_notes;
pub mod mkv;
pub mod png;
pub mod raw;
pub mod selective;
pub mod simd;
pub mod tiff;
pub mod video;
pub mod zero_copy;

// Re-export commonly used parsers
pub use bmp::BmpParser;
pub use heif::HeifParser;
pub use jpeg::JpegParser;
pub use mkv::MkvParser;
pub use png::PngParser;
pub use raw::RawParser;
pub use selective::{SelectiveFieldExtractor, FieldExtractors};
pub use simd::SimdHexParser;
pub use video::VideoParser;
pub use zero_copy::ZeroCopyExifParser;
