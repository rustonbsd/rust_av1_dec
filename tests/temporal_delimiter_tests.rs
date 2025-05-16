use std::{fs::File, io::{self, Read}};

use bitstream_io::BitReader;
use rust_av1_dec::{consts::OBU_TYPE, obu::OBU, Leb128};

// [!] find av1 files with temporal delimiters
#[test]
fn test_extract_temporal_delimiter_obus() -> io::Result<()> {
    let file_path = "./samples/output_av1_stream.bin";
    
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let mut position = 0;
    let mut temporal_delimiters_found = 0;
    
    while position < buffer.len() {
        let mut segment_reader = BitReader::endian(&buffer[position..], bitstream_io::BigEndian);
        
        match OBU::open_bitstream_unit(&mut segment_reader, (buffer.len() - position) as u64) {
            Ok(obu) => {
                if obu.header.obu_type == OBU_TYPE::OBU_TEMPORAL_DELIMITER {
                    temporal_delimiters_found += 1;
                    println!("Temporal Delimiter #{} found at position {}", 
                             temporal_delimiters_found, position);
                    println!(" -> {}", obu);
                    println!("");
                }
                
                position += obu.size.value as usize + 1;
                if obu.header.has_size_field == 1 {
                    let size_bytes = Leb128::size_in_bytes(obu.size.value);
                    position += size_bytes;
                }
            },
            Err(e) => {
                println!("Error parsing OBU at position {}: {}", position, e);
                position += 1;
            }
        }
    }
    
    println!("Found {} temporal delimiter OBUs", temporal_delimiters_found);
    
    assert!(temporal_delimiters_found == 0, "Temporal delimiter OBUs found even tough \"output_av1_stream.bin\" shouldn't have any");
    
    Ok(())
}