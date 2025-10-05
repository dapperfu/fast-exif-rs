use fast_exif_reader::FastExifReader;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <image_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("Debugging fast-exif-rs with file: {}", file_path);

    // Read the file as bytes to debug EXIF segment detection
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    println!("File size: {} bytes", data.len());

    // Look for EXIF segment manually
    let mut pos = 2;
    let mut exif_found = false;
    while pos < data.len().saturating_sub(6) {
        if data[pos] == 0xFF && data[pos + 1] == 0xE1 {
            println!("Found APP1 segment at position {}", pos);
            let length = ((data[pos + 2] as u16) << 8) | (data[pos + 3] as u16);
            println!("APP1 segment length: {}", length);
            let segment_end = pos + 2 + length as usize;
            
            if segment_end > data.len() {
                println!("Segment extends beyond file end");
                break;
            }

            // Look for "Exif" identifier
            let segment_start = pos + 4;
            for exif_start in segment_start..segment_end.saturating_sub(4) {
                if &data[exif_start..exif_start + 4] == b"Exif" {
                    println!("Found EXIF identifier at position {}", exif_start);
                    exif_found = true;
                    
                    // Show some bytes around the EXIF identifier
                    let start = exif_start.saturating_sub(10);
                    let end = (exif_start + 50).min(data.len());
                    println!("Bytes around EXIF identifier:");
                    for i in start..end {
                        print!("{:02X} ", data[i]);
                        if (i - start + 1) % 16 == 0 {
                            println!();
                        }
                    }
                    println!();
                    
                    // Try to find TIFF header
                    let exif_data_start = exif_start + 4;
                    if exif_data_start < segment_end {
                        let exif_data = &data[exif_data_start..segment_end];
                        println!("EXIF data length: {} bytes", exif_data.len());
                        
                        // Look for TIFF header
                        for i in 0..exif_data.len().saturating_sub(8) {
                            if &exif_data[i..i + 2] == b"II" || &exif_data[i..i + 2] == b"MM" {
                                println!("Found TIFF header at EXIF offset {}", i);
                                let byte_order = if &exif_data[i..i + 2] == b"II" {
                                    "Little-endian"
                                } else {
                                    "Big-endian"
                                };
                                println!("Byte order: {}", byte_order);
                                
                                // Show TIFF header bytes
                                let end = (i + 20).min(exif_data.len());
                                println!("TIFF header bytes:");
                                for j in i..end {
                                    print!("{:02X} ", exif_data[j]);
                                }
                                println!();
                                break;
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

    if !exif_found {
        println!("No EXIF segment found in file");
    }

    // Now test with fast-exif-rs
    let mut reader = FastExifReader::new();
    match reader.read_file(file_path) {
        Ok(metadata) => {
            println!("\nFast-exif-rs extracted {} metadata fields:", metadata.len());
            
            // Look for timestamp-related fields
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
            
            println!("\nTimestamp-related fields:");
            for field in &timestamp_fields {
                if let Some(value) = metadata.get(*field) {
                    println!("  {}: {}", field, value);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading EXIF data: {}", e);
        }
    }

    Ok(())
}
