use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result as SqlResult, params};

/// Persistent EXIF cache for fast-exif-rs 2.0
/// 
/// This module provides SQLite-based caching of EXIF metadata
/// to avoid re-parsing unchanged files, providing significant
/// performance improvements for repeated operations.
#[derive(Serialize, Deserialize, Debug)]
pub struct CacheEntry {
    pub file_path: String,
    pub file_size: u64,
    pub modification_time: u64,
    pub exif_data: HashMap<String, String>,
    pub cache_timestamp: u64,
}

pub struct ExifCache {
    db_path: String,
    max_cache_size: usize,
    cache_ttl: u64, // Time to live in seconds
}

impl ExifCache {
    /// Create a new EXIF cache
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self, String> {
        let cache_dir = cache_dir.as_ref();
        
        // Create cache directory if it doesn't exist
        fs::create_dir_all(cache_dir).map_err(|e| format!("Failed to create cache directory: {}", e))?;
        
        let db_path = cache_dir.join("exif_cache.db").to_string_lossy().to_string();
        
        let cache = Self {
            db_path,
            max_cache_size: 10000, // 10,000 entries
            cache_ttl: 86400, // 24 hours
        };
        
        cache.initialize_database()?;
        Ok(cache)
    }
    
    /// Create cache with custom settings
    pub fn with_settings<P: AsRef<Path>>(
        cache_dir: P,
        max_size: usize,
        ttl_seconds: u64,
    ) -> Result<Self, String> {
        let cache_dir = cache_dir.as_ref();
        fs::create_dir_all(cache_dir).map_err(|e| format!("Failed to create cache directory: {}", e))?;
        
        let db_path = cache_dir.join("exif_cache.db").to_string_lossy().to_string();
        
        let cache = Self {
            db_path,
            max_cache_size: max_size,
            cache_ttl: ttl_seconds,
        };
        
        cache.initialize_database()?;
        Ok(cache)
    }
    
