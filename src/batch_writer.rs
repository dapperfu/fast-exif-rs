use crate::types::{ExifError, ProcessingStats};
use crate::writer::ExifWriter;
use crate::exif_copier::ExifCopier;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

/// Batch EXIF writing operation
#[derive(Debug, Clone)]
pub struct BatchWriteOperation {
    pub input_path: String,
    pub output_path: String,
    pub metadata: HashMap<String, String>,
}

/// Batch EXIF copy operation
#[derive(Debug, Clone)]
pub struct BatchCopyOperation {
    pub source_path: String,
    pub target_path: String,
    pub output_path: String,
}

/// Result of a batch write operation
#[derive(Debug, Clone)]
pub struct BatchWriteResult {
    pub input_path: String,
    pub output_path: String,
    pub success: bool,
    pub processing_time: f64,
    pub error: Option<String>,
    pub fields_written: usize,
}

impl BatchWriteResult {
    fn to_python_dict(&self, py: Python) -> Py<PyAny> {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("input_path", &self.input_path).unwrap();
        dict.set_item("output_path", &self.output_path).unwrap();
        dict.set_item("success", self.success).unwrap();
        dict.set_item("processing_time", self.processing_time).unwrap();
        dict.set_item("error", &self.error).unwrap();
        dict.set_item("fields_written", self.fields_written).unwrap();
        dict.into()
    }
}

/// Batch EXIF writer for parallel processing
#[pyclass]
#[derive(Clone)]
pub struct BatchExifWriter {
    writer: ExifWriter,
    copier: ExifCopier,
    max_workers: Option<usize>,
}

#[pymethods]
impl BatchExifWriter {
    #[new]
    fn new(max_workers: Option<usize>) -> Self {
        Self {
            writer: ExifWriter::new(),
            copier: ExifCopier::new(),
            max_workers,
        }
    }

    /// Write EXIF metadata to multiple files in parallel
    fn write_exif_batch(
        &self,
        operations: Vec<Py<PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            // Convert Python objects to batch operations
            let mut batch_ops = Vec::new();
            for op in operations {
                let dict = op.bind(py).downcast::<pyo3::types::PyDict>()?;
                
                let input_path: String = dict.get_item("input_path")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing input_path"))?
                    .extract()?;
                
                let output_path: String = dict.get_item("output_path")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing output_path"))?
                    .extract()?;
                
                let metadata_item = dict.get_item("metadata")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing metadata"))?;
                let metadata_dict = metadata_item.downcast::<pyo3::types::PyDict>()?;
                
                let mut metadata = HashMap::new();
                for (key, value) in metadata_dict.iter() {
                    let key_str: String = key.extract()?;
                    let value_str: String = value.extract()?;
                    metadata.insert(key_str, value_str);
                }
                
                batch_ops.push(BatchWriteOperation {
                    input_path,
                    output_path,
                    metadata,
                });
            }
            
            let results = self.process_write_batch(batch_ops)?;
            let py_dict = pyo3::types::PyDict::new(py);
            for (k, v) in results {
                py_dict.set_item(k, v).unwrap();
            }
            Ok(py_dict.into())
        })
    }

    /// Copy EXIF metadata between multiple file pairs in parallel
    fn copy_exif_batch(
        &mut self,
        operations: Vec<Py<PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            // Convert Python objects to batch copy operations
            let mut batch_ops = Vec::new();
            for op in operations {
                let dict = op.bind(py).downcast::<pyo3::types::PyDict>()?;
                
                let source_path: String = dict.get_item("source_path")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing source_path"))?
                    .extract()?;
                
                let target_path: String = dict.get_item("target_path")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing target_path"))?
                    .extract()?;
                
                let output_path: String = dict.get_item("output_path")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing output_path"))?
                    .extract()?;
                
                batch_ops.push(BatchCopyOperation {
                    source_path,
                    target_path,
                    output_path,
                });
            }
            
            let results = self.process_copy_batch(batch_ops)?;
            let py_dict = pyo3::types::PyDict::new(py);
            for (k, v) in results {
                py_dict.set_item(k, v).unwrap();
            }
            Ok(py_dict.into())
        })
    }

    /// Write EXIF metadata to multiple files with high-priority field filtering
    fn write_high_priority_exif_batch(
        &self,
        operations: Vec<Py<PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            // Convert Python objects to batch operations
            let mut batch_ops = Vec::new();
            for op in operations {
                let dict = op.bind(py).downcast::<pyo3::types::PyDict>()?;
                
                let input_path: String = dict.get_item("input_path")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing input_path"))?
                    .extract()?;
                
                let output_path: String = dict.get_item("output_path")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing output_path"))?
                    .extract()?;
                
                let metadata_item = dict.get_item("metadata")?
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing metadata"))?;
                let metadata_dict = metadata_item.downcast::<pyo3::types::PyDict>()?;
                
                let mut metadata = HashMap::new();
                for (key, value) in metadata_dict.iter() {
                    let key_str: String = key.extract()?;
                    let value_str: String = value.extract()?;
                    metadata.insert(key_str, value_str);
                }
                
                batch_ops.push(BatchWriteOperation {
                    input_path,
                    output_path,
                    metadata,
                });
            }
            
            let results = self.process_write_batch_high_priority(batch_ops)?;
            let py_dict = pyo3::types::PyDict::new(py);
            for (k, v) in results {
                py_dict.set_item(k, v).unwrap();
            }
            Ok(py_dict.into())
        })
    }
}

