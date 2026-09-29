//! JPEG APP1 XMP and APP13 IPTC segments, plus splicing them around EXIF.

use crate::exif_encode;
use crate::types::ExifError;
use std::collections::HashMap;

const XMP_HEADER: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";
const PS_HEADER: &[u8] = b"Photoshop 3.0\0";

const IPTC_KEYS: &[&str] = &["IPTC:By-line", "IPTC:Byline", "By-line", "Byline"];
const XMP_CREATOR_KEYS: &[&str] = &["XMP-dc:Creator", "XMP:Creator", "dc:Creator"];
const XMP_TOOL_KEYS: &[&str] = &[
    "XMP-xmp:CreatorTool",
    "XMP:CreatorTool",
    "xmp:CreatorTool",
    "CreatorTool",
];

struct IrbBlock {
    id: u16,
    /// Pascal name field, including the length byte and even padding.
    name: Vec<u8>,
    data: Vec<u8>,
}

struct IptcDataset {
    record: u8,
    dataset: u8,
    data: Vec<u8>,
}

struct JpegParts {
    app0: Vec<Vec<u8>>,
    body: Vec<Vec<u8>>,
    rest: Vec<u8>,
}

fn first_value<'a>(metadata: &'a HashMap<String, String>, keys: &[&str]) -> Option<&'a str> {
    for key in keys {
        if let Some(value) = metadata.get(*key) {
            if !value.is_empty() {
                return Some(value.as_str());
            }
        }
    }
    None
}

pub(crate) fn requests_jpeg_sidecar(metadata: &HashMap<String, String>) -> bool {
    first_value(metadata, IPTC_KEYS).is_some()
        || first_value(metadata, XMP_CREATOR_KEYS).is_some()
        || first_value(metadata, XMP_TOOL_KEYS).is_some()
}

/// Write EXIF, XMP, and IPTC into a JPEG. Image scans and non-targeted markers stay put.
pub(crate) fn write_jpeg_metadata(
    input: &[u8],
    little_endian: bool,
    metadata: &HashMap<String, String>,
) -> Result<Vec<u8>, ExifError> {
    if input.len() < 2 || input[0] != 0xFF || input[1] != 0xD8 {
        return Err(ExifError::InvalidExif("Invalid JPEG format".to_string()));
    }

    let mut parts = split_jpeg(input)?;
    let exif = exif_encode::encode_jpeg_app1_opt(little_endian, metadata)?;
    let creator = first_value(metadata, XMP_CREATOR_KEYS);
    let tool = first_value(metadata, XMP_TOOL_KEYS);
    let xmp = match (creator, tool) {
        (None, None) => None,
        _ => Some(wrap_marker(0xE1, &xmp_payload(creator, tool)?)?),
    };
    let by_line = first_value(metadata, IPTC_KEYS);
    let iptc = if let Some(by_line) = by_line {
        let existing = parts
            .body
            .iter()
            .find(|segment| is_photoshop_app13(segment));
        let mut blocks = existing
            .map(|segment| irb_blocks(segment))
            .unwrap_or_default();
        upsert_iptc_byline(&mut blocks, by_line)?;
        Some(wrap_marker(0xED, &photoshop_payload(&blocks))?)
    } else {
        None
    };

    if exif.is_none() && xmp.is_none() && iptc.is_none() {
        return Err(ExifError::InvalidExif(
            "No writable EXIF fields in metadata".to_string(),
        ));
    }

    if exif.is_some() {
        parts.body.retain(|segment| !is_exif_app1(segment));
    }
    if xmp.is_some() {
        parts.body.retain(|segment| !is_xmp_app1(segment));
    }
    if iptc.is_some() {
        parts.body.retain(|segment| !is_photoshop_app13(segment));
    }

    let mut out = Vec::with_capacity(input.len() + 4096);
    out.extend_from_slice(&[0xFF, 0xD8]);
    for segment in &parts.app0 {
        out.extend_from_slice(segment);
    }
    if let Some(segment) = &exif {
        out.extend_from_slice(segment);
    }
    if let Some(segment) = &xmp {
        out.extend_from_slice(segment);
    }
    if let Some(segment) = &iptc {
        out.extend_from_slice(segment);
    }
    for segment in &parts.body {
        out.extend_from_slice(segment);
    }
    out.extend_from_slice(&parts.rest);
    Ok(out)
}

