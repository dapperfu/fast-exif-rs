//! EXIF Tool RS - A fast EXIF metadata extraction tool
//! 
//! This CLI tool provides fast EXIF metadata extraction with support for:
//! - Short tags (compact output)
//! - Known EXIF parameters and values
//! - Multiple output formats
//! - Batch processing

use clap::{Parser, Subcommand};
use colored::*;
use fast_exif_reader::FastExifReader;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};
use walkdir::WalkDir;

/// A fast EXIF metadata extraction tool written in Rust
#[derive(Parser)]
#[command(name = "exiftool-rs")]
#[command(version = "0.2.0")]
#[command(about = "A fast EXIF metadata extraction tool")]
#[command(long_about = "A high-performance EXIF metadata extraction tool that supports short tags, known parameters, and multiple output formats.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract EXIF metadata from files
    Extract {
        /// Input files or directories
        #[arg(required = true)]
        inputs: Vec<String>,
        
        /// Use short tags (compact output)
        #[arg(short, long)]
        short: bool,
        
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
        
        /// Recursively process directories
        #[arg(short, long)]
        recursive: bool,
        
        /// Show only specific tags
        #[arg(short, long)]
        tags: Option<Vec<String>>,
        
        /// Show file names
        #[arg(long)]
        filenames: bool,
        
        /// Quiet mode (minimal output)
        #[arg(short, long)]
        quiet: bool,
    },
    /// List known EXIF tags
    ListTags {
        /// Show only short tag names
        #[arg(short, long)]
        short: bool,
        
        /// Filter by tag category
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Show tool information
    Info,
    /// Benchmark EXIF extraction performance
    Benchmark {
        /// Input files or directories to benchmark
        #[arg(required = true)]
        inputs: Vec<String>,
        
        /// Recursively process directories
        #[arg(short, long)]
        recursive: bool,
        
        /// Number of iterations to run for more accurate timing
        #[arg(short, long, default_value = "1")]
        iterations: u32,
        
        /// Show detailed per-file timing
        #[arg(long)]
        detailed: bool,
        
        /// Output format for benchmark results
        #[arg(short, long, default_value = "text")]
        format: BenchmarkFormat,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(clap::ValueEnum, Clone)]
enum BenchmarkFormat {
    Text,
    Json,
    Csv,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Extract { 
            inputs, 
            short, 
            format, 
            recursive, 
            tags, 
            filenames, 
            quiet 
        } => {
            extract_exif_data(inputs, short, format, recursive, tags, filenames, quiet)?;
        }
        Commands::ListTags { short, category } => {
            list_known_tags(short, category)?;
        }
        Commands::Info => {
            show_info()?;
        }
        Commands::Benchmark { 
            inputs, 
            recursive, 
            iterations, 
            detailed, 
            format 
        } => {
            benchmark_exif_extraction(inputs, recursive, iterations, detailed, format)?;
        }
    }
    
    Ok(())
}

fn extract_exif_data(
    inputs: Vec<String>,
    short: bool,
    format: OutputFormat,
    recursive: bool,
    tags: Option<Vec<String>>,
    filenames: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = FastExifReader::new();
    let mut all_results = Vec::new();
    
    for input in inputs {
        let path = Path::new(&input);
        
        if path.is_file() {
            process_file(&mut reader, path, &mut all_results, short, &tags, filenames, quiet)?;
        } else if path.is_dir() {
            process_directory(&mut reader, path, &mut all_results, short, &tags, filenames, quiet, recursive)?;
        } else {
            eprintln!("{}: File or directory not found", input.red());
        }
    }
    
    // Output results in requested format
    match format {
        OutputFormat::Text => output_text_format(&all_results, short, quiet),
        OutputFormat::Json => output_json_format(&all_results)?,
        OutputFormat::Csv => output_csv_format(&all_results)?,
    }
    
    Ok(())
}

fn process_file(
    reader: &mut FastExifReader,
    path: &Path,
    results: &mut Vec<FileResult>,
    _short: bool,
    tags: &Option<Vec<String>>,
    _filenames: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match reader.read_file(path.to_str().unwrap()) {
        Ok(metadata) => {
            let filtered_metadata = if let Some(tag_list) = tags {
                filter_tags(&metadata, tag_list)
            } else {
                metadata
            };
            
            if !quiet {
                println!("{}: {} EXIF fields extracted", 
                    path.display().to_string().green(), 
                    filtered_metadata.len()
                );
            }
            
            results.push(FileResult {
                filename: path.to_string_lossy().to_string(),
                metadata: filtered_metadata,
            });
        }
        Err(e) => {
            eprintln!("{}: Error reading EXIF data: {}", path.display().to_string().red(), e);
        }
    }
    
    Ok(())
}

