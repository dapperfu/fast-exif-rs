use fast_exif_reader::FastExifReader;
use std::path::Path;

fn be_box(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + payload.len());
    out.extend_from_slice(&((8 + payload.len()) as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(payload);
    out
}

/// Little-endian TIFF whose only IFD0 tag is ASCII Make.
fn tiff_make(make: &str) -> Vec<u8> {
    let mut text = make.as_bytes().to_vec();
    text.push(0);
    let mut tiff = Vec::new();
    tiff.extend_from_slice(b"II*\0");
    tiff.extend_from_slice(&8u32.to_le_bytes());
    tiff.extend_from_slice(&1u16.to_le_bytes());
    tiff.extend_from_slice(&0x010Fu16.to_le_bytes());
    tiff.extend_from_slice(&2u16.to_le_bytes());
    tiff.extend_from_slice(&(text.len() as u32).to_le_bytes());
    tiff.extend_from_slice(&26u32.to_le_bytes());
    tiff.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(tiff.len(), 26);
    tiff.extend_from_slice(&text);
    tiff
}

fn exif_item(tiff: &[u8]) -> Vec<u8> {
    let mut item = Vec::new();
    item.extend_from_slice(&6u32.to_be_bytes());
    item.extend_from_slice(b"Exif\0\0");
    item.extend_from_slice(tiff);
    item
}

fn infe(id: u16, kind: &[u8; 4]) -> Vec<u8> {
    let mut payload = vec![2, 0, 0, 0];
    payload.extend_from_slice(&id.to_be_bytes());
    payload.extend_from_slice(&0u16.to_be_bytes());
    payload.extend_from_slice(kind);
    payload.push(0);
    be_box(b"infe", &payload)
}

fn iinf(entries: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = vec![0, 0, 0, 0];
    payload.extend_from_slice(&(entries.len() as u16).to_be_bytes());
    for entry in entries {
        payload.extend_from_slice(entry);
    }
    be_box(b"iinf", &payload)
}

fn iloc(id: u16, construction: u16, offset: u32, len: u32) -> Vec<u8> {
    let mut payload = vec![1, 0, 0, 0, 0x44, 0x00];
    payload.extend_from_slice(&1u16.to_be_bytes());
    payload.extend_from_slice(&id.to_be_bytes());
    payload.extend_from_slice(&construction.to_be_bytes());
    payload.extend_from_slice(&0u16.to_be_bytes());
    payload.extend_from_slice(&1u16.to_be_bytes());
    payload.extend_from_slice(&offset.to_be_bytes());
    payload.extend_from_slice(&len.to_be_bytes());
    be_box(b"iloc", &payload)
}

fn hdlr_pict() -> Vec<u8> {
    let mut payload = vec![0u8; 8];
    payload.extend_from_slice(b"pict");
    payload.extend_from_slice(&[0u8; 12]);
    be_box(b"hdlr", &payload)
}

fn pitm(id: u16) -> Vec<u8> {
    let mut payload = vec![0, 0, 0, 0];
    payload.extend_from_slice(&id.to_be_bytes());
    be_box(b"pitm", &payload)
}

fn ispe(width: u32, height: u32) -> Vec<u8> {
    let mut payload = vec![0, 0, 0, 0];
    payload.extend_from_slice(&width.to_be_bytes());
    payload.extend_from_slice(&height.to_be_bytes());
    be_box(b"ispe", &payload)
}

fn iprp(width: u32, height: u32) -> Vec<u8> {
    let ipco = be_box(
        b"ipco",
        &[ispe(width, height), be_box(b"irot", &[3])].concat(),
    );
    let mut ipma = vec![0, 0, 0, 0];
    ipma.extend_from_slice(&1u32.to_be_bytes());
    ipma.extend_from_slice(&1u16.to_be_bytes());
    ipma.push(2);
    ipma.push(0x81);
    ipma.push(0x82);
    be_box(b"iprp", &[ipco, be_box(b"ipma", &ipma)].concat())
}

fn meta(parts: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = vec![0, 0, 0, 0];
    for part in parts {
        payload.extend_from_slice(part);
    }
    be_box(b"meta", &payload)
}

/// Samsung-style layout: `mdat` first, `meta` after it, Exif item inside `mdat`.
fn heic_exif_in_mdat(make: &str) -> Vec<u8> {
    let exif = exif_item(&tiff_make(make));
    let mut mdat_payload = vec![0u8; 16];
    mdat_payload.extend_from_slice(&exif);
    let ftyp = be_box(b"ftyp", b"heic\0\0\0\0mif1heic");
    let mdat = be_box(b"mdat", &mdat_payload);
    let exif_off = (ftyp.len() + 8 + 16) as u32;
    let meta = meta(&[
        hdlr_pict(),
        pitm(1),
        iinf(&[infe(1, b"grid"), infe(2, b"Exif")]),
        iprp(3024, 3024),
        iloc(2, 0, exif_off, exif.len() as u32),
    ]);
    let mut file = ftyp;
    file.extend(mdat);
    file.extend(meta);
    file
}

/// Exif item stored in `idat` (construction_method = 1), not in `mdat`.
fn heic_exif_in_idat(make: &str) -> Vec<u8> {
    let exif = exif_item(&tiff_make(make));
    let ftyp = be_box(b"ftyp", b"heic\0\0\0\0mif1heic");
    let mdat = be_box(b"mdat", &[0u8; 32]);
    let idat = be_box(b"idat", &exif);
    let meta = meta(&[
        hdlr_pict(),
        pitm(1),
        iinf(&[infe(1, b"grid"), infe(2, b"Exif")]),
        iprp(64, 48),
        iloc(2, 1, 0, exif.len() as u32),
        idat,
    ]);
    let mut file = ftyp;
    file.extend(mdat);
    file.extend(meta);
    file
}

#[test]
fn heic_reads_exif_item_that_follows_mdat() {
    let tags = FastExifReader::new()
        .read_bytes(&heic_exif_in_mdat("samsung"))
        .unwrap();
    assert_eq!(tags.get("Make").map(String::as_str), Some("samsung"));
    assert_eq!(tags.get("Format").map(String::as_str), Some("HEIC"));
    assert_eq!(tags.get("FileType").map(String::as_str), Some("HEIC"));
    assert_eq!(tags.get("MIMEType").map(String::as_str), Some("image/heic"));
    assert_eq!(tags.get("ImageWidth").map(String::as_str), Some("3024"));
    assert_eq!(tags.get("ImageHeight").map(String::as_str), Some("3024"));
    assert_eq!(tags.get("Rotation").map(String::as_str), Some("270"));
    assert_eq!(tags.get("HandlerType").map(String::as_str), Some("Picture"));
    assert_eq!(
        tags.get("PrimaryItemReference").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        tags.get("CompatibleBrands").map(String::as_str),
        Some("mif1, heic")
    );
    assert_ne!(
        tags.get("File:MIMEType").map(String::as_str),
        Some("image/jpeg")
    );
    assert!(tags.get("BlueBalance").is_none());
}

#[test]
fn heic_reads_exif_item_stored_in_idat() {
    let tags = FastExifReader::new()
        .read_bytes(&heic_exif_in_idat("samsung"))
        .unwrap();
    assert_eq!(tags.get("Make").map(String::as_str), Some("samsung"));
    assert_eq!(tags.get("MIMEType").map(String::as_str), Some("image/heic"));
    assert_eq!(tags.get("ImageWidth").map(String::as_str), Some("64"));
    assert_eq!(tags.get("Rotation").map(String::as_str), Some("270"));
}

#[test]
fn samsung_heic_matches_exiftool_camera_tags() {
    let path = "/keg/jed/Desktop/Camera/20240315_171931.heic";
    if !Path::new(path).exists() {
        return;
    }
    let tags = FastExifReader::new().read_file(path).unwrap();
    assert_eq!(tags.get("Make").map(String::as_str), Some("samsung"));
    assert_eq!(tags.get("Model").map(String::as_str), Some("SM-G970U"));
    assert_eq!(
        tags.get("DateTimeOriginal").map(String::as_str),
        Some("2024:03:15 17:19:31")
    );
    assert_eq!(
        tags.get("ModifyDate").map(String::as_str),
        Some("2024:03:15 17:19:31")
    );
    assert_eq!(tags.get("ExposureTime").map(String::as_str), Some("1/120"));
    assert_eq!(tags.get("FNumber").map(String::as_str), Some("2.4"));
    assert_eq!(tags.get("ISO").map(String::as_str), Some("200"));
    assert_eq!(
        tags.get("GPSLatitudeRef").map(String::as_str),
        Some("North")
    );
    assert_eq!(
        tags.get("GPSLongitudeRef").map(String::as_str),
        Some("West")
    );
    assert_eq!(tags.get("MIMEType").map(String::as_str), Some("image/heic"));
    assert_eq!(tags.get("FileType").map(String::as_str), Some("HEIC"));
    assert_eq!(tags.get("ImageWidth").map(String::as_str), Some("3024"));
    assert_eq!(tags.get("Rotation").map(String::as_str), Some("270"));
    assert_eq!(
        tags.get("PrimaryItemReference").map(String::as_str),
        Some("37")
    );
    assert!(
        tags.get("GPSLatitude")
            .map(|v| v.contains("43"))
            .unwrap_or(false),
        "GPSLatitude missing: {:?}",
        tags.get("GPSLatitude")
    );
    assert_ne!(
        tags.get("File:MIMEType").map(String::as_str),
        Some("image/jpeg")
    );
}