/// Read IPTC By-line and XMP creator fields out of a JPEG.
pub(crate) fn read_jpeg_sidecars(data: &[u8], metadata: &mut HashMap<String, String>) {
    let Ok(parts) = split_jpeg(data) else {
        return;
    };
    for segment in parts.body.iter().chain(parts.app0.iter()) {
        if is_xmp_app1(segment) {
            if let Some(xml) = segment_payload(segment).and_then(|payload| {
                payload
                    .strip_prefix(XMP_HEADER)
                    .and_then(|xml| std::str::from_utf8(xml).ok())
            }) {
                if let Some(creator) = xmp_creator(xml) {
                    insert_alias(metadata, &["XMP-dc:Creator", "Creator"], creator);
                }
                if let Some(tool) = xmp_creator_tool(xml) {
                    insert_alias(metadata, &["XMP-xmp:CreatorTool", "CreatorTool"], tool);
                }
            }
        }
        if is_photoshop_app13(segment) {
            for block in irb_blocks(segment) {
                if block.id != 0x0404 {
                    continue;
                }
                if let Some(by_line) = iptc_byline(&block.data) {
                    insert_alias(metadata, &["IPTC:By-line", "By-line"], by_line);
                }
            }
        }
    }
}

pub(crate) fn has_sidecar_tags(metadata: &HashMap<String, String>) -> bool {
    metadata.contains_key("IPTC:By-line")
        || metadata.contains_key("XMP-dc:Creator")
        || metadata.contains_key("XMP-xmp:CreatorTool")
}

fn insert_alias(metadata: &mut HashMap<String, String>, keys: &[&str], value: String) {
    for key in keys {
        metadata
            .entry((*key).to_string())
            .or_insert_with(|| value.clone());
    }
}

fn split_jpeg(data: &[u8]) -> Result<JpegParts, ExifError> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err(ExifError::InvalidExif("Invalid JPEG format".to_string()));
    }

    let mut app0 = Vec::new();
    let mut body = Vec::new();
    let mut rest = Vec::new();
    let mut pos = 2usize;

    while pos + 1 < data.len() {
        if data[pos] != 0xFF {
            rest = data[pos..].to_vec();
            break;
        }
        let mut marker_ff = pos;
        pos += 1;
        while pos < data.len() && data[pos] == 0xFF {
            marker_ff = pos;
            pos += 1;
        }
        if pos >= data.len() {
            rest = data[marker_ff..].to_vec();
            break;
        }
        let marker = data[pos];
        let segment_start = marker_ff;
        pos += 1;

        if marker == 0xD9 || marker == 0xDA {
            rest = data[segment_start..].to_vec();
            break;
        }
        if marker == 0x01 || (0xD0..=0xD8).contains(&marker) {
            body.push(data[segment_start..pos].to_vec());
            continue;
        }
        if pos + 2 > data.len() {
            rest = data[segment_start..].to_vec();
            break;
        }
        let seglen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        if seglen < 2 || pos + seglen > data.len() {
            rest = data[segment_start..].to_vec();
            break;
        }
        let segment_end = pos + seglen;
        let segment = data[segment_start..segment_end].to_vec();
        if marker == 0xE0 {
            app0.push(segment);
        } else {
            body.push(segment);
        }
        pos = segment_end;
    }

    Ok(JpegParts { app0, body, rest })
}

fn segment_payload(segment: &[u8]) -> Option<&[u8]> {
    if segment.len() < 4 || segment[0] != 0xFF {
        return None;
    }
    Some(&segment[4..])
}

fn is_exif_app1(segment: &[u8]) -> bool {
    segment.get(1) == Some(&0xE1)
        && segment_payload(segment).is_some_and(|payload| payload.starts_with(b"Exif\0\0"))
}

fn is_xmp_app1(segment: &[u8]) -> bool {
    segment.get(1) == Some(&0xE1)
        && segment_payload(segment).is_some_and(|payload| payload.starts_with(XMP_HEADER))
}

