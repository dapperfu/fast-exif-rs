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
            // Show ALL fields, not just date fields
            println!("Total fields extracted: {}", metadata.len());
            for (key, value) in &metadata {
                println!("{}: {}", key, value);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
}
