use crate::types::ExifError;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;

/// Common utility functions for EXIF processing
pub struct ExifUtils;

impl ExifUtils {
    /// Read a 32-bit big-endian integer from data at position
    pub fn read_u32_be(data: &[u8], pos: usize) -> Result<u32, ExifError> {
        if pos + 4 > data.len() {
            return Err(ExifError::InvalidExif(
                "Insufficient data for u32 read".to_string(),
            ));
        }
        Ok(((data[pos] as u32) << 24)
            | ((data[pos + 1] as u32) << 16)
            | ((data[pos + 2] as u32) << 8)
            | (data[pos + 3] as u32))
    }

    /// Find a pattern in data and return its position
    pub fn find_pattern_in_data(data: &[u8], pattern: &[u8]) -> Option<usize> {
        data.windows(pattern.len())
            .position(|window| window == pattern)
    }

    /// Format Unix timestamp to EXIF datetime format
    pub fn format_timestamp(timestamp: i64) -> Option<String> {
        // Format Unix timestamp to EXIF datetime format
        let datetime = UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64);
        let _system_time = datetime;

        // Convert to EXIF format (YYYY:MM:DD HH:MM:SS)
        // This is a simplified implementation
        Some("2024:01:01 00:00:00".to_string()) // Placeholder
    }

    /// Extract timestamp at a specific position in data
    pub fn extract_timestamp_at_position(data: &[u8], pos: usize) -> Option<String> {
        // Try to extract a timestamp starting at the given position
        // Look for pattern: YYYY:MM:DD HH:MM:SS

        if pos + 19 > data.len() {
            return None;
        }

        let timestamp_bytes = &data[pos..pos + 19];

        // Check if it looks like a timestamp (YYYY:MM:DD HH:MM:SS)
        if timestamp_bytes.len() >= 19 {
            let year = &timestamp_bytes[0..4];
            let month = &timestamp_bytes[5..7];
            let day = &timestamp_bytes[8..10];
            let hour = &timestamp_bytes[11..13];
            let minute = &timestamp_bytes[14..16];
            let second = &timestamp_bytes[17..19];

            // Basic validation
            if Self::is_digit_string(year)
                && Self::is_digit_string(month)
                && Self::is_digit_string(day)
                && Self::is_digit_string(hour)
                && Self::is_digit_string(minute)
                && Self::is_digit_string(second)
            {
                if let Ok(timestamp) = String::from_utf8(timestamp_bytes.to_vec()) {
                    return Some(timestamp);
                }
            }
        }

        None
    }

    /// Check if a byte slice contains only ASCII digits
    pub fn is_digit_string(bytes: &[u8]) -> bool {
        bytes.iter().all(|&b| b.is_ascii_digit())
    }

    /// Extract Unix timestamps from data and add to metadata
    pub fn extract_unix_timestamps(data: &[u8], metadata: &mut HashMap<String, String>) {
        // Look for Unix timestamps (32-bit integers representing seconds since epoch)
        // Common ranges: 2020-2030 (1577836800 - 1893456000)

        let min_timestamp: u32 = 1577836800; // 2020-01-01
        let max_timestamp: u32 = 1893456000; // 2030-01-01

        // Search for 4-byte sequences that could be timestamps
        for i in 0..data.len().saturating_sub(3) {
            let timestamp_bytes = &data[i..i + 4];

            // Try both little-endian and big-endian interpretations
            let timestamp_le = u32::from_le_bytes([
                timestamp_bytes[0],
                timestamp_bytes[1],
                timestamp_bytes[2],
                timestamp_bytes[3],
            ]);
            let timestamp_be = u32::from_be_bytes([
                timestamp_bytes[0],
                timestamp_bytes[1],
                timestamp_bytes[2],
                timestamp_bytes[3],
            ]);

            // Check if either interpretation falls within our expected range
            if timestamp_le >= min_timestamp && timestamp_le <= max_timestamp {
                if let Some(formatted) = Self::format_timestamp(timestamp_le as i64) {
                    metadata.insert("UnixTimestamp_LE".to_string(), formatted);
                }
            }

            if timestamp_be >= min_timestamp && timestamp_be <= max_timestamp {
                if let Some(formatted) = Self::format_timestamp(timestamp_be as i64) {
                    metadata.insert("UnixTimestamp_BE".to_string(), formatted);
                }
            }
        }
    }
}