    /// Initialize SQLite database
    fn initialize_database(&self) -> Result<(), String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS exif_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT UNIQUE NOT NULL,
                file_size INTEGER NOT NULL,
                modification_time INTEGER NOT NULL,
                exif_data TEXT NOT NULL,
                cache_timestamp INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| format!("Failed to create table: {}", e))?;
        
        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_path ON exif_cache(file_path)",
            [],
        ).map_err(|e| format!("Failed to create index: {}", e))?;
        
        // Create index for cleanup
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cache_timestamp ON exif_cache(cache_timestamp)",
            [],
        ).map_err(|e| format!("Failed to create timestamp index: {}", e))?;
        
        Ok(())
    }
    
    /// Get cached EXIF data for a file
    pub fn get<P: AsRef<Path>>(&self, file_path: P) -> Result<Option<HashMap<String, String>>, String> {
        let file_path = file_path.as_ref();
        let file_path_str = file_path.to_string_lossy().to_string();
        
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        // Get file metadata
        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        
        let file_size = metadata.len();
        let modification_time = metadata.modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_secs();
        
        // Query cache
        let mut stmt = conn.prepare(
            "SELECT file_size, modification_time, exif_data, cache_timestamp 
             FROM exif_cache WHERE file_path = ?"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
        
        let result: Result<(u64, u64, String, u64), rusqlite::Error> = stmt.query_row(params![file_path_str], |row| {
            let cached_size: u64 = row.get(0)?;
            let cached_mtime: u64 = row.get(1)?;
            let exif_data_json: String = row.get(2)?;
            let cache_timestamp: u64 = row.get(3)?;
            
            Ok((cached_size, cached_mtime, exif_data_json, cache_timestamp))
        });
        
        match result {
            Ok((cached_size, cached_mtime, exif_data_json, cache_timestamp)) => {
                // Check if file has been modified
                if cached_size == file_size && cached_mtime == modification_time {
                    // Check if cache entry is still valid
                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| format!("Failed to get current time: {}", e))?
                        .as_secs();
                    
                    if current_time - cache_timestamp < self.cache_ttl {
                        // Cache hit - deserialize and return
                        let exif_data: HashMap<String, String> = serde_json::from_str(&exif_data_json)
                            .map_err(|e| format!("Failed to deserialize EXIF data: {}", e))?;
                        
                        return Ok(Some(exif_data));
                    }
                }
                
                // Cache miss or expired - remove old entry
                self.remove_entry(&conn, &file_path_str)?;
                Ok(None)
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // No cache entry found
                Ok(None)
            },
            Err(e) => Err(format!("Database query failed: {}", e)),
        }
    }
    
    /// Store EXIF data in cache
    pub fn store<P: AsRef<Path>>(
        &self,
        file_path: P,
        exif_data: HashMap<String, String>,
    ) -> Result<(), String> {
        let file_path = file_path.as_ref();
        let file_path_str = file_path.to_string_lossy().to_string();
        
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        // Get file metadata
        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        
        let file_size = metadata.len();
        let modification_time = metadata.modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_secs();
        
        let cache_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get current time: {}", e))?
            .as_secs();
        
        // Serialize EXIF data
        let exif_data_json = serde_json::to_string(&exif_data)
            .map_err(|e| format!("Failed to serialize EXIF data: {}", e))?;
        
        // Insert or update cache entry
        conn.execute(
            "INSERT OR REPLACE INTO exif_cache 
             (file_path, file_size, modification_time, exif_data, cache_timestamp)
             VALUES (?, ?, ?, ?, ?)",
            params![file_path_str, file_size, modification_time, exif_data_json, cache_timestamp],
        ).map_err(|e| format!("Failed to insert cache entry: {}", e))?;
        
        // Cleanup old entries if cache is too large
        self.cleanup_if_needed(&conn)?;
        
        Ok(())
    }
    
    /// Remove a specific cache entry
    pub fn remove<P: AsRef<Path>>(&self, file_path: P) -> Result<(), String> {
        let file_path_str = file_path.as_ref().to_string_lossy().to_string();
        
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        self.remove_entry(&conn, &file_path_str)
    }
    
    /// Remove cache entry from database
    fn remove_entry(&self, conn: &Connection, file_path: &str) -> Result<(), String> {
        conn.execute(
            "DELETE FROM exif_cache WHERE file_path = ?",
            params![file_path],
        ).map_err(|e| format!("Failed to delete cache entry: {}", e))?;
        
        Ok(())
    }
    
    /// Clean up old cache entries if needed
    fn cleanup_if_needed(&self, conn: &Connection) -> Result<(), String> {
        // Count total entries
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM exif_cache",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count cache entries: {}", e))?;
        
        if count as usize > self.max_cache_size {
            // Remove oldest entries
            let entries_to_remove = (count as usize) - self.max_cache_size;
            
            conn.execute(
                "DELETE FROM exif_cache WHERE id IN (
                    SELECT id FROM exif_cache 
                    ORDER BY cache_timestamp ASC 
                    LIMIT ?
                )",
                params![entries_to_remove],
            ).map_err(|e| format!("Failed to cleanup cache entries: {}", e))?;
        }
        
        // Remove expired entries
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get current time: {}", e))?
            .as_secs();
        
        conn.execute(
            "DELETE FROM exif_cache WHERE cache_timestamp < ?",
            params![current_time - self.cache_ttl],
        ).map_err(|e| format!("Failed to remove expired entries: {}", e))?;
        
        Ok(())
    }
    
    /// Clear all cache entries
    pub fn clear(&self) -> Result<(), String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        conn.execute("DELETE FROM exif_cache", [])
            .map_err(|e| format!("Failed to clear cache: {}", e))?;
        
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> Result<CacheStats, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        let total_entries: i64 = conn.query_row(
            "SELECT COUNT(*) FROM exif_cache",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count entries: {}", e))?;
        
        let total_size: i64 = conn.query_row(
            "SELECT SUM(LENGTH(exif_data)) FROM exif_cache",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to calculate total size: {}", e))?;
        
        let oldest_entry: Option<i64> = conn.query_row(
            "SELECT MIN(cache_timestamp) FROM exif_cache",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to get oldest entry: {}", e))?;
        
        let newest_entry: Option<i64> = conn.query_row(
            "SELECT MAX(cache_timestamp) FROM exif_cache",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to get newest entry: {}", e))?;
        
        Ok(CacheStats {
            total_entries: total_entries as usize,
            total_size_bytes: total_size as usize,
            oldest_entry_timestamp: oldest_entry,
            newest_entry_timestamp: newest_entry,
            max_cache_size: self.max_cache_size,
            cache_ttl_seconds: self.cache_ttl,
        })
    }
    
    /// Check if a file is cached and valid
    pub fn is_cached<P: AsRef<Path>>(&self, file_path: P) -> Result<bool, String> {
        match self.get(file_path)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub oldest_entry_timestamp: Option<i64>,
    pub newest_entry_timestamp: Option<i64>,
    pub max_cache_size: usize,
    pub cache_ttl_seconds: u64,
}

impl CacheStats {
    /// Get cache hit rate (requires external tracking)
    pub fn get_hit_rate(&self, hits: usize, misses: usize) -> f64 {
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
    
    /// Get cache size in MB
    pub fn get_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }
    
    /// Check if cache is full
    pub fn is_full(&self) -> bool {
        self.total_entries >= self.max_cache_size
    }
}
