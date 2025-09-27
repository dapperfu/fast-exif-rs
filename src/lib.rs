// Allow non-local definitions for PyO3 macros
#![allow(non_local_definitions)]

use memmap2::Mmap;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::collections::HashMap;
use std::fs::File;

// Module declarations
mod format_detection;
mod multiprocessing;
mod parsers;
mod types;
mod utils;
mod v2_reader;
mod writer;
mod exif_copier;
mod batch_writer;

// Re-export commonly used types
pub use format_detection::FormatDetector;
pub use multiprocessing::MultiprocessingExifReader;
pub use parsers::{BmpParser, HeifParser, JpegParser, MkvParser, PngParser, RawParser, VideoParser};
pub use types::{ExifError, ExifResult, ProcessingStats};
pub use utils::ExifUtils;
pub use v2_reader::FastExifReaderV2;
pub use writer::ExifWriter;
pub use exif_copier::ExifCopier;
pub use batch_writer::{BatchExifWriter, write_exif_batch_parallel, copy_exif_batch_parallel};

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
    fn read_file(&mut self, file_path: &str) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let metadata = self.read_exif_fast(file_path)?;
            Ok(metadata.into_pyobject(py)?.into())
        })
    }

    /// Read EXIF data from bytes
    fn read_bytes(&mut self, data: &[u8]) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let metadata = self.read_exif_from_bytes(data)?;
            Ok(metadata.into_pyobject(py)?.into())
        })
    }

    /// Support for pickle protocol
    fn __getstate__(&self, py: Python) -> PyResult<Py<PyAny>> {
        // Serialize the buffer as bytes
        let buffer_bytes = PyBytes::new(py, &self.buffer);
        Ok(buffer_bytes.into())
    }

    /// Support for pickle protocol
    fn __setstate__(&mut self, py: Python, state: Py<PyAny>) -> PyResult<()> {
        // Deserialize the buffer from bytes
        let buffer_bytes: &[u8] = state.bind(py).extract()?;
        self.buffer = buffer_bytes.to_vec();
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
            "PNG" => PngParser::parse_png_exif(data, &mut metadata)?,
            "BMP" => BmpParser::parse_bmp_exif(data, &mut metadata)?,
            "MKV" => MkvParser::parse_mkv_exif(data, &mut metadata)?,
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

/// Fast EXIF writer for adding/modifying EXIF metadata
#[pyclass]
#[derive(Clone)]
#[allow(non_local_definitions)]
pub struct FastExifWriter {
    writer: ExifWriter,
}

/// Fast EXIF copier for copying metadata between images
#[pyclass]
#[derive(Clone)]
#[allow(non_local_definitions)]
pub struct FastExifCopier {
    copier: ExifCopier,
}

#[allow(non_local_definitions)]
#[pymethods]
impl FastExifWriter {
    #[new]
    fn new() -> Self {
        Self {
            writer: ExifWriter::new(),
        }
    }

    /// Write EXIF metadata to an image file (auto-detects format)
    fn write_exif(
        &self,
        input_path: &str,
        output_path: &str,
        metadata: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let mut exif_metadata = HashMap::new();
        
        // Convert Python dict to HashMap
        for (key, value) in metadata.iter() {
            let key_str: String = key.extract()?;
            let value_str: String = value.extract()?;
            exif_metadata.insert(key_str, value_str);
        }
        
        self.writer.write_exif(input_path, output_path, &exif_metadata)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(())
    }

    /// Write EXIF metadata to a JPEG file (legacy method)
    fn write_jpeg_exif(
        &self,
        input_path: &str,
        output_path: &str,
        metadata: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        self.write_exif(input_path, output_path, metadata)
    }

    /// Write EXIF metadata to image bytes (auto-detects format)
    fn write_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &Bound<'_, PyDict>,
    ) -> PyResult<Vec<u8>> {
        let mut exif_metadata = HashMap::new();
        
        // Convert Python dict to HashMap
        for (key, value) in metadata.iter() {
            let key_str: String = key.extract()?;
            let value_str: String = value.extract()?;
            exif_metadata.insert(key_str, value_str);
        }
        
        self.writer.write_exif_to_bytes(input_data, &exif_metadata)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Write EXIF metadata to JPEG bytes (legacy method)
    fn write_jpeg_exif_to_bytes(
        &self,
        input_data: &[u8],
        metadata: &Bound<'_, PyDict>,
    ) -> PyResult<Vec<u8>> {
        self.write_exif_to_bytes(input_data, metadata)
    }

    /// Copy high-priority EXIF fields from source to target image
    fn copy_high_priority_exif(
        &self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> PyResult<()> {
        self.writer.copy_high_priority_exif(source_path, target_path, output_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Copy high-priority EXIF fields from source bytes to target bytes
    fn copy_high_priority_exif_to_bytes(
        &self,
        source_data: &[u8],
        target_data: &[u8],
    ) -> PyResult<Vec<u8>> {
        self.writer.copy_high_priority_exif_to_bytes(source_data, target_data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

#[allow(non_local_definitions)]
#[pymethods]
impl FastExifCopier {
    #[new]
    fn new() -> Self {
        Self {
            copier: ExifCopier::new(),
        }
    }

    /// Copy high-priority EXIF fields from source to target image
    fn copy_high_priority_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> PyResult<()> {
        self.copier.copy_high_priority_exif(source_path, target_path, output_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Copy high-priority EXIF fields from source bytes to target bytes
    fn copy_high_priority_exif_to_bytes(
        &mut self,
        source_data: &[u8],
        target_data: &[u8],
    ) -> PyResult<Vec<u8>> {
        self.copier.copy_high_priority_exif_to_bytes(source_data, target_data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Copy all EXIF fields from source to target image
    fn copy_all_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
    ) -> PyResult<()> {
        self.copier.copy_all_exif(source_path, target_path, output_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Copy specific EXIF fields from source to target image
    fn copy_specific_exif(
        &mut self,
        source_path: &str,
        target_path: &str,
        output_path: &str,
        field_names: Vec<String>,
    ) -> PyResult<()> {
        let field_refs: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();
        self.copier.copy_specific_exif(source_path, target_path, output_path, &field_refs)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get available EXIF fields from source image
    fn get_available_fields(&mut self, source_path: &str) -> PyResult<Vec<String>> {
        self.copier.get_available_fields(source_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get high-priority EXIF fields from source image
    fn get_high_priority_fields(&mut self, source_path: &str) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let metadata = self.copier.get_high_priority_fields(source_path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            let dict = PyDict::new(py);
            for (key, value) in metadata {
                dict.set_item(key, value)?;
            }
            
            Ok(dict.into())
        })
    }
}

/// Python module definition
#[pymodule]
fn fast_exif_reader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FastExifReader>()?;
    m.add_class::<FastExifWriter>()?;
    m.add_class::<FastExifCopier>()?;
    m.add_class::<MultiprocessingExifReader>()?;
    m.add_class::<BatchExifWriter>()?;
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
    m.add_function(wrap_pyfunction!(
        batch_writer::write_exif_batch_parallel,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        batch_writer::copy_exif_batch_parallel,
        m
    )?)?;
    Ok(())
}