fn process_directory(
    reader: &mut FastExifReader,
    path: &Path,
    results: &mut Vec<FileResult>,
    short: bool,
    tags: &Option<Vec<String>>,
    filenames: bool,
    quiet: bool,
    recursive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let walker = if recursive {
        WalkDir::new(path).into_iter()
    } else {
        WalkDir::new(path).max_depth(1).into_iter()
    };
    
    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && is_image_file(path) {
            process_file(reader, path, results, short, tags, filenames, quiet)?;
        }
    }
    
    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            let ext_lower = ext_str.to_lowercase();
            return matches!(ext_lower.as_str(), 
                "jpg" | "jpeg" | "tiff" | "tif" | "png" | "bmp" | "gif" | "webp" | 
                "cr2" | "nef" | "arw" | "raf" | "srw" | "pef" | "rw2" | "orf" | 
                "dng" | "heic" | "heif" | "mov" | "mp4" | "3gp" | "avi" | "wmv" | 
                "webm" | "mkv"
            );
        }
    }
    false
}

fn filter_tags(metadata: &HashMap<String, String>, tags: &[String]) -> HashMap<String, String> {
    let mut filtered = HashMap::new();
    
    for tag in tags {
        if let Some(value) = metadata.get(tag) {
            filtered.insert(tag.clone(), value.clone());
        }
    }
    
    filtered
}

fn output_text_format(results: &[FileResult], short: bool, quiet: bool) {
    for result in results {
        if !quiet {
            println!("\n{}", format!("=== {} ===", result.filename).bold().blue());
        }
        
        for (key, value) in &result.metadata {
            let display_key = if short {
                get_short_tag(key)
            } else {
                key.clone()
            };
            
            println!("{}: {}", display_key.cyan(), value);
        }
    }
}

fn output_json_format(results: &[FileResult]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(results)?);
    Ok(())
}

fn output_csv_format(results: &[FileResult]) -> Result<(), Box<dyn std::error::Error>> {
    // Simple CSV output
    println!("filename,tag,value");
    for result in results {
        for (tag, value) in &result.metadata {
            println!("{},{},{}", result.filename, tag, value);
        }
    }
    Ok(())
}

fn list_known_tags(short: bool, category: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let tags = get_known_exif_tags();
    
    println!("{}", "Known EXIF Tags".bold().green());
    println!("{}", "===============".green());
    
    for (tag, info) in tags {
        if let Some(ref cat) = category {
            if !info.category.to_lowercase().contains(&cat.to_lowercase()) {
                continue;
            }
        }
        
        let display_tag = if short {
            info.short_name.clone()
        } else {
            tag.clone()
        };
        
        println!("{}: {}", display_tag.cyan(), info.description);
        if !short {
            println!("  Category: {}", info.category.yellow());
            println!("  Short: {}", info.short_name.green());
        }
        println!();
    }
    
    Ok(())
}

fn show_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "EXIF Tool RS".bold().blue());
    println!("{}", "============".blue());
    println!("Version: {}", "0.1.0".green());
    println!("Description: {}", "A fast EXIF metadata extraction tool written in Rust".yellow());
    println!("Repository: {}", "https://github.com/dapperfu/fast-exif-rs".cyan());
    println!();
    println!("{}", "Features:".bold().green());
    println!("• High-performance EXIF extraction");
    println!("• Support for 20+ image and video formats");
    println!("• Short tags for compact output");
    println!("• Multiple output formats (text, JSON, CSV)");
    println!("• Batch processing and recursive directory scanning");
    println!("• Known EXIF parameter definitions");
    println!();
    println!("{}", "Supported Formats:".bold().green());
    println!("Images: JPEG, CR2, NEF, ARW, RAF, SRW, PEF, RW2, ORF, DNG, HEIF/HEIC, PNG, BMP, GIF, WEBP");
    println!("Videos: MOV, MP4, 3GP, AVI, WMV, WEBM, MKV");
    
    Ok(())
}

