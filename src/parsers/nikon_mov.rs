//! Nikon QuickTime metadata (`NCTG` inside `udta/NCDT`).
//!
//! Z-series cameras store exposure, lens, and maker-note tags in this
//! big-endian record list instead of a TIFF EXIF IFD. Record layout matches
//! ExifTool `ProcessNikonMOV`: u32 tag, u16 TIFF format, u16 count, then
//! `count * format_size` bytes.

use crate::types::ExifError;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

const QT_EPOCH_OFFSET: i64 = 2_082_844_800;

const XLAT0: [u8; 256] = [
    0xc1, 0xbf, 0x6d, 0x0d, 0x59, 0xc5, 0x13, 0x9d, 0x83, 0x61, 0x6b, 0x4f, 0xc7, 0x7f, 0x3d, 0x3d,
    0x53, 0x59, 0xe3, 0xc7, 0xe9, 0x2f, 0x95, 0xa7, 0x95, 0x1f, 0xdf, 0x7f, 0x2b, 0x29, 0xc7, 0x0d,
    0xdf, 0x07, 0xef, 0x71, 0x89, 0x3d, 0x13, 0x3d, 0x3b, 0x13, 0xfb, 0x0d, 0x89, 0xc1, 0x65, 0x1f,
    0xb3, 0x0d, 0x6b, 0x29, 0xe3, 0xfb, 0xef, 0xa3, 0x6b, 0x47, 0x7f, 0x95, 0x35, 0xa7, 0x47, 0x4f,
    0xc7, 0xf1, 0x59, 0x95, 0x35, 0x11, 0x29, 0x61, 0xf1, 0x3d, 0xb3, 0x2b, 0x0d, 0x43, 0x89, 0xc1,
    0x9d, 0x9d, 0x89, 0x65, 0xf1, 0xe9, 0xdf, 0xbf, 0x3d, 0x7f, 0x53, 0x97, 0xe5, 0xe9, 0x95, 0x17,
    0x1d, 0x3d, 0x8b, 0xfb, 0xc7, 0xe3, 0x67, 0xa7, 0x07, 0xf1, 0x71, 0xa7, 0x53, 0xb5, 0x29, 0x89,
    0xe5, 0x2b, 0xa7, 0x17, 0x29, 0xe9, 0x4f, 0xc5, 0x65, 0x6d, 0x6b, 0xef, 0x0d, 0x89, 0x49, 0x2f,
    0xb3, 0x43, 0x53, 0x65, 0x1d, 0x49, 0xa3, 0x13, 0x89, 0x59, 0xef, 0x6b, 0xef, 0x65, 0x1d, 0x0b,
    0x59, 0x13, 0xe3, 0x4f, 0x9d, 0xb3, 0x29, 0x43, 0x2b, 0x07, 0x1d, 0x95, 0x59, 0x59, 0x47, 0xfb,
    0xe5, 0xe9, 0x61, 0x47, 0x2f, 0x35, 0x7f, 0x17, 0x7f, 0xef, 0x7f, 0x95, 0x95, 0x71, 0xd3, 0xa3,
    0x0b, 0x71, 0xa3, 0xad, 0x0b, 0x3b, 0xb5, 0xfb, 0xa3, 0xbf, 0x4f, 0x83, 0x1d, 0xad, 0xe9, 0x2f,
    0x71, 0x65, 0xa3, 0xe5, 0x07, 0x35, 0x3d, 0x0d, 0xb5, 0xe9, 0xe5, 0x47, 0x3b, 0x9d, 0xef, 0x35,
    0xa3, 0xbf, 0xb3, 0xdf, 0x53, 0xd3, 0x97, 0x53, 0x49, 0x71, 0x07, 0x35, 0x61, 0x71, 0x2f, 0x43,
    0x2f, 0x11, 0xdf, 0x17, 0x97, 0xfb, 0x95, 0x3b, 0x7f, 0x6b, 0xd3, 0x25, 0xbf, 0xad, 0xc7, 0xc5,
    0xc5, 0xb5, 0x8b, 0xef, 0x2f, 0xd3, 0x07, 0x6b, 0x25, 0x49, 0x95, 0x25, 0x49, 0x6d, 0x71, 0xc7,
];

const XLAT1: [u8; 256] = [
    0xa7, 0xbc, 0xc9, 0xad, 0x91, 0xdf, 0x85, 0xe5, 0xd4, 0x78, 0xd5, 0x17, 0x46, 0x7c, 0x29, 0x4c,
    0x4d, 0x03, 0xe9, 0x25, 0x68, 0x11, 0x86, 0xb3, 0xbd, 0xf7, 0x6f, 0x61, 0x22, 0xa2, 0x26, 0x34,
    0x2a, 0xbe, 0x1e, 0x46, 0x14, 0x68, 0x9d, 0x44, 0x18, 0xc2, 0x40, 0xf4, 0x7e, 0x5f, 0x1b, 0xad,
    0x0b, 0x94, 0xb6, 0x67, 0xb4, 0x0b, 0xe1, 0xea, 0x95, 0x9c, 0x66, 0xdc, 0xe7, 0x5d, 0x6c, 0x05,
    0xda, 0xd5, 0xdf, 0x7a, 0xef, 0xf6, 0xdb, 0x1f, 0x82, 0x4c, 0xc0, 0x68, 0x47, 0xa1, 0xbd, 0xee,
    0x39, 0x50, 0x56, 0x4a, 0xdd, 0xdf, 0xa5, 0xf8, 0xc6, 0xda, 0xca, 0x90, 0xca, 0x01, 0x42, 0x9d,
    0x8b, 0x0c, 0x73, 0x43, 0x75, 0x05, 0x94, 0xde, 0x24, 0xb3, 0x80, 0x34, 0xe5, 0x2c, 0xdc, 0x9b,
    0x3f, 0xca, 0x33, 0x45, 0xd0, 0xdb, 0x5f, 0xf5, 0x52, 0xc3, 0x21, 0xda, 0xe2, 0x22, 0x72, 0x6b,
    0x3e, 0xd0, 0x5b, 0xa8, 0x87, 0x8c, 0x06, 0x5d, 0x0f, 0xdd, 0x09, 0x19, 0x93, 0xd0, 0xb9, 0xfc,
    0x8b, 0x0f, 0x84, 0x60, 0x33, 0x1c, 0x9b, 0x45, 0xf1, 0xf0, 0xa3, 0x94, 0x3a, 0x12, 0x77, 0x33,
    0x4d, 0x44, 0x78, 0x28, 0x3c, 0x9e, 0xfd, 0x65, 0x57, 0x16, 0x94, 0x6b, 0xfb, 0x59, 0xd0, 0xc8,
    0x22, 0x36, 0xdb, 0xd2, 0x63, 0x98, 0x43, 0xa1, 0x04, 0x87, 0x86, 0xf7, 0xa6, 0x26, 0xbb, 0xd6,
    0x59, 0x4d, 0xbf, 0x6a, 0x2e, 0xaa, 0x2b, 0xef, 0xe6, 0x78, 0xb6, 0x4e, 0xe0, 0x2f, 0xdc, 0x7c,
    0xbe, 0x57, 0x19, 0x32, 0x7e, 0x2a, 0xd0, 0xb8, 0xba, 0x29, 0x00, 0x3c, 0x52, 0x7d, 0xa8, 0x49,
    0x3b, 0x2d, 0xeb, 0x25, 0x49, 0xfa, 0xa3, 0xaa, 0x39, 0xa7, 0xc5, 0xa7, 0x50, 0x11, 0x36, 0xfb,
    0xc6, 0x67, 0x4a, 0xf5, 0xa5, 0x12, 0x65, 0x7e, 0xb0, 0xdf, 0xaf, 0x4e, 0xb3, 0x61, 0x7f, 0x2f,
];

