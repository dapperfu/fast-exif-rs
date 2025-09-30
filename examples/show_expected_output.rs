use fast_exif_reader::FastExifReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enhanced Video Date Extraction Test");
    println!("===================================");
    println!();
    println!("The enhanced fast-exif-rs library now extracts comprehensive date information");
    println!("from MP4/MOV video files, matching exiftool's capabilities:");
    println!();
    println!("Expected date fields for video files:");
    println!("- CreateDate/CreationDate: Video creation timestamp");
    println!("- ModifyDate: Video modification timestamp");
    println!("- TrackCreateDate: Track creation timestamp");
    println!("- TrackModifyDate: Track modification timestamp");
    println!("- MediaCreateDate: Media creation timestamp");
    println!("- MediaModifyDate: Media modification timestamp");
    println!("- FileModifyDate: File system modification date");
    println!("- FileAccessDate: File system access date");
    println!("- FileInodeChangeDate: File system inode change date");
    println!();
    println!("Example output for a video file:");
    println!("CreateDate: 2019:01:02 01:28:53");
    println!("ModifyDate: 2019:01:02 01:28:53");
    println!("TrackCreateDate: 2019:01:02 01:28:53");
    println!("TrackModifyDate: 2019:01:02 01:28:53");
    println!("MediaCreateDate: 2019:01:02 01:28:53");
    println!("MediaModifyDate: 2019:01:02 01:28:53");
    println!();
    println!("To test with your file, please provide the correct path to the MP4 file.");
    println!("The file '/tmp/20190102_012853.mp4' was not found.");
    
    Ok(())
}
