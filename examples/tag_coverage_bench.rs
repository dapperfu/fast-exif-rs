//! Tag coverage vs speed.
//!
//! Probe 1–2 files from the same camera to learn the full tag set, then time
//! the first N images under narrower read options. The point is the trade-off:
//! a photographer who only needs the capture timestamp can skip maker notes
//! and GPS and get a large speedup in exchange for covering far fewer tags.
//!
//! ```text
//! cargo run --release --example tag_coverage_bench
//! cargo run --release --example tag_coverage_bench -- ~/Desktop/Pictures -n 50 --probe 2
//! ```

use fast_exif_reader::{FastExifReader, ReadOptions};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const IMAGE_EXTS: &[&str] = &[
    "jpg", "jpeg", "tif", "tiff", "cr2", "cr3", "nef", "orf", "dng", "arw", "rw2", "raf",
    "heic", "heif", "hif", "png",
];

const DATETIME_TAGS: &[&str] = &[
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
];

const SHOT_TAGS: &[&str] = &[
    "DateTimeOriginal",
    "CreateDate",
    "DateTime",
    "ModifyDate",
    "Make",
    "Model",
    "ISO",
    "ExposureTime",
    "FNumber",
    "FocalLength",
    "FocalLengthIn35mmFormat",
    "LensModel",
    "LensID",
    "ExposureCompensation",
    "Flash",
    "WhiteBalance",
];

struct Args {
    dir: PathBuf,
    n: usize,
    probe: usize,
    iters: u32,
    warmup: u32,
}

fn main() {
    let args = parse_args();
    if let Err(err) = run(&args) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn parse_args() -> Args {
    let mut dir = default_pictures_dir();
    let mut n = 50usize;
    let mut probe = 2usize;
    let mut iters = 5u32;
    let mut warmup = 1u32;
    let raw: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "-n" | "--n" => {
                n = raw.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(n);
                i += 2;
            }
            "--probe" => {
                probe = raw.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(probe);
                i += 2;
            }
            "--iters" | "--iterations" => {
                iters = raw.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(iters);
                i += 2;
            }
            "--warmup" => {
                warmup = raw.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(warmup);
                i += 2;
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other if other.starts_with('-') => {
                eprintln!("Unknown flag: {other}");
                print_help();
                std::process::exit(2);
            }
            other => {
                dir = expand_tilde(other);
                i += 1;
            }
        }
    }
    Args {
        dir,
        n: n.max(1),
        probe: probe.clamp(1, 2),
        iters: iters.max(1),
        warmup,
    }
}

fn print_help() {
    eprintln!(
        "Usage: tag_coverage_bench [DIR] [-n N] [--probe 1|2] [--iters K] [--warmup K]\n\
         \n\
         Benchmarks the first N images in DIR (default: ~/Desktop/Pictures).\n\
         Probe 1–2 files for the full tag set (same camera assumed), then time\n\
         narrower reads so you can see N% tag coverage at M% of full-read time."
    );
}