impl BatchExifWriter {
    /// Process batch write operations in parallel
    fn process_write_batch(
        &self,
        operations: Vec<BatchWriteOperation>,
    ) -> Result<HashMap<String, Py<PyAny>>, ExifError> {
        let start_time = Instant::now();

        // Configure rayon thread pool if max_workers is specified
        if let Some(max_workers) = self.max_workers {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(max_workers)
                .build_global();
        }

        // Process operations in parallel
        let results: Vec<BatchWriteResult> = operations
            .par_iter()
            .map(|op| self.process_single_write(op))
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

        let total_fields_written: usize = results.iter()
            .map(|r| r.fields_written)
            .sum();

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
        Python::attach(|py| {
            let mut result_map = HashMap::new();

            // Add statistics
            result_map.insert(
                "stats".to_string(),
                {
                    let dict = pyo3::types::PyDict::new(py);
                    dict.set_item("total_files", stats.total_files).unwrap();
                    dict.set_item("success_count", stats.success_count).unwrap();
                    dict.set_item("error_count", stats.error_count).unwrap();
                    dict.set_item("success_rate", stats.success_rate).unwrap();
                    dict.set_item("total_time", stats.total_time).unwrap();
                    dict.set_item("avg_processing_time", stats.avg_processing_time).unwrap();
                    dict.set_item("files_per_second", stats.files_per_second).unwrap();
                    dict.into()
                },
            );

            // Add field statistics
            result_map.insert(
                "total_fields_written".to_string(),
                total_fields_written.into_pyobject(py).unwrap().into(),
            );

            // Add individual results
            for result in results {
                let file_key = format!(
                    "file_{}",
                    result.input_path.replace("/", "_").replace("\\", "_")
                );
                result_map.insert(
                    file_key,
                    result.to_python_dict(py),
                );
            }

            Ok(result_map)
        })
    }

    /// Process batch copy operations in parallel
    fn process_copy_batch(
        &mut self,
        operations: Vec<BatchCopyOperation>,
    ) -> Result<HashMap<String, Py<PyAny>>, ExifError> {
        let start_time = Instant::now();

        // Configure rayon thread pool if max_workers is specified
        if let Some(max_workers) = self.max_workers {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(max_workers)
                .build_global();
        }

        // Process operations in parallel
        // Since we need mutable access, we'll process sequentially for now
        let mut results = Vec::new();
        for op in operations {
            results.push(self.process_single_copy(&op));
        }

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
        Python::attach(|py| {
            let mut result_map = HashMap::new();

            // Add statistics
            result_map.insert(
                "stats".to_string(),
                {
                    let dict = pyo3::types::PyDict::new(py);
                    dict.set_item("total_files", stats.total_files).unwrap();
                    dict.set_item("success_count", stats.success_count).unwrap();
                    dict.set_item("error_count", stats.error_count).unwrap();
                    dict.set_item("success_rate", stats.success_rate).unwrap();
                    dict.set_item("total_time", stats.total_time).unwrap();
                    dict.set_item("avg_processing_time", stats.avg_processing_time).unwrap();
                    dict.set_item("files_per_second", stats.files_per_second).unwrap();
                    dict.into()
                },
            );

            // Add individual results
            for result in results {
                let file_key = format!(
                    "file_{}",
                    result.input_path.replace("/", "_").replace("\\", "_")
                );
                result_map.insert(
                    file_key,
                    result.to_python_dict(py),
                );
            }

            Ok(result_map)
        })
    }

