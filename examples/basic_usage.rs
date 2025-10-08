//! Example usage of the fast-exif-reader crate
//! 
//! This example demonstrates how to use the pure Rust API to read EXIF metadata
//! from image files, including the optimal parser.

use fast_exif_reader::{
    FastExifReader, FastExifWriter, FastExifCopier, 
    OptimalExifParser, OptimalBatchProcessor,
    ExifError
};
use std::collections::HashMap;

fn main() -> Result<(), ExifError> {
    // Example 1: Standard FastExifReader
    println!("=== Standard FastExifReader ===");
    let mut reader = FastExifReader::new();
    
    // Example: Read EXIF data from a file
    // Note: Replace "example.jpg" with an actual image file path
    if let Ok(metadata) = reader.read_file("example.jpg") {
        println!("EXIF Metadata:");
        for (key, value) in &metadata {
            println!("  {}: {}", key, value);
        }
        
        // Extract specific fields
        if let Some(make) = metadata.get("Make") {
            println!("Camera Make: {}", make);
        }
        
        if let Some(model) = metadata.get("Model") {
            println!("Camera Model: {}", model);
        }
        
        if let Some(date_time) = metadata.get("DateTime") {
            println!("Date Taken: {}", date_time);
        }
    } else {
        println!("No EXIF data found or file not found");
    }
    
    // Example 2: Optimal EXIF Parser
    println!("\n=== Optimal EXIF Parser ===");
    let mut optimal_parser = OptimalExifParser::new();
    
    if let Ok(metadata) = optimal_parser.parse_file("example.jpg") {
        println!("Optimal parser extracted {} fields", metadata.len());
        
        // Get performance stats
        let stats = optimal_parser.get_stats();
        println!("Optimal parser stats: {:?}", stats);
    }
    
    // Example 3: Optimal Parser with Target Fields
    println!("\n=== Optimal Parser with Target Fields ===");
    let mut targeted_parser = OptimalExifParser::with_target_fields(
        vec!["Make".to_string(), "Model".to_string(), "DateTime".to_string()]
    );
    
    if let Ok(metadata) = targeted_parser.parse_file("example.jpg") {
        println!("Targeted parsing extracted {} fields", metadata.len());
        for (key, value) in &metadata {
            println!("  {}: {}", key, value);
        }
    }
    
    // Example 4: Batch Processing
    println!("\n=== Batch Processing ===");
    let file_paths = vec!["example1.jpg".to_string(), "example2.jpg".to_string()];
    
    // Standard parallel processing
    if let Ok(results) = reader.read_files_parallel(file_paths.clone()) {
        println!("Processed {} files in parallel", results.len());
        for (i, metadata) in results.iter().enumerate() {
            println!("  File {}: {} fields", i + 1, metadata.len());
        }
    }
    
    // Optimal batch processing
    let mut batch_processor = OptimalBatchProcessor::new(10);
    if let Ok(results) = batch_processor.process_files(&file_paths) {
        println!("Optimal batch processed {} files", results.len());
        for (i, metadata) in results.iter().enumerate() {
            println!("  File {}: {} fields", i + 1, metadata.len());
        }
    }
    
    // Example 5: EXIF Writing
    println!("\n=== EXIF Writing ===");
    let writer = FastExifWriter::new();
    
    let mut metadata = HashMap::new();
    metadata.insert("Make".to_string(), "Example Camera".to_string());
    metadata.insert("Model".to_string(), "Example Model".to_string());
    metadata.insert("DateTime".to_string(), "2024:01:01 12:00:00".to_string());
    
    // Note: This would write to a new file
    // writer.write_exif("input.jpg", "output.jpg", &metadata)?;
    println!("EXIF writing example prepared (commented out to avoid file operations)");
    
    // Example 6: EXIF Copying
    println!("\n=== EXIF Copying ===");
    let mut copier = FastExifCopier::new();
    
    // Note: This would copy EXIF from source to target
    // copier.copy_high_priority_exif("source.jpg", "target.jpg", "output.jpg")?;
    println!("EXIF copying example prepared (commented out to avoid file operations)");
    
    println!("\n=== Examples Complete ===");
    println!("The OptimalExifParser automatically chooses the best strategy:");
    println!("- Memory mapping for small files (< 8MB)");
    println!("- Hybrid approach for medium files (8-32MB)");
    println!("- Seek optimization for large files (> 32MB)");
    println!("- 10-100x faster than full file reading for large files");
    
    Ok(())
}