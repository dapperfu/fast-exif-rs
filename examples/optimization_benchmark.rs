use std::time::Instant;
use fast_exif_reader::{FastExifReader, OptimalExifParser, OptimalBatchProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("EXIF Parser Optimization Benchmark");
    println!("==================================");

    let test_files: Vec<String> = std::env::args()
        .skip(1)
        .collect();

    let existing_files: Vec<String> = if test_files.is_empty() {
        vec![
            "test_images/sample1.jpg".to_string(),
            "test_images/sample2.jpg".to_string(),
        ]
        .into_iter()
        .filter(|path| std::path::Path::new(path).exists())
        .collect()
    } else {
        test_files
            .into_iter()
            .filter(|path| std::path::Path::new(path).exists())
            .collect()
    };

    if existing_files.is_empty() {
        println!("No test files found. Pass file paths as arguments or add JPEGs under test_images/.");
        return Ok(());
    }

    println!("Testing with {} files\n", existing_files.len());

    let start = Instant::now();
    let mut optimal = OptimalExifParser::new();
    let mut optimal_fields = 0usize;
    for path in &existing_files {
        if let Ok(meta) = optimal.parse_file(path) {
            optimal_fields += meta.len();
        }
    }
    println!(
        "OptimalExifParser: {:.3}s ({} fields)",
        start.elapsed().as_secs_f64(),
        optimal_fields
    );

    let start = Instant::now();
    let mut reader = FastExifReader::new();
    let mut reader_fields = 0usize;
    for path in &existing_files {
        if let Ok(meta) = reader.read_file(path) {
            reader_fields += meta.len();
        }
    }
    println!(
        "FastExifReader: {:.3}s ({} fields)",
        start.elapsed().as_secs_f64(),
        reader_fields
    );

    let start = Instant::now();
    let mut batch = OptimalBatchProcessor::new(existing_files.len());
    let batch_results = batch.process_files(&existing_files)?;
    println!(
        "OptimalBatchProcessor: {:.3}s ({} files)",
        start.elapsed().as_secs_f64(),
        batch_results.len()
    );

    Ok(())
}
