use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Which expensive EXIF subtrees to walk.
///
/// Standard IFD0 / ExifIFD tags are always parsed. GPS, Interop, and maker
/// notes are optional because they dominate parse time on camera files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseScope {
    pub maker_notes: bool,
    pub gps: bool,
    pub interop: bool,
}

impl ParseScope {
    pub fn all() -> Self {
        Self {
            maker_notes: true,
            gps: true,
            interop: true,
        }
    }

    pub fn standard_exif() -> Self {
        Self {
            maker_notes: false,
            gps: true,
            interop: true,
        }
    }

    pub fn core() -> Self {
        Self {
            maker_notes: false,
            gps: false,
            interop: false,
        }
    }
}

impl Default for ParseScope {
    fn default() -> Self {
        Self::all()
    }
}

/// Options for a metadata read.
///
/// Use [`ReadOptions::tags`] when you only need a handful of fields (for
/// example date/time). That skips maker notes and GPS unless those tags were
/// requested, and keeps only the named fields after parse.
#[derive(Debug, Clone)]
pub struct ReadOptions {
    pub include_maker_notes: bool,
    pub include_gps: bool,
    pub include_interop: bool,
    pub include_computed_fields: bool,
    pub include_file_system: bool,
    /// If set, only these tag names are returned (ExifTool-style names).
    pub wanted_tags: Option<Vec<String>>,
    /// Read the EXIF segment with seeking instead of mapping the whole file.
    pub exif_segment_only: bool,
}

impl ReadOptions {
    pub fn full() -> Self {
        Self {
            include_maker_notes: true,
            include_gps: true,
            include_interop: true,
            include_computed_fields: true,
            include_file_system: true,
            wanted_tags: None,
            exif_segment_only: false,
        }
    }

    /// Standard TIFF/EXIF/GPS tags, no manufacturer maker notes.
    pub fn standard() -> Self {
        Self {
            include_maker_notes: false,
            include_gps: true,
            include_interop: true,
            include_computed_fields: true,
            include_file_system: true,
            wanted_tags: None,
            exif_segment_only: true,
        }
    }

    /// Date and time fields only.
    pub fn datetime() -> Self {
        let mut opts = Self::tags([
            "DateTimeOriginal",
            "CreateDate",
            "DateTime",
            "ModifyDate",
            "DateTimeDigitized",
            "SubSecTime",
            "SubSecTimeOriginal",
            "SubSecTimeDigitized",
            "OffsetTime",
            "OffsetTimeOriginal",
            "OffsetTimeDigitized",
            "SubSecCreateDate",
            "SubSecDateTimeOriginal",
            "SubSecModifyDate",
        ]);
        // Composite SubSec* tags are built after parse.
        opts.include_computed_fields = true;
        opts
    }

    /// Keep only `tags`. Maker notes and GPS are enabled only if a requested
    /// name looks like it lives in those IFDs.
    pub fn tags<I, S>(tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let wanted: Vec<String> = tags.into_iter().map(Into::into).collect();
        let include_gps = wanted.iter().any(|t| {
            t.len() >= 3 && t[..3].eq_ignore_ascii_case("gps")
        });
        let include_maker_notes = wanted.iter().any(|t| looks_like_maker_note_tag(t));
        Self {
            include_maker_notes,
            include_gps,
            include_interop: false,
            include_computed_fields: false,
            include_file_system: false,
            wanted_tags: Some(wanted),
            exif_segment_only: true,
        }
    }

    pub fn parse_scope(&self) -> ParseScope {
        ParseScope {
            maker_notes: self.include_maker_notes,
            gps: self.include_gps,
            interop: self.include_interop,
        }
    }
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self::full()
    }
}

pub(crate) fn looks_like_maker_note_tag(tag: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "MakerNote",
        "Canon",
        "Nikon",
        "Olympus",
        "Sony",
        "Samsung",
        "Ricoh",
        "Fujifilm",
        "Pentax",
        "Panasonic",
        "Minolta",
        "Sigma",
        "Leica",
    ];
    PREFIXES.iter().any(|p| tag.starts_with(p))
}

pub(crate) fn tag_is_wanted(tag: &str, wanted: &[String]) -> bool {
    let wanted_n = wanted_normalized_set(wanted);
    tag_is_wanted_normalized(tag, &wanted_n)
}

pub(crate) fn wanted_normalized_set(wanted: &[String]) -> HashSet<String> {
    let mut set = HashSet::with_capacity(wanted.len() * 3);
    for w in wanted {
        set.insert(normalize_tag(w));
        for alias in aliases_of(w) {
            set.insert(normalize_tag(alias));
        }
    }
    set
}

pub(crate) fn tag_is_wanted_normalized(tag: &str, wanted: &HashSet<String>) -> bool {
    wanted.contains(&normalize_tag(tag))
}

pub(crate) fn normalize_tag(tag: &str) -> String {
    let base = tag.rsplit_once(':').map(|(_, rest)| rest).unwrap_or(tag);
    base.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn aliases_of(tag: &str) -> &'static [&'static str] {
    match normalize_tag(tag).as_str() {
        "datetime" | "modifydate" | "subsecmodifydate" => &[
            "DateTime",
            "ModifyDate",
            "SubSecTime",
            "OffsetTime",
            "SubSecModifyDate",
        ],
        "datetimeoriginal" | "subsecdatetimeoriginal" => &[
            "DateTimeOriginal",
            "SubSecTimeOriginal",
            "OffsetTimeOriginal",
            "OffsetTime",
            "SubSecDateTimeOriginal",
        ],
        "createdate" | "datetimedigitized" | "datetimecreated" | "subseccreatedate" => &[
            "CreateDate",
            "DateTimeDigitized",
            "DateTimeCreated",
            "DateTimeOriginal",
            "SubSecTimeDigitized",
            "SubSecTime",
            "OffsetTimeDigitized",
            "OffsetTime",
            "SubSecCreateDate",
        ],
        "iso" | "isospeedratings" | "isospeed" => &["ISO", "ISOSpeedRatings", "ISOSpeed"],
        _ => &[],
    }
}

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


/// Result structure for multiprocessing operations
#[derive(Debug, Clone)]
pub struct ExifResult {
    pub file_path: String,
    pub metadata: HashMap<String, String>,
    pub processing_time: f64,
    pub success: bool,
    pub error: Option<String>,
}

/// Statistics for multiprocessing operations
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub total_files: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub success_rate: f64,
    pub total_time: f64,
    pub avg_processing_time: f64,
    pub files_per_second: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datetime_options_skip_maker_notes() {
        let opts = ReadOptions::datetime();
        assert!(!opts.include_maker_notes);
        assert!(!opts.include_gps);
        assert!(opts.exif_segment_only);
        assert!(opts.include_computed_fields);
        let wanted = opts.wanted_tags.unwrap();
        assert!(wanted.contains(&"DateTimeOriginal".to_string()));
        assert!(wanted.contains(&"SubSecCreateDate".to_string()));
    }

    #[test]
    fn gps_tag_request_enables_gps_ifd() {
        let opts = ReadOptions::tags(["GPSLatitude"]);
        assert!(opts.include_gps);
        assert!(!opts.include_maker_notes);
    }
}
