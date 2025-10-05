use fast_exif_reader::FastExifReader;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <image_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("Testing fast-exif-rs with file: {}", file_path);

    let mut reader = FastExifReader::new();
    
    match reader.read_file(file_path) {
        Ok(metadata) => {
            println!("Successfully extracted {} metadata fields:", metadata.len());
            
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
            
            println!("\nAll metadata fields:");
            let mut sorted_keys: Vec<_> = metadata.keys().collect();
            sorted_keys.sort();
            for key in sorted_keys {
                println!("  {}: {}", key, metadata[key]);
            }
        }
        Err(e) => {
            eprintln!("Error reading EXIF data: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
