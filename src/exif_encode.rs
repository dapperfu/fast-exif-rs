//! Encode EXIF metadata into a TIFF IFD tree (IFD0 + ExifIFD + optional GPS/Interop).

use crate::types::ExifError;
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum IfdKind {
    Ifd0,
    Exif,
    Gps,
    Interop,
}

struct TagSpec {
    names: &'static [&'static str],
    tag: u16,
    dtype: u16,
    kind: IfdKind,
}

#[derive(Clone)]
struct EncodedTag {
    tag: u16,
    dtype: u16,
    count: u32,
    data: Vec<u8>,
}

const TAGS: &[TagSpec] = &[
    TagSpec { names: &["ImageDescription"], tag: 0x010E, dtype: 2, kind: IfdKind::Ifd0 },
    TagSpec { names: &["Make"], tag: 0x010F, dtype: 2, kind: IfdKind::Ifd0 },
    TagSpec { names: &["Model", "CameraModelName"], tag: 0x0110, dtype: 2, kind: IfdKind::Ifd0 },
    TagSpec { names: &["Orientation"], tag: 0x0112, dtype: 3, kind: IfdKind::Ifd0 },
    TagSpec { names: &["XResolution"], tag: 0x011A, dtype: 5, kind: IfdKind::Ifd0 },
    TagSpec { names: &["YResolution"], tag: 0x011B, dtype: 5, kind: IfdKind::Ifd0 },
    TagSpec { names: &["ResolutionUnit"], tag: 0x0128, dtype: 3, kind: IfdKind::Ifd0 },
    TagSpec { names: &["Software"], tag: 0x0131, dtype: 2, kind: IfdKind::Ifd0 },
    TagSpec { names: &["DateTime", "ModifyDate"], tag: 0x0132, dtype: 2, kind: IfdKind::Ifd0 },
    TagSpec { names: &["Artist"], tag: 0x013B, dtype: 2, kind: IfdKind::Ifd0 },
    TagSpec { names: &["YCbCrPositioning"], tag: 0x0213, dtype: 3, kind: IfdKind::Ifd0 },
    TagSpec { names: &["Copyright"], tag: 0x8298, dtype: 2, kind: IfdKind::Ifd0 },
    TagSpec { names: &["Rating"], tag: 0x4746, dtype: 3, kind: IfdKind::Ifd0 },
    TagSpec { names: &["ExposureTime", "ShutterSpeed"], tag: 0x829A, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["FNumber", "Aperture"], tag: 0x829D, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["ExposureProgram"], tag: 0x8822, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["SpectralSensitivity"], tag: 0x8824, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["ISO", "ISOSpeedRatings", "ISOSpeed"], tag: 0x8827, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["SensitivityType"], tag: 0x8830, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["RecommendedExposureIndex"], tag: 0x8832, dtype: 4, kind: IfdKind::Exif },
    TagSpec { names: &["ExifVersion"], tag: 0x9000, dtype: 7, kind: IfdKind::Exif },
    TagSpec { names: &["DateTimeOriginal"], tag: 0x9003, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["DateTimeDigitized", "CreateDate", "DateTimeCreated"], tag: 0x9004, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["OffsetTime"], tag: 0x9010, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["OffsetTimeOriginal"], tag: 0x9011, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["OffsetTimeDigitized"], tag: 0x9012, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["ComponentsConfiguration"], tag: 0x9101, dtype: 7, kind: IfdKind::Exif },
    TagSpec { names: &["CompressedBitsPerPixel"], tag: 0x9102, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["ShutterSpeedValue"], tag: 0x9201, dtype: 10, kind: IfdKind::Exif },
    TagSpec { names: &["ApertureValue"], tag: 0x9202, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["BrightnessValue"], tag: 0x9203, dtype: 10, kind: IfdKind::Exif },
    TagSpec { names: &["ExposureCompensation", "ExposureBiasValue"], tag: 0x9204, dtype: 10, kind: IfdKind::Exif },
    TagSpec { names: &["MaxApertureValue"], tag: 0x9205, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["SubjectDistance"], tag: 0x9206, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["MeteringMode"], tag: 0x9207, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["LightSource"], tag: 0x9208, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["Flash"], tag: 0x9209, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["FocalLength"], tag: 0x920A, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["UserComment"], tag: 0x9286, dtype: 7, kind: IfdKind::Exif },
    TagSpec { names: &["SubSecTime"], tag: 0x9290, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["SubSecTimeOriginal"], tag: 0x9291, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["SubSecTimeDigitized"], tag: 0x9292, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["FlashpixVersion"], tag: 0xA000, dtype: 7, kind: IfdKind::Exif },
    TagSpec { names: &["ColorSpace"], tag: 0xA001, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["ExifImageWidth", "PixelXDimension"], tag: 0xA002, dtype: 4, kind: IfdKind::Exif },
    TagSpec { names: &["ExifImageHeight", "PixelYDimension"], tag: 0xA003, dtype: 4, kind: IfdKind::Exif },
    TagSpec { names: &["RelatedSoundFile"], tag: 0xA004, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["SensingMethod"], tag: 0xA217, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["FileSource"], tag: 0xA300, dtype: 7, kind: IfdKind::Exif },
    TagSpec { names: &["SceneType"], tag: 0xA301, dtype: 7, kind: IfdKind::Exif },
    TagSpec { names: &["CustomRendered"], tag: 0xA401, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["ExposureMode"], tag: 0xA402, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["WhiteBalance"], tag: 0xA403, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["DigitalZoomRatio"], tag: 0xA404, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["FocalLengthIn35mmFormat", "FocalLengthIn35mmFilm"], tag: 0xA405, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["SceneCaptureType"], tag: 0xA406, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["GainControl"], tag: 0xA407, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["Contrast"], tag: 0xA408, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["Saturation"], tag: 0xA409, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["Sharpness"], tag: 0xA40A, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["SubjectDistanceRange"], tag: 0xA40C, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["ImageUniqueID"], tag: 0xA420, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["CameraOwnerName"], tag: 0xA430, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["SerialNumber", "BodySerialNumber"], tag: 0xA431, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["LensInfo", "Lens", "LensSpecification"], tag: 0xA432, dtype: 5, kind: IfdKind::Exif },
    TagSpec { names: &["LensMake"], tag: 0xA433, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["LensModel"], tag: 0xA434, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["LensSerialNumber"], tag: 0xA435, dtype: 2, kind: IfdKind::Exif },
    TagSpec { names: &["CompositeImage"], tag: 0xA460, dtype: 3, kind: IfdKind::Exif },
    TagSpec { names: &["GPSVersionID"], tag: 0x0000, dtype: 1, kind: IfdKind::Gps },
    TagSpec { names: &["GPSLatitudeRef"], tag: 0x0001, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSLatitude"], tag: 0x0002, dtype: 5, kind: IfdKind::Gps },
    TagSpec { names: &["GPSLongitudeRef"], tag: 0x0003, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSLongitude"], tag: 0x0004, dtype: 5, kind: IfdKind::Gps },
    TagSpec { names: &["GPSAltitudeRef"], tag: 0x0005, dtype: 1, kind: IfdKind::Gps },
    TagSpec { names: &["GPSAltitude"], tag: 0x0006, dtype: 5, kind: IfdKind::Gps },
    TagSpec { names: &["GPSTimeStamp"], tag: 0x0007, dtype: 5, kind: IfdKind::Gps },
    TagSpec { names: &["GPSSatellites"], tag: 0x0008, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSStatus"], tag: 0x0009, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSMeasureMode"], tag: 0x000A, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSDOP"], tag: 0x000B, dtype: 5, kind: IfdKind::Gps },
    TagSpec { names: &["GPSSpeedRef"], tag: 0x000C, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSSpeed"], tag: 0x000D, dtype: 5, kind: IfdKind::Gps },
    TagSpec { names: &["GPSTrackRef"], tag: 0x000E, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSTrack"], tag: 0x000F, dtype: 5, kind: IfdKind::Gps },
    TagSpec { names: &["GPSImgDirectionRef"], tag: 0x0010, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSImgDirection"], tag: 0x0011, dtype: 5, kind: IfdKind::Gps },
    TagSpec { names: &["GPSMapDatum"], tag: 0x0012, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSDateStamp"], tag: 0x001D, dtype: 2, kind: IfdKind::Gps },
    TagSpec { names: &["GPSDifferential"], tag: 0x001E, dtype: 3, kind: IfdKind::Gps },
    TagSpec { names: &["InteropIndex"], tag: 0x0001, dtype: 2, kind: IfdKind::Interop },
    TagSpec { names: &["InteropVersion"], tag: 0x0002, dtype: 7, kind: IfdKind::Interop },
];

fn strip_units(value: &str) -> String {
    let mut s = value.trim().to_string();
    for suffix in [" mm", "mm", " m", " inches", " deg"] {
        if let Some(stripped) = s.strip_suffix(suffix) {
            s = stripped.trim().to_string();
        }
    }
    s
}

fn parse_u16_enum(tag: u16, value: &str) -> Option<u16> {
    let cleaned = strip_units(value);
    if let Ok(n) = cleaned.parse::<u16>() {
        return Some(n);
    }
    let v = cleaned.to_lowercase();
    match tag {
        0x0112 => match v.as_str() {
            "horizontal (normal)" | "normal" => Some(1),
            "mirror horizontal" => Some(2),
            "rotate 180" => Some(3),
            "mirror vertical" => Some(4),
            "rotate 90 cw" => Some(6),
            "rotate 270 cw" => Some(8),
            _ => None,
        },
        0x0128 => match v.as_str() {
            "none" => Some(1),
            "inches" => Some(2),
            "centimeters" | "cm" => Some(3),
            _ => None,
        },
        0x0213 => match v.as_str() {
            "centered" => Some(1),
            "co-sited" | "cosited" => Some(2),
            _ => None,
        },
        0x8822 => match v.as_str() {
            "not defined" | "unknown" => Some(0),
            "manual" => Some(1),
            "program" | "program ae" | "normal program" => Some(2),
            "aperture-priority" | "aperture priority" | "aperture-priority ae" => Some(3),
            "shutter-priority" | "shutter priority" | "shutter-priority ae" => Some(4),
            "creative" | "program creative" => Some(5),
            "action" | "program action" => Some(6),
            "portrait" | "portrait mode" => Some(7),
            "landscape" | "landscape mode" => Some(8),
            _ => None,
        },
        0x8830 => match v.as_str() {
            "unknown" => Some(0),
            "standard output sensitivity" => Some(1),
            "recommended exposure index" => Some(2),
            "iso speed" => Some(3),
            _ => None,
        },
        0x9207 => match v.as_str() {
            "unknown" => Some(0),
            "average" => Some(1),
            "center-weighted average" => Some(2),
            "spot" => Some(3),
            "multi-spot" => Some(4),
            "multi-segment" | "pattern" | "evaluative" => Some(5),
            "partial" => Some(6),
            "other" => Some(255),
            _ => None,
        },
        0x9208 => match v.as_str() {
            "unknown" => Some(0),
            "daylight" => Some(1),
            "fluorescent" => Some(2),
            "tungsten" => Some(3),
            "flash" => Some(4),
            _ => None,
        },
        0x9209 => match v.as_str() {
            "off, did not fire" | "off" | "no flash" | "did not fire" => Some(16),
            "fired" | "on" => Some(1),
            _ => None,
        },
        0xA001 => match v.as_str() {
            "srgb" => Some(1),
            "adobe rgb" | "uncalibrated" => Some(65535),
            _ => None,
        },
        0xA217 => match v.as_str() {
            "not defined" => Some(1),
            "one-chip color area" | "one-chip color area sensor" => Some(2),
            "two-chip color area" => Some(3),
            "three-chip color area" => Some(4),
            _ => None,
        },
        0xA401 => match v.as_str() {
            "normal" => Some(0),
            "custom" => Some(1),
            _ => None,
        },
        0xA402 => match v.as_str() {
            "auto" => Some(0),
            "manual" => Some(1),
            "auto bracket" => Some(2),
            _ => None,
        },
        0xA403 => match v.as_str() {
            "auto" | "auto1" | "natural auto" | "auto white balance" => Some(0),
            "manual" => Some(1),
            _ if v.contains("auto") => Some(0),
            _ => None,
        },
        0xA406 => match v.as_str() {
            "standard" => Some(0),
            "landscape" => Some(1),
            "portrait" => Some(2),
            "night scene" | "night" => Some(3),
            _ => None,
        },
        0xA407 => match v.as_str() {
            "none" => Some(0),
            "low gain up" => Some(1),
            "high gain up" => Some(2),
            "low gain down" => Some(3),
            "high gain down" => Some(4),
            _ => None,
        },
        0xA408 | 0xA409 | 0xA40A => match v.as_str() {
            "normal" | "soft" | "low" => Some(0),
            "hard" | "high" => Some(1),
            _ => None,
        },
        0xA40C => match v.as_str() {
            "unknown" => Some(0),
            "macro" => Some(1),
            "close" | "close view" => Some(2),
            "distant" | "distant view" => Some(3),
            _ => None,
        },
        0xA460 => match v.as_str() {
            "unknown" => Some(0),
            "not a composite image" => Some(1),
            "general composite image" => Some(2),
            "composite image captured while shooting" => Some(3),
            _ => None,
        },
        _ => None,
    }
}

fn parse_rational_parts(value: &str) -> Option<(u32, u32)> {
    let cleaned = strip_units(value);
    if let Some((n, d)) = cleaned.split_once('/') {
        let numerator = n.trim().parse::<u32>().ok()?;
        let denominator = d.trim().parse::<u32>().ok()?;
        return Some((numerator, denominator));
    }
    let float_value = cleaned.parse::<f64>().ok()?;
    if float_value.fract() == 0.0 && float_value >= 0.0 {
        Some((float_value as u32, 1))
    } else {
        Some(((float_value * 1_000_000.0).round() as u32, 1_000_000))
    }
}

fn parse_srational_parts(value: &str) -> Option<(i32, i32)> {
    let cleaned = strip_units(value);
    if let Some((n, d)) = cleaned.split_once('/') {
        let numerator = n.trim().parse::<i32>().ok()?;
        let denominator = d.trim().parse::<i32>().ok()?;
        return Some((numerator, denominator));
    }
    let float_value = cleaned.parse::<f64>().ok()?;
    if float_value.fract() == 0.0 {
        Some((float_value as i32, 1))
    } else {
        Some(((float_value * 1_000_000.0).round() as i32, 1_000_000))
    }
}

fn write_u32_bytes(little_endian: bool, value: u32) -> [u8; 4] {
    if little_endian {
        value.to_le_bytes()
    } else {
        value.to_be_bytes()
    }
}

fn write_u16_bytes(little_endian: bool, value: u16) -> [u8; 2] {
    if little_endian {
        value.to_le_bytes()
    } else {
        value.to_be_bytes()
    }
}

fn ascii_version_bytes(value: &str) -> Option<Vec<u8>> {
    if let Ok(n) = value.parse::<u32>() {
        let bytes = n.to_le_bytes();
        if bytes.iter().all(|b| b.is_ascii_digit()) {
            return Some(bytes.to_vec());
        }
    }
    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        Some(digits.as_bytes()[..4].to_vec())
    } else if value.len() == 4 {
        Some(value.as_bytes().to_vec())
    } else {
        Some(value.as_bytes().to_vec())
    }
}

fn encode_undefined(tag: u16, value: &str) -> Option<Vec<u8>> {
    match tag {
        0x9000 | 0xA000 | 0x0002 => ascii_version_bytes(value),
        0x9101 => {
            if let Ok(n) = value.parse::<u32>() {
                return Some(n.to_le_bytes().to_vec());
            }
            let v = value.to_lowercase();
            if v.contains("y") && v.contains("cb") {
                Some(vec![1, 2, 3, 0])
            } else {
                let parts: Vec<u8> = value
                    .split(|c: char| !c.is_ascii_digit())
                    .filter_map(|p| p.parse::<u8>().ok())
                    .collect();
                if parts.len() == 4 {
                    Some(parts)
                } else {
                    None
                }
            }
        }
        0xA300 => match value.to_lowercase().as_str() {
            "film scanner" => Some(vec![1]),
            "reflection print scanner" => Some(vec![2]),
            "digital camera" | "dsc" => Some(vec![3]),
            _ => value.parse::<u8>().ok().map(|n| vec![n]),
        },
        0xA301 => match value.to_lowercase().as_str() {
            "directly photographed" => Some(vec![1]),
            _ => value.parse::<u8>().ok().map(|n| vec![n]),
        },
        0x9286 => {
            if value.trim().is_empty() || value.trim().parse::<i64>().is_ok() {
                return None;
            }
            let mut out = b"ASCII\0\0\0".to_vec();
            out.extend_from_slice(value.as_bytes());
            Some(out)
        }
        _ => Some(value.as_bytes().to_vec()),
    }
}

fn encode_lens_info(little_endian: bool, value: &str) -> Option<Vec<u8>> {
    let nums: Vec<f64> = value
        .replace("mm", " ")
        .replace('f', " ")
        .replace('/', " ")
        .replace('-', " ")
        .split_whitespace()
        .filter_map(|p| p.parse::<f64>().ok())
        .collect();
    if nums.len() < 4 {
        return None;
    }
    let mut out = Vec::with_capacity(32);
    for n in &nums[..4] {
        let (num, den) = if n.fract() == 0.0 {
            (*n as u32, 1u32)
        } else {
            ((*n * 10.0).round() as u32, 10u32)
        };
        if little_endian {
            out.write_u32::<LittleEndian>(num).ok()?;
            out.write_u32::<LittleEndian>(den).ok()?;
        } else {
            out.write_u32::<BigEndian>(num).ok()?;
            out.write_u32::<BigEndian>(den).ok()?;
        }
    }
    Some(out)
}

fn encode_gps_bytes(value: &str) -> Option<Vec<u8>> {
    if let Ok(n) = value.parse::<u8>() {
        return Some(vec![n]);
    }
    let parts: Vec<u8> = value
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|p| if p.is_empty() { None } else { p.parse::<u8>().ok() })
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts)
    }
}

fn encode_rationals(little_endian: bool, value: &str, signed: bool) -> Option<(u32, Vec<u8>)> {
    let cleaned = strip_units(value);
    let owned: Vec<String> = if cleaned.contains(',') {
        cleaned
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if cleaned.matches('/').count() >= 2 {
        cleaned.split_whitespace().map(|s| s.to_string()).collect()
    } else {
        vec![cleaned]
    };

    let mut out = Vec::new();
    let mut count = 0u32;
    for chunk in &owned {
        if chunk.is_empty() {
            continue;
        }
        if signed {
            let (n, d) = parse_srational_parts(chunk)?;
            if little_endian {
                out.write_i32::<LittleEndian>(n).ok()?;
                out.write_i32::<LittleEndian>(d).ok()?;
            } else {
                out.write_i32::<BigEndian>(n).ok()?;
                out.write_i32::<BigEndian>(d).ok()?;
            }
        } else {
            let (n, d) = parse_rational_parts(chunk)?;
            if little_endian {
                out.write_u32::<LittleEndian>(n).ok()?;
                out.write_u32::<LittleEndian>(d).ok()?;
            } else {
                out.write_u32::<BigEndian>(n).ok()?;
                out.write_u32::<BigEndian>(d).ok()?;
            }
        }
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some((count, out))
    }
}

fn encode_tag(little_endian: bool, spec: &TagSpec, value: &str) -> Option<EncodedTag> {
    match spec.dtype {
        1 => {
            let data = encode_gps_bytes(value)?;
            Some(EncodedTag { tag: spec.tag, dtype: 1, count: data.len() as u32, data })
        }
        2 => {
            let mut data = value.as_bytes().to_vec();
            data.push(0);
            Some(EncodedTag { tag: spec.tag, dtype: 2, count: data.len() as u32, data })
        }
        3 => {
            let n = parse_u16_enum(spec.tag, value)?;
            if matches!(spec.tag, 0xA408 | 0xA409 | 0xA40A) && n > 2 {
                return None;
            }
            Some(EncodedTag {
                tag: spec.tag,
                dtype: 3,
                count: 1,
                data: write_u16_bytes(little_endian, n).to_vec(),
            })
        }
        4 => {
            let cleaned = strip_units(value);
            let n = cleaned.parse::<u32>().ok()?;
            Some(EncodedTag {
                tag: spec.tag,
                dtype: 4,
                count: 1,
                data: write_u32_bytes(little_endian, n).to_vec(),
            })
        }
        5 => {
            if spec.tag == 0xA432 {
                let data = encode_lens_info(little_endian, value)?;
                return Some(EncodedTag { tag: spec.tag, dtype: 5, count: 4, data });
            }
            let (count, data) = encode_rationals(little_endian, value, false)?;
            Some(EncodedTag { tag: spec.tag, dtype: 5, count, data })
        }
        7 => {
            let data = encode_undefined(spec.tag, value)?;
            Some(EncodedTag { tag: spec.tag, dtype: 7, count: data.len() as u32, data })
        }
        10 => {
            let (count, data) = encode_rationals(little_endian, value, true)?;
            Some(EncodedTag { tag: spec.tag, dtype: 10, count, data })
        }
        _ => None,
    }
}

fn overflow_size(tags: &[EncodedTag]) -> usize {
    tags.iter().map(|t| if t.data.len() > 4 { t.data.len() } else { 0 }).sum()
}

fn ifd_dir_size(count: usize) -> usize {
    2 + 12 * count + 4
}

fn write_ifd(
    little_endian: bool,
    tags: &[EncodedTag],
    overflow_start: u32,
) -> Result<Vec<u8>, ExifError> {
    let mut buf = Vec::new();
    let count = tags.len() as u16;
    if little_endian {
        buf.write_u16::<LittleEndian>(count)?;
    } else {
        buf.write_u16::<BigEndian>(count)?;
    }

    let mut overflow = Vec::new();
    for tag in tags {
        if little_endian {
            buf.write_u16::<LittleEndian>(tag.tag)?;
            buf.write_u16::<LittleEndian>(tag.dtype)?;
            buf.write_u32::<LittleEndian>(tag.count)?;
        } else {
            buf.write_u16::<BigEndian>(tag.tag)?;
            buf.write_u16::<BigEndian>(tag.dtype)?;
            buf.write_u32::<BigEndian>(tag.count)?;
        }

        if tag.data.len() <= 4 {
            let mut padded = [0u8; 4];
            padded[..tag.data.len()].copy_from_slice(&tag.data);
            buf.extend_from_slice(&padded);
        } else {
            let offset = overflow_start + overflow.len() as u32;
            if little_endian {
                buf.write_u32::<LittleEndian>(offset)?;
            } else {
                buf.write_u32::<BigEndian>(offset)?;
            }
            overflow.extend_from_slice(&tag.data);
        }
    }

    if little_endian {
        buf.write_u32::<LittleEndian>(0)?;
    } else {
        buf.write_u32::<BigEndian>(0)?;
    }
    buf.extend_from_slice(&overflow);
    Ok(buf)
}

fn pointer_tag(little_endian: bool, tag: u16, offset: u32) -> EncodedTag {
    EncodedTag {
        tag,
        dtype: 4,
        count: 1,
        data: write_u32_bytes(little_endian, offset).to_vec(),
    }
}

/// Build a TIFF payload (starting at the byte-order marker) for the given metadata.
pub fn encode_tiff_exif(
    little_endian: bool,
    metadata: &HashMap<String, String>,
) -> Result<Vec<u8>, ExifError> {
    let mut ifd0 = Vec::new();
    let mut exif = Vec::new();
    let mut gps = Vec::new();
    let mut interop = Vec::new();

    for spec in TAGS {
        for name in spec.names {
            if let Some(value) = metadata.get(*name) {
                if value.trim().is_empty() {
                    continue;
                }
                if let Some(encoded) = encode_tag(little_endian, spec, value) {
                    match spec.kind {
                        IfdKind::Ifd0 => ifd0.push(encoded),
                        IfdKind::Exif => exif.push(encoded),
                        IfdKind::Gps => gps.push(encoded),
                        IfdKind::Interop => interop.push(encoded),
                    }
                    break;
                }
            }
        }
    }

    if ifd0.is_empty() && exif.is_empty() {
        return Err(ExifError::InvalidExif("No writable EXIF fields in metadata".to_string()));
    }

    let has_exif = !exif.is_empty() || !interop.is_empty();
    let has_gps = !gps.is_empty();
    let has_interop = !interop.is_empty();

    // Layout: header(8) IFD0+overflow Exif+overflow GPS+overflow Interop+overflow
    let n0 = ifd0.len() + usize::from(has_exif) + usize::from(has_gps);
    let ifd0_off = 8u32;
    let overflow0_off = ifd0_off + ifd_dir_size(n0) as u32;
    let exif_off = overflow0_off + overflow_size(&ifd0) as u32;
    let n_exif = exif.len() + usize::from(has_interop);
    let overflow_exif_off = if has_exif {
        exif_off + ifd_dir_size(n_exif) as u32
    } else {
        exif_off
    };
    let gps_off = overflow_exif_off + if has_exif { overflow_size(&exif) as u32 } else { 0 };
    let overflow_gps_off = if has_gps {
        gps_off + ifd_dir_size(gps.len()) as u32
    } else {
        gps_off
    };
    let interop_off = overflow_gps_off + if has_gps { overflow_size(&gps) as u32 } else { 0 };
    let overflow_interop_off = if has_interop {
        interop_off + ifd_dir_size(interop.len()) as u32
    } else {
        interop_off
    };

    if has_exif {
        ifd0.push(pointer_tag(little_endian, 0x8769, exif_off));
    }
    if has_gps {
        ifd0.push(pointer_tag(little_endian, 0x8825, gps_off));
    }
    if has_interop {
        exif.push(pointer_tag(little_endian, 0xA005, interop_off));
    }

    ifd0.sort_by_key(|t| t.tag);
    exif.sort_by_key(|t| t.tag);
    gps.sort_by_key(|t| t.tag);
    interop.sort_by_key(|t| t.tag);

    let mut tiff = Vec::new();
    if little_endian {
        tiff.extend_from_slice(b"II");
        tiff.write_u16::<LittleEndian>(42)?;
        tiff.write_u32::<LittleEndian>(ifd0_off)?;
    } else {
        tiff.extend_from_slice(b"MM");
        tiff.write_u16::<BigEndian>(42)?;
        tiff.write_u32::<BigEndian>(ifd0_off)?;
    }

    tiff.extend(write_ifd(little_endian, &ifd0, overflow0_off)?);
    if has_exif {
        tiff.extend(write_ifd(little_endian, &exif, overflow_exif_off)?);
    }
    if has_gps {
        tiff.extend(write_ifd(little_endian, &gps, overflow_gps_off)?);
    }
    if has_interop {
        tiff.extend(write_ifd(little_endian, &interop, overflow_interop_off)?);
    }

    let _ = overflow_interop_off;
    Ok(tiff)
}

/// Wrap TIFF EXIF in a JPEG APP1 segment.
pub fn encode_jpeg_app1(
    little_endian: bool,
    metadata: &HashMap<String, String>,
) -> Result<Vec<u8>, ExifError> {
    let tiff = encode_tiff_exif(little_endian, metadata)?;
    let payload_len = 2 + 6 + tiff.len();
    if payload_len > u16::MAX as usize {
        return Err(ExifError::InvalidExif(format!(
            "EXIF APP1 segment too large ({} bytes)",
            payload_len
        )));
    }

    let mut app1 = Vec::with_capacity(2 + payload_len);
    app1.write_u8(0xFF)?;
    app1.write_u8(0xE1)?;
    app1.write_u16::<BigEndian>(payload_len as u16)?;
    app1.extend_from_slice(b"Exif\0\0");
    app1.extend_from_slice(&tiff);
    Ok(app1)
}
