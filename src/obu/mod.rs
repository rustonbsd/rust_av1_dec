mod impls;
mod handlers;

use crate::{consts::{self, OBU_TYPE}, generics::uvlc, leb_128};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU {
    obu_size: leb_128,          // leb128
    obu_header: OBU_Header,     // 16 bits
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU_Header {
    obu_forbidden_bit: u8,                              // 1 bit
    obu_type: OBU_TYPE,                                 // 4 bits
    obu_extension_flag: u8,                             // 1 bit
    obu_has_size_field: u8,                             // 1 bit
    obu_reserved_1bit: u8,                              // 1 bit
    obu_extension_header: Option<OBU_Extension_Header>, // 8 bits
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU_Extension_Header {
    temporal_id: u8,                     // 3 bits
    spatial_id: u8,                      // 2 bits
    extension_header_reserved_3bits: u8, // 3 bits
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU_Sequence_Header {
    seq_profile: u8,                    // 3 bits
    still_picture: u8,                  // 1 bit
    timing_info: Option<Timing_Info>,
    decoder_model_info: Option<Decoder_Model_Info>,
    operating_point_idc: Vec<u16>,      // 12 bits
    seq_level_idx: Vec<u8>,                  // 5 bits
    seq_tier: Vec<u8>,                       // 1 bit
    decoder_model_present_for_this_op: Vec<u8>, // 1 bit
    operating_parameters_info: Option<Operating_Parameters_Info>,
    initial_display_delay_present_for_this_op: Vec<u8>, // 1 bit
    initial_display_delay_minus_1: Option<Vec<u8>>,    // 4 bits
    c_operating_point_idc: u16,         // 12 bits
    max_frame_width_minus_one: u16,     //  2**frame_width_bits_minus_1+1 
    max_frame_height_minus_one: u16,    //  2**frame_height_bits_minus_1+1
    delta_frame_id_length_minus_2: Option<u8>, // 4 bits
    additional_frame_id_length_minus_1: Option<u8>, // 3 bits
    use_128x128_superblock: u8,         // 1 bit
    enable_filter_intra: u8,             // 1 bit
    enable_intra_edge_filter: u8,        // 1 bit
    enable_interintra_compound: u8,      // 1 bit
    enable_masked_compound: u8,          // 1 bit
    enable_warped_motion: u8,            // 1 bit
    enable_dual_filter: u8,              // 1 bit
    enable_order_hint: u8,               // 1 bit
    enable_jnt_comp: u8,                 // 1 bit
    enable_ref_frame_mvs: u8,            // 1 bit
    seq_force_screen_content_tools: u8, // 1 bit
    seq_force_integer_mv: u8,           // 1 bit
    order_hint_bits: u8,                 // 3 bits
    enable_superres: u8,                 // 1 bit
    enable_cdef: u8,                     // 1 bit
    enable_restoration: u8,              // 1 bit
    color_config: Color_Config,
    film_grain_params_present: u8,       // 1 bit
}

// 5.5.3 Timing info syntax
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Timing_Info {
    num_units_in_display_tick: u32,                 // 32 bits
    time_scale: u32,                        // 32 bits
    equal_picture_interval: u8,             //  1 bit
    num_ticks_per_picture_minus_1: Option<uvlc>,   // UVLC
}

// 5.5.4 Decoder model info
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Decoder_Model_Info {
    buffer_delay_length_minus_1: u8,   // 5 bits
    num_units_in_decoding_tick: u32,   // 32 bits
    buffer_removal_delay_length_minus_1: u8, // 5 bits
    frame_presentation_delay_length_minus_1: u8, // 5 bits
}

// 5.5.5 Operating parameters info
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Operating_Parameters_Info {
    decoder_buffer_delay: Vec<u32>, // 2**5=32 bits max
    encoder_buffer_delay: Vec<u32>, // 2**5=32 bits max
    low_delay_mode_flag: Vec<u8>,   // 1 bit
}

// 5.5.2 Color Config
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Color_Config {
    bit_depth: u8,
    mono_chrome: u8,
    num_planes: u8,
    color_primaries: consts::COLOR_PRIMARIES,
    transfer_characteristics: consts::TRANSFER_CHARACTERISTICS,
    matrix_coefficients: consts::MATRIX_COEFFICIENTS,
    color_range: u8,
    subsampling_x: u8,
    subsampling_y: u8,
    chroma_sample_position: consts::CHROMA_SAMPLE_POSITION,
    separate_uv_delta_q: u8,
}