fn is_photoshop_app13(segment: &[u8]) -> bool {
    segment.get(1) == Some(&0xED)
        && segment_payload(segment).is_some_and(|payload| payload.starts_with(PS_HEADER))
}

fn wrap_marker(marker: u8, payload: &[u8]) -> Result<Vec<u8>, ExifError> {
    let len = payload.len() + 2;
    if len > u16::MAX as usize {
        return Err(ExifError::InvalidExif(format!(
            "JPEG segment is too large ({len} bytes)"
        )));
    }
    let mut out = Vec::with_capacity(4 + payload.len());
    out.push(0xFF);
    out.push(marker);
    out.extend_from_slice(&(len as u16).to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

fn xmp_payload(creator: Option<&str>, tool: Option<&str>) -> Result<Vec<u8>, ExifError> {
    let xml = build_xmp(creator, tool);
    let mut payload = Vec::with_capacity(XMP_HEADER.len() + xml.len());
    payload.extend_from_slice(XMP_HEADER);
    payload.extend_from_slice(xml.as_bytes());
    Ok(payload)
}

fn build_xmp(creator: Option<&str>, tool: Option<&str>) -> String {
    let mut body = String::new();
    if let Some(creator) = creator {
        body.push_str("   <dc:creator><rdf:Seq><rdf:li>");
        body.push_str(&xml_escape(creator));
        body.push_str("</rdf:li></rdf:Seq></dc:creator>\n");
    }
    if let Some(tool) = tool {
        body.push_str("   <xmp:CreatorTool>");
        body.push_str(&xml_escape(tool));
        body.push_str("</xmp:CreatorTool>\n");
    }
    format!(
        "<?xpacket begin=\"\u{FEFF}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
         <x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"fast-exif-rs\">\n\
         <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n\
         <rdf:Description rdf:about=\"\"\n\
         xmlns:dc=\"http://purl.org/dc/elements/1.1/\"\n\
         xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\">\n\
         {body}\
         </rdf:Description>\n\
         </rdf:RDF>\n\
         </x:xmpmeta>\n\
         <?xpacket end=\"w\"?>\n"
    )
}

fn xml_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            ch if ch.is_control() => {}
            ch => out.push(ch),
        }
    }
    out
}

fn xml_unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find(';') {
            let entity = &after[..end];
            let decoded = match entity {
                "amp" => Some("&"),
                "lt" => Some("<"),
                "gt" => Some(">"),
                "quot" => Some("\""),
                "apos" => Some("'"),
                _ => None,
            };
            if let Some(decoded) = decoded {
                out.push_str(decoded);
                rest = &after[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = after;
    }
    out.push_str(rest);
    out
}

fn element_inner<'a>(xml: &'a str, local: &str) -> Option<&'a str> {
    let mut search_from = 0;
    while let Some(rel) = xml[search_from..].find(local) {
        let idx = search_from + rel;
        let prev = xml[..idx].chars().next_back();
        let after = &xml[idx + local.len()..];
        let next = after.chars().next();
        let tag_boundary = matches!(next, Some('>' | '/' | ' ' | '\t' | '\n' | '\r'));
        if matches!(prev, Some('<' | ':')) && tag_boundary {
            if after.trim_start().starts_with("/>") {
                return Some("");
            }
            let content_at = after.find('>')? + 1;
            let content = &after[content_at..];
            if let Some(close_at) = find_close(content, local) {
                return Some(&content[..close_at]);
            }
        }
        search_from = idx + local.len();
    }
    None
}

fn find_close(content: &str, local: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(rel) = content[offset..].find("</") {
        let start = offset + rel;
        let after = &content[start + 2..];
        let name_len = after
            .find(|c: char| c == '>' || c.is_whitespace())
            .unwrap_or(after.len());
        let name = &after[..name_len];
        let local_name = name.rsplit(':').next().unwrap_or(name);
        if local_name == local {
            return Some(start);
        }
        offset = start + 2;
    }
    None
}

