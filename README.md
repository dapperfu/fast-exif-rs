# fast-exif-rs

I wanted ExifTool's tag coverage without starting Perl for every file. Most Rust/Python EXIF crates are either slow on a big library or they drop maker notes, GPS, and half the computed fields.

This crate memory-maps the file, finds the metadata (JPEG APP1, TIFF IFDs, HEIF boxes, video atoms), parses those bits only, then names and formats fields the way ExifTool does. Batch reads use rayon.

It is not a full ExifTool port. Reads are the point. There's a writer and a copier; treat them as experimental.

## Install

```toml
[dependencies]
fast-exif-reader = "0.9"
```

Needs Rust 1.70+.

## Usage

```rust
use fast_exif_reader::FastExifReader;

let mut reader = FastExifReader::new();
let tags = reader.read_file("photo.jpg")?;

println!("{} {}", tags["Make"], tags["Model"]);
```

A directory of files:

```rust
let tags = reader.read_files_parallel(paths)?;
```

Or bytes: `reader.read_bytes(&buf)?`.

There's a CLI in `exiftool-rs/` if you just want to dump tags.

## Formats

JPEG, TIFF, PNG, BMP, HEIF/HIF, Canon CR2, Nikon NEF, Olympus ORF, DNG, MOV/MP4/3GP, MKV.

Maker notes are best on Canon/Nikon. Other cameras usually still get the standard EXIF/GPS tags.

## License

MIT. ExifTool is Phil Harvey's; this just tries to be faster at the subset I actually use.
