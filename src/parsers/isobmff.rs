//! ISO Base Media File Format reader for HEIF/HEIC still images.
//!
//! HEIC stores the TIFF EXIF blob as an `Exif` item. The `iloc` box gives its
//! file offset, which is often near the end of `mdat`, and `meta` itself may
//! follow that `mdat`. Scanning the file for a JPEG APP1 marker misses it.

use crate::parsers::tiff::TiffParser;
use crate::types::{ExifError, ParseScope};
use std::collections::HashMap;

struct BmffBox {
    kind: [u8; 4],
    header_len: usize,
    start: usize,
    end: usize,
}

struct ItemLoc {
    item_id: u32,
    construction: u8,
    base_offset: u64,
    extents: Vec<(u64, u64)>,
}

pub(crate) fn parse_heif(
    data: &[u8],
    metadata: &mut HashMap<String, String>,
    scope: &ParseScope,
) -> Result<(), ExifError> {
    if data.len() < 12 || &data[4..8] != b"ftyp" {
        return Err(ExifError::InvalidExif("Not a HEIF file".to_string()));
    }

    let mut pos = 0;
    let mut ftyp_payload: Option<(usize, usize)> = None;
    let mut mdat: Option<(usize, usize)> = None;
    let mut meta_payload: Option<(usize, usize)> = None;

    while pos + 8 <= data.len() {
        let Some(bx) = next_box(data, &mut pos, data.len()) else {
            break;
        };
        let payload_start = bx.start + bx.header_len;
        match &bx.kind {
            b"ftyp" => ftyp_payload = Some((payload_start, bx.end)),
            b"mdat" => {
                if mdat.is_none() {
                    mdat = Some((payload_start, bx.end));
                }
            }
            b"meta" => meta_payload = Some((payload_start, bx.end)),
            _ => {}
        }
    }

    let (ftyp_start, ftyp_end) =
        ftyp_payload.ok_or_else(|| ExifError::InvalidExif("HEIF ftyp missing".to_string()))?;
    apply_ftyp(&data[ftyp_start..ftyp_end], metadata);

    if let Some((payload_start, payload_end)) = mdat {
        metadata.insert("MediaDataOffset".to_string(), payload_start.to_string());
        metadata.insert(
            "MediaDataSize".to_string(),
            payload_end.saturating_sub(payload_start).to_string(),
        );
    }

    let Some((meta_start, meta_end)) = meta_payload else {
        return Ok(());
    };
    let meta = &data[meta_start..meta_end];
    // `meta` is a FullBox: 4-byte version/flags, then child boxes.
    let child_off = if meta.len() >= 8 { 4 } else { 0 };

    let mut handler = None;
    let mut item_types: HashMap<u32, [u8; 4]> = HashMap::new();
    let mut locations: Vec<ItemLoc> = Vec::new();
    let mut primary: Option<u32> = None;
    let mut properties: Vec<([u8; 4], usize, usize)> = Vec::new();
    let mut associations: HashMap<u32, Vec<u16>> = HashMap::new();
    let mut idat: Option<(usize, usize)> = None;

    let mut child = child_off;
    while child + 8 <= meta.len() {
        let Some(bx) = next_box(meta, &mut child, meta.len()) else {
            break;
        };
        let abs_payload = meta_start + bx.start + bx.header_len;
        let payload = &meta[bx.start + bx.header_len..bx.end];
        match &bx.kind {
            b"hdlr" => handler = handler_type(payload),
            b"iinf" => item_types = parse_iinf(payload),
            b"iloc" => locations = parse_iloc(payload),
            b"pitm" => primary = parse_pitm(payload),
            b"idat" => idat = Some((abs_payload, meta_start + bx.end)),
            b"iprp" => parse_iprp(
                data,
                abs_payload - bx.header_len,
                meta_start + bx.end,
                &mut properties,
                &mut associations,
            ),
            _ => {}
        }
    }

    if let Some(handler) = handler {
        metadata.insert("HandlerType".to_string(), handler);
    }

    let primary_id = primary.or_else(|| {
        item_types
            .iter()
            .find(|(_, kind)| *kind == b"grid")
            .map(|(id, _)| *id)
    });
    if let Some(id) = primary_id {
        metadata.insert("PrimaryItemReference".to_string(), id.to_string());
        apply_primary_properties(data, id, &properties, &associations, metadata);
    }

    if let Some(exif) = load_exif_item(data, &item_types, &locations, idat) {
        if let Some(tiff) = tiff_from_exif_item(&exif) {
            let _ = TiffParser::parse_tiff_exif_scoped(tiff, metadata, scope);
        }
    }

    Ok(())
}

