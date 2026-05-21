use fast_exif_reader::{
    FormatDetector, MemoryOptimizedExifReader, OptimalExifParser, FastExifReader,
};

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
    let b = fast_exif_reader::EnhancedFormatDetector::detect_format(&header).unwrap();
    assert_eq!(a, b);
}

#[test]
fn optimal_parser_exposes_io_stats() {
    let parser = OptimalExifParser::new();
    let stats = parser.get_stats();
    assert_eq!(stats.get("parser_type").map(String::as_str), Some("OptimalExif"));
}

#[test]
fn reader_post_processing_on_bytes() {
    let mut reader = FastExifReader::new();
    let minimal_jpeg = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00,
        0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
    ];
    let metadata = reader.read_bytes(&minimal_jpeg).unwrap();
    assert!(metadata.contains_key("Format") || metadata.contains_key("FileType"));
}

#[test]
fn memory_optimized_reader_delegates_to_fast_reader() {
    let mut reader = MemoryOptimizedExifReader::new();
    assert_eq!(reader.get_batch_size(), 50);
    reader.set_batch_size(10);
    assert_eq!(reader.get_batch_size(), 10);
}
