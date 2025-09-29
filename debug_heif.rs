use std::collections::HashMap;
use fast_exif_reader::parsers::tiff::TiffParser;
use std::fs;

fn main() {
    // Read the HEIF file
    let data = fs::read("/keg/pictures/incoming/2025/09-Sep/20250924_082340.850.hif").unwrap();
    
    // Find EXIF data manually
    for i in 0..data.len().saturating_sub(8) {
        if &data[i..i + 4] == b"Exif" && i + 8 < data.len() {
            let tiff_start = i + 4;
            if &data[tiff_start..tiff_start + 2] == b"II" || &data[tiff_start..tiff_start + 2] == b"MM" {
                println!("Found EXIF at offset: 0x{:x}", i);
                println!("TIFF starts at offset: 0x{:x}", tiff_start);
                
                // Parse with TIFF parser
                let mut metadata = HashMap::new();
                match TiffParser::parse_tiff_exif(&data[tiff_start..], &mut metadata) {
                    Ok(_) => {
                        println!("TIFF parsing succeeded!");
                        println!("DateTimeOriginal: {:?}", metadata.get("DateTimeOriginal"));
                        println!("SubSecTimeOriginal: {:?}", metadata.get("SubSecTimeOriginal"));
                        println!("Make: {:?}", metadata.get("Make"));
                        println!("Model: {:?}", metadata.get("Model"));
                        
                        // Print all keys to see what we got
                        let mut keys: Vec<_> = metadata.keys().collect();
                        keys.sort();
                        println!("All extracted fields:");
                        for key in keys {
                            println!("  {}: {}", key, metadata.get(key).unwrap());
                        }
                    }
                    Err(e) => {
                        println!("TIFF parsing failed: {:?}", e);
                    }
                }
                break;
            }
        }
    }
}