struct NctgPending {
    serial: Option<u32>,
    shutter_count: Option<u32>,
    lens_data: Option<Vec<u8>>,
}

/// Walk QuickTime atoms in an in-memory buffer (a whole file, or `ftyp`+`moov`).
pub fn walk_atoms(data: &[u8], metadata: &mut HashMap<String, String>) {
    walk_slice(data, 0, data.len(), metadata, 0);
}

/// Read a QuickTime file without loading `mdat`.
///
/// Returns `(media_data_offset, media_data_size)` when an `mdat` atom is found.
/// The offset points at the media payload, past the atom header.
pub fn scan_file<R: Read + Seek>(
    file: &mut R,
    file_len: u64,
    metadata: &mut HashMap<String, String>,
) -> Result<Option<(u64, u64)>, ExifError> {
    let mut mdat = None;
    scan_range(file, 0, file_len, metadata, &mut mdat, 0)?;
    Ok(mdat)
}

fn scan_range<R: Read + Seek>(
    file: &mut R,
    start: u64,
    end: u64,
    metadata: &mut HashMap<String, String>,
    mdat: &mut Option<(u64, u64)>,
    depth: u32,
) -> Result<(), ExifError> {
    if depth > 12 || start >= end {
        return Ok(());
    }
    let mut pos = start;
    while pos + 8 <= end {
        file.seek(SeekFrom::Start(pos))?;
        let mut hdr = [0u8; 16];
        let n = file.read(&mut hdr)?;
        if n < 8 {
            break;
        }
        let (size, header_len) = atom_size(&hdr, n, end.saturating_sub(pos))?;
        let Some(atom_end) = pos.checked_add(size) else {
            break;
        };
        if size < header_len || atom_end > end {
            break;
        }
        let typ = [hdr[4], hdr[5], hdr[6], hdr[7]];
        let payload = pos + header_len;
        let payload_len = size - header_len;
        if typ == *b"mdat" {
            *mdat = Some((payload, payload_len));
        } else if is_container(&typ) {
            let child = if typ == *b"meta" {
                payload.saturating_add(4).min(atom_end)
            } else {
                payload
            };
            scan_range(file, child, atom_end, metadata, mdat, depth + 1)?;
        } else if payload_len > 0 && payload_len <= 1_048_576 && is_leaf(&typ) {
            let mut buf = vec![0u8; payload_len as usize];
            file.seek(SeekFrom::Start(payload))?;
            file.read_exact(&mut buf)?;
            handle_atom(&typ, &buf, metadata);
        }
        pos = atom_end;
    }
    Ok(())
}

fn atom_size(hdr: &[u8], n: usize, remaining: u64) -> Result<(u64, u64), ExifError> {
    let size32 = u32::from_be_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]);
    if size32 == 1 {
        if n < 16 {
            return Err(ExifError::InvalidExif(
                "Truncated extended QuickTime atom".to_string(),
            ));
        }
        let size = u64::from_be_bytes(hdr[8..16].try_into().unwrap());
        Ok((size, 16))
    } else if size32 == 0 {
        Ok((remaining, 8))
    } else {
        Ok((u64::from(size32), 8))
    }
}

fn walk_slice(
    data: &[u8],
    start: usize,
    end: usize,
    metadata: &mut HashMap<String, String>,
    depth: u32,
) {
    if depth > 12 || start >= end {
        return;
    }
    let end = end.min(data.len());
    let mut pos = start;
    while pos + 8 <= end {
        let size32 = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let (size, header_len) = if size32 == 1 && pos + 16 <= end {
            let size = u64::from_be_bytes(data[pos + 8..pos + 16].try_into().unwrap());
            (size, 16usize)
        } else if size32 == 0 {
            ((end - pos) as u64, 8usize)
        } else {
            (u64::from(size32), 8usize)
        };
        if size < header_len as u64 {
            break;
        }
        let size = size as usize;
        if pos + size > end {
            break;
        }
        let typ = [data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]];
        let payload_at = pos + header_len;
        let payload_end = pos + size;
        if typ == *b"mdat" {
            metadata.insert("MediaDataOffset".to_string(), payload_at.to_string());
            metadata.insert("MediaDataSize".to_string(), (size - header_len).to_string());
        } else if is_container(&typ) {
            let child = if typ == *b"meta" {
                payload_at.saturating_add(4).min(payload_end)
            } else {
                payload_at
            };
            walk_slice(data, child, payload_end, metadata, depth + 1);
        } else if is_leaf(&typ) {
            handle_atom(&typ, &data[payload_at..payload_end], metadata);
        }
        pos += size;
    }
}