fn benchmark_exif_extraction(
    inputs: Vec<String>,
    recursive: bool,
    iterations: u32,
    detailed: bool,
    format: BenchmarkFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = FastExifReader::new();
    let mut all_files = Vec::new();
    
    // Collect all files to benchmark
    for input in inputs {
        let path = Path::new(&input);
        
        if path.is_file() {
            if is_image_file(path) {
                all_files.push(path.to_path_buf());
            }
        } else if path.is_dir() {
            let walker = if recursive {
                WalkDir::new(path).into_iter()
            } else {
                WalkDir::new(path).max_depth(1).into_iter()
            };
            
            for entry in walker {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() && is_image_file(path) {
                    all_files.push(path.to_path_buf());
                }
            }
        } else {
            eprintln!("{}: File or directory not found", input.red());
        }
    }
    
    if all_files.is_empty() {
        eprintln!("{}", "No valid image files found to benchmark".red());
        return Ok(());
    }
    
    println!("{}", "EXIF Extraction Benchmark".bold().blue());
    println!("{}", "=========================".blue());
    println!("Files to process: {}", all_files.len().to_string().green());
    println!("Iterations: {}", iterations.to_string().green());
    println!();
    
    let mut total_duration = Duration::new(0, 0);
    let mut successful_files = 0;
    let mut total_exif_fields = 0;
    let mut file_timings = Vec::new();
    
    // Run benchmark iterations
    for iteration in 1..=iterations {
        if iterations > 1 {
            println!("{}", format!("Iteration {}/{}", iteration, iterations).bold().yellow());
        }
        
        let iteration_start = Instant::now();
        let mut iteration_successful = 0;
        let mut iteration_exif_fields = 0;
        
        for file_path in &all_files {
            let file_start = Instant::now();
            
            match reader.read_file(file_path.to_str().unwrap()) {
                Ok(metadata) => {
                    let file_duration = file_start.elapsed();
                    let field_count = metadata.len();
                    
                    iteration_successful += 1;
                    iteration_exif_fields += field_count;
                    
                    if detailed {
                        file_timings.push(FileTiming {
                            filename: file_path.to_string_lossy().to_string(),
                            duration: file_duration,
                            exif_fields: field_count,
                            iteration,
                        });
                    }
                    
                    if detailed && iterations == 1 {
                        println!("  {}: {} fields in {:.3}ms", 
                            file_path.file_name().unwrap().to_string_lossy().cyan(),
                            field_count.to_string().green(),
                            file_duration.as_secs_f64() * 1000.0
                        );
                    }
                }
                Err(e) => {
                    if detailed {
                        eprintln!("  {}: Error - {}", 
                            file_path.file_name().unwrap().to_string_lossy().red(),
                            e
                        );
                    }
                }
            }
        }
        
        let iteration_duration = iteration_start.elapsed();
        total_duration += iteration_duration;
        successful_files += iteration_successful;
        total_exif_fields += iteration_exif_fields;
        
        if iterations > 1 {
            println!("  Iteration {}: {} files, {} fields, {:.3}s", 
                iteration,
                iteration_successful,
                iteration_exif_fields,
                iteration_duration.as_secs_f64()
            );
        }
    }
    
    // Calculate statistics
    let avg_duration = total_duration / iterations;
    let files_per_second = if avg_duration.as_secs_f64() > 0.0 {
        all_files.len() as f64 / avg_duration.as_secs_f64()
    } else {
        0.0
    };
    
    let fields_per_second = if avg_duration.as_secs_f64() > 0.0 {
        total_exif_fields as f64 / avg_duration.as_secs_f64()
    } else {
        0.0
    };
    
    let success_rate = if !all_files.is_empty() {
        (successful_files as f64 / (all_files.len() as f64 * iterations as f64)) * 100.0
    } else {
        0.0
    };
    
    // Create benchmark results
    let results = BenchmarkResults {
        total_files: all_files.len(),
        iterations,
        total_duration: avg_duration,
        successful_files: successful_files / iterations as usize,
        total_exif_fields: total_exif_fields / iterations as usize,
        files_per_second,
        fields_per_second,
        success_rate,
        file_timings: if detailed { Some(file_timings) } else { None },
    };
    
    // Output results
    match format {
        BenchmarkFormat::Text => output_benchmark_text(&results)?,
        BenchmarkFormat::Json => output_benchmark_json(&results)?,
        BenchmarkFormat::Csv => output_benchmark_csv(&results)?,
    }
    
    Ok(())
}

