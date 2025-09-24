// Allow non-local definitions for PyO3 macros
#![allow(non_local_definitions)]

use memmap2::Mmap;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::time::Instant;

use crate::format_detection::FormatDetector;
use crate::parsers::{HeifParser, JpegParser, RawParser, VideoParser};
use crate::types::{ExifError, ExifResult, ProcessingStats};

/// Multiprocessing EXIF reader using Rayon for parallel processing
#[pyclass]
#[allow(non_local_definitions)]
pub struct MultiprocessingExifReader {
    max_workers: Option<usize>,
}

#[allow(non_local_definitions)]
#[pymethods]
impl MultiprocessingExifReader {
    #[new]
    fn new(max_workers: Option<usize>) -> Self {
        Self { max_workers }
    }

    /// Read EXIF data from multiple files using Rust parallel processing
    fn read_files(&self, file_paths: Vec<String>) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let results = self.process_files_parallel(file_paths)?;
            Ok(results.into_py(py))
        })
    }

    /// Read EXIF data from all image files in a directory
    fn read_directory(
        &self,
        directory: String,
        extensions: Option<Vec<String>>,
        max_files: Option<usize>,
    ) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let file_paths = self.scan_directory(directory, extensions, max_files)?;
            let results = self.process_files_parallel(file_paths)?;
            Ok(results.into_py(py))
        })
    }
}

impl MultiprocessingExifReader {
    fn process_files_parallel(
        &self,
        file_paths: Vec<String>,
    ) -> Result<HashMap<String, PyObject>, ExifError> {
        let start_time = Instant::now();

        // Configure rayon thread pool if max_workers is specified
        // Note: We can't reinitialize the global thread pool, so we'll use the default
        // The rayon crate will automatically use all available CPU cores
        if let Some(max_workers) = self.max_workers {
            // Try to set the thread count, but don't fail if it's already initialized
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(max_workers)
                .build_global();
        }

        // Process files in parallel using rayon
        let results: Vec<ExifResult> = file_paths
            .par_iter()
            .map(|file_path| self.process_single_file(file_path))
            .collect();

        let total_time = start_time.elapsed().as_secs_f64();

        // Calculate statistics
        let success_count = results.iter().filter(|r| r.success).count();
        let error_count = results.len() - success_count;
        let success_rate = if !results.is_empty() {
            success_count as f64 / results.len() as f64 * 100.0
        } else {
            0.0
        };

        let avg_processing_time = if !results.is_empty() {
            results.iter().map(|r| r.processing_time).sum::<f64>() / results.len() as f64
        } else {
            0.0
        };

        let files_per_second = if total_time > 0.0 {
            results.len() as f64 / total_time
        } else {
            0.0
        };

        let stats = ProcessingStats {
            total_files: results.len(),
            success_count,
            error_count,
            success_rate,
            total_time,
            avg_processing_time,
            files_per_second,
        };

        // Convert results to Python objects
        Python::with_gil(|py| {
            let mut result_map = HashMap::new();

            // Add statistics
            result_map.insert("stats".to_string(), stats.into_py(py));

            // Add individual results
            for result in results {
                let file_key = format!(
                    "file_{}",
                    result.file_path.replace("/", "_").replace("\\", "_")
                );
                result_map.insert(file_key, result.into_py(py));
            }

            Ok(result_map)
        })
    }

    fn process_single_file(&self, file_path: &str) -> ExifResult {
        let start_time = Instant::now();

        match self.read_file_internal(file_path) {
            Ok(metadata) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                ExifResult {
                    file_path: file_path.to_string(),
                    metadata,
                    processing_time,
                    success: true,
                    error: None,
                }
            }
            Err(e) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                ExifResult {
                    file_path: file_path.to_string(),
                    metadata: HashMap::new(),
                    processing_time,
                    success: false,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    fn read_file_internal(&self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        self.read_exif_from_bytes(&mmap)
    }

    fn read_exif_from_bytes(&self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
        let mut metadata = HashMap::new();

        // Detect file format
        let format = FormatDetector::detect_format(data)?;
        metadata.insert("Format".to_string(), format.clone());

        // Parse EXIF based on format
        match format.as_str() {
            "JPEG" => JpegParser::parse_jpeg_exif(data, &mut metadata)?,
            "CR2" => RawParser::parse_cr2_exif(data, &mut metadata)?,
            "NEF" => RawParser::parse_nef_exif(data, &mut metadata)?,
            "ORF" => RawParser::parse_orf_exif(data, &mut metadata)?,
            "DNG" => RawParser::parse_dng_exif(data, &mut metadata)?,
            "HEIF" | "HIF" => HeifParser::parse_heif_exif(data, &mut metadata)?,
            "MOV" => VideoParser::parse_mov_exif(data, &mut metadata)?,
            "MP4" => VideoParser::parse_mp4_exif(data, &mut metadata)?,
            "3GP" => VideoParser::parse_3gp_exif(data, &mut metadata)?,
            _ => {
                return Err(ExifError::UnsupportedFormat(format!(
                    "Unsupported format: {}",
                    format
                )))
            }
        }

        Ok(metadata)
    }

    fn scan_directory(
        &self,
        directory: String,
        extensions: Option<Vec<String>>,
        max_files: Option<usize>,
    ) -> Result<Vec<String>, ExifError> {
        let default_extensions = vec![
            ".jpg".to_string(),
            ".jpeg".to_string(),
            ".cr2".to_string(),
            ".nef".to_string(),
            ".heic".to_string(),
            ".heif".to_string(),
            ".tiff".to_string(),
            ".tif".to_string(),
            ".png".to_string(),
            ".bmp".to_string(),
        ];

        let extensions = extensions.unwrap_or(default_extensions);
        let mut file_paths = Vec::new();

        let dir = std::fs::read_dir(&directory).map_err(ExifError::IoError)?;

        for entry in dir {
            let entry = entry.map_err(ExifError::IoError)?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    let ext_with_dot = format!(".{}", ext_str);

                    if extensions.contains(&ext_with_dot) {
                        file_paths.push(path.to_string_lossy().to_string());

                        if let Some(max) = max_files {
                            if file_paths.len() >= max {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(file_paths)
    }
}

/// Standalone function for parallel EXIF processing
#[pyfunction]
pub fn process_files_parallel(
    file_paths: Vec<String>,
    max_workers: Option<usize>,
) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let reader = MultiprocessingExifReader::new(max_workers);
        let results = reader
            .process_files_parallel(file_paths)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(results.into_py(py))
    })
}

/// Standalone function for directory processing
#[pyfunction]
pub fn process_directory_parallel(
    directory: String,
    extensions: Option<Vec<String>>,
    max_files: Option<usize>,
    max_workers: Option<usize>,
) -> PyResult<PyObject> {
    Python::with_gil(|_py| {
        let reader = MultiprocessingExifReader::new(max_workers);
        let results = reader
            .read_directory(directory, extensions, max_files)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(results)
    })
}
