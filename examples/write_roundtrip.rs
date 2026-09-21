//! Write EXIF from a source image onto a no-EXIF JPEG and round-trip verify.
//!
//! ```text
//! cargo run --release --example write_roundtrip -- \
//!   ~/Desktop/DSC_3865.JPG ~/Desktop/DSC_3865_noexif.JPG /tmp/DSC_3865_written.JPG
//! ```

use fast_exif_reader::{FastExifReader, FastExifWriter};
use std::env;
use std::fs;

const SKIP_COMPARE: &[&str] = &[
    "SourceFile",
    "FileName",
    "Directory",
    "FileSize",
    "FileModifyDate",
    "FileAccessDate",
    "FileInodeChangeDate",
    "FilePermissions",
    "ThumbnailOffset",
    "ThumbnailLength",
    "ThumbnailImage",
    "PreviewImage",
    "MPImage3",
    "MPFVersion",
    "NumberOfImages",
    "MPImageFlags",
    "MPImageFormat",
    "MPImageType",
    "MPImageLength",
    "MPImageStart",
    "DependentImage1EntryNumber",
    "DependentImage2EntryNumber",
    "MakerNote",
    "MakerNoteVersion",
    "MakerNoteType",
    "NikonMakerNote",
    "CFAPattern",
    "UserComment",
    "CircleOfConfusion",
    "HyperfocalDistance",
    "LensSpecification",
    "EncodingProcess",
    "BitsPerSample",
    "ColorComponents",
    "YCbCrSubSampling",
    "JPEGQuality",
    "ExifToolVersion",
];

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: write_roundtrip <source-with-exif> <no-exif-base> <output.jpg>");
        std::process::exit(2);
    }
    if let Err(err) = run(&args[0], &args[1], &args[2]) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run(source: &str, base: &str, output: &str) -> Result<(), String> {
    fs::copy(base, output).map_err(|e| format!("copy {base} -> {output}: {e}"))?;

    let mut reader = FastExifReader::new();
    let source_meta = reader
        .read_file(source)
        .map_err(|e| format!("read source: {e}"))?;
    println!("source tags: {}", source_meta.len());

    let writer = FastExifWriter::new();
    writer
        .write_exif(base, output, &source_meta)
        .map_err(|e| format!("write: {e}"))?;

    let written_meta = reader
        .read_file(output)
        .map_err(|e| format!("read written: {e}"))?;
    println!("written tags: {}", written_meta.len());

    let mut matched = 0usize;
    let mut mismatched = Vec::new();
    let mut missing = Vec::new();

    for (key, want) in &source_meta {
        if SKIP_COMPARE.iter().any(|k| k == key) || key.contains(':') {
            continue;
        }
        match written_meta.get(key) {
            Some(got) if values_close(key, want, got) => matched += 1,
            Some(got) => mismatched.push((key.clone(), want.clone(), got.clone())),
            None => missing.push((key.clone(), want.clone())),
        }
    }

    println!("matched: {matched}");
    println!("mismatched: {}", mismatched.len());
    println!("missing from written: {}", missing.len());

    println!("\n--- mismatched (up to 40) ---");
    for (key, want, got) in mismatched.iter().take(40) {
        println!("  {key}: source={want:?} written={got:?}");
    }
    println!("\n--- missing (writable-looking, up to 40) ---");
    let mut missing_sorted = missing.clone();
    missing_sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (key, want) in missing_sorted.iter().take(40) {
        println!("  {key}: {want}");
    }

    let core = [
        "Make",
        "Model",
        "DateTimeOriginal",
        "CreateDate",
        "ModifyDate",
        "ISO",
        "ExposureTime",
        "FNumber",
        "FocalLength",
        "LensModel",
        "Artist",
        "Copyright",
        "SerialNumber",
        "OffsetTimeOriginal",
        "SubSecTimeOriginal",
    ];
    let mut failed = false;
    println!("\n--- core round-trip ---");
    for key in core {
        let want = source_meta.get(key);
        let got = written_meta.get(key);
        let ok = match (want, got) {
            (Some(w), Some(g)) => values_close(key, w, g),
            (None, _) => true,
            (Some(_), None) => false,
        };
        println!(
            "  {key}: {} (source={:?} written={:?})",
            if ok { "ok" } else { "FAIL" },
            want,
            got
        );
        if !ok {
            failed = true;
        }
    }

    if let Ok(status) = std::process::Command::new("exiftool")
        .args(["-s", "-G1", output])
        .status()
    {
        if !status.success() {
            eprintln!("exiftool exited {status}");
        }
    }

    if failed {
        Err("core EXIF tags failed to round-trip".into())
    } else {
        println!("\ncore EXIF round-trip passed: {output}");
        Ok(())
    }
}

fn values_close(key: &str, want: &str, got: &str) -> bool {
    if want == got {
        return true;
    }
    let wn = want.replace(" mm", "").replace("mm", "");
    let gn = got.replace(" mm", "").replace("mm", "");
    if wn == gn {
        return true;
    }
    if let (Ok(a), Ok(b)) = (want.parse::<f64>(), got.parse::<f64>()) {
        return (a - b).abs() < 0.05;
    }
    if key == "Flash" {
        let a = want.to_lowercase();
        let b = got.to_lowercase();
        return a.contains("not fire") && (b == "16" || b.contains("not fire"))
            || a == "16" && (b == "16" || b.contains("not fire"));
    }
    false
}