fn is_container(typ: &[u8; 4]) -> bool {
    matches!(
        typ,
        b"moov"
            | b"trak"
            | b"mdia"
            | b"minf"
            | b"stbl"
            | b"udta"
            | b"edts"
            | b"dinf"
            | b"NCDT"
            | b"meta"
    )
}

fn is_leaf(typ: &[u8; 4]) -> bool {
    matches!(
        typ,
        b"ftyp" | b"mvhd" | b"tkhd" | b"mdhd" | b"stsd" | b"NCTG"
    )
}

fn handle_atom(typ: &[u8; 4], payload: &[u8], metadata: &mut HashMap<String, String>) {
    match typ {
        b"ftyp" => parse_ftyp(payload, metadata),
        b"mvhd" => parse_mvhd(payload, metadata),
        b"tkhd" => parse_tkhd(payload, metadata),
        b"mdhd" => parse_mdhd(payload, metadata),
        b"stsd" => parse_stsd(payload, metadata),
        b"NCTG" => parse_nctg(payload, metadata),
        _ => {}
    }
}

fn parse_ftyp(payload: &[u8], metadata: &mut HashMap<String, String>) {
    if payload.len() < 8 {
        return;
    }
    let brand = &payload[0..4];
    metadata.insert("MajorBrand".to_string(), major_brand_name(brand));
    let minor = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
    metadata.insert("MinorVersion".to_string(), format_minor_version(minor));
    let mut brands = Vec::new();
    let mut i = 8;
    while i + 4 <= payload.len() {
        brands.push(String::from_utf8_lossy(&payload[i..i + 4]).into_owned());
        i += 4;
    }
    if !brands.is_empty() {
        metadata.insert("CompatibleBrands".to_string(), brands.join(", "));
    }
}

fn major_brand_name(brand: &[u8]) -> String {
    match brand {
        b"qt  " => "Apple QuickTime (.MOV/QT)".to_string(),
        b"mp41" | b"mp42" => "MP4 Base Media v1".to_string(),
        b"isom" => "MP4 Base Media v1".to_string(),
        _ => latin1_trim(brand),
    }
}

fn format_minor_version(minor: u32) -> String {
    let hex = format!("{minor:08x}");
    let year = &hex[0..4];
    let month = hex[4..6].parse::<u32>().unwrap_or(0);
    let rev = hex[6..8].parse::<u32>().unwrap_or(0);
    format!("{year}.{month}.{rev}")
}

fn parse_mvhd(payload: &[u8], metadata: &mut HashMap<String, String>) {
    if payload.len() < 20 {
        return;
    }
    let version = payload[0];
    if version == 0 && payload.len() >= 20 {
        let create = u32::from_be_bytes(payload[4..8].try_into().unwrap());
        let modify = u32::from_be_bytes(payload[8..12].try_into().unwrap());
        let scale = u32::from_be_bytes(payload[12..16].try_into().unwrap());
        let duration = u32::from_be_bytes(payload[16..20].try_into().unwrap());
        insert_qt_date(metadata, "ModifyDate", modify);
        // Movie header creation is UTC. Nikon NCTG CreateDate (local) overwrites
        // this later when it is present.
        if !metadata.contains_key("CreateDate") {
            insert_qt_date(metadata, "CreateDate", create);
        }
        if scale > 0 {
            metadata.insert("TimeScale".to_string(), scale.to_string());
            let secs = duration as f64 / scale as f64;
            metadata.insert("Duration".to_string(), format!("{secs:.2} s"));
            metadata.insert("MediaDuration".to_string(), format!("{secs:.2} s"));
            metadata.insert("TrackDuration".to_string(), format!("{secs:.2} s"));
        }
    }
}

fn parse_tkhd(payload: &[u8], metadata: &mut HashMap<String, String>) {
    if payload.is_empty() || payload[0] != 0 || payload.len() < 84 {
        return;
    }
    let create = u32::from_be_bytes(payload[4..8].try_into().unwrap());
    let modify = u32::from_be_bytes(payload[8..12].try_into().unwrap());
    insert_qt_date(metadata, "TrackCreateDate", create);
    insert_qt_date(metadata, "TrackModifyDate", modify);
    let width = fixed_16_16(u32::from_be_bytes(payload[76..80].try_into().unwrap()));
    let height = fixed_16_16(u32::from_be_bytes(payload[80..84].try_into().unwrap()));
    if width > 0 && height > 0 {
        metadata.insert("ImageWidth".to_string(), width.to_string());
        metadata.insert("ImageHeight".to_string(), height.to_string());
        metadata.insert("ImageSize".to_string(), format!("{width}x{height}"));
        metadata.insert("SourceImageWidth".to_string(), width.to_string());
        metadata.insert("SourceImageHeight".to_string(), height.to_string());
    }
}

fn parse_mdhd(payload: &[u8], metadata: &mut HashMap<String, String>) {
    if payload.is_empty() || payload[0] != 0 || payload.len() < 20 {
        return;
    }
    let create = u32::from_be_bytes(payload[4..8].try_into().unwrap());
    let modify = u32::from_be_bytes(payload[8..12].try_into().unwrap());
    let scale = u32::from_be_bytes(payload[12..16].try_into().unwrap());
    insert_qt_date(metadata, "MediaCreateDate", create);
    insert_qt_date(metadata, "MediaModifyDate", modify);
    if scale > 0 {
        metadata.insert("MediaTimeScale".to_string(), scale.to_string());
    }
}