fn all_element_texts(xml: &str, local: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut offset = 0;
    while offset < xml.len() {
        let Some(rel) = xml[offset..].find(local) else {
            break;
        };
        let idx = offset + rel;
        let prev = xml[..idx].chars().next_back();
        let after = &xml[idx + local.len()..];
        let next = after.chars().next();
        if matches!(prev, Some('<' | ':'))
            && matches!(next, Some('>' | '/' | ' ' | '\t' | '\n' | '\r'))
        {
            if let Some(content_at) = after.find('>') {
                let content = &after[content_at + 1..];
                if let Some(close_at) = find_close(content, local) {
                    let text = content[..close_at].trim();
                    if !text.is_empty() && !text.contains('<') {
                        items.push(xml_unescape(text));
                    }
                    offset = idx + local.len() + content_at + 1 + close_at;
                    continue;
                }
            }
        }
        offset = idx + local.len();
    }
    items
}

fn attr_value(xml: &str, local: &str) -> Option<String> {
    let key = format!("{local}=\"");
    let mut search_from = 0;
    while let Some(rel) = xml[search_from..].find(&key) {
        let idx = search_from + rel;
        let prev = xml[..idx].chars().next_back();
        if matches!(prev, Some(':' | ' ' | '\t' | '\n' | '\r')) {
            let val_start = idx + key.len();
            let val_end = xml[val_start..].find('"')?;
            return Some(xml_unescape(&xml[val_start..val_start + val_end]));
        }
        search_from = idx + key.len();
    }
    None
}

fn xmp_creator(xml: &str) -> Option<String> {
    if let Some(block) = element_inner(xml, "creator") {
        let items = all_element_texts(block, "li");
        if !items.is_empty() {
            return Some(items.join(", "));
        }
        let flat = block.trim();
        if !flat.is_empty() && !flat.contains('<') {
            return Some(xml_unescape(flat));
        }
    }
    attr_value(xml, "creator")
}

fn xmp_creator_tool(xml: &str) -> Option<String> {
    if let Some(block) = element_inner(xml, "CreatorTool") {
        let flat = block.trim();
        if !flat.is_empty() && !flat.contains('<') {
            return Some(xml_unescape(flat));
        }
    }
    attr_value(xml, "CreatorTool")
}

fn photoshop_payload(blocks: &[IrbBlock]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(PS_HEADER);
    payload.extend_from_slice(&write_irb(blocks));
    payload
}

fn irb_blocks(segment: &[u8]) -> Vec<IrbBlock> {
    let Some(payload) = segment_payload(segment) else {
        return Vec::new();
    };
    let Some(irb) = payload.strip_prefix(PS_HEADER) else {
        return Vec::new();
    };
    parse_irb(irb)
}

fn parse_irb(data: &[u8]) -> Vec<IrbBlock> {
    let mut pos = 0;
    let mut blocks = Vec::new();
    while pos + 8 <= data.len() && data[pos..].starts_with(b"8BIM") {
        let id = u16::from_be_bytes([data[pos + 4], data[pos + 5]]);
        let name_len = data[pos + 6] as usize;
        let mut name_size = 1 + name_len;
        if name_size % 2 == 1 {
            name_size += 1;
        }
        let size_at = pos + 6 + name_size;
        if size_at + 4 > data.len() {
            break;
        }
        let data_len = u32::from_be_bytes(data[size_at..size_at + 4].try_into().unwrap()) as usize;
        let data_at = size_at + 4;
        if data_at + data_len > data.len() {
            break;
        }
        blocks.push(IrbBlock {
            id,
            name: data[pos + 6..size_at].to_vec(),
            data: data[data_at..data_at + data_len].to_vec(),
        });
        let mut next = data_at + data_len;
        if data_len % 2 == 1 {
            next += 1;
        }
        pos = next;
    }
    blocks
}

fn write_irb(blocks: &[IrbBlock]) -> Vec<u8> {
    let mut out = Vec::new();
    for block in blocks {
        out.extend_from_slice(b"8BIM");
        out.extend_from_slice(&block.id.to_be_bytes());
        if block.name.is_empty() {
            out.extend_from_slice(&[0, 0]);
        } else {
            out.extend_from_slice(&block.name);
        }
        out.extend_from_slice(&(block.data.len() as u32).to_be_bytes());
        out.extend_from_slice(&block.data);
        if block.data.len() % 2 == 1 {
            out.push(0);
        }
    }
    out
}

