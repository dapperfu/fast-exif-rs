//! Integration example for fast-exif-reader

use fast_exif_reader::{ExifError, FastExifReader, MemoryOptimizedExifReader};

fn main() -> Result<(), ExifError> {
    println!("Fast EXIF Reader - Integration Example");

    let test_files = std::env::args().skip(1).map(String::from).collect::<Vec<_>>();

    if test_files.is_empty() {
        println!("Pass image paths as arguments to run this example.");
        return Ok(());
    }

    let mut reader = FastExifReader::new();
    for file in &test_files {
        match reader.read_file(file) {
            Ok(metadata) => {
                println!("{}: {} fields", file, metadata.len());
                if let Some(make) = metadata.get("Make") {
                    println!("  Camera: {}", make);
                }
            }
            Err(e) => println!("{}: {}", file, e),
        }
    }

    println!("\nParallel processing:");
    let results = reader.read_files_parallel(test_files.clone())?;
    println!("Processed {} files", results.len());

    println!("\nBatch reader:");
    let mut batch_reader = MemoryOptimizedExifReader::new();
    let batch_results = batch_reader.read_files_batch(test_files)?;
    println!("Batch processed {} files", batch_results.len());

    println!("\nParser I/O stats: {:?}", reader.parser_stats());

    Ok(())
}
