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
    println!("Analyzing EXIF segments in file: {}", file_path);

    // Read the file as bytes
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // Find all EXIF segments and analyze their content
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
                        
                        // Look for TIFF header
                        for i in 0..exif_data.len().saturating_sub(8) {
                            if &exif_data[i..i + 2] == b"II" || &exif_data[i..i + 2] == b"MM" {
                                let byte_order = if &exif_data[i..i + 2] == b"II" {
                                    "Little-endian"
                                } else {
                                    "Big-endian"
                                };
                                println!("TIFF header found at offset {}: {}", i, byte_order);
                                
                                // Read TIFF version
                                if i + 4 < exif_data.len() {
                                    let version = u16::from_le_bytes([exif_data[i + 2], exif_data[i + 3]]);
                                    println!("TIFF version: {}", version);
                                }
                                
                                // Read IFD offset
                                if i + 8 < exif_data.len() {
                                    let ifd_offset = u32::from_le_bytes([
                                        exif_data[i + 4], exif_data[i + 5], 
                                        exif_data[i + 6], exif_data[i + 7]
                                    ]);
                                    println!("IFD offset: {}", ifd_offset);
                                    
                                    // Try to read IFD
                                    let ifd_pos = i + ifd_offset as usize;
                                    if ifd_pos + 2 < exif_data.len() {
                                        let entry_count = u16::from_le_bytes([
                                            exif_data[ifd_pos], exif_data[ifd_pos + 1]
                                        ]);
                                        println!("IFD entry count: {}", entry_count);
                                        
                                        // Look for DateTimeOriginal tag (0x9003)
                                        for entry_idx in 0..entry_count {
                                            let entry_pos = ifd_pos + 2 + (entry_idx as usize * 12);
                                            if entry_pos + 12 <= exif_data.len() {
                                                let tag_id = u16::from_le_bytes([
                                                    exif_data[entry_pos], exif_data[entry_pos + 1]
                                                ]);
                                                if tag_id == 0x9003 { // DateTimeOriginal
                                                    println!("Found DateTimeOriginal tag!");
                                                    let data_type = u16::from_le_bytes([
                                                        exif_data[entry_pos + 2], exif_data[entry_pos + 3]
                                                    ]);
                                                    let count = u32::from_le_bytes([
                                                        exif_data[entry_pos + 4], exif_data[entry_pos + 5],
                                                        exif_data[entry_pos + 6], exif_data[entry_pos + 7]
                                                    ]);
                                                    let value_offset = u32::from_le_bytes([
                                                        exif_data[entry_pos + 8], exif_data[entry_pos + 9],
                                                        exif_data[entry_pos + 10], exif_data[entry_pos + 11]
                                                    ]);
                                                    println!("  Data type: {}", data_type);
                                                    println!("  Count: {}", count);
                                                    println!("  Value offset: {}", value_offset);
                                                    
                                                    // Read the actual string value
                                                    let value_pos = i + value_offset as usize;
                                                    if value_pos + count as usize <= exif_data.len() {
                                                        let value_bytes = &exif_data[value_pos..value_pos + count as usize];
                                                        if let Ok(value_str) = std::str::from_utf8(value_bytes) {
                                                            println!("  DateTimeOriginal value: '{}'", value_str.trim_end_matches('\0'));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
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

    println!("\nTotal EXIF segments found: {}", segment_count);
    Ok(())
}
