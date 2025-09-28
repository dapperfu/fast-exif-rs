# Benchmark Results Summary

## Key Findings

**Test**: 1000 random photos from /keg/pictures/
**fast-exif-rs**: 252.17s total (0.2522s avg per file)
**PyExifTool**: 18.94s total (0.0189s avg per file)
**Overall**: PyExifTool is 13.3x faster

## Format Performance

- **CR2 files**: fast-exif-rs is 7.79x faster
- **DNG files**: fast-exif-rs is 5.55x faster  
- **JPEG files**: PyExifTool is 15.7x faster
- **HEIC files**: PyExifTool is 3.3x faster

## Success Rates

- **fast-exif-rs**: 998/1000 (99.8%)
- **PyExifTool**: 1000/1000 (100.0%)

## Field Coverage

- **fast-exif-rs**: 105.3 fields average
- **PyExifTool**: 238.3 fields average

## Conclusion

fast-exif-rs excels at RAW files but struggles with JPEG/HEIC. PyExifTool is faster overall for mixed formats.