fn apply_ftyp(payload: &[u8], metadata: &mut HashMap<String, String>) {
    if payload.len() < 8 {
        return;
    }
    let major: [u8; 4] = payload[0..4].try_into().unwrap_or(*b"heic");
    let minor = u32::from_be_bytes(payload[4..8].try_into().unwrap_or([0; 4]));
    let mut brands = Vec::new();
    let mut off = 8;
    while off + 4 <= payload.len() {
        brands.push(payload[off..off + 4].to_vec());
        off += 4;
    }

    let heic = &major == b"heic"
        || &major == b"heix"
        || &major == b"hevc"
        || &major == b"hevx"
        || brands
            .iter()
            .any(|b| b.as_slice() == b"heic" || b.as_slice() == b"heix");
    let avif = &major == b"avif"
        || &major == b"avis"
        || brands
            .iter()
            .any(|b| b.as_slice() == b"avif" || b.as_slice() == b"avis");

    let (file_type, ext, mime) = if avif && !heic {
        ("AVIF", "avif", "image/avif")
    } else if heic {
        ("HEIC", "heic", "image/heic")
    } else {
        ("HEIF", "heif", "image/heif")
    };

    metadata.insert("Format".to_string(), file_type.to_string());
    metadata.insert("FileType".to_string(), file_type.to_string());
    metadata.insert("FileTypeExtension".to_string(), ext.to_string());
    metadata.insert("MIMEType".to_string(), mime.to_string());
    metadata.insert("MajorBrand".to_string(), major_brand_name(&major));
    let minor_bytes = minor.to_be_bytes();
    metadata.insert(
        "MinorVersion".to_string(),
        format!("{}.{}.{}", minor_bytes[1], minor_bytes[2], minor_bytes[3]),
    );
    if !brands.is_empty() {
        let text = brands
            .iter()
            .map(|b| String::from_utf8_lossy(b).trim().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        metadata.insert("CompatibleBrands".to_string(), text);
    }
}

fn major_brand_name(brand: &[u8; 4]) -> String {
    match brand {
        b"heic" => "High Efficiency Image Format HEVC still image (.HEIC)".to_string(),
        b"heix" => "High Efficiency Image Format HEVC image (.HEIC)".to_string(),
        b"hevc" | b"hevx" => "High Efficiency Image Format HEVC (.HEIC)".to_string(),
        b"mif1" | b"heif" => "HEIF Image".to_string(),
        b"msf1" => "HEIF Sequence".to_string(),
        b"avif" => "AV1 Image File Format (.AVIF)".to_string(),
        b"avis" => "AV1 Image Sequence (.AVIF)".to_string(),
        other => String::from_utf8_lossy(other).trim().to_string(),
    }
}

fn apply_primary_properties(
    data: &[u8],
    item_id: u32,
    properties: &[([u8; 4], usize, usize)],
    associations: &HashMap<u32, Vec<u16>>,
    metadata: &mut HashMap<String, String>,
) {
    let Some(indexes) = associations.get(&item_id) else {
        return;
    };
    let mut best_area = 0u64;
    let mut size = None;
    let mut rotation = None;
    for index in indexes {
        let idx = *index as usize;
        if idx == 0 || idx > properties.len() {
            continue;
        }
        let (kind, start, end) = properties[idx - 1];
        let payload = data.get(start..end).unwrap_or(&[]);
        if &kind == b"ispe" && payload.len() >= 12 {
            let width = u32::from_be_bytes(payload[4..8].try_into().unwrap_or([0; 4]));
            let height = u32::from_be_bytes(payload[8..12].try_into().unwrap_or([0; 4]));
            let area = width as u64 * height as u64;
            if area >= best_area {
                best_area = area;
                size = Some((width, height));
            }
        } else if &kind == b"irot" && !payload.is_empty() && rotation.is_none() {
            rotation = Some((payload[0] & 0x03) as u32 * 90);
        }
    }
    if let Some((width, height)) = size {
        metadata.insert("ImageWidth".to_string(), width.to_string());
        metadata.insert("ImageHeight".to_string(), height.to_string());
        metadata.insert("ImageSize".to_string(), format!("{width}x{height}"));
        metadata.insert("MetaImageSize".to_string(), format!("{width}x{height}"));
        metadata.insert(
            "ImageSpatialExtent".to_string(),
            format!("{width}x{height}"),
        );
    }
    if let Some(angle) = rotation {
        metadata.insert("Rotation".to_string(), angle.to_string());
    }
}

fn load_exif_item(
    data: &[u8],
    item_types: &HashMap<u32, [u8; 4]>,
    locations: &[ItemLoc],
    idat: Option<(usize, usize)>,
) -> Option<Vec<u8>> {
    let exif_id = item_types
        .iter()
        .find(|(_, kind)| *kind == b"Exif")
        .map(|(id, _)| *id)?;
    let loc = locations.iter().find(|loc| loc.item_id == exif_id)?;
    let mut buf = Vec::new();
    for &(offset, length) in &loc.extents {
        let abs = match loc.construction {
            0 => loc.base_offset.checked_add(offset)?,
            1 => {
                let (idat_start, _) = idat?;
                (idat_start as u64)
                    .checked_add(loc.base_offset)?
                    .checked_add(offset)?
            }
            _ => return None,
        };
        let start = usize::try_from(abs).ok()?;
        let len = usize::try_from(length).ok()?;
        if len == 0 {
            continue;
        }
        let end = start.checked_add(len)?;
        buf.extend_from_slice(data.get(start..end)?);
    }
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

/// Exif item payload: big-endian offset to the TIFF header, usually 6 (`Exif\0\0`).
fn tiff_from_exif_item(payload: &[u8]) -> Option<&[u8]> {
    if let Some(slice) = tiff_at(payload, 0) {
        return Some(slice);
    }
    if payload.len() >= 8 {
        let off = u32::from_be_bytes(payload[0..4].try_into().ok()?) as usize;
        if let Some(slice) = tiff_at(payload, off) {
            return Some(slice);
        }
    }
    let limit = payload.len().min(64);
    for i in 0..limit.saturating_sub(8) {
        if let Some(slice) = tiff_at(payload, i) {
            return Some(slice);
        }
    }
    None
}

fn tiff_at(payload: &[u8], off: usize) -> Option<&[u8]> {
    if off + 8 > payload.len() {
        return None;
    }
    let le = &payload[off..off + 2] == b"II";
    let be = &payload[off..off + 2] == b"MM";
    if !le && !be {
        return None;
    }
    let version = if le {
        u16::from_le_bytes([payload[off + 2], payload[off + 3]])
    } else {
        u16::from_be_bytes([payload[off + 2], payload[off + 3]])
    };
    if version == 42 {
        Some(&payload[off..])
    } else {
        None
    }
}

fn handler_type(payload: &[u8]) -> Option<String> {
    // FullBox + pre_defined u32 + handler_type.
    if payload.len() < 12 {
        return None;
    }
    let kind = &payload[8..12];
    let name = match kind {
        b"pict" => "Picture",
        b"vide" => "Video Handler",
        b"soun" => "Sound Handler",
        b"meta" => "Meta",
        other => return Some(String::from_utf8_lossy(other).trim().to_string()),
    };
    Some(name.to_string())
}

fn parse_iinf(payload: &[u8]) -> HashMap<u32, [u8; 4]> {
    let mut types = HashMap::new();
    if payload.len() < 6 {
        return types;
    }
    let version = payload[0];
    let (count, mut pos) = if version == 0 {
        (u16::from_be_bytes([payload[4], payload[5]]) as u32, 6)
    } else if payload.len() >= 8 {
        (
            u32::from_be_bytes(payload[4..8].try_into().unwrap_or([0; 4])),
            8,
        )
    } else {
        return types;
    };
    for _ in 0..count {
        let Some(bx) = next_box(payload, &mut pos, payload.len()) else {
            break;
        };
        if &bx.kind != b"infe" {
            continue;
        }
        let entry = &payload[bx.start + bx.header_len..bx.end];
        if let Some((id, kind)) = parse_infe(entry) {
            types.insert(id, kind);
        }
    }
    types
}

fn parse_infe(payload: &[u8]) -> Option<(u32, [u8; 4])> {
    if payload.len() < 8 {
        return None;
    }
    let version = payload[0];
    let mut pos = 4;
    let item_id = if version <= 2 {
        let id = u16::from_be_bytes(payload.get(pos..pos + 2)?.try_into().ok()?) as u32;
        pos += 2;
        id
    } else if version == 3 {
        let id = u32::from_be_bytes(payload.get(pos..pos + 4)?.try_into().ok()?);
        pos += 4;
        id
    } else {
        return None;
    };
    pos += 2; // item_protection_index
    if version >= 2 {
        let kind = payload.get(pos..pos + 4)?.try_into().ok()?;
        Some((item_id, kind))
    } else {
        let _ = item_id;
        None
    }
}

fn parse_iloc(payload: &[u8]) -> Vec<ItemLoc> {
    parse_iloc_inner(payload).unwrap_or_default()
}

fn parse_iloc_inner(payload: &[u8]) -> Option<Vec<ItemLoc>> {
    if payload.len() < 8 {
        return None;
    }
    let version = payload[0];
    let mut pos = 4;
    let offset_size = (payload[pos] >> 4) as usize;
    let length_size = (payload[pos] & 0x0f) as usize;
    pos += 1;
    let base_offset_size = (payload[pos] >> 4) as usize;
    let index_size = if version == 1 || version == 2 {
        (payload[pos] & 0x0f) as usize
    } else {
        0
    };
    pos += 1;
    if offset_size > 8 || length_size > 8 || base_offset_size > 8 || index_size > 8 {
        return None;
    }
    let item_count = if version < 2 {
        read_uint(payload, &mut pos, 2)? as u32
    } else {
        read_uint(payload, &mut pos, 4)? as u32
    };
    let mut items = Vec::new();
    for _ in 0..item_count {
        let item_id = if version < 2 {
            read_uint(payload, &mut pos, 2)? as u32
        } else {
            read_uint(payload, &mut pos, 4)? as u32
        };
        let construction = if version == 1 || version == 2 {
            (read_uint(payload, &mut pos, 2)? & 0x0f) as u8
        } else {
            0
        };
        let _data_ref = read_uint(payload, &mut pos, 2)?;
        let base_offset = read_uint(payload, &mut pos, base_offset_size)?;
        let extent_count = read_uint(payload, &mut pos, 2)? as u32;
        let mut extents = Vec::new();
        for _ in 0..extent_count {
            if (version == 1 || version == 2) && index_size > 0 {
                let _ = read_uint(payload, &mut pos, index_size)?;
            }
            let offset = read_uint(payload, &mut pos, offset_size)?;
            let length = read_uint(payload, &mut pos, length_size)?;
            extents.push((offset, length));
        }
        items.push(ItemLoc {
            item_id,
            construction,
            base_offset,
            extents,
        });
    }
    Some(items)
}

fn parse_pitm(payload: &[u8]) -> Option<u32> {
    if payload.len() < 6 {
        return None;
    }
    let version = payload[0];
    if version == 0 {
        Some(u16::from_be_bytes([payload[4], payload[5]]) as u32)
    } else if payload.len() >= 8 {
        Some(u32::from_be_bytes(payload[4..8].try_into().ok()?))
    } else {
        None
    }
}

fn parse_iprp(
    file: &[u8],
    box_start: usize,
    box_end: usize,
    properties: &mut Vec<([u8; 4], usize, usize)>,
    associations: &mut HashMap<u32, Vec<u16>>,
) {
    let Some(region) = file.get(box_start..box_end) else {
        return;
    };
    let mut pos = 8; // skip iprp header; caller passed the box start
    if region.len() >= 8 && &region[4..8] == b"iprp" {
        // box_start includes the iprp header
    } else {
        pos = 0;
    }
    while pos + 8 <= region.len() {
        let Some(bx) = next_box(region, &mut pos, region.len()) else {
            break;
        };
        let payload = &region[bx.start + bx.header_len..bx.end];
        if &bx.kind == b"ipco" {
            let mut inner = 0;
            while inner + 8 <= payload.len() {
                let Some(prop) = next_box(payload, &mut inner, payload.len()) else {
                    break;
                };
                let abs = box_start + bx.start + bx.header_len + prop.start + prop.header_len;
                let abs_end = box_start + bx.start + bx.header_len + prop.end;
                properties.push((prop.kind, abs, abs_end));
            }
        } else if &bx.kind == b"ipma" {
            *associations = parse_ipma(payload);
        }
    }
}

fn parse_ipma(payload: &[u8]) -> HashMap<u32, Vec<u16>> {
    let mut map = HashMap::new();
    if payload.len() < 8 {
        return map;
    }
    let version = payload[0];
    let flags = u32::from_be_bytes([0, payload[1], payload[2], payload[3]]);
    let mut pos = 4;
    let entry_count = read_uint(payload, &mut pos, 4).unwrap_or(0) as u32;
    let wide_index = flags & 1 != 0;
    for _ in 0..entry_count {
        let item_id = if version == 0 {
            match read_uint(payload, &mut pos, 2) {
                Some(v) => v as u32,
                None => break,
            }
        } else {
            match read_uint(payload, &mut pos, 4) {
                Some(v) => v as u32,
                None => break,
            }
        };
        let Some(count_byte) = payload.get(pos).copied() else {
            break;
        };
        pos += 1;
        let mut indexes = Vec::with_capacity(count_byte as usize);
        for _ in 0..count_byte {
            let index = if wide_index {
                match read_uint(payload, &mut pos, 2) {
                    Some(v) => (v & 0x7fff) as u16,
                    None => break,
                }
            } else {
                match read_uint(payload, &mut pos, 1) {
                    Some(v) => (v & 0x7f) as u16,
                    None => break,
                }
            };
            if index != 0 {
                indexes.push(index);
            }
        }
        map.insert(item_id, indexes);
    }
    map
}

fn read_uint(data: &[u8], pos: &mut usize, nbytes: usize) -> Option<u64> {
    if nbytes == 0 {
        return Some(0);
    }
    if nbytes > 8 || *pos + nbytes > data.len() {
        return None;
    }
    let mut value = 0u64;
    for _ in 0..nbytes {
        value = (value << 8) | data[*pos] as u64;
        *pos += 1;
    }
    Some(value)
}

fn next_box(data: &[u8], pos: &mut usize, end: usize) -> Option<BmffBox> {
    let end = end.min(data.len());
    if *pos + 8 > end {
        return None;
    }
    let mut size = u32::from_be_bytes(data[*pos..*pos + 4].try_into().ok()?) as u64;
    let kind: [u8; 4] = data[*pos + 4..*pos + 8].try_into().ok()?;
    let mut header_len = 8usize;
    if size == 1 {
        if *pos + 16 > end {
            return None;
        }
        size = u64::from_be_bytes(data[*pos + 8..*pos + 16].try_into().ok()?);
        header_len = 16;
    } else if size == 0 {
        size = (end - *pos) as u64;
    }
    if size < header_len as u64 {
        return None;
    }
    let start = *pos;
    let box_end = start.checked_add(size as usize)?;
    if box_end > end {
        return None;
    }
    *pos = box_end;
    Some(BmffBox {
        kind,
        header_len,
        start,
        end: box_end,
    })
}
