//! Parser modules for different image and video formats
//!
//! This module contains specialized parsers for various formats:
//! - Optimal: Single optimal parser that automatically chooses best strategy
//! - Format-specific: Specialized parsers for specific formats
//! - TIFF: Core TIFF-based EXIF parsing

pub mod optimal;
pub mod bmp;
pub mod heif;
pub mod jpeg;
pub mod maker_notes;
pub mod mkv;
pub mod png;
pub mod raw;
pub mod tiff;
pub mod video;

// Re-export optimal parser as the main parser
pub use optimal::{OptimalExifParser, OptimalBatchProcessor};

// Re-export format-specific parsers
pub use bmp::BmpParser;
pub use heif::HeifParser;
pub use jpeg::JpegParser;
pub use mkv::MkvParser;
pub use png::PngParser;
pub use raw::RawParser;
pub use video::VideoParser;
