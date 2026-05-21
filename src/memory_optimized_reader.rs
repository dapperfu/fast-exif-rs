use std::collections::HashMap;
use crate::types::ExifError;
use crate::FastExifReader;

/// Batch-oriented reader that delegates to [`FastExifReader`] with configurable batch size.
///
/// Post-processing (computed fields, ExifTool field names/values) is applied by the underlying
/// reader, matching [`FastExifReader::read_file`].
pub struct MemoryOptimizedExifReader {
    reader: FastExifReader,
    batch_size: usize,
}

impl MemoryOptimizedExifReader {
    pub fn new() -> Self {
        Self {
            reader: FastExifReader::new(),
            batch_size: 50,
        }
    }

    pub fn read_file(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        self.reader.read_file(file_path)
    }

    pub fn read_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        self.reader.read_bytes(data)
    }

    pub fn read_files_batch(
        &mut self,
        file_paths: Vec<String>,
    ) -> Result<Vec<HashMap<String, String>>, ExifError> {
        let mut results = Vec::new();
        for chunk in file_paths.chunks(self.batch_size) {
            for path in chunk {
                results.push(self.read_file(path)?);
            }
        }
        Ok(results)
    }

    pub fn set_batch_size(&mut self, batch_size: usize) {
        self.batch_size = batch_size;
    }

    pub fn get_batch_size(&self) -> usize {
        self.batch_size
    }
}

impl Default for MemoryOptimizedExifReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark memory-optimized reader against standard reader.
pub fn benchmark_memory_optimization(
    file_paths: Vec<String>,
) -> Result<HashMap<String, String>, ExifError> {
    use std::time::Instant;

    let mut standard_reader = FastExifReader::new();
    let mut memory_reader = MemoryOptimizedExifReader::new();

    let mut standard_times = Vec::new();
    let mut memory_times = Vec::new();

    for file_path in file_paths {
        let start = Instant::now();
        let _ = standard_reader.read_file(&file_path);
        standard_times.push(start.elapsed().as_secs_f64());

        let start = Instant::now();
        let _ = memory_reader.read_file(&file_path);
        memory_times.push(start.elapsed().as_secs_f64());
    }

    let standard_avg = standard_times.iter().sum::<f64>() / standard_times.len().max(1) as f64;
    let memory_avg = memory_times.iter().sum::<f64>() / memory_times.len().max(1) as f64;
    let speedup = if memory_avg > 0.0 {
        standard_avg / memory_avg
    } else {
        1.0
    };

    let mut results = HashMap::new();
    results.insert("standard_avg_time".to_string(), standard_avg.to_string());
    results.insert("memory_avg_time".to_string(), memory_avg.to_string());
    results.insert("speedup".to_string(), speedup.to_string());
    results.insert(
        "files_tested".to_string(),
        standard_times.len().to_string(),
    );

    Ok(results)
}

/// Profile processing time for a single file.
pub fn profile_memory_usage(file_path: &str) -> Result<HashMap<String, String>, ExifError> {
    use std::time::Instant;

    let mut memory_reader = MemoryOptimizedExifReader::new();

    let start_time = Instant::now();
    let _metadata = memory_reader.read_file(file_path)?;
    let elapsed = start_time.elapsed().as_secs_f64();

    let mut profile = HashMap::new();
    profile.insert("processing_time".to_string(), elapsed.to_string());
    profile.insert("file_path".to_string(), file_path.to_string());

    Ok(profile)
}
