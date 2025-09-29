use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Simplified memory pool for efficient allocation reuse
pub struct MemoryPool {
    /// Pool of reusable HashMap instances
    hashmap_pool: Arc<Mutex<Vec<HashMap<String, String>>>>,
    /// Pool of reusable Vec<u8> buffers
    buffer_pool: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Maximum pool size to prevent memory bloat
    max_pool_size: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new() -> Self {
        Self {
            hashmap_pool: Arc::new(Mutex::new(Vec::new())),
            buffer_pool: Arc::new(Mutex::new(Vec::new())),
            max_pool_size: 50, // Keep up to 50 instances of each type
        }
    }
    
    /// Get a HashMap from the pool or create a new one
    pub fn get_hashmap(&self) -> HashMap<String, String> {
        if let Ok(mut pool) = self.hashmap_pool.lock() {
            if let Some(mut map) = pool.pop() {
                map.clear(); // Clear but keep capacity
                return map;
            }
        }
        
        // Create new HashMap with pre-allocated capacity
        HashMap::with_capacity(200) // Pre-allocate for ~200 fields
    }
    
    /// Return a HashMap to the pool for reuse
    pub fn return_hashmap(&self, mut map: HashMap<String, String>) {
        if let Ok(mut pool) = self.hashmap_pool.lock() {
            if pool.len() < self.max_pool_size {
                map.clear();
                pool.push(map);
            }
        }
    }
    
    /// Get a buffer from the pool or create a new one
    pub fn get_buffer(&self, capacity: usize) -> Vec<u8> {
        if let Ok(mut pool) = self.buffer_pool.lock() {
            if let Some(mut buffer) = pool.pop() {
                buffer.clear();
                if buffer.capacity() >= capacity {
                    return buffer;
                }
            }
        }
        
        // Create new buffer with requested capacity
        Vec::with_capacity(capacity.max(64 * 1024)) // Minimum 64KB
    }
    
    /// Return a buffer to the pool for reuse
    pub fn return_buffer(&self, mut buffer: Vec<u8>) {
        if let Ok(mut pool) = self.buffer_pool.lock() {
            if pool.len() < self.max_pool_size && buffer.capacity() <= 1024 * 1024 {
                buffer.clear();
                pool.push(buffer);
            }
        }
    }
    
    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        let hashmap_count = self.hashmap_pool.lock().map(|p| p.len()).unwrap_or(0);
        let buffer_count = self.buffer_pool.lock().map(|p| p.len()).unwrap_or(0);
        
        PoolStats {
            hashmap_count,
            buffer_count,
            max_pool_size: self.max_pool_size,
        }
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about memory pool usage
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub hashmap_count: usize,
    pub buffer_count: usize,
    pub max_pool_size: usize,
}

/// Memory-optimized EXIF reader with simplified allocation management
pub struct MemoryOptimizedReader {
    /// Memory pool for reuse
    memory_pool: MemoryPool,
    /// Pre-allocated metadata buffer
    metadata_buffer: HashMap<String, String>,
    /// Pre-allocated parsing buffer
    parsing_buffer: Vec<u8>,
}

impl MemoryOptimizedReader {
    /// Create a new memory-optimized reader
    pub fn new() -> Self {
        Self {
            memory_pool: MemoryPool::new(),
            metadata_buffer: HashMap::with_capacity(200),
            parsing_buffer: Vec::with_capacity(1024 * 1024), // 1MB buffer
        }
    }
    
    /// Parse EXIF data with memory optimization
    pub fn parse_exif_optimized(
        &mut self,
        data: &[u8],
        file_extension: &str,
    ) -> Result<HashMap<String, String>, crate::types::ExifError> {
        // Clear and reuse metadata buffer
        self.metadata_buffer.clear();
        
        // Parse based on file extension with optimized allocations
        match file_extension.to_lowercase().as_str() {
            "jpg" | "jpeg" => {
                self.parse_jpeg_optimized(data)?;
            },
            "cr2" => {
                crate::parsers::raw::RawParser::parse_cr2_exif(data, &mut self.metadata_buffer)?;
            },
            "heic" | "heif" => {
                crate::parsers::heif::HeifParser::parse_heif_exif(data, &mut self.metadata_buffer)?;
            },
            _ => {
                return Err(crate::types::ExifError::UnsupportedFormat(
                    file_extension.to_string()
                ));
            }
        }
        
        Ok(self.metadata_buffer.clone())
    }
    
