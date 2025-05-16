use std::fs::File;
use std::io::{self, Read};
use bitstream_io::{BitReader, BitRead};
use rust_av1_dec::{leb_128, consts::OBU_TYPE};
use rust_av1_dec::obu::OBU;

#[test]
fn test_parse_sequence_headers() -> io::Result<()> {
    let file_path = "./samples/output_av1_stream.bin";
    
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let mut position = 0;
    let mut sequence_headers_found = 0;
    let mut sequence_headers_parsed = 0;
    
    while position < buffer.len() {
        let mut segment_reader = BitReader::endian(&buffer[position..], bitstream_io::BigEndian);
        
        match OBU::open_bitstream_unit(&mut segment_reader, (buffer.len() - position) as u64) {
            Ok(obu) => {
                if obu.obu_header.obu_type == OBU_TYPE::OBU_SEQUENCE_HEADER {
                    sequence_headers_found += 1;
                    
                    if let Some(seq_header) = &obu.obu_sequence_header {
                        sequence_headers_parsed += 1;
                        println!("Sequence Header #{} found:", sequence_headers_parsed);
                        println!(" -> {}", seq_header);
                        println!("");
                    }
                }
                
                position += obu.obu_size.value as usize + 1;
                if obu.obu_header.obu_has_size_field == 1 {
                    let size_bytes = leb_128::size_in_bytes(obu.obu_size.value);
                    position += size_bytes;
                }
            },
            Err(e) => {
                println!("Error parsing OBU at position {}: {}", position, e);
                position += 1;
            }
        }
    }
    
    println!("Found {} sequence headers, successfully parsed {}", 
             sequence_headers_found, sequence_headers_parsed);
    
    assert!(sequence_headers_found > 0, "No sequence headers found in the file");
    assert_eq!(sequence_headers_found, sequence_headers_parsed, 
               "Not all sequence headers were successfully parsed");
    
    Ok(())
}

#[test]
fn test_sequence_header_values_match_reference() -> io::Result<()> {
    let file_path = "./samples/output_av1_stream.bin";
    
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let mut position = 0;
    let mut sequence_headers = Vec::new();
    
    while position < buffer.len() {
        let mut segment_reader = BitReader::endian(&buffer[position..], bitstream_io::BigEndian);
        
        match OBU::open_bitstream_unit(&mut segment_reader, (buffer.len() - position) as u64) {
            Ok(obu) => {
                if obu.obu_header.obu_type == OBU_TYPE::OBU_SEQUENCE_HEADER {
                    if let Some(seq_header) = &obu.obu_sequence_header {
                        sequence_headers.push(seq_header.clone());
                    }
                }
                
                position += obu.obu_size.value as usize + 1;
                if obu.obu_header.obu_has_size_field == 1 {
                    let size_bytes = leb_128::size_in_bytes(obu.obu_size.value);
                    position += size_bytes;
                }
            },
            Err(_) => {
                position += 1;
            }
        }
    }
    
    assert_eq!(sequence_headers.len(), 12, "Expected 12 sequence headers, found {}", sequence_headers.len());
    
    for header in &sequence_headers {
        assert_eq!(header.seq_profile, 0);
        assert_eq!(header.still_picture, 0);
        
        assert_eq!(header.max_frame_width_minus_one, 1920);
        assert_eq!(header.max_frame_height_minus_one, 818);
        
        assert_eq!(header.use_128x128_superblock, 1);
        assert_eq!(header.enable_filter_intra, 1);
        assert_eq!(header.enable_intra_edge_filter, 1);
        assert_eq!(header.enable_interintra_compound, 0);
        assert_eq!(header.enable_masked_compound, 1);
        assert_eq!(header.enable_warped_motion, 1);
        assert_eq!(header.enable_dual_filter, 0);
        assert_eq!(header.enable_order_hint, 1);
        assert_eq!(header.enable_jnt_comp, 1);
        assert_eq!(header.enable_ref_frame_mvs, 1);
        
        assert_eq!(header.seq_force_screen_content_tools, 2);
        assert_eq!(header.seq_force_integer_mv, 2);
        
        assert_eq!(header.order_hint_bits, 7);
        assert_eq!(header.enable_superres, 0);
        assert_eq!(header.enable_cdef, 1);
        assert_eq!(header.enable_restoration, 0);
        
        assert_eq!(header.color_config.bit_depth, 8);
        assert_eq!(header.color_config.mono_chrome, 0);
        assert_eq!(header.color_config.num_planes, 3);
        assert_eq!(header.color_config.color_range, 1);
        assert_eq!(header.color_config.subsampling_x, 1);
        assert_eq!(header.color_config.subsampling_y, 1);
        assert_eq!(header.color_config.separate_uv_delta_q, 0);
        
        assert_eq!(header.film_grain_params_present, 0);
    }
    
    Ok(())
}