fn upsert_iptc_byline(blocks: &mut Vec<IrbBlock>, by_line: &str) -> Result<(), ExifError> {
    if by_line.len() > 32767 {
        return Err(ExifError::InvalidExif(
            "IPTC By-line is too long".to_string(),
        ));
    }
    let iim = if let Some(block) = blocks.iter().find(|block| block.id == 0x0404) {
        upsert_byline_dataset(&block.data, by_line)
    } else {
        upsert_byline_dataset(&[], by_line)
    };
    if let Some(block) = blocks.iter_mut().find(|block| block.id == 0x0404) {
        block.data = iim;
    } else {
        blocks.push(IrbBlock {
            id: 0x0404,
            name: vec![0, 0],
            data: iim,
        });
    }
    Ok(())
}

fn upsert_byline_dataset(iim: &[u8], by_line: &str) -> Vec<u8> {
    let mut sets = parse_iim(iim);
    let bytes = by_line.as_bytes().to_vec();
    if let Some(existing) = sets
        .iter_mut()
        .find(|set| set.record == 2 && set.dataset == 80)
    {
        existing.data = bytes;
    } else {
        sets.push(IptcDataset {
            record: 2,
            dataset: 80,
            data: bytes,
        });
    }
    let mut prefix = Vec::new();
    if !sets.iter().any(|set| set.record == 1 && set.dataset == 0) {
        prefix.push(IptcDataset {
            record: 1,
            dataset: 0,
            data: vec![0, 4],
        });
    }
    if !sets.iter().any(|set| set.record == 1 && set.dataset == 90) {
        prefix.push(IptcDataset {
            record: 1,
            dataset: 90,
            data: vec![0x1B, 0x25, 0x47],
        });
    }
    if !sets.iter().any(|set| set.record == 2 && set.dataset == 0) {
        prefix.push(IptcDataset {
            record: 2,
            dataset: 0,
            data: vec![0, 4],
        });
    }
    prefix.append(&mut sets);
    write_iim(&prefix)
}

fn parse_iim(data: &[u8]) -> Vec<IptcDataset> {
    let mut pos = 0;
    let mut sets = Vec::new();
    while pos + 5 <= data.len() {
        if data[pos] != 0x1C {
            break;
        }
        let record = data[pos + 1];
        let dataset = data[pos + 2];
        let len_word = u16::from_be_bytes([data[pos + 3], data[pos + 4]]);
        pos += 5;
        let data_len = if len_word & 0x8000 != 0 {
            let nbytes = (len_word & 0x7FFF) as usize;
            if nbytes == 0 || pos + nbytes > data.len() {
                break;
            }
            let mut value = 0usize;
            for _ in 0..nbytes {
                value = (value << 8) | data[pos] as usize;
                pos += 1;
            }
            value
        } else {
            len_word as usize
        };
        if pos + data_len > data.len() {
            break;
        }
        sets.push(IptcDataset {
            record,
            dataset,
            data: data[pos..pos + data_len].to_vec(),
        });
        pos += data_len;
    }
    sets
}

fn write_iim(sets: &[IptcDataset]) -> Vec<u8> {
    let mut out = Vec::new();
    for set in sets {
        out.push(0x1C);
        out.push(set.record);
        out.push(set.dataset);
        if set.data.len() > 32767 {
            out.extend_from_slice(&0x8004u16.to_be_bytes());
            out.extend_from_slice(&(set.data.len() as u32).to_be_bytes());
        } else {
            out.extend_from_slice(&(set.data.len() as u16).to_be_bytes());
        }
        out.extend_from_slice(&set.data);
    }
    out
}

