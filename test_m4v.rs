use fast_exif_reader::{FastExifReader, ExifError};
use std::collections::HashMap;

fn main() -> Result<(), ExifError> {
    let file_path = "/keg/takeout/2012/01-Jan/20120113_044330.000.m4v";
    
    println!("Testing M4V file: {}", file_path);
    
    let mut reader = FastExifReader::new();
    
    match reader.read_file(file_path) {
        Ok(metadata) => {
            println!("Successfully extracted {} fields:", metadata.len());
            
            // Filter for date-related fields
            let date_fields: Vec<_> = metadata.iter()
                .filter(|(key, _)| key.to_lowercase().contains("date"))
                .collect();
            
            println!("\nDate-related fields:");
            for (key, value) in &date_fields {
                println!("  {}: {}", key, value);
            }
            
            // Show all fields if there are few
            if metadata.len() <= 20 {
                println!("\nAll fields:");
                for (key, value) in &metadata {
                    println!("  {}: {}", key, value);
                }
            } else {
                println!("\nFirst 20 fields:");
                for (key, value) in metadata.iter().take(20) {
                    println!("  {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            println!("Error reading file: {:?}", e);
        }
    }
    
    Ok(())
}