fn output_benchmark_text(results: &BenchmarkResults) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("{}", "Benchmark Results".bold().green());
    println!("{}", "================".green());
    println!("Total files processed: {}", results.total_files.to_string().cyan());
    println!("Iterations: {}", results.iterations.to_string().cyan());
    println!("Total time: {:.3}s", results.total_duration.as_secs_f64().to_string().cyan());
    println!("Successful files: {}", results.successful_files.to_string().green());
    println!("Total EXIF fields: {}", results.total_exif_fields.to_string().green());
    println!("Success rate: {:.1}%", results.success_rate.to_string().yellow());
    println!();
    println!("{}", "Performance Metrics".bold().blue());
    println!("{}", "==================".blue());
    println!("Files per second: {:.1}", results.files_per_second.to_string().green());
    println!("Fields per second: {:.0}", results.fields_per_second.to_string().green());
    println!("Average time per file: {:.3}ms", 
        (results.total_duration.as_secs_f64() * 1000.0) / results.total_files as f64
    );
    
    if let Some(ref timings) = results.file_timings {
        println!();
        println!("{}", "Detailed File Timings".bold().purple());
        println!("{}", "=====================".purple());
        
        // Sort by duration (slowest first)
        let mut sorted_timings = timings.clone();
        sorted_timings.sort_by(|a, b| b.duration.cmp(&a.duration));
        
        for timing in sorted_timings.iter().take(10) {
            println!("  {}: {:.3}ms ({} fields)", 
                timing.filename.cyan(),
                timing.duration.as_secs_f64() * 1000.0,
                timing.exif_fields.to_string().green()
            );
        }
        
        if timings.len() > 10 {
            println!("  ... and {} more files", timings.len() - 10);
        }
    }
    
    Ok(())
}

fn output_benchmark_json(results: &BenchmarkResults) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(results)?);
    Ok(())
}

fn output_benchmark_csv(results: &BenchmarkResults) -> Result<(), Box<dyn std::error::Error>> {
    println!("metric,value");
    println!("total_files,{}", results.total_files);
    println!("iterations,{}", results.iterations);
    println!("total_duration_seconds,{:.6}", results.total_duration.as_secs_f64());
    println!("successful_files,{}", results.successful_files);
    println!("total_exif_fields,{}", results.total_exif_fields);
    println!("files_per_second,{:.2}", results.files_per_second);
    println!("fields_per_second,{:.0}", results.fields_per_second);
    println!("success_rate_percent,{:.1}", results.success_rate);
    
    if let Some(ref timings) = results.file_timings {
        println!();
        println!("filename,duration_ms,exif_fields,iteration");
        for timing in timings {
            println!("{},{:.3},{},{}", 
                timing.filename,
                timing.duration.as_secs_f64() * 1000.0,
                timing.exif_fields,
                timing.iteration
            );
        }
    }
    
    Ok(())
}

fn get_short_tag(tag: &str) -> String {
    let tags = get_known_exif_tags();
    if let Some(info) = tags.get(tag) {
        info.short_name.clone()
    } else {
        tag.to_string()
    }
}

#[derive(serde::Serialize)]
struct FileResult {
    filename: String,
    metadata: HashMap<String, String>,
}

#[derive(serde::Serialize, Clone)]
struct FileTiming {
    filename: String,
    duration: Duration,
    exif_fields: usize,
    iteration: u32,
}

#[derive(serde::Serialize)]
struct BenchmarkResults {
    total_files: usize,
    iterations: u32,
    total_duration: Duration,
    successful_files: usize,
    total_exif_fields: usize,
    files_per_second: f64,
    fields_per_second: f64,
    success_rate: f64,
    file_timings: Option<Vec<FileTiming>>,
}

#[derive(Clone)]
struct ExifTagInfo {
    short_name: String,
    description: String,
    category: String,
}

