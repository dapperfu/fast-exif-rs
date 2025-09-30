use std::collections::HashMap;
use std::time::Instant;
use fast_exif_reader::parsers::{
    ultra_fast_jpeg::{UltraFastJpegParser, UltraFastBatchProcessor},
    ultra_seek_optimized::{UltraSeekOptimizedParser, UltraSeekBatchProcessor},
    adaptive_memory::{AdaptiveMemoryParser, AdaptiveMemoryBatchProcessor},
    lazy_parser::{LazyExifParser, LazyExifBatchProcessor},
};
use fast_exif_reader::FastExifReader;

/// Benchmark different optimization strategies for EXIF parsing
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EXIF Parser Optimization Benchmark");
    println!("=====================================");
    
    // Test file paths (you'll need to provide actual JPEG files)
    let test_files = vec![
        "test_images/sample1.jpg".to_string(),
        "test_images/sample2.jpg".to_string(),
        "test_images/sample3.jpg".to_string(),
        "test_images/sample4.jpg".to_string(),
        "test_images/sample5.jpg".to_string(),
    ];
    
    // Filter to only existing files
    let existing_files: Vec<String> = test_files
        .into_iter()
        .filter(|path| std::path::Path::new(path).exists())
        .collect();
    
    if existing_files.is_empty() {
        println!("❌ No test files found. Please add JPEG files to test_images/ directory.");
        println!("   Expected files: sample1.jpg, sample2.jpg, sample3.jpg, sample4.jpg, sample5.jpg");
        return Ok(());
    }
    
    println!("📁 Testing with {} files", existing_files.len());
    println!();
    
    // Benchmark 1: Original UltraFast implementation
    benchmark_ultra_fast(&existing_files)?;
    
    // Benchmark 2: Ultra-seek optimized implementation
    benchmark_ultra_seek(&existing_files)?;
    
    // Benchmark 3: Adaptive memory implementation
    benchmark_adaptive_memory(&existing_files)?;
    
    // Benchmark 4: Lazy parsing implementation
    benchmark_lazy_parsing(&existing_files)?;
    
    // Benchmark 5: Original FastExifReader for comparison
    benchmark_original_reader(&existing_files)?;
    
    // Benchmark 6: Selective field parsing
    benchmark_selective_fields(&existing_files)?;
    
    // Benchmark 7: Batch processing comparison
    benchmark_batch_processing(&existing_files)?;
    
    println!("✅ Benchmark completed!");
    
    Ok(())
}

/// Benchmark the original UltraFast implementation
fn benchmark_ultra_fast(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Benchmarking UltraFast Implementation");
    println!("----------------------------------------");
    
    let start_time = Instant::now();
    let mut parser = UltraFastJpegParser::new();
    let mut total_fields = 0;
    let mut successful_files = 0;
    
    for file_path in files {
        match std::fs::read(file_path) {
            Ok(data) => {
                let mut metadata = HashMap::new();
                if parser.parse_jpeg_exif_ultra_fast(&data, &mut metadata).is_ok() {
                    total_fields += metadata.len();
                    successful_files += 1;
                }
            }
            Err(_) => {
                println!("   ⚠️  Could not read file: {}", file_path);
            }
        }
    }
    
    let total_time = start_time.elapsed();
    let files_per_second = if total_time.as_secs_f64() > 0.0 {
        files.len() as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };
    
    println!("   📊 Results:");
    println!("      Files processed: {}/{}", successful_files, files.len());
    println!("      Total fields extracted: {}", total_fields);
    println!("      Total time: {:.3}s", total_time.as_secs_f64());
    println!("      Files per second: {:.1}", files_per_second);
    println!("      Avg fields per file: {:.1}", total_fields as f64 / successful_files.max(1) as f64);
    println!();
    
    Ok(())
}

