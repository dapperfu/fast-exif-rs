// Allow non-local definitions for PyO3 macros
#![allow(non_local_definitions)]

use memmap2::Mmap;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::collections::HashMap;
use std::fs::File;

// Module declarations
mod format_detection;
mod multiprocessing;
mod parsers;
mod types;
mod utils;

// Re-export commonly used types
pub use format_detection::FormatDetector;
pub use multiprocessing::MultiprocessingExifReader;
pub use parsers::{HeifParser, JpegParser, RawParser, VideoParser};
pub use types::{ExifError, ExifResult, ProcessingStats};
pub use utils::ExifUtils;

/// Fast EXIF reader optimized for Canon 70D and Nikon Z50 II
#[pyclass]
#[derive(Clone)]
#[allow(non_local_definitions)]
pub struct FastExifReader {
    // Pre-allocated buffers for performance
    buffer: Vec<u8>,
}

#[allow(non_local_definitions)]
#[pymethods]
impl FastExifReader {
    #[new]
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(1024 * 1024), // 1MB buffer
        }
    }

    /// Read EXIF data from file path
    fn read_file(&mut self, file_path: &str) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let metadata = self.read_exif_fast(file_path)?;
            Ok(metadata.into_py(py))
        })
    }

    /// Read EXIF data from bytes
    fn read_bytes(&mut self, data: &[u8]) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let metadata = self.read_exif_from_bytes(data)?;
            Ok(metadata.into_py(py))
        })
    }

    /// Support for pickle protocol
    fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        // Serialize the buffer as bytes
        let buffer_bytes = PyBytes::new(py, &self.buffer);
        Ok(buffer_bytes.into())
    }

    /// Support for pickle protocol
    fn __setstate__(&mut self, py: Python, state: PyObject) -> PyResult<()> {
        // Deserialize the buffer from bytes
        let buffer_bytes: &PyBytes = state.extract(py)?;
        self.buffer = buffer_bytes.as_bytes().to_vec();
        Ok(())
    }
}

impl FastExifReader {
    fn read_exif_fast(&mut self, file_path: &str) -> Result<HashMap<String, String>, ExifError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        self.read_exif_from_bytes(&mmap)
    }

    fn read_exif_from_bytes(&mut self, data: &[u8]) -> Result<HashMap<String, String>, ExifError> {
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
}

/// Python module definition
#[pymodule]
fn fast_exif_reader(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<FastExifReader>()?;
    m.add_class::<MultiprocessingExifReader>()?;
    m.add_class::<ExifResult>()?;
    m.add_class::<ProcessingStats>()?;
    m.add_function(wrap_pyfunction!(
        multiprocessing::process_files_parallel,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        multiprocessing::process_directory_parallel,
        m
    )?)?;
    Ok(())
}
