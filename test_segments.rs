use fast_exif_reader::parsers::tiff::TiffParser;
use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <image_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("Testing individual EXIF segments with file: {}", file_path);

    // Read the file as bytes
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // Find all EXIF segments
    let mut pos = 2;
    let mut segment_count = 0;
    while pos < data.len().saturating_sub(6) {
        if data[pos] == 0xFF && data[pos + 1] == 0xE1 {
            let length = ((data[pos + 2] as u16) << 8) | (data[pos + 3] as u16);
            let segment_end = pos + 2 + length as usize;
            
            if segment_end > data.len() {
                break;
            }

            // Look for "Exif" identifier
            let segment_start = pos + 4;
            for exif_start in segment_start..segment_end.saturating_sub(4) {
                if &data[exif_start..exif_start + 4] == b"Exif" {
                    segment_count += 1;
                    println!("\n=== EXIF Segment {} ===", segment_count);
                    println!("Position: {}", pos);
                    println!("Length: {} bytes", length);
                    println!("EXIF data length: {} bytes", segment_end - exif_start - 4);
                    
                    let exif_data_start = exif_start + 4;
                    if exif_data_start < segment_end {
                        let exif_data = &data[exif_data_start..segment_end];
                        
                        // Try to parse this EXIF segment
                        let mut metadata = HashMap::new();
                        match TiffParser::parse_tiff_exif(exif_data, &mut metadata) {
                            Ok(_) => {
                                println!("Successfully parsed {} fields:", metadata.len());
                                
                                // Look for timestamp fields
                                let timestamp_fields = [
                                    "DateTimeOriginal",
                                    "DateTime",
                                    "DateTimeDigitized", 
                                    "CreateDate",
                                    "ModifyDate",
                                    "SubSecTimeOriginal",
                                    "SubSecDateTimeOriginal",
                                    "OffsetTimeOriginal",
                                    "TimeZone"
                                ];
                                
                                println!("Timestamp fields found:");
                                for field in &timestamp_fields {
                                    if let Some(value) = metadata.get(*field) {
                                        println!("  {}: {}", field, value);
                                    }
                                }
                                
                                // Show all fields for debugging
                                if segment_count == 2 { // Show all fields for the second segment
                                    println!("\nAll fields in segment {}:", segment_count);
                                    let mut sorted_keys: Vec<_> = metadata.keys().collect();
                                    sorted_keys.sort();
                                    for key in sorted_keys {
                                        println!("  {}: {}", key, metadata[key]);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to parse EXIF segment: {}", e);
                            }
                        }
                    }
                    break;
                }
            }
            pos = segment_end;
        } else {
            pos += 1;
        }
    }

    println!("\nTotal EXIF segments found: {}", segment_count);
    Ok(())
}