/// Benchmark the ultra-seek optimized implementation
fn benchmark_ultra_seek(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Benchmarking Ultra-Seek Optimized Implementation");
    println!("---------------------------------------------------");
    
    let start_time = Instant::now();
    let mut parser = UltraSeekOptimizedParser::new();
    let mut total_fields = 0;
    let mut successful_files = 0;
    
    for file_path in files {
        match parser.parse_file(file_path) {
            Ok(metadata) => {
                total_fields += metadata.len();
                successful_files += 1;
            }
            Err(e) => {
                println!("   ⚠️  Error processing {}: {}", file_path, e);
            }
        }
    }
    
    let total_time = start_time.elapsed();
    let files_per_second = if total_time.as_secs_f64() > 0.0 {
        files.len() as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };
    
    println!("   📊 Results:");
    println!("      Files processed: {}/{}", successful_files, files.len());
    println!("      Total fields extracted: {}", total_fields);
    println!("      Total time: {:.3}s", total_time.as_secs_f64());
    println!("      Files per second: {:.1}", files_per_second);
    println!("      Avg fields per file: {:.1}", total_fields as f64 / successful_files.max(1) as f64);
    
    // Get parser statistics
    let stats = parser.get_stats();
    println!("      Parser stats: {:?}", stats);
    println!();
    
    Ok(())
}

/// Benchmark the adaptive memory implementation
fn benchmark_adaptive_memory(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Benchmarking Adaptive Memory Implementation");
    println!("----------------------------------------------");
    
    let start_time = Instant::now();
    let mut parser = AdaptiveMemoryParser::new();
    let mut total_fields = 0;
    let mut successful_files = 0;
    
    for file_path in files {
        match parser.parse_file(file_path) {
            Ok(metadata) => {
                total_fields += metadata.len();
                successful_files += 1;
            }
            Err(e) => {
                println!("   ⚠️  Error processing {}: {}", file_path, e);
            }
        }
    }
    
    let total_time = start_time.elapsed();
    let files_per_second = if total_time.as_secs_f64() > 0.0 {
        files.len() as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };
    
    println!("   📊 Results:");
    println!("      Files processed: {}/{}", successful_files, files.len());
    println!("      Total fields extracted: {}", total_fields);
    println!("      Total time: {:.3}s", total_time.as_secs_f64());
    println!("      Files per second: {:.1}", files_per_second);
    println!("      Avg fields per file: {:.1}", total_fields as f64 / successful_files.max(1) as f64);
    
    // Get parser statistics
    let stats = parser.get_stats();
    println!("      Parser stats: {:?}", stats);
    println!();
    
    Ok(())
}

