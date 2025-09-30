use fast_exif_reader::{UltraFastJpegReader, benchmark_ultra_fast_jpeg};
use std::path::Path;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "/keg/pictures/incoming/2025.old";
    
    println!("Benchmarking UltraFastJPEG on: {}", test_dir);
    
    // Find all JPEG files
    let mut jpeg_files = Vec::new();
    for entry in WalkDir::new(test_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "jpg" || ext == "jpeg" {
                    jpeg_files.push(entry.path().to_path_buf());
                }
            }
        }
    }
    
    println!("Found {} JPEG files", jpeg_files.len());
    
    if jpeg_files.is_empty() {
        println!("No JPEG files found in directory");
        return Ok(());
    }
    
    // Test first 10 files individually to identify problematic ones
    println!("\nTesting first 10 files individually:");
    let mut success_count = 0;
    let mut error_count = 0;
    
    for (i, file_path) in jpeg_files.iter().take(10).enumerate() {
        print!("{}. {}: ", i + 1, file_path.file_name().unwrap().to_string_lossy());
        
        match test_single_file(file_path) {
            Ok(_) => {
                println!("✓ OK");
                success_count += 1;
            }
            Err(e) => {
                println!("✗ ERROR: {}", e);
                error_count += 1;
            }
        }
    }
    
    println!("\nIndividual test results: {} success, {} errors", success_count, error_count);
    
    // Run benchmark on all files
    println!("\nRunning benchmark on all files...");
    let file_paths: Vec<String> = jpeg_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
    match benchmark_ultra_fast_jpeg(file_paths) {
        Ok(stats) => {
            println!("Benchmark completed successfully!");
            println!("Total files: {}", stats.get("total_files").unwrap_or(&"0".to_string()));
            println!("Success rate: {}%", stats.get("success_rate").unwrap_or(&"0".to_string()));
            println!("Average time: {}ms", stats.get("avg_time_ms").unwrap_or(&"0".to_string()));
        }
        Err(e) => {
            println!("Benchmark failed: {}", e);
        }
    }
    
    Ok(())
}

fn test_single_file(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = UltraFastJpegReader::new();
    reader.read_file(file_path.to_str().unwrap())?;
    Ok(())
}