fn default_pictures_dir() -> PathBuf {
    expand_tilde("~/Desktop/Pictures")
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    if path == "~" {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(path)
}

fn run(args: &Args) -> Result<(), String> {
    if !args.dir.is_dir() {
        return Err(format!(
            "Directory not found: {}\nPass an image directory, e.g. ~/Desktop/Pictures",
            args.dir.display()
        ));
    }

    let files = collect_images(&args.dir, args.n);
    if files.is_empty() {
        return Err(format!(
            "No image files found in {}",
            args.dir.display()
        ));
    }

    let probe_n = args.probe.min(files.len());
    let probe_files = &files[..probe_n];

    let mut reader = FastExifReader::new();
    let (universe, camera) = probe_tag_universe(&mut reader, probe_files)?;
    if universe.is_empty() {
        return Err("Probe images produced no tags.".to_string());
    }

    println!("Tag coverage vs speed");
    println!("Directory: {}", args.dir.display());
    println!(
        "Files:     {} (first {} images, sorted by path)",
        files.len(),
        files.len()
    );
    println!(
        "Probe:     {} file{}{}",
        probe_n,
        if probe_n == 1 { "" } else { "s" },
        camera
            .as_deref()
            .map(|c| format!("  ({c})"))
            .unwrap_or_default()
    );
    for path in probe_files {
        println!("           {}", path.display());
    }
    println!("Full tags: {} unique names from probe", universe.len());
    let mut preview: Vec<_> = universe.iter().cloned().collect();
    preview.sort();
    let date_preview: Vec<_> = preview
        .iter()
        .filter(|t| {
            let u = t.to_ascii_lowercase();
            u.contains("date") || u.contains("time")
        })
        .take(8)
        .cloned()
        .collect();
    if !date_preview.is_empty() {
        println!("           date/time fields: {}", date_preview.join(", "));
    }
    println!(
        "Timing:    {} warmup + {} timed iteration(s)\n",
        args.warmup, args.iters
    );

    let datetime_wanted = present_in(&universe, DATETIME_TAGS);
    let shot_wanted = present_in(&universe, SHOT_TAGS);

    let mut profiles: Vec<Profile> = Vec::new();
    if !datetime_wanted.is_empty() {
        profiles.push(Profile {
            name: "DateTime only".into(),
            options: ReadOptions::tags(datetime_wanted.clone()),
            note: Some("capture time without maker notes / GPS".into()),
        });
    }
    if !shot_wanted.is_empty() {
        profiles.push(Profile {
            name: "Shot basics".into(),
            options: ReadOptions::tags(shot_wanted),
            note: Some("date, camera, exposure, lens".into()),
        });
    }
    profiles.push(Profile {
        name: "Standard EXIF (no maker notes)".into(),
        options: ReadOptions::standard(),
        note: Some("IFD0 / ExifIFD / GPS; skip manufacturer blob".into()),
    });
    profiles.push(Profile {
        name: "Full (all tags)".into(),
        options: ReadOptions::full(),
        note: None,
    });

    // Coverage ladder: 1 tag, then growing prefixes of photographer-priority tags
    // that actually exist on this camera, plus 25/50/100% of the full name list.
    let priority = photographer_priority(&universe);
    let mut ladder_counts: BTreeSet<usize> = BTreeSet::new();
    ladder_counts.insert(1.min(priority.len()));
    if priority.len() >= 2 {
        ladder_counts.insert(2);
    }
    for pct in [5, 10, 25, 50, 100] {
        let count = ((priority.len() * pct) + 99) / 100;
        if count > 0 {
            ladder_counts.insert(count.min(priority.len()));
        }
    }

    let results = bench_profiles(
        &mut reader,
        &files,
        &universe,
        &profiles,
        args.warmup,
        args.iters,
    )?;

    println!(
        "{:<34} {:>8} {:>10} {:>12} {:>12} {:>10}",
        "Profile", "Tags", "Coverage", "Time", "vs full", "files/s"
    );
    println!("{}", "-".repeat(92));

    let full = results
        .iter()
        .find(|r| r.name == "Full (all tags)")
        .cloned()
        .ok_or_else(|| "full profile missing".to_string())?;

    for row in &results {
        print_row(row, &full);
    }

    println!();
    println!("Coverage ladder (requested tag names that exist on this camera)");
    println!(
        "{:<34} {:>8} {:>10} {:>12} {:>12} {:>10}",
        "Requested tags", "Got", "Coverage", "Time", "vs full", "files/s"
    );
    println!("{}", "-".repeat(92));

    let ladder_profiles: Vec<Profile> = ladder_counts
        .into_iter()
        .map(|count| {
            let wanted: Vec<String> = priority.iter().take(count).cloned().collect();
            Profile {
                name: format!("{} tag{}", count, if count == 1 { "" } else { "s" }),
                options: ReadOptions::tags(wanted),
                note: None,
            }
        })
        .collect();
    let ladder_rows = bench_profiles(
        &mut reader,
        &files,
        &universe,
        &ladder_profiles,
        args.warmup,
        args.iters,
    )?;
    for row in &ladder_rows {
        print_row(row, &full);
    }

    println!();
    if let Some(dt) = results.iter().find(|r| r.name == "DateTime only") {
        if full.avg_secs > 0.0 && dt.avg_secs > 0.0 {
            let speedup = full.avg_secs / dt.avg_secs;
            println!(
                "Takeaway: DateTime-only covered {:.1}% of probe tags at {:.1}% of full-read time ({:.1}×).",
                dt.coverage_pct,
                100.0 * dt.avg_secs / full.avg_secs,
                speedup
            );
        }
    }
    println!(
        "Full read is 100% of tags. Narrower options skip maker notes and unused IFDs; \
that is the usual trade-off when you only need a timestamp."
    );

    Ok(())
}

struct Profile {
    name: String,
    options: ReadOptions,
    note: Option<String>,
}

#[derive(Clone)]
struct BenchRow {
    name: String,
    note: Option<String>,
    tags_returned: usize,
    coverage_pct: f64,
    avg_secs: f64,
    files_per_sec: f64,
}

fn probe_tag_universe(
    reader: &mut FastExifReader,
    files: &[PathBuf],
) -> Result<(HashSet<String>, Option<String>), String> {
    let mut universe = HashSet::new();
    let mut camera = None;
    for path in files {
        let meta = read_full(reader, path)?;
        if camera.is_none() {
            camera = camera_from_metadata(&meta);
        }
        universe.extend(meta.into_keys());
    }
    Ok((universe, camera))
}

fn camera_from_metadata(meta: &HashMap<String, String>) -> Option<String> {
    let lookup = |names: &[&str]| {
        names.iter().find_map(|n| {
            meta.iter().find(|(k, v)| {
                normalize_tag_name(k) == normalize_tag_name(n) && !v.trim().is_empty()
            }).map(|(_, v)| v.trim().to_string())
        })
    };
    let make = lookup(&["Make", "CameraMake"]);
    let model = lookup(&["Model", "CameraModelName", "UniqueCameraModel"]);
    match (make, model) {
        (Some(a), Some(b)) => Some(format!("{a} {b}")),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn read_full(
    reader: &mut FastExifReader,
    path: &Path,
) -> Result<HashMap<String, String>, String> {
    reader
        .read_file_with_options(path.to_str().unwrap_or_default(), &ReadOptions::full())
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn bench_profiles(
    reader: &mut FastExifReader,
    files: &[PathBuf],
    universe: &HashSet<String>,
    profiles: &[Profile],
    warmup: u32,
    iters: u32,
) -> Result<Vec<BenchRow>, String> {
    for _ in 0..warmup {
        for profile in profiles {
            for path in files {
                let _ = reader.read_file_with_options(
                    path.to_str().unwrap_or_default(),
                    &profile.options,
                );
            }
        }
    }

    let mut times = vec![Vec::new(); profiles.len()];
    let mut last_tags: Vec<HashSet<String>> = vec![HashSet::new(); profiles.len()];
    for _ in 0..iters {
        for (i, profile) in profiles.iter().enumerate() {
            let start = Instant::now();
            last_tags[i].clear();
            for path in files {
                match reader.read_file_with_options(
                    path.to_str().unwrap_or_default(),
                    &profile.options,
                ) {
                    Ok(meta) => last_tags[i].extend(meta.into_keys()),
                    Err(_) => {}
                }
            }
            times[i].push(start.elapsed());
        }
    }

    Ok(profiles
        .iter()
        .enumerate()
        .map(|(i, profile)| row_from_times(profile, universe, files.len(), &times[i], &last_tags[i]))
        .collect())
}

fn row_from_times(
    profile: &Profile,
    universe: &HashSet<String>,
    file_count: usize,
    times: &[Duration],
    last_tags: &HashSet<String>,
) -> BenchRow {
    let avg = median_duration(times);
    let covered = last_tags.iter().filter(|t| universe.contains(*t)).count();
    let coverage_pct = if universe.is_empty() {
        0.0
    } else {
        100.0 * covered as f64 / universe.len() as f64
    };
    let secs = avg.as_secs_f64();
    BenchRow {
        name: profile.name.clone(),
        note: profile.note.clone(),
        tags_returned: last_tags.len(),
        coverage_pct,
        avg_secs: secs,
        files_per_sec: if secs > 0.0 {
            file_count as f64 / secs
        } else {
            f64::INFINITY
        },
    }
}

fn print_row(row: &BenchRow, full: &BenchRow) {
    let vs = if full.avg_secs > 0.0 {
        format!("{:>5.1}% time", 100.0 * row.avg_secs / full.avg_secs)
    } else {
        "     n/a".into()
    };
    println!(
        "{:<34} {:>8} {:>9.1}% {:>12} {:>12} {:>10.1}",
        truncate(&row.name, 34),
        row.tags_returned,
        row.coverage_pct,
        format_secs(row.avg_secs),
        vs,
        row.files_per_sec
    );
    if let Some(note) = &row.note {
        println!("  {note}");
    }
}

fn photographer_priority(universe: &HashSet<String>) -> Vec<String> {
    let mut ordered: Vec<String> = Vec::new();
    for tag in DATETIME_TAGS.iter().chain(SHOT_TAGS.iter()) {
        if let Some(actual) = universe.iter().find(|t| normalize_tag_name(t) == normalize_tag_name(tag)) {
            if !ordered.iter().any(|t| normalize_tag_name(t) == normalize_tag_name(actual)) {
                ordered.push(actual.clone());
            }
        }
    }
    let mut rest: Vec<String> = universe
        .iter()
        .filter(|t| !ordered.iter().any(|o| normalize_tag_name(o) == normalize_tag_name(t)))
        .cloned()
        .collect();
    rest.sort();
    // Cheaper groups first so the ladder actually gets faster as coverage drops:
    // remaining non-GPS non-maker-note, then GPS, then maker notes.
    let (mn, rest): (Vec<_>, Vec<_>) = rest
        .into_iter()
        .partition(|t| looks_mn(t));
    let (gps, core): (Vec<_>, Vec<_>) = rest.into_iter().partition(|t| t.starts_with("GPS"));
    ordered.extend(core);
    ordered.extend(gps);
    ordered.extend(mn);
    ordered
}

fn looks_mn(tag: &str) -> bool {
    tag.starts_with("MakerNote")
        || tag.starts_with("Canon")
        || tag.starts_with("Nikon")
        || tag.starts_with("Olympus")
        || tag.starts_with("Sony")
        || tag.starts_with("Samsung")
        || tag.starts_with("Ricoh")
        || tag.starts_with("Fujifilm")
        || tag.starts_with("Pentax")
        || tag.starts_with("Panasonic")
}

fn present_in(universe: &HashSet<String>, tags: &[&str]) -> Vec<String> {
    tags.iter()
        .filter(|want| {
            universe.iter().any(|have| normalize_tag_name(have) == normalize_tag_name(want))
        })
        .map(|t| (*t).to_string())
        .collect()
}

fn normalize_tag_name(tag: &str) -> String {
    let base = tag.rsplit_once(':').map(|(_, rest)| rest).unwrap_or(tag);
    base.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn collect_images(dir: &Path, n: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_images_into(dir, &mut files);
    files.sort();
    files.truncate(n);
    files
}

fn collect_images_into(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_images_into(&path, files);
        } else if path.is_file() && is_image(&path) {
            files.push(path);
        }
    }
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTS.iter().any(|x| e.eq_ignore_ascii_case(x)))
        .unwrap_or(false)
}

fn median_duration(times: &[Duration]) -> Duration {
    if times.is_empty() {
        return Duration::ZERO;
    }
    let mut sorted = times.to_vec();
    sorted.sort();
    sorted[sorted.len() / 2]
}

fn format_secs(secs: f64) -> String {
    if secs < 0.001 {
        format!("{:.0}µs", secs * 1_000_000.0)
    } else if secs < 1.0 {
        format!("{:.1}ms", secs * 1000.0)
    } else {
        format!("{:.3}s", secs)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