    /// Process batch write operations with high-priority field filtering
    fn process_write_batch_high_priority(
        &self,
        operations: Vec<BatchWriteOperation>,
    ) -> Result<HashMap<String, Py<PyAny>>, ExifError> {
        let start_time = Instant::now();

        // Configure rayon thread pool if max_workers is specified
        if let Some(max_workers) = self.max_workers {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(max_workers)
                .build_global();
        }

        // Process operations in parallel with high-priority filtering
        let results: Vec<BatchWriteResult> = operations
            .par_iter()
            .map(|op| self.process_single_write_high_priority(op))
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

        let total_fields_written: usize = results.iter()
            .map(|r| r.fields_written)
            .sum();

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
        Python::attach(|py| {
            let mut result_map = HashMap::new();

            // Add statistics
            result_map.insert(
                "stats".to_string(),
                {
                    let dict = pyo3::types::PyDict::new(py);
                    dict.set_item("total_files", stats.total_files).unwrap();
                    dict.set_item("success_count", stats.success_count).unwrap();
                    dict.set_item("error_count", stats.error_count).unwrap();
                    dict.set_item("success_rate", stats.success_rate).unwrap();
                    dict.set_item("total_time", stats.total_time).unwrap();
                    dict.set_item("avg_processing_time", stats.avg_processing_time).unwrap();
                    dict.set_item("files_per_second", stats.files_per_second).unwrap();
                    dict.into()
                },
            );

            // Add field statistics
            result_map.insert(
                "total_fields_written".to_string(),
                total_fields_written.into_pyobject(py).unwrap().into(),
            );

            // Add individual results
            for result in results {
                let file_key = format!(
                    "file_{}",
                    result.input_path.replace("/", "_").replace("\\", "_")
                );
                result_map.insert(
                    file_key,
                    result.to_python_dict(py),
                );
            }

            Ok(result_map)
        })
    }

    /// Process a single write operation
    fn process_single_write(&self, operation: &BatchWriteOperation) -> BatchWriteResult {
        let start_time = Instant::now();

        match self.writer.write_exif(
            &operation.input_path,
            &operation.output_path,
            &operation.metadata,
        ) {
            Ok(_) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                BatchWriteResult {
                    input_path: operation.input_path.clone(),
                    output_path: operation.output_path.clone(),
                    success: true,
                    processing_time,
                    error: None,
                    fields_written: operation.metadata.len(),
                }
            }
            Err(e) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                BatchWriteResult {
                    input_path: operation.input_path.clone(),
                    output_path: operation.output_path.clone(),
                    success: false,
                    processing_time,
                    error: Some(e.to_string()),
                    fields_written: 0,
                }
            }
        }
    }

    /// Process a single copy operation
    fn process_single_copy(&mut self, operation: &BatchCopyOperation) -> BatchWriteResult {
        let start_time = Instant::now();

        match self.copier.copy_high_priority_exif(
            &operation.source_path,
            &operation.target_path,
            &operation.output_path,
        ) {
            Ok(_) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                BatchWriteResult {
                    input_path: operation.source_path.clone(),
                    output_path: operation.output_path.clone(),
                    success: true,
                    processing_time,
                    error: None,
                    fields_written: 0, // Will be determined by copier
                }
            }
            Err(e) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                BatchWriteResult {
                    input_path: operation.source_path.clone(),
                    output_path: operation.output_path.clone(),
                    success: false,
                    processing_time,
                    error: Some(e.to_string()),
                    fields_written: 0,
                }
            }
        }
    }

    /// Process a single write operation with high-priority field filtering
    fn process_single_write_high_priority(&self, operation: &BatchWriteOperation) -> BatchWriteResult {
        let start_time = Instant::now();

        // Filter to high-priority fields
        let high_priority_metadata = crate::utils::ExifUtils::filter_high_priority_fields(&operation.metadata);

        match self.writer.write_exif(
            &operation.input_path,
            &operation.output_path,
            &high_priority_metadata,
        ) {
            Ok(_) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                BatchWriteResult {
                    input_path: operation.input_path.clone(),
                    output_path: operation.output_path.clone(),
                    success: true,
                    processing_time,
                    error: None,
                    fields_written: high_priority_metadata.len(),
                }
            }
            Err(e) => {
                let processing_time = start_time.elapsed().as_secs_f64();
                BatchWriteResult {
                    input_path: operation.input_path.clone(),
                    output_path: operation.output_path.clone(),
                    success: false,
                    processing_time,
                    error: Some(e.to_string()),
                    fields_written: 0,
                }
            }
        }
    }
}

impl Default for BatchExifWriter {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Standalone function for batch EXIF writing
#[pyfunction]
pub fn write_exif_batch_parallel(
    operations: Vec<Py<PyAny>>,
    max_workers: Option<usize>,
) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        let writer = BatchExifWriter::new(max_workers);
        let results = writer
            .write_exif_batch(operations)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(results)
    })
}

/// Standalone function for batch EXIF copying
#[pyfunction]
pub fn copy_exif_batch_parallel(
    operations: Vec<Py<PyAny>>,
    max_workers: Option<usize>,
) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        let mut writer = BatchExifWriter::new(max_workers);
        let results = writer
            .copy_exif_batch(operations)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(results)
    })
}
