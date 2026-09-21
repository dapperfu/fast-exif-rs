use fast_exif_reader::{
    EnhancedFormatDetector, ExifWriter, FastExifReader, FormatDetector, OptimalExifParser,
};
use std::collections::HashMap;

#[test]
fn format_detector_recognizes_jpeg() {
    let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0];
    let format = FormatDetector::detect_format(&jpeg_header).unwrap();
    assert_eq!(format, "JPEG");
}

#[test]
fn format_detector_recognizes_png() {
    let png_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let format = FormatDetector::detect_format(&png_header).unwrap();
    assert_eq!(format, "PNG");
}

#[test]
fn enhanced_format_detector_alias_matches() {
    let header = [0xFF, 0xD8, 0xFF, 0xE0];
    let a = FormatDetector::detect_format(&header).unwrap();
    let b = EnhancedFormatDetector::detect_format(&header).unwrap();
    assert_eq!(a, b);
}

#[test]
fn optimal_parser_exposes_io_stats() {
    let parser = OptimalExifParser::new();
    let stats = parser.get_stats();
    assert_eq!(
        stats.get("parser_type").map(String::as_str),
        Some("OptimalExif")
    );
}

#[test]
fn reader_roundtrip_written_jpeg_exif() {
    let jpeg = [0xFF, 0xD8, 0xFF, 0xD9];
    let mut metadata = HashMap::new();
    metadata.insert("Make".to_string(), "Canon".to_string());
    metadata.insert("Model".to_string(), "EOS 70D".to_string());

    let written = ExifWriter::new()
        .write_jpeg_exif_to_bytes(&jpeg, &metadata)
        .unwrap();
    let back = FastExifReader::new().read_bytes(&written).unwrap();

    assert_eq!(back.get("Make").map(String::as_str), Some("Canon"));
    assert_eq!(back.get("Model").map(String::as_str), Some("EOS 70D"));
}
