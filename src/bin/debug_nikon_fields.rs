use fast_exif_reader::FastExifReader;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    
    let mut reader = FastExifReader::new();
    let file_path = &args[1];
    
    match reader.read_file(file_path) {
        Ok(metadata) => {
            // Look for Nikon-specific fields
            let nikon_fields: Vec<_> = metadata.iter()
                .filter(|(key, _)| {
                    let key_lower = key.to_lowercase();
                    key_lower.contains("nikon") || 
                    key_lower.contains("active") ||
                    key_lower.contains("afarea") ||
                    key_lower.contains("affine") ||
                    key_lower.contains("contrast") ||
                    key_lower.contains("vrmode") ||
                    key_lower.contains("shooting") ||
                    key_lower.contains("colorspace") ||
                    key_lower.contains("noise") ||
                    key_lower.contains("flash")
                })
                .collect();
            
            if nikon_fields.is_empty() {
                println!("❌ No Nikon-specific fields found");
            } else {
                println!("✅ Found {} Nikon-specific fields:", nikon_fields.len());
                for (key, value) in nikon_fields {
                    println!("  {}: {}", key, value);
                }
            }
            
            // Also show all fields for debugging
            println!("\nAll fields:");
            for (key, value) in &metadata {
                println!("{}: {}", key, value);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
}