fn get_known_exif_tags() -> HashMap<String, ExifTagInfo> {
    let mut tags = HashMap::new();
    
    // Camera Information
    tags.insert("Make".to_string(), ExifTagInfo {
        short_name: "Make".to_string(),
        description: "Camera manufacturer".to_string(),
        category: "Camera".to_string(),
    });
    
    tags.insert("Model".to_string(), ExifTagInfo {
        short_name: "Model".to_string(),
        description: "Camera model".to_string(),
        category: "Camera".to_string(),
    });
    
    tags.insert("SerialNumber".to_string(), ExifTagInfo {
        short_name: "Serial".to_string(),
        description: "Camera serial number".to_string(),
        category: "Camera".to_string(),
    });
    
    // Image Properties
    tags.insert("ImageWidth".to_string(), ExifTagInfo {
        short_name: "Width".to_string(),
        description: "Image width in pixels".to_string(),
        category: "Image".to_string(),
    });
    
    tags.insert("ImageHeight".to_string(), ExifTagInfo {
        short_name: "Height".to_string(),
        description: "Image height in pixels".to_string(),
        category: "Image".to_string(),
    });
    
    tags.insert("Orientation".to_string(), ExifTagInfo {
        short_name: "Orientation".to_string(),
        description: "Image orientation".to_string(),
        category: "Image".to_string(),
    });
    
    // Date/Time
    tags.insert("DateTime".to_string(), ExifTagInfo {
        short_name: "DateTime".to_string(),
        description: "Date and time when image was taken".to_string(),
        category: "DateTime".to_string(),
    });
    
    tags.insert("DateTimeOriginal".to_string(), ExifTagInfo {
        short_name: "DateTimeOriginal".to_string(),
        description: "Original date and time".to_string(),
        category: "DateTime".to_string(),
    });
    
    tags.insert("DateTimeDigitized".to_string(), ExifTagInfo {
        short_name: "DateTimeDigitized".to_string(),
        description: "Date and time when image was digitized".to_string(),
        category: "DateTime".to_string(),
    });
    
    // Camera Settings
    tags.insert("ExposureTime".to_string(), ExifTagInfo {
        short_name: "ExposureTime".to_string(),
        description: "Exposure time in seconds".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("FNumber".to_string(), ExifTagInfo {
        short_name: "FNumber".to_string(),
        description: "Aperture f-number".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("ISO".to_string(), ExifTagInfo {
        short_name: "ISO".to_string(),
        description: "ISO sensitivity".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("FocalLength".to_string(), ExifTagInfo {
        short_name: "FocalLength".to_string(),
        description: "Focal length of lens".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("Flash".to_string(), ExifTagInfo {
        short_name: "Flash".to_string(),
        description: "Flash firing status".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("WhiteBalance".to_string(), ExifTagInfo {
        short_name: "WhiteBalance".to_string(),
        description: "White balance mode".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("ExposureMode".to_string(), ExifTagInfo {
        short_name: "ExposureMode".to_string(),
        description: "Exposure mode".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("MeteringMode".to_string(), ExifTagInfo {
        short_name: "MeteringMode".to_string(),
        description: "Metering mode".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    // GPS Information
    tags.insert("GPSLatitude".to_string(), ExifTagInfo {
        short_name: "GPSLatitude".to_string(),
        description: "GPS latitude".to_string(),
        category: "GPS".to_string(),
    });
    
    tags.insert("GPSLongitude".to_string(), ExifTagInfo {
        short_name: "GPSLongitude".to_string(),
        description: "GPS longitude".to_string(),
        category: "GPS".to_string(),
    });
    
    tags.insert("GPSAltitude".to_string(), ExifTagInfo {
        short_name: "GPSAltitude".to_string(),
        description: "GPS altitude".to_string(),
        category: "GPS".to_string(),
    });
    
    // File Information
    tags.insert("FileName".to_string(), ExifTagInfo {
        short_name: "FileName".to_string(),
        description: "File name".to_string(),
        category: "File".to_string(),
    });
    
    tags.insert("FileSize".to_string(), ExifTagInfo {
        short_name: "FileSize".to_string(),
        description: "File size in bytes".to_string(),
        category: "File".to_string(),
    });
    
    tags.insert("Directory".to_string(), ExifTagInfo {
        short_name: "Directory".to_string(),
        description: "Directory path".to_string(),
        category: "File".to_string(),
    });
    
    tags.insert("SourceFile".to_string(), ExifTagInfo {
        short_name: "SourceFile".to_string(),
        description: "Source file path".to_string(),
        category: "File".to_string(),
    });
    
    // Additional Camera Settings
    tags.insert("ApertureValue".to_string(), ExifTagInfo {
        short_name: "ApertureValue".to_string(),
        description: "Aperture value".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("BrightnessValue".to_string(), ExifTagInfo {
        short_name: "BrightnessValue".to_string(),
        description: "Brightness value".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("ExposureBiasValue".to_string(), ExifTagInfo {
        short_name: "ExposureBiasValue".to_string(),
        description: "Exposure bias value".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("MaxApertureValue".to_string(), ExifTagInfo {
        short_name: "MaxApertureValue".to_string(),
        description: "Maximum aperture value".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("SubjectDistance".to_string(), ExifTagInfo {
        short_name: "SubjectDistance".to_string(),
        description: "Subject distance".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags.insert("FocalLengthIn35mmFilm".to_string(), ExifTagInfo {
        short_name: "FocalLengthIn35mmFilm".to_string(),
        description: "35mm equivalent focal length".to_string(),
        category: "Camera Settings".to_string(),
    });
    
    tags
}
