#!/usr/bin/env python3
"""
Analyze the specific pattern around the correct SubSecTime values
"""

import sys
from pathlib import Path

def analyze_subsec_pattern(file_path: str):
    """Analyze the pattern around the correct SubSecTime values"""
    print(f"Analyzing SubSecTime pattern: {file_path}")
    print("=" * 60)
    
    try:
        with open(file_path, 'rb') as f:
            file_data = f.read()
        
        # Look for the specific pattern we found
        target_positions = [2646, 2658, 2670]
        
        for pos in target_positions:
            print(f"\nPosition {pos}:")
            # Show more context
            start = max(0, pos - 50)
            end = min(len(file_data), pos + 50)
            context = file_data[start:end]
            print(f"Context: {context.hex()}")
            
            # Try to interpret the bytes around position 92
            if pos + 10 < len(file_data):
                # Look for tag patterns
                for i in range(max(0, pos - 20), min(len(file_data) - 4, pos + 20)):
                    # Check if this looks like a SubSecTime tag
                    if file_data[i:i+2] == b'\x92\x90':  # SubSecTime
                        print(f"  Found SubSecTime tag at {i}")
                        # The value should be in the next few bytes
                        if i + 4 < len(file_data):
                            value_bytes = file_data[i+2:i+4]
                            print(f"    Value bytes: {list(value_bytes)}")
                            try:
                                value_str = value_bytes.decode('ascii', errors='replace')
                                print(f"    As string: '{value_str}'")
                            except:
                                print(f"    As string: (decode error)")
                    elif file_data[i:i+2] == b'\x92\x91':  # SubSecTimeOriginal
                        print(f"  Found SubSecTimeOriginal tag at {i}")
                        if i + 4 < len(file_data):
                            value_bytes = file_data[i+2:i+4]
                            print(f"    Value bytes: {list(value_bytes)}")
                            try:
                                value_str = value_bytes.decode('ascii', errors='replace')
                                print(f"    As string: '{value_str}'")
                            except:
                                print(f"    As string: (decode error)")
                    elif file_data[i:i+2] == b'\x92\x92':  # SubSecTimeDigitized
                        print(f"  Found SubSecTimeDigitized tag at {i}")
                        if i + 4 < len(file_data):
                            value_bytes = file_data[i+2:i+4]
                            print(f"    Value bytes: {list(value_bytes)}")
                            try:
                                value_str = value_bytes.decode('ascii', errors='replace')
                                print(f"    As string: '{value_str}'")
                            except:
                                print(f"    As string: (decode error)")
        
        # Look for the pattern more systematically
        print(f"\n" + "=" * 60)
        print("Systematic search for SubSecTime tag patterns:")
        print("-" * 50)
        
        subsec_tags = [b'\x92\x90', b'\x92\x91', b'\x92\x92']
        tag_names = {b'\x92\x90': "SubSecTime", b'\x92\x91': "SubSecTimeOriginal", b'\x92\x92': "SubSecTimeDigitized"}
        
        for tag_bytes in subsec_tags:
            positions = []
            for i in range(len(file_data) - 10):
                if file_data[i:i+2] == tag_bytes:
                    positions.append(i)
            
            print(f"\n{tag_names[tag_bytes]} (0x{tag_bytes.hex()}): {len(positions)} occurrences")
            
            for pos in positions[:10]:  # Show first 10
                print(f"  At position {pos}:")
                # Show context
                start = max(0, pos - 10)
                end = min(len(file_data), pos + 10)
                context = file_data[start:end]
                print(f"    Context: {context.hex()}")
                
                # Try to extract value (assuming it's in the next 2 bytes)
                if pos + 4 < len(file_data):
                    value_bytes = file_data[pos+2:pos+4]
                    print(f"    Value bytes: {list(value_bytes)}")
                    try:
                        value_str = value_bytes.decode('ascii', errors='replace')
                        print(f"    As string: '{value_str}'")
                        if value_str == "92":
                            print(f"    *** CORRECT VALUE! ***")
                    except:
                        print(f"    As string: (decode error)")
        
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python analyze_subsec_pattern.py <heif_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    analyze_subsec_pattern(file_path)
