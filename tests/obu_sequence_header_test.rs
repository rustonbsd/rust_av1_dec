use std::fs::File;
use std::io::{self, Read};
use bitstream_io::{BitReader, BitRead};
use rust_av1_dec::{leb_128, consts::OBU_TYPE};
use rust_av1_dec::obu::OBU;

#[test]
fn test_parse_sequence_headers() -> io::Result<()> {
    // Path to the sample AV1 file
    let file_path = "./samples/output_av1_stream.bin";
    
    // Read the file
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Create a bit reader from the buffer
    let mut reader = BitReader::endian(&buffer[..], bitstream_io::BigEndian);
    
    let mut position = 0;
    let mut sequence_headers_found = 0;
    let mut sequence_headers_parsed = 0;
    
    // Process the file until we reach the end
    while position < buffer.len() {
        // Create a bit reader at the current position
        let mut segment_reader = BitReader::endian(&buffer[position..], bitstream_io::BigEndian);
        
        // Try to parse an OBU
        match OBU::open_bitstream_unit(&mut segment_reader, (buffer.len() - position) as u64) {
            Ok(obu) => {
                // Check if this is a sequence header
                if obu.obu_header.obu_type == OBU_TYPE::OBU_SEQUENCE_HEADER {
                    sequence_headers_found += 1;
                    
                    // If the OBU has a sequence header, it was successfully parsed
                    if let Some(seq_header) = &obu.obu_sequence_header {
                        sequence_headers_parsed += 1;
                        println!("Sequence Header #{} found:", sequence_headers_parsed);

                        println!(" -> {}", seq_header);
                        println!(" -> {}", seq_header.max_frame_width_minus_one);
                        
                        println!(""); // Empty line for readability
                    }
                }
                
                // Move to the next OBU
                position += obu.obu_size.value as usize + 1; // +1 for the header byte
                if obu.obu_header.obu_has_size_field == 1 {
                    // Add the size of the leb128 field
                    let size_bytes = leb_128::size_in_bytes(obu.obu_size.value);
                    position += size_bytes;
                }
            },
            Err(e) => {
                // If we can't parse an OBU, try moving forward one byte
                println!("Error parsing OBU at position {}: {}", position, e);
                position += 1;
            }
        }
    }
    
    println!("Found {} sequence headers, successfully parsed {}", 
             sequence_headers_found, sequence_headers_parsed);
    
    // Test passes if we found at least one sequence header and parsed all of them
    assert!(sequence_headers_found > 0, "No sequence headers found in the file");
    assert_eq!(sequence_headers_found, sequence_headers_parsed, 
               "Not all sequence headers were successfully parsed");
    
    Ok(())
}