fn parse_stsd(payload: &[u8], metadata: &mut HashMap<String, String>) {
    if payload.len() < 8 {
        return;
    }
    let count = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let mut pos = 8usize;
    for _ in 0..count {
        if pos + 8 > payload.len() {
            break;
        }
        let size = u32::from_be_bytes(payload[pos..pos + 4].try_into().unwrap()) as usize;
        if size < 8 || pos + size > payload.len() {
            break;
        }
        let codec = latin1_trim(&payload[pos + 4..pos + 8]);
        if !codec.is_empty() && codec.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
            metadata.insert("CompressorID".to_string(), codec.clone());
            metadata.insert("VideoCodec".to_string(), codec);
        }
        if pos + 36 <= pos + size {
            let width = u16::from_be_bytes([payload[pos + 32], payload[pos + 33]]);
            let height = u16::from_be_bytes([payload[pos + 34], payload[pos + 35]]);
            if width > 0 && height > 0 && !metadata.contains_key("ImageWidth") {
                metadata.insert("ImageWidth".to_string(), width.to_string());
                metadata.insert("ImageHeight".to_string(), height.to_string());
                metadata.insert("ImageSize".to_string(), format!("{width}x{height}"));
            }
        }
        if pos + 86 <= pos + size {
            let depth = u16::from_be_bytes([payload[pos + 82], payload[pos + 83]]);
            if depth > 0 && depth < 128 {
                metadata.insert("BitDepth".to_string(), depth.to_string());
            }
            let hres = u32::from_be_bytes(payload[pos + 36..pos + 40].try_into().unwrap());
            let vres = u32::from_be_bytes(payload[pos + 40..pos + 44].try_into().unwrap());
            let x_res = fixed_16_16(hres);
            let y_res = fixed_16_16(vres);
            if x_res > 0 {
                metadata.insert("XResolution".to_string(), x_res.to_string());
            }
            if y_res > 0 {
                metadata.insert("YResolution".to_string(), y_res.to_string());
            }
        }
        if let Some(colr) = find_typed_box(&payload[pos..pos + size], b"colr") {
            parse_colr(colr, metadata);
        }
        pos += size;
    }
}

fn find_typed_box<'a>(data: &'a [u8], typ: &[u8; 4]) -> Option<&'a [u8]> {
    // Sample entries mix a fixed header with nested boxes, so scan for the
    // type instead of treating the whole entry as one atom.
    let mut pos = 0;
    while pos + 8 <= data.len() {
        if &data[pos + 4..pos + 8] == typ {
            let size = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            if size >= 8 && pos + size <= data.len() {
                return Some(&data[pos + 8..pos + size]);
            }
        }
        pos += 1;
    }
    None
}

fn parse_colr(payload: &[u8], metadata: &mut HashMap<String, String>) {
    if payload.len() < 10 || &payload[0..4] != b"nclc" {
        return;
    }
    metadata.insert("ColorProfiles".to_string(), "nclc".to_string());
    let primaries = u16::from_be_bytes([payload[4], payload[5]]);
    let transfer = u16::from_be_bytes([payload[6], payload[7]]);
    let matrix = u16::from_be_bytes([payload[8], payload[9]]);
    metadata.insert(
        "ColorPrimaries".to_string(),
        color_param_name(primaries).to_string(),
    );
    metadata.insert(
        "TransferCharacteristics".to_string(),
        color_param_name(transfer).to_string(),
    );
    metadata.insert(
        "MatrixCoefficients".to_string(),
        color_param_name(matrix).to_string(),
    );
}

fn color_param_name(v: u16) -> &'static str {
    match v {
        1 => "BT.709",
        6 => "SMPTE 170M",
        _ => "Unknown",
    }
}

fn parse_nctg(data: &[u8], metadata: &mut HashMap<String, String>) {
    metadata.insert(
        "ExifByteOrder".to_string(),
        "Big-endian (Motorola, MM)".to_string(),
    );
    let mut pending = NctgPending {
        serial: None,
        shutter_count: None,
        lens_data: None,
    };
    let mut pos = 0;
    let mut records = 0;
    while pos + 8 <= data.len() && records < 400 {
        let tag = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap());
        let fmt = u16::from_be_bytes([data[pos + 4], data[pos + 5]]);
        let count = u16::from_be_bytes([data[pos + 6], data[pos + 7]]) as usize;
        let Some(unit) = format_size(fmt) else {
            break;
        };
        let size = count.saturating_mul(unit);
        pos += 8;
        if size > data.len() - pos {
            break;
        }
        let raw = &data[pos..pos + size];
        dispatch_nctg(tag, fmt, raw, metadata, &mut pending);
        pos += size;
        records += 1;
    }
    if let Some(lens) = pending.lens_data {
        parse_lens_data(&lens, pending.serial, pending.shutter_count, metadata);
    }
}

fn dispatch_nctg(
    tag: u32,
    fmt: u16,
    raw: &[u8],
    metadata: &mut HashMap<String, String>,
    pending: &mut NctgPending,
) {
    match tag {
        0x0001 => put(metadata, "Make", ascii(raw)),
        0x0002 => put(metadata, "Model", ascii(raw)),
        0x0003 => put(metadata, "Software", ascii(raw)),
        0x0011 => put(metadata, "CreateDate", ascii(raw)),
        0x0012 => put(metadata, "DateTimeOriginal", ascii(raw)),
        0x0013 => {
            if let Some(v) = be_u32s(raw).first().copied() {
                put(metadata, "FrameCount", v.to_string());
            }
        }
        0x0016 => {
            if let Some((n, d)) = first_rational(raw) {
                let text = format_frame_rate(n, d);
                put(metadata, "FrameRate", text.clone());
                put(metadata, "VideoFrameRate", text);
            }
        }
        0x0019 => put(metadata, "TimeZone", ascii(raw)),
        0x0022 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "FrameWidth", v.to_string());
            }
        }
        0x0023 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "FrameHeight", v.to_string());
            }
        }
        0x0032 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "AudioChannels", v.to_string());
            }
        }
        0x0033 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "AudioBitsPerSample", v.to_string());
            }
        }
        0x0034 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "AudioSampleRate", v.to_string());
            }
        }
        0x1002 => put(metadata, "NikonDateTime", ascii(raw)),
        0x1013 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "ElectronicVR", on_off(v));
            }
        }
        0x0110_829a => {
            if let Some((n, d)) = first_rational(raw) {
                let text = reduce_fraction(n, d);
                put(metadata, "ExposureTime", text.clone());
                put(metadata, "ShutterSpeed", text);
            }
        }
        0x0110_829d => {
            if let Some((n, d)) = first_rational(raw) {
                let text = format_fnumber(n, d);
                put(metadata, "FNumber", text.clone());
                put(metadata, "Aperture", text);
            }
        }
        0x0110_8822 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "ExposureProgram", exposure_program(v));
            }
        }
        0x0110_8832 => {
            if let Some(v) = be_u32s(raw).first().copied() {
                put(metadata, "ISO", v.to_string());
            }
        }
        0x0110_9204 => {
            if let Some((n, d)) = first_srational(raw) {
                put(metadata, "ExposureCompensation", format_ev(n, d));
            }
        }
        0x0110_9207 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "MeteringMode", metering_mode(v));
            }
        }
        0x0110_920a => {
            if let Some((n, d)) = first_rational(raw) {
                put(metadata, "FocalLength", format_focal(n, d));
            }
        }
        0x0110_a431 => {
            let text = ascii(raw);
            put(metadata, "SerialNumber", text.clone());
            pending.serial = text.parse().ok();
        }
        0x0110_a432 => put(metadata, "LensInfo", format_lens_info(raw)),
        0x0110_a433 => put(metadata, "LensMake", ascii(raw)),
        0x0110_a434 => put(metadata, "LensModel", ascii(raw)),
        0x0110_a435 => put(metadata, "LensSerialNumber", ascii(raw)),
        0x0200_0001 => put(metadata, "MakerNoteVersion", maker_note_version(raw)),
        0x0200_0005 => put(metadata, "WhiteBalance", ascii(raw)),
        0x0200_0007 => {
            let mode = ascii(raw);
            let auto = if mode.to_ascii_lowercase().starts_with("manual") {
                "Off"
            } else {
                "On"
            };
            put(metadata, "FocusMode", mode);
            put(metadata, "AutoFocus", auto.to_string());
        }
        0x0200_001b => put(metadata, "CropHiSpeed", format_crop(raw)),
        0x0200_001f => parse_vr_info(raw, metadata),
        0x0200_0022 => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "ActiveD-Lighting", active_d_lighting(v));
            }
        }
        0x0200_0023 => parse_picture_control(raw, metadata),
        0x0200_0024 => parse_world_time(raw, metadata),
        0x0200_0025 => parse_iso_info(raw, metadata),
        0x0200_002a => {
            if let Some(v) = be_u16s(raw).first().copied() {
                put(metadata, "VignetteControl", vignette_control(v));
            }
        }
        0x0200_003f => put(metadata, "WhiteBalanceFineTune", format_fine_tune(raw)),
        0x0200_0083 => {
            if let Some(v) = raw.first().copied() {
                put(metadata, "LensType", lens_type(v));
            }
        }
        0x0200_0084 => put(metadata, "Lens", format_lens_info(raw)),
        0x0200_0087 => {
            if let Some(v) = raw.first().copied() {
                put(metadata, "FlashMode", flash_mode(v));
            }
        }
        0x0200_0098 => pending.lens_data = Some(raw.to_vec()),
        0x0200_00a7 => {
            if let Some(v) = be_u32s(raw).first().copied() {
                put(metadata, "ShutterCount", v.to_string());
                pending.shutter_count = Some(v);
            }
        }
        0x0200_00ab => put(metadata, "VariProgram", ascii(raw)),
        0x0200_00b1 => {
            if fmt == 3 {
                if let Some(v) = be_u16s(raw).first().copied() {
                    put(metadata, "HighISONoiseReduction", high_iso_nr(v));
                }
            }
        }
        0x0200_00b7 => parse_af_info2(raw, metadata),
        _ => {}
    }
}

fn parse_vr_info(raw: &[u8], metadata: &mut HashMap<String, String>) {
    if raw.len() < 9 {
        return;
    }
    put(metadata, "VRInfoVersion", ascii(&raw[0..4]));
    put(
        metadata,
        "VibrationReduction",
        match raw[4] {
            0 => "n/a".to_string(),
            1 => "On".to_string(),
            2 => "Off".to_string(),
            v => format!("Unknown ({v})"),
        },
    );
    put(
        metadata,
        "VRMode",
        match raw[6] {
            0 => "Off".to_string(),
            1 => "Normal".to_string(),
            3 => "Sport".to_string(),
            v => format!("Unknown ({v})"),
        },
    );
    put(
        metadata,
        "VRType",
        match raw[8] {
            2 => "In-body".to_string(),
            3 => "In-body + Lens".to_string(),
            v => format!("Unknown ({v})"),
        },
    );
}

fn parse_world_time(raw: &[u8], metadata: &mut HashMap<String, String>) {
    if raw.len() < 4 {
        return;
    }
    let minutes = i16::from_be_bytes([raw[0], raw[1]]);
    let sign = if minutes < 0 { '-' } else { '+' };
    let abs = minutes.unsigned_abs();
    put(
        metadata,
        "TimeZone",
        format!("{sign}{:02}:{:02}", abs / 60, abs % 60),
    );
    put(
        metadata,
        "DaylightSavings",
        if raw[2] == 0 {
            "No".to_string()
        } else {
            "Yes".to_string()
        },
    );
    put(
        metadata,
        "DateDisplayFormat",
        match raw[3] {
            0 => "Y/M/D".to_string(),
            1 => "M/D/Y".to_string(),
            2 => "D/M/Y".to_string(),
            v => format!("Unknown ({v})"),
        },
    );
}

fn parse_iso_info(raw: &[u8], metadata: &mut HashMap<String, String>) {
    if raw.is_empty() {
        return;
    }
    put(metadata, "ISO", nikon_iso(raw[0]).to_string());
    if raw.len() >= 6 {
        let exp = u16::from_be_bytes([raw[4], raw[5]]);
        put(metadata, "ISOExpansion", iso_expansion(exp));
    }
    if raw.len() >= 7 {
        put(metadata, "ISO2", nikon_iso(raw[6]).to_string());
    }
    if raw.len() >= 12 {
        let exp = u16::from_be_bytes([raw[10], raw[11]]);
        put(metadata, "ISOExpansion2", iso_expansion(exp));
    }
}

fn nikon_iso(raw: u8) -> u32 {
    let v = 100.0 * 2f64.powf(f64::from(raw) / 12.0 - 5.0);
    (v + 0.5) as u32
}

fn iso_expansion(v: u16) -> String {
    match v {
        0 => "Off".to_string(),
        0x101 => "Hi 0.3".to_string(),
        0x102 => "Hi 0.5".to_string(),
        0x103 => "Hi 0.7".to_string(),
        0x104 => "Hi 1.0".to_string(),
        0x201 => "Lo 0.3".to_string(),
        0x204 => "Lo 1.0".to_string(),
        other => format!("Unknown ({other})"),
    }
}

