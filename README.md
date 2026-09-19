# fast-exif-rs

I needed camera metadata off a large photo library. ExifTool has the tags I actually use — maker notes, GPS, the computed fields — but it boots Perl for every file. Fine for a few shots. Miserable at tens of thousands.

The usual Rust and Python EXIF crates go the other way: they skip maker notes, GPS, or they name things differently than ExifTool, so anything already keyed on ExifTool tags breaks.

This crate sits in the middle. It memory-maps the file, finds the metadata (JPEG APP1, TIFF IFDs, HEIF boxes, video atoms), parses those bytes only, then names and formats fields the way ExifTool does. Directories go through rayon.

It is not a full ExifTool port. Reads are the point. There's a writer and a copier; don't trust them yet.

## Install

Rust 1.70+.

```toml
[dependencies]
fast-exif-reader = "0.10.5"
```

From git:

```toml
fast-exif-reader = { git = "https://github.com/dapperfu/fast-exif-rs" }
```

CLI, if you just want to dump tags:

```bash
make install
exiftool-rs extract photo.jpg
```

That puts the binary in `~/.local/bin/`. Make sure that directory is on your `PATH`.

## Usage

```rust
use fast_exif_reader::FastExifReader;

let mut reader = FastExifReader::new();
let tags = reader.read_file("photo.jpg")?;

println!("{} {}", tags["Make"], tags["Model"]);
```

A pile of files:

```rust
let tags = reader.read_files_parallel(paths)?;
```

Or bytes: `reader.read_bytes(&buf)?`.

## Formats

JPEG, TIFF, PNG, BMP, HEIF/HIF, Canon CR2, Nikon NEF, Olympus ORF, DNG, MOV/MP4/3GP, MKV.

Maker notes are best on Canon and Nikon. Other cameras usually still get the standard EXIF/GPS tags.

## License

MIT. ExifTool is Phil Harvey's; this is just faster at the subset I actually use.
