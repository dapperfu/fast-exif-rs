use fast_exif_reader::FastExifReader;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test with a sample MP4 file path
    let test_file = "/tmp/20190102_012853.mp4";
    
    println!("Testing enhanced video date extraction...");
    println!("File: {}", test_file);
    
    let mut reader = FastExifReader::new();
    
    match reader.read_file(test_file) {
        Ok(metadata) => {
            println!("\nExtracted metadata:");
            println!("==================");
            
            // Print all date-related fields
            let date_fields = [
                "CreateDate",
                "CreationDate", 
                "ModifyDate",
                "TrackCreateDate",
                "TrackModifyDate",
                "MediaCreateDate",
                "MediaModifyDate",
                "FileModifyDate",
                "FileAccessDate",
                "FileInodeChangeDate"
            ];
            
            for field in &date_fields {
                if let Some(value) = metadata.get(*field) {
                    println!("{}: {}", field, value);
                }
            }
            
            // Print a few other relevant fields
            let other_fields = [
                "Format",
                "FileType",
                "Duration",
                "ImageWidth",
                "ImageHeight"
            ];
            
            println!("\nOther metadata:");
            println!("==============");
            for field in &other_fields {
                if let Some(value) = metadata.get(*field) {
                    println!("{}: {}", field, value);
                }
            }
            
            // Check if we found a creation date (any date from 2019:01:02)
            if let Some(create_date) = metadata.get("CreateDate") {
                if create_date.contains("2019:01:02") {
                    println!("\n✅ SUCCESS: Found correct CreateDate: {}", create_date);
                } else {
                    println!("\n⚠️  WARNING: Found CreateDate but it's different: {}", create_date);
                }
            } else if let Some(creation_date) = metadata.get("CreationDate") {
                if creation_date.contains("2019:01:02") {
                    println!("\n✅ SUCCESS: Found correct CreationDate: {}", creation_date);
                } else {
                    println!("\n⚠️  WARNING: Found CreationDate but it's different: {}", creation_date);
                }
            } else {
                println!("\n❌ ERROR: No CreateDate or CreationDate found");
            }
            
        },
        Err(e) => {
            println!("Error reading file: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
