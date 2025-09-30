#!/bin/bash

# Test script to compare fast-exif-rs vs exiftool date extraction
echo "=== COMPREHENSIVE DATE EXTRACTION TEST ==="
echo "Testing fast-exif-rs vs exiftool for actual EXIF metadata"
echo ""

# Test files for each extension
declare -a jpg_files=(
    "/keg/pictures/SchoolPictures/Calvin_SchoolPhoto_3rd.jpg"
    "/keg/pictures/SchoolPictures/Josie_SchoolPhoto_1st.jpg"
    "/keg/pictures/SchoolPictures/Calvin_SchoolPhoto_4th.jpg"
    "/keg/pictures/SchoolPictures/Josie_SchoolPhoto_K.jpg"
    "/keg/pictures/2021/04-Apr/20210402_151624.000.jpg"
)

declare -a cr2_files=(
    "/keg/pictures/2015/04-Apr/20150401_081554.000-2.cr2"
    "/keg/pictures/2015/04-Apr/20150408_170400.000.cr2"
    "/keg/pictures/2015/04-Apr/20150427_140630.090.cr2"
    "/keg/pictures/2015/04-Apr/20150424_071040.090.cr2"
    "/keg/pictures/2015/04-Apr/20150414_181152.000.cr2"
)

declare -a mp4_files=(
    "/keg/pictures/2021/04-Apr/20210405_185527.000.mp4"
    "/keg/pictures/2021/04-Apr/20210427_234148.000.mp4"
    "/keg/pictures/2021/04-Apr/20210404_133207.000.mp4"
    "/keg/pictures/2021/04-Apr/20210404_115733.000.mp4"
    "/keg/pictures/2021/04-Apr/20210405_201537.000.mp4"
)

declare -a heic_files=(
    "/keg/pictures/2021/04-Apr/20210423_083752.000.heic"
    "/keg/pictures/2021/04-Apr/20210422_194223.000.heic"
    "/keg/pictures/2021/04-Apr/20210411_093433.000.heic"
    "/keg/pictures/2021/04-Apr/20210427_193813.000.heic"
    "/keg/pictures/2021/04-Apr/20210430_121153.000.heic"
)

declare -a dng_files=(
    "/keg/pictures/2021/11-Nov/20211119_212738.000.dng"
    "/keg/pictures/2021/11-Nov/20211119_212701.000.dng"
    "/keg/pictures/2021/11-Nov/20211119_212820.000.dng"
    "/keg/pictures/2021/11-Nov/20211119_212642.000.dng"
    "/keg/pictures/2021/11-Nov/20211119_212840.000.dng"
)

declare -a hif_files=(
    "/keg/pictures/2025/09-Sep/20250921_120522.130.hif"
    "/keg/pictures/2025/09-Sep/20250920_105857.240.hif"
    "/keg/pictures/2025/09-Sep/20250920_123106.560.hif"
    "/keg/pictures/2025/09-Sep/20250921_120231.920.hif"
    "/keg/pictures/2025/09-Sep/20250920_130510.530.hif"
)

declare -a mov_files=(
    "/keg/pictures/2019/04-Apr/20190428_180739.000.mov"
    "/keg/pictures/2019/01-Jan/20190120_230209.000.mov"
    "/keg/pictures/2019/09-Sep/20190924_120634.000.mov"
    "/keg/pictures/2023/03-Mar/20230305_021222.000.mov"
    "/keg/pictures/2023/03-Mar/20230320_080741.000.mov"
)

declare -a 3gp_files=(
    "/keg/pictures/2019/06-Jun/20190623_151056.000.3gp"
    "/keg/pictures/2019/06-Jun/20190623_150354.000.3gp"
    "/keg/pictures/2019/06-Jun/20190623_150450.000.3gp"
    "/keg/pictures/2019/01-Jan/20190109_151642.000.3gp"
    "/keg/pictures/2019/01-Jan/20190109_143416.000.3gp"
)

declare -a mkv_files=(
    "/keg/pictures/Digitized VHS Videos/jedabby_cards.vp9.mkv"
    "/keg/pictures/Digitized VHS Videos/pokagan_89.vp9.mkv"
    "/keg/pictures/Digitized VHS Videos/bees_89.vp9.mkv"
    "/keg/pictures/Digitized VHS Videos/christmas_98.vp9.mkv"
    "/keg/pictures/Digitized VHS Videos/abby_biking_21jun90.vp9.mkv"
)

test_file_type() {
    local file_type="$1"
    local -n files_array="$2"
    
    echo "=================================================================================="
    echo "TESTING $file_type FILES"
    echo "=================================================================================="
    
    local success_count=0
    local total_count=${#files_array[@]}
    
    for file_path in "${files_array[@]}"; do
        echo ""
        echo "--- Testing: $(basename "$file_path") ---"
        
        # Get exiftool dates
        echo "EXIFTOOL dates:"
        exiftool -s "$file_path" | grep -i date | head -10
        
        echo ""
        echo "FAST-EXIF-RS dates:"
        cd /projects/fast-exif-rs
        cargo run --quiet --bin test_single_file -- "$file_path" 2>/dev/null || echo "Error running fast-exif-rs"
        
        echo ""
        echo "--- Comparison ---"
        
        # Check if fast-exif-rs found any meaningful dates (not just file system dates)
        cd /projects/fast-exif-rs
        local fast_exif_output=$(cargo run --quiet --bin test_single_file -- "$file_path" 2>/dev/null)
        local meaningful_dates=$(echo "$fast_exif_output" | grep -v "FileModifyDate\|FileAccessDate\|FileInodeChangeDate" | grep -i "date\|time\|create\|modify" | wc -l)
        
        if [ "$meaningful_dates" -gt 0 ]; then
            echo "✅ SUCCESS: Found $meaningful_dates meaningful date fields"
            ((success_count++))
        else
            echo "❌ FAILURE: Only found file system dates"
        fi
    done
    
    echo ""
    echo "SUMMARY: $success_count/$total_count $file_type files successful"
    echo ""
}

# Create a simple test binary
cd /projects/fast-exif-rs
cat > src/bin/test_single_file.rs << 'EOF'
use fast_exif_reader::FastExifReader;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    
    let mut reader = FastExifReader::new();
    let file_path = &args[1];
    
    match reader.read_file(file_path) {
        Ok(metadata) => {
            // Look for date-related fields
            let date_fields: Vec<_> = metadata.iter()
                .filter(|(key, _)| {
                    let key_lower = key.to_lowercase();
                    key_lower.contains("date") || key_lower.contains("time") || 
                    key_lower.contains("create") || key_lower.contains("modify")
                })
                .filter(|(_, value)| !value.is_empty())
                .collect();
            
            for (key, value) in date_fields {
                println!("  {}: {}", key, value);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
EOF

# Build the test binary
cargo build --bin test_single_file --quiet

# Test each file type
test_file_type "JPG" jpg_files
test_file_type "CR2" cr2_files  
test_file_type "MP4" mp4_files
test_file_type "HEIC" heic_files
test_file_type "DNG" dng_files
test_file_type "HIF" hif_files
test_file_type "MOV" mov_files
test_file_type "3GP" 3gp_files
test_file_type "MKV" mkv_files

echo "=================================================================================="
echo "FINAL SUMMARY"
echo "=================================================================================="
echo "Test completed. Check individual results above for detailed comparison."
