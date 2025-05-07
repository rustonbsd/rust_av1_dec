mod impls;

use crate::{consts::OBU_TYPE, leb_128};


#[derive(Debug, PartialEq, Eq)]
pub struct OBU {
    obu_size: leb_128,          // leb128
    obu_header: OBU_Header,     // 16 bits
}


#[derive(Debug, PartialEq, Eq)]
pub struct OBU_Header {
    obu_forbidden_bit: u8,                              // 1 bit
    obu_type: OBU_TYPE,                                 // 4 bits
    obu_extension_flag: u8,                             // 1 bit
    obu_has_size_field: u8,                             // 1 bit
    obu_reserved_1bit: u8,                              // 1 bit
    obu_extension_header: Option<OBU_Extension_Header>, // 8 bits
}

#[derive(Debug, PartialEq, Eq)]
pub struct OBU_Extension_Header {
    temporal_id: u8,                     // 3 bits
    spatial_id: u8,                      // 2 bits
    extension_header_reserved_3bits: u8, // 3 bits
}

#[derive(Debug, PartialEq, Eq)]
pub struct OBU_Sequence_Header {

}

pub struct Timing_Info {
    num_units_in_tick: u32,                 // 32 bits
    time_scale: u32,                        // 32 bits
    equal_picture_interval: u8,             //  1 bit
    num_tickets_per_picture_minus_1: Option<u16>,   // UVLC
}