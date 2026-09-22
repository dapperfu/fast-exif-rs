# fast-exif-rs

I needed camera metadata off a large photo library. Not just Make/Model —
maker notes, GPS, and the computed dates ExifTool prints (`CreateDate` and
friends). I already had scripts keyed on those names.

ExifTool is the right answer for coverage. It also starts a Perl process per
file. That is fine for a dozen shots and miserable at tens of thousands.

The usual Rust and Python EXIF crates are faster, but they skip maker notes
or GPS, or they invent their own tag names. Then none of the old scripts
work.

This crate is the middle path I wanted: mmap the file, find the metadata
(JPEG APP1, TIFF IFDs, HEIF boxes, video atoms), parse those bytes only, and
emit ExifTool names and formatting. Directories go through rayon.

Not a full ExifTool port. Reads are the point. There is a writer and a
copier; do not trust them yet.

## Install

Rust 1.70+.

```toml
[dependencies]
fast-exif-reader = "0.11.3"
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

That puts the binary in `~/.local/bin/`. Put that directory on your `PATH`.

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

JPEG, TIFF, PNG, BMP, HEIF/HIF, Canon CR2, Nikon NEF, Olympus ORF, DNG,
MOV/MP4/3GP, MKV.

Maker notes are best on Canon and Nikon. Other cameras usually still get
the standard EXIF/GPS tags.

## License

MIT. ExifTool is Phil Harvey's. This is just faster at the subset I actually
use.