fn iptc_byline(iim: &[u8]) -> Option<String> {
    let sets = parse_iim(iim);
    let set = sets
        .iter()
        .rev()
        .find(|set| set.record == 2 && set.dataset == 80)?;
    let end = set
        .data
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(set.data.len());
    let text = String::from_utf8_lossy(&set.data[..end]);
    let text = text.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shop_meta() -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        let when = "2024:06:15 08:30:00";
        metadata.insert("DateTimeOriginal".to_string(), when.to_string());
        metadata.insert("CreateDate".to_string(), when.to_string());
        metadata.insert("ModifyDate".to_string(), when.to_string());
        metadata.insert("Make".to_string(), "LaView".to_string());
        metadata.insert("Model".to_string(), "LV-T9708MHS".to_string());
        metadata.insert("Artist".to_string(), "gate-3".to_string());
        metadata.insert("IPTC:By-line".to_string(), "A&B <cam>".to_string());
        metadata.insert("XMP-dc:Creator".to_string(), "A&B <cam>".to_string());
        metadata.insert("XMP-xmp:CreatorTool".to_string(), "shop-nvr".to_string());
        metadata.insert("XPAuthor".to_string(), "gate-3".to_string());
        metadata.insert("XPComment".to_string(), "Z".to_string());
        metadata
    }

    #[test]
    fn shop_nvr_tags_roundtrip_jpeg() {
        let jpeg = [
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0, 1, 1, 0, 0, 1, 0, 1, 0,
            0, 0xFF, 0xDA, 0x00, 0x02, 0x11, 0x22, 0x33, 0xFF, 0xD9,
        ];
        let written = write_jpeg_metadata(&jpeg, true, &shop_meta()).unwrap();
        assert!(written.windows(5).any(|window| window == b"JFIF\0"));
        assert!(written
            .windows(3)
            .any(|window| window == [0x11, 0x22, 0x33]));

        let mut reader = crate::FastExifReader::new();
        let back = reader.read_bytes(&written).unwrap();
        assert_eq!(back.get("Make").map(String::as_str), Some("LaView"));
        assert_eq!(back.get("Model").map(String::as_str), Some("LV-T9708MHS"));
        assert_eq!(back.get("Artist").map(String::as_str), Some("gate-3"));
        assert_eq!(
            back.get("DateTimeOriginal").map(String::as_str),
            Some("2024:06:15 08:30:00")
        );
        assert_eq!(
            back.get("CreateDate").map(String::as_str),
            Some("2024:06:15 08:30:00")
        );
        assert_eq!(
            back.get("ModifyDate").map(String::as_str),
            Some("2024:06:15 08:30:00")
        );
        assert_eq!(back.get("XPAuthor").map(String::as_str), Some("gate-3"));
        assert_eq!(back.get("XPComment").map(String::as_str), Some("Z"));
        assert_eq!(
            back.get("IPTC:By-line").map(String::as_str),
            Some("A&B <cam>")
        );
        assert_eq!(
            back.get("XMP-dc:Creator").map(String::as_str),
            Some("A&B <cam>")
        );
        assert_eq!(
            back.get("XMP-xmp:CreatorTool").map(String::as_str),
            Some("shop-nvr")
        );
    }

    #[test]
    fn iptc_byline_keeps_other_datasets() {
        let caption = upsert_byline_dataset(&[], "old");
        let mut sets = parse_iim(&caption);
        sets.push(IptcDataset {
            record: 2,
            dataset: 120,
            data: b"hello-cap".to_vec(),
        });
        let iim = write_iim(&sets);
        let block = IrbBlock {
            id: 0x0404,
            name: vec![0, 0],
            data: iim,
        };
        let app13 = wrap_marker(0xED, &photoshop_payload(&[block])).unwrap();
        let mut jpeg = vec![0xFF, 0xD8];
        jpeg.extend_from_slice(&app13);
        jpeg.extend_from_slice(&[0xFF, 0xD9]);

        let mut metadata = HashMap::new();
        metadata.insert("IPTC:By-line".to_string(), "gate-3".to_string());
        let written = write_jpeg_metadata(&jpeg, true, &metadata).unwrap();
        assert!(written.windows(9).any(|window| window == b"hello-cap"));

        let mut reader = crate::FastExifReader::new();
        let back = reader.read_bytes(&written).unwrap();
        assert_eq!(back.get("IPTC:By-line").map(String::as_str), Some("gate-3"));
    }
}