/// Benchmark the lazy parsing implementation
fn benchmark_lazy_parsing(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("😴 Benchmarking Lazy Parsing Implementation");
    println!("-------------------------------------------");
    
    let start_time = Instant::now();
    let mut parser = LazyExifParser::new();
    let mut total_fields = 0;
    let mut successful_files = 0;
    
    // Test with common fields
    let target_fields = vec!["Make", "Model", "DateTime", "ExposureTime", "FNumber", "ISO"];
    
    for file_path in files {
        match parser.load_file(file_path) {
            Ok(_) => {
                match parser.get_fields(&target_fields) {
                    Ok(metadata) => {
                        total_fields += metadata.len();
                        successful_files += 1;
                    }
                    Err(e) => {
                        println!("   ⚠️  Error parsing fields for {}: {}", file_path, e);
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Error loading {}: {}", file_path, e);
            }
        }
    }
    
    let total_time = start_time.elapsed();
    let files_per_second = if total_time.as_secs_f64() > 0.0 {
        files.len() as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };
    
    println!("   📊 Results:");
    println!("      Files processed: {}/{}", successful_files, files.len());
    println!("      Total fields extracted: {}", total_fields);
    println!("      Total time: {:.3}s", total_time.as_secs_f64());
    println!("      Files per second: {:.1}", files_per_second);
    println!("      Avg fields per file: {:.1}", total_fields as f64 / successful_files.max(1) as f64);
    
    // Get parser statistics
    let stats = parser.get_stats();
    println!("      Parser stats: {:?}", stats);
    println!();
    
    Ok(())
}

/// Benchmark the original FastExifReader for comparison
fn benchmark_original_reader(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 Benchmarking Original FastExifReader");
    println!("---------------------------------------");
    
    let start_time = Instant::now();
    let mut reader = FastExifReader::new();
    let mut total_fields = 0;
    let mut successful_files = 0;
    
    for file_path in files {
        match reader.read_file(file_path) {
            Ok(metadata) => {
                total_fields += metadata.len();
                successful_files += 1;
            }
            Err(e) => {
                println!("   ⚠️  Error processing {}: {}", file_path, e);
            }
        }
    }
    
    let total_time = start_time.elapsed();
    let files_per_second = if total_time.as_secs_f64() > 0.0 {
        files.len() as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };
    
    println!("   📊 Results:");
    println!("      Files processed: {}/{}", successful_files, files.len());
    println!("      Total fields extracted: {}", total_fields);
    println!("      Total time: {:.3}s", total_time.as_secs_f64());
    println!("      Files per second: {:.1}", files_per_second);
    println!("      Avg fields per file: {:.1}", total_fields as f64 / successful_files.max(1) as f64);
    println!();
    
    Ok(())
}

/// Benchmark selective field parsing
fn benchmark_selective_fields(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Benchmarking Selective Field Parsing");
    println!("---------------------------------------");
    
    // Test different field sets
    let field_sets = vec![
        (vec!["Make", "Model"], "Basic Info"),
        (vec!["Make", "Model", "DateTime", "ExposureTime", "FNumber", "ISO"], "Common Fields"),
        (vec!["Make", "Model", "DateTime", "ExposureTime", "FNumber", "ISO", "FocalLength", "WhiteBalance", "Flash", "MeteringMode"], "Extended Fields"),
    ];
    
    for (fields, description) in field_sets {
        println!("   🔍 Testing {}: {:?}", description, fields);
        
        let start_time = Instant::now();
        let mut parser = UltraSeekOptimizedParser::with_target_fields(fields.iter().map(|s| s.to_string()).collect());
        let mut total_fields = 0;
        let mut successful_files = 0;
        
        for file_path in files {
            match parser.parse_file(file_path) {
                Ok(metadata) => {
                    total_fields += metadata.len();
                    successful_files += 1;
                }
                Err(_) => {
                    // Ignore errors for this benchmark
                }
            }
        }
        
        let total_time = start_time.elapsed();
        let files_per_second = if total_time.as_secs_f64() > 0.0 {
            files.len() as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };
        
        println!("      Files processed: {}/{}", successful_files, files.len());
        println!("      Total fields extracted: {}", total_fields);
        println!("      Total time: {:.3}s", total_time.as_secs_f64());
        println!("      Files per second: {:.1}", files_per_second);
        println!();
    }
    
    Ok(())
}

/// Benchmark batch processing
fn benchmark_batch_processing(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Benchmarking Batch Processing");
    println!("-------------------------------");
    
    // Test different batch processors
    let processors = vec![
        ("UltraFast Batch", || UltraFastBatchProcessor::new(10)),
        ("Ultra-Seek Batch", || UltraSeekBatchProcessor::new(10)),
        ("Adaptive Memory Batch", || AdaptiveMemoryBatchProcessor::new(10)),
        ("Lazy Parsing Batch", || LazyExifBatchProcessor::new(10)),
    ];
    
    for (name, processor_factory) in processors {
        println!("   🔄 Testing {} Processor", name);
        
        let start_time = Instant::now();
        let mut processor = processor_factory();
        let mut total_fields = 0;
        let mut successful_files = 0;
        
        match processor.process_files(files) {
            Ok(results) => {
                for metadata in results {
                    if !metadata.is_empty() {
                        total_fields += metadata.len();
                        successful_files += 1;
                    }
                }
            }
            Err(e) => {
                println!("      ⚠️  Error in batch processing: {}", e);
            }
        }
        
        let total_time = start_time.elapsed();
        let files_per_second = if total_time.as_secs_f64() > 0.0 {
            files.len() as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };
        
        println!("      Files processed: {}/{}", successful_files, files.len());
        println!("      Total fields extracted: {}", total_fields);
        println!("      Total time: {:.3}s", total_time.as_secs_f64());
        println!("      Files per second: {:.1}", files_per_second);
        println!();
    }
    
    Ok(())
}

/// Create test directory and sample files for benchmarking
#[allow(dead_code)]
fn create_test_files() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    
    // Create test directory
    fs::create_dir_all("test_images")?;
    
    println!("📁 Created test_images directory");
    println!("   Please add JPEG files to this directory for benchmarking");
    println!("   Expected files: sample1.jpg, sample2.jpg, sample3.jpg, sample4.jpg, sample5.jpg");
    
    Ok(())
}
