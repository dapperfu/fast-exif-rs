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
            // Look for meaningful date fields (exclude file system dates)
            let meaningful_dates: Vec<_> = metadata.iter()
                .filter(|(key, _)| {
                    let key_lower = key.to_lowercase();
                    (key_lower.contains("date") || key_lower.contains("time") || 
                     key_lower.contains("create") || key_lower.contains("modify")) &&
                    !key_lower.contains("filemodify") && 
                    !key_lower.contains("fileaccess") && 
                    !key_lower.contains("fileinode")
                })
                .filter(|(_, value)| !value.is_empty())
                .collect();
            
            if meaningful_dates.is_empty() {
                println!("❌ No meaningful EXIF dates found");
            } else {
                println!("✅ Found {} meaningful EXIF date fields:", meaningful_dates.len());
                for (key, value) in meaningful_dates {
                    println!("  {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
}