    /// Parse JPEG with memory optimization
    fn parse_jpeg_optimized(
        &mut self,
        data: &[u8],
    ) -> Result<(), crate::types::ExifError> {
        // Clear and reuse parsing buffer
        self.parsing_buffer.clear();
        
        // Find EXIF segment efficiently
        if let Some(exif_data) = self.find_exif_segment_optimized(data) {
            crate::parsers::tiff::TiffParser::parse_tiff_exif(exif_data, &mut self.metadata_buffer)?;
        }
        
        Ok(())
    }
    
    /// Find EXIF segment with optimized buffer usage
    fn find_exif_segment_optimized<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        // Efficient EXIF segment finding with minimal allocations
        for i in 0..data.len().saturating_sub(4) {
            if data[i] == 0xFF && data[i + 1] == 0xE1 {
                // Found EXIF marker, extract segment
                if i + 4 < data.len() {
                    let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                    if i + 4 + length <= data.len() {
                        return Some(&data[i + 4..i + 4 + length]);
                    }
                }
            }
        }
        None
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        let pool_stats = self.memory_pool.get_stats();
        
        MemoryStats {
            pool_stats,
            buffer_capacity: self.parsing_buffer.capacity(),
            metadata_capacity: self.metadata_buffer.capacity(),
        }
    }
    
    /// Clear all caches and reset memory usage
    pub fn clear_caches(&mut self) {
        self.metadata_buffer.clear();
        self.parsing_buffer.clear();
    }
}

impl Default for MemoryOptimizedReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub pool_stats: PoolStats,
    pub buffer_capacity: usize,
    pub metadata_capacity: usize,
}

/// Batch processing with memory optimization
pub struct BatchMemoryOptimizer {
    /// Memory pool for batch operations
    memory_pool: MemoryPool,
    /// Batch size for optimal memory usage
    batch_size: usize,
    /// Current batch buffer
    batch_buffer: Vec<HashMap<String, String>>,
}

impl BatchMemoryOptimizer {
    /// Create a new batch memory optimizer
    pub fn new(batch_size: usize) -> Self {
        Self {
            memory_pool: MemoryPool::new(),
            batch_size,
            batch_buffer: Vec::with_capacity(batch_size),
        }
    }
    
    /// Process files in batches with memory optimization
    pub fn process_files_batch(
        &mut self,
        file_paths: &[String],
        processor: impl Fn(&str) -> Result<HashMap<String, String>, crate::types::ExifError>,
    ) -> Result<Vec<HashMap<String, String>>, crate::types::ExifError> {
        let mut results = Vec::with_capacity(file_paths.len());
        
        for chunk in file_paths.chunks(self.batch_size) {
            self.batch_buffer.clear();
            
            for file_path in chunk {
                match processor(file_path) {
                    Ok(metadata) => self.batch_buffer.push(metadata),
                    Err(e) => return Err(e),
                }
            }
            
            results.extend(self.batch_buffer.drain(..));
        }
        
        Ok(results)
    }
    
    /// Get batch processing statistics
    pub fn get_batch_stats(&self) -> BatchStats {
        let pool_stats = self.memory_pool.get_stats();
        
        BatchStats {
            pool_stats,
            batch_size: self.batch_size,
            buffer_capacity: self.batch_buffer.capacity(),
        }
    }
}

impl Default for BatchMemoryOptimizer {
    fn default() -> Self {
        Self::new(100) // Default batch size of 100
    }
}

/// Batch processing statistics
#[derive(Debug, Clone)]
pub struct BatchStats {
    pub pool_stats: PoolStats,
    pub batch_size: usize,
    pub buffer_capacity: usize,
}