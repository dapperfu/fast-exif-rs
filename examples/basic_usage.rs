//! Example usage of the fast-exif-reader crate
//! 
//! This example demonstrates how to use the pure Rust API to read EXIF metadata
//! from image files, including the specialized ultra-fast and hybrid readers.

use fast_exif_reader::{
    FastExifReader, FastExifWriter, FastExifCopier, 
    UltraFastJpegReader, HybridExifReader,
    benchmark_ultra_fast_jpeg, benchmark_hybrid_vs_standard,
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
    
    // Example 2: Ultra-Fast JPEG Reader
    println!("\n=== Ultra-Fast JPEG Reader ===");
    let mut ultra_reader = UltraFastJpegReader::new();
    
    if let Ok(metadata) = ultra_reader.read_file("example.jpg") {
        println!("Ultra-fast processing extracted {} fields", metadata.len());
        
        // Get performance stats
        if let Ok(stats) = ultra_reader.get_stats() {
            println!("Ultra-fast stats: {:?}", stats);
        }
    }
    
    // Example 3: Hybrid Reader
    println!("\n=== Hybrid Reader ===");
    let mut hybrid_reader = HybridExifReader::new();
    
    if let Ok(metadata) = hybrid_reader.read_file("example.jpg") {
        println!("Hybrid processing extracted {} fields", metadata.len());
        
        // Get performance stats
        if let Ok(stats) = hybrid_reader.get_performance_stats() {
            println!("Hybrid stats: {:?}", stats);
        }
    }
    
    // Example 4: Parallel Processing
    println!("\n=== Parallel Processing ===");
    let file_paths = vec!["example1.jpg".to_string(), "example2.jpg".to_string()];
    
    // Standard parallel processing
    if let Ok(results) = reader.read_files_parallel(file_paths.clone()) {
        println!("Processed {} files in parallel", results.len());
        for (i, metadata) in results.iter().enumerate() {
            println!("  File {}: {} fields", i + 1, metadata.len());
        }
    }
    
    // Ultra-fast parallel processing
    if let Ok(results) = ultra_reader.read_files_batch(file_paths.clone()) {
        println!("Ultra-fast processed {} files in parallel", results.len());
        for (i, metadata) in results.iter().enumerate() {
            println!("  File {}: {} fields", i + 1, metadata.len());
        }
    }
    
    // Hybrid parallel processing
    if let Ok(results) = hybrid_reader.read_files_parallel(file_paths.clone()) {
        println!("Hybrid processed {} files in parallel", results.len());
        for (i, metadata) in results.iter().enumerate() {
            println!("  File {}: {} fields", i + 1, metadata.len());
        }
    }
    
    // Example 5: Benchmarking
    println!("\n=== Benchmarking ===");
    let test_files = vec!["example.jpg".to_string()];
    
    // Benchmark ultra-fast JPEG processing
    if let Ok(benchmark_results) = benchmark_ultra_fast_jpeg(test_files.clone()) {
        println!("Ultra-fast benchmark results:");
        for (key, value) in &benchmark_results {
            println!("  {}: {}", key, value);
        }
    }
    
    // Benchmark hybrid vs standard
    if let Ok(comparison_results) = benchmark_hybrid_vs_standard(test_files) {
        println!("Hybrid vs Standard comparison:");
        for (key, value) in &comparison_results {
            println!("  {}: {}", key, value);
        }
    }
    
    // Example 6: Read from Memory
    println!("\n=== Reading from Memory ===");
    if let Ok(image_data) = std::fs::read("example.jpg") {
        let metadata = reader.read_bytes(&image_data)?;
        println!("Read {} fields from image bytes", metadata.len());
    }
    
    // Example 7: Write EXIF metadata
    println!("\n=== Writing EXIF Metadata ===");
    let writer = FastExifWriter::new();
    let mut new_metadata = HashMap::new();
    new_metadata.insert("Make".to_string(), "Example Camera".to_string());
    new_metadata.insert("Model".to_string(), "Example Model".to_string());
    
    // writer.write_exif("input.jpg", "output.jpg", &new_metadata)?;
    
    // Example 8: Copy EXIF metadata between files
    println!("\n=== Copying EXIF Metadata ===");
    let mut copier = FastExifCopier::new();
    // copier.copy_high_priority_exif("source.jpg", "target.jpg", "output.jpg")?;
    
    println!("\n=== All examples completed successfully! ===");
    Ok(())
}
