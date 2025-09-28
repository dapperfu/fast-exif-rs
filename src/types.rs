use pyo3::prelude::*;
use std::collections::HashMap;
use thiserror::Error;

/// Error types for EXIF operations
#[derive(Error, Debug)]
pub enum ExifError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Invalid EXIF data: {0}")]
    InvalidExif(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<ExifError> for PyErr {
    fn from(err: ExifError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string())
    }
}

/// Result structure for multiprocessing operations
#[derive(Debug, Clone)]
#[pyclass]
pub struct ExifResult {
    pub file_path: String,
    pub metadata: HashMap<String, String>,
    pub processing_time: f64,
    pub success: bool,
    pub error: Option<String>,
}

/// Statistics for multiprocessing operations
#[derive(Debug, Clone)]
#[pyclass]
pub struct ProcessingStats {
    pub total_files: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub success_rate: f64,
    pub total_time: f64,
    pub avg_processing_time: f64,
    pub files_per_second: f64,
}
