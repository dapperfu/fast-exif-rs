use fast_exif_reader::{FastExifReader, OptimalExifParser};
use std::path::Path;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());

    println!("Benchmarking EXIF readers on: {}", test_dir);

    let mut jpeg_files = Vec::new();
    for entry in WalkDir::new(&test_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "jpg" || ext == "jpeg" {
                    jpeg_files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    println!("Found {} JPEG files", jpeg_files.len());

    if jpeg_files.is_empty() {
        return Ok(());
    }

    let mut reader = FastExifReader::new();
    let mut optimal = OptimalExifParser::new();

    for file_path in jpeg_files.iter().take(10) {
        let path_str = file_path.to_string_lossy();
        match reader.read_file(&path_str) {
            Ok(meta) => println!("FastExifReader {}: {} fields", path_str, meta.len()),
            Err(e) => println!("FastExifReader {}: {}", path_str, e),
        }
        match optimal.parse_file(Path::new(file_path.as_os_str())) {
            Ok(meta) => println!("OptimalExifParser {}: {} fields", path_str, meta.len()),
            Err(e) => println!("OptimalExifParser {}: {}", path_str, e),
        }
    }

    Ok(())
}