fn parse_picture_control(raw: &[u8], metadata: &mut HashMap<String, String>) {
    if raw.len() < 4 || &raw[0..2] != b"03" {
        return;
    }
    put(metadata, "PictureControlVersion", ascii(&raw[0..4]));
    if raw.len() >= 28 {
        put(
            metadata,
            "PictureControlName",
            nikon_title_case(&ascii(&raw[8..28])),
        );
    }
    if raw.len() >= 48 {
        put(
            metadata,
            "PictureControlBase",
            nikon_title_case(&ascii(&raw[28..48])),
        );
    }
    if raw.len() > 54 {
        put(
            metadata,
            "PictureControlAdjust",
            match raw[54] {
                0 => "Default Settings".to_string(),
                1 => "Quick Adjust".to_string(),
                2 => "Full Control".to_string(),
                v => format!("Unknown ({v})"),
            },
        );
    }
    const FIELDS: &[(usize, &str)] = &[
        (55, "PictureControlQuickAdjust"),
        (57, "Sharpness"),
        (59, "MidRangeSharpness"),
        (61, "Clarity"),
        (63, "Contrast"),
        (65, "Brightness"),
        (67, "Saturation"),
        (69, "Hue"),
        (71, "FilterEffect"),
        (72, "ToningEffect"),
        (73, "ToningSaturation"),
    ];
    for (offset, name) in FIELDS {
        if let Some(b) = raw.get(*offset).copied() {
            put(
                metadata,
                name,
                if b == 0xff {
                    "n/a".to_string()
                } else {
                    b.to_string()
                },
            );
        }
    }
}

fn parse_af_info2(raw: &[u8], metadata: &mut HashMap<String, String>) {
    if raw.len() < 7 {
        return;
    }
    put(metadata, "AFInfo2Version", ascii(&raw[0..4]));
    put(
        metadata,
        "ContrastDetectAF",
        match raw[4] {
            0 => "Off".to_string(),
            1 => "On".to_string(),
            2 => "On (2)".to_string(),
            v => format!("Unknown ({v})"),
        },
    );
    put(metadata, "AFAreaMode", af_area_mode(raw[5]));
    put(metadata, "PhaseDetectAF", format!("Unknown ({})", raw[6]));
    if raw.len() >= 17 {
        let hex = raw[10..17]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(" ");
        put(metadata, "AFPointsUsed", format!("Unknown ({hex})"));
    }
}

fn af_area_mode(v: u8) -> String {
    match v {
        192 => "Pinpoint".to_string(),
        193 => "Single".to_string(),
        195 => "Wide (S)".to_string(),
        196 => "Wide (L)".to_string(),
        197 => "Auto".to_string(),
        204 => "Dynamic Area (S)".to_string(),
        205 => "Dynamic Area (M)".to_string(),
        206 => "Dynamic Area (L)".to_string(),
        207 => "3D-tracking".to_string(),
        208 => "Wide (C1/C2)".to_string(),
        other => format!("Unknown ({other})"),
    }
}

fn parse_lens_data(
    raw: &[u8],
    serial: Option<u32>,
    shutter: Option<u32>,
    metadata: &mut HashMap<String, String>,
) {
    if raw.len() < 4 {
        return;
    }
    let version = ascii(&raw[0..4]);
    put(metadata, "LensDataVersion", version.clone());
    if !version.starts_with("080") {
        return;
    }
    let (Some(serial), Some(shutter)) = (serial, shutter) else {
        return;
    };
    let decoded = decrypt_nikon(raw, 4, serial, shutter);
    if decoded.len() < 0x60 {
        return;
    }
    let lens_id = u16::from_le_bytes([decoded[0x30], decoded[0x31]]);
    if let Some(name) = z_lens_name(lens_id) {
        put(metadata, "LensID", name.to_string());
    }
    let max_ap = u16::from_le_bytes([decoded[0x36], decoded[0x37]]);
    let aperture = 2f64.powf(f64::from(max_ap) / 384.0 - 1.0);
    put(metadata, "MaxAperture", format!("{aperture:.1}"));
    let focus_raw = u16::from_le_bytes([decoded[0x4e], decoded[0x4f]]);
    let focus_steps = f64::from(focus_raw) / 256.0;
    let meters = 2f64.powf((focus_steps - 80.0) / 12.0);
    put(metadata, "FocusDistance", format_distance(meters));
    let position = i32::from_le_bytes(decoded[0x5a..0x5e].try_into().unwrap());
    put(metadata, "LensPositionAbsolute", position.to_string());
    let mount = decoded[0x5f] & 0x01;
    put(
        metadata,
        "LensMountType",
        if mount == 0 {
            "Z-mount Lens".to_string()
        } else {
            "F-mount Lens".to_string()
        },
    );
}

fn decrypt_nikon(data: &[u8], start: usize, serial: u32, count: u32) -> Vec<u8> {
    let mut out = data.to_vec();
    if start >= out.len() {
        return out;
    }
    let mut key = 0u32;
    for i in 0..4 {
        key ^= (count >> (i * 8)) & 0xff;
    }
    let ci = u32::from(XLAT0[(serial & 0xff) as usize]);
    let mut cj = u32::from(XLAT1[(key & 0xff) as usize]);
    let mut ck = 0x60u32;
    for byte in &mut out[start..] {
        cj = (cj + ci * ck) & 0xff;
        ck = (ck + 1) & 0xff;
        *byte ^= cj as u8;
    }
    out
}

fn z_lens_name(id: u16) -> Option<&'static str> {
    Some(match id {
        1 => "Nikkor Z 24-70mm f/4 S",
        2 => "Nikkor Z 14-30mm f/4 S",
        4 => "Nikkor Z 35mm f/1.8 S",
        9 => "Nikkor Z 50mm f/1.8 S",
        11 => "NIKKOR Z DX 16-50mm f/3.5-6.3 VR",
        12 => "Nikkor Z DX 50-250mm f/4.5-6.3 VR",
        13 => "Nikkor Z 24-70mm f/2.8 S",
        14 => "Nikkor Z 85mm f/1.8 S",
        _ => return None,
    })
}

fn format_distance(meters: f64) -> String {
    if meters < 10.0 {
        format!("{meters:.2} m")
    } else if meters < 100.0 {
        format!("{meters:.1} m")
    } else {
        format!("{meters:.0} m")
    }
}

fn format_size(fmt: u16) -> Option<usize> {
    Some(match fmt {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 => 4,
        5 | 10 | 12 => 8,
        _ => return None,
    })
}

fn ascii(raw: &[u8]) -> String {
    let end = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).trim().to_string()
}

fn latin1_trim(raw: &[u8]) -> String {
    raw.iter().map(|b| *b as char).collect::<String>()
}

fn nikon_title_case(value: &str) -> String {
    let mut out = String::new();
    let mut cap = true;
    for ch in value.chars() {
        if ch.is_whitespace() {
            cap = true;
            out.push(ch);
        } else if cap {
            out.extend(ch.to_uppercase());
            cap = false;
        } else {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

fn be_u16s(raw: &[u8]) -> Vec<u16> {
    raw.chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect()
}

fn be_u32s(raw: &[u8]) -> Vec<u32> {
    raw.chunks_exact(4)
        .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn first_rational(raw: &[u8]) -> Option<(u32, u32)> {
    if raw.len() < 8 {
        return None;
    }
    Some((
        u32::from_be_bytes(raw[0..4].try_into().unwrap()),
        u32::from_be_bytes(raw[4..8].try_into().unwrap()),
    ))
}

fn first_srational(raw: &[u8]) -> Option<(i32, i32)> {
    if raw.len() < 8 {
        return None;
    }
    Some((
        i32::from_be_bytes(raw[0..4].try_into().unwrap()),
        i32::from_be_bytes(raw[4..8].try_into().unwrap()),
    ))
}

fn reduce_fraction(n: u32, d: u32) -> String {
    if d == 0 {
        return "0".to_string();
    }
    let g = gcd(n, d);
    let n = n / g;
    let d = d / g;
    if d == 1 {
        n.to_string()
    } else {
        format!("{n}/{d}")
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.max(1)
}

fn format_fnumber(n: u32, d: u32) -> String {
    if d == 0 {
        return "0".to_string();
    }
    format!("{:.1}", n as f64 / d as f64)
}

fn format_focal(n: u32, d: u32) -> String {
    if d == 0 {
        return "0 mm".to_string();
    }
    format!("{:.1} mm", n as f64 / d as f64)
}

fn format_ev(n: i32, d: i32) -> String {
    if d == 0 {
        return "0".to_string();
    }
    let v = n as f64 / d as f64;
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

fn format_frame_rate(n: u32, d: u32) -> String {
    if d == 0 {
        return "0".to_string();
    }
    let v = n as f64 / d as f64;
    let rounded = (v * 1000.0 + 0.5).floor() / 1000.0;
    let text = format!("{rounded:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn format_lens_info(raw: &[u8]) -> String {
    if raw.len() < 32 {
        return String::new();
    }
    let vals: Vec<f64> = (0..4)
        .map(|i| {
            let n = u32::from_be_bytes(raw[i * 8..i * 8 + 4].try_into().unwrap());
            let d = u32::from_be_bytes(raw[i * 8 + 4..i * 8 + 8].try_into().unwrap());
            if d == 0 {
                0.0
            } else {
                n as f64 / d as f64
            }
        })
        .collect();
    format!(
        "{}-{}mm f/{}-{}",
        trim_float(vals[0]),
        trim_float(vals[1]),
        trim_float(vals[2]),
        trim_float(vals[3])
    )
}

fn trim_float(v: f64) -> String {
    if (v - v.round()).abs() < 0.05 {
        format!("{:.0}", v.round())
    } else {
        format!("{v:.1}")
    }
}

fn format_fine_tune(raw: &[u8]) -> String {
    let mut parts = Vec::new();
    let mut i = 0;
    while i + 8 <= raw.len() {
        let n = i32::from_be_bytes(raw[i..i + 4].try_into().unwrap());
        let d = i32::from_be_bytes(raw[i + 4..i + 8].try_into().unwrap());
        if d != 0 && n % d == 0 {
            parts.push((n / d).to_string());
        } else if d != 0 {
            parts.push(format!("{:.1}", n as f64 / d as f64));
        } else {
            parts.push("0".to_string());
        }
        i += 8;
    }
    parts.join(" ")
}

fn format_crop(raw: &[u8]) -> String {
    let vals = be_u16s(raw);
    if vals.len() < 7 {
        return vals
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
    }
    let name = match vals[0] {
        0 => "Off",
        1 => "1.3x Crop",
        2 => "DX Crop",
        3 => "5:4 Crop",
        4 => "3:2 Crop",
        6 => "16:9 Crop",
        8 => "2.7x Crop",
        9 => "DX Movie Crop",
        10 => "1.3x Movie Crop",
        11 => "FX Uncropped",
        12 => "DX Uncropped",
        other => return format!("Unknown ({other})"),
    };
    format!(
        "{name} ({}x{} cropped to {}x{} at pixel {},{})",
        vals[1], vals[2], vals[3], vals[4], vals[5], vals[6]
    )
}

fn maker_note_version(raw: &[u8]) -> String {
    let text = ascii(raw);
    if text.len() >= 4 && text.chars().all(|c| c.is_ascii_digit()) {
        let major = text[..2].trim_start_matches('0');
        let major = if major.is_empty() { "0" } else { major };
        format!("{major}.{}", &text[2..4])
    } else {
        text
    }
}

fn lens_type(v: u8) -> String {
    if v == 0 {
        return "AF".to_string();
    }
    let mut parts = Vec::new();
    if v & 0x01 != 0 {
        parts.push("MF");
    }
    if v & 0x02 != 0 {
        parts.push("D");
    }
    if v & 0x04 != 0 {
        parts.push("G");
    }
    if v & 0x08 != 0 {
        parts.push("VR");
    }
    if v & 0x40 != 0 {
        parts.push("E");
    }
    if parts.is_empty() {
        format!("Unknown ({v})")
    } else {
        parts.join(" ")
    }
}

fn exposure_program(v: u16) -> String {
    match v {
        0 => "Not Defined",
        1 => "Manual",
        2 => "Program AE",
        3 => "Aperture-priority AE",
        4 => "Shutter speed priority AE",
        5 => "Creative (Slow speed)",
        6 => "Action (High speed)",
        7 => "Portrait",
        8 => "Landscape",
        _ => return format!("Unknown ({v})"),
    }
    .to_string()
}

fn metering_mode(v: u16) -> String {
    match v {
        1 => "Average",
        2 => "Center-weighted average",
        3 => "Spot",
        4 => "Multi-spot",
        5 => "Multi-segment",
        6 => "Partial",
        255 => "Other",
        _ => return format!("Unknown ({v})"),
    }
    .to_string()
}

fn active_d_lighting(v: u16) -> String {
    match v {
        0 => "Off",
        1 => "Low",
        3 => "Normal",
        5 => "High",
        7 => "Extra High",
        0xffff => "Auto",
        _ => return format!("Unknown ({v})"),
    }
    .to_string()
}

fn vignette_control(v: u16) -> String {
    match v {
        0 => "Off",
        1 => "Low",
        3 => "Normal",
        5 => "High",
        _ => return format!("Unknown ({v})"),
    }
    .to_string()
}

fn flash_mode(v: u8) -> String {
    match v {
        0 => "Did Not Fire",
        1 => "Fired, Manual",
        3 => "Not Ready",
        7 => "Fired, External",
        8 => "Fired, Commander Mode",
        9 => "Fired, TTL Mode",
        18 => "LED Light",
        _ => return format!("Unknown ({v})"),
    }
    .to_string()
}

fn high_iso_nr(v: u16) -> String {
    match v {
        0 => "Off",
        1 => "Minimal",
        2 => "Low",
        3 => "Medium Low",
        4 => "Normal",
        5 => "Medium High",
        6 => "High",
        _ => return format!("Unknown ({v})"),
    }
    .to_string()
}

fn on_off(v: u16) -> String {
    if v == 0 {
        "Off".to_string()
    } else {
        "On".to_string()
    }
}

fn put(metadata: &mut HashMap<String, String>, key: &str, value: String) {
    if !value.is_empty() {
        metadata.insert(key.to_string(), value);
    }
}

fn insert_qt_date(metadata: &mut HashMap<String, String>, key: &str, qt_seconds: u32) {
    let unix = i64::from(qt_seconds) - QT_EPOCH_OFFSET;
    if unix <= 0 {
        return;
    }
    if let Some(dt) = chrono::DateTime::from_timestamp(unix, 0) {
        metadata.insert(key.to_string(), dt.format("%Y:%m:%d %H:%M:%S").to_string());
    }
}

fn fixed_16_16(v: u32) -> u32 {
    v >> 16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let size = (8 + payload.len()) as u32;
        let mut out = size.to_be_bytes().to_vec();
        out.extend_from_slice(tag);
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn nikon_z50_mov_matches_exiftool_camera_tags() {
        let path = "/keg/x/2026/01-Jan/20260102_204044.000.mov";
        if !std::path::Path::new(path).exists() {
            return;
        }
        let mut file = std::fs::File::open(path).unwrap();
        let len = file.metadata().unwrap().len();
        let metadata =
            crate::parsers::video::VideoParser::parse_quicktime_file(&mut file, len, true).unwrap();
        assert_eq!(
            metadata.get("Make").map(String::as_str),
            Some("NIKON CORPORATION")
        );
        assert_eq!(
            metadata.get("Model").map(String::as_str),
            Some("NIKON Z50_2")
        );
        assert_eq!(
            metadata.get("DateTimeOriginal").map(String::as_str),
            Some("2026:01:02 15:40:44")
        );
        assert_eq!(
            metadata.get("ExposureTime").map(String::as_str),
            Some("1/125")
        );
        assert_eq!(metadata.get("FNumber").map(String::as_str), Some("3.5"));
        assert_eq!(metadata.get("ISO").map(String::as_str), Some("3200"));
        assert_eq!(
            metadata.get("FocalLength").map(String::as_str),
            Some("16.0 mm")
        );
        assert_eq!(
            metadata.get("LensModel").map(String::as_str),
            Some("NIKKOR Z DX 16-50mm f/3.5-6.3 VR")
        );
        assert_eq!(
            metadata.get("ShutterCount").map(String::as_str),
            Some("44879")
        );
        assert_eq!(
            metadata.get("FocusDistance").map(String::as_str),
            Some("9.98 m")
        );
        assert_eq!(metadata.get("MaxAperture").map(String::as_str), Some("3.6"));
        assert_eq!(
            metadata.get("LensMountType").map(String::as_str),
            Some("Z-mount Lens")
        );
        assert_eq!(
            metadata.get("Duration").map(String::as_str),
            Some("10.11 s")
        );
        assert_eq!(
            metadata.get("CompressorID").map(String::as_str),
            Some("hvc1")
        );
        assert_eq!(
            metadata.get("ColorPrimaries").map(String::as_str),
            Some("BT.709")
        );
        assert_eq!(metadata.get("ImageWidth").map(String::as_str), Some("3840"));
        assert_eq!(
            metadata.get("MediaDataOffset").map(String::as_str),
            Some("3407880")
        );
        assert_eq!(
            metadata.get("ModifyDate").map(String::as_str),
            Some("2026:01:02 20:40:44")
        );
        assert_eq!(
            metadata.get("CreateDate").map(String::as_str),
            Some("2026:01:02 15:40:44")
        );
    }

    #[test]
    fn nctg_make_inside_moov_is_not_tiff() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&1u32.to_be_bytes());
        payload.extend_from_slice(&2u16.to_be_bytes());
        payload.extend_from_slice(&5u16.to_be_bytes());
        payload.extend_from_slice(b"NIKON");
        let nctg = atom(b"NCTG", &payload);
        let ncdt = atom(b"NCDT", &nctg);
        let udta = atom(b"udta", &ncdt);
        let moov = atom(b"moov", &udta);
        let ftyp = atom(b"ftyp", b"qt  \x00\x00\x00\x00qt  ");
        let mut file = ftyp;
        file.extend_from_slice(&moov);
        let mut metadata = HashMap::new();
        walk_atoms(&file, &mut metadata);
        assert_eq!(metadata.get("Make").map(String::as_str), Some("NIKON"));
        assert_eq!(
            metadata.get("MajorBrand").map(String::as_str),
            Some("Apple QuickTime (.MOV/QT)")
        );
    }
}
