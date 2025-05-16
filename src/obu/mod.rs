mod impls;
mod handlers;

use crate::{consts::{self, OBU_TYPE}, generics::uvlc, leb_128};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU {
    pub obu_size: leb_128,          // leb128
    pub obu_header: OBU_Header,     // 16 bits
    pub obu_sequence_header: Option<OBU_Sequence_Header>, // Add this field
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU_Header {
    pub obu_forbidden_bit: u8,                              // 1 bit
    pub obu_type: OBU_TYPE,                                 // 4 bits
    pub obu_extension_flag: u8,                             // 1 bit
    pub obu_has_size_field: u8,                             // 1 bit
    pub obu_reserved_1bit: u8,                              // 1 bit
    pub obu_extension_header: Option<OBU_Extension_Header>, // 8 bits
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU_Extension_Header {
    pub temporal_id: u8,                     // 3 bits
    pub spatial_id: u8,                      // 2 bits
    pub extension_header_reserved_3bits: u8, // 3 bits
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU_Sequence_Header {
    pub seq_profile: u8,                    // 3 bits
    pub still_picture: u8,                  // 1 bit
    pub timing_info: Option<Timing_Info>,
    pub decoder_model_info: Option<Decoder_Model_Info>,
    pub operating_point_idc: Vec<u16>,      // 12 bits
    pub seq_level_idx: Vec<u8>,                  // 5 bits
    pub seq_tier: Vec<u8>,                       // 1 bit
    pub decoder_model_present_for_this_op: Vec<u8>, // 1 bit
    pub operating_parameters_info: Option<Operating_Parameters_Info>,
    pub initial_display_delay_present_for_this_op: Vec<u8>, // 1 bit
    pub initial_display_delay_minus_1: Option<Vec<u8>>,    // 4 bits
    pub c_operating_point_idc: u16,         // 12 bits
    pub max_frame_width_minus_one: u16,     //  2**frame_width_bits_minus_1+1 
    pub max_frame_height_minus_one: u16,    //  2**frame_height_bits_minus_1+1
    pub delta_frame_id_length_minus_2: Option<u8>, // 4 bits
    pub additional_frame_id_length_minus_1: Option<u8>, // 3 bits
    pub use_128x128_superblock: u8,         // 1 bit
    pub enable_filter_intra: u8,             // 1 bit
    pub enable_intra_edge_filter: u8,        // 1 bit
    pub enable_interintra_compound: u8,      // 1 bit
    pub enable_masked_compound: u8,          // 1 bit
    pub enable_warped_motion: u8,            // 1 bit
    pub enable_dual_filter: u8,              // 1 bit
    pub enable_order_hint: u8,               // 1 bit
    pub enable_jnt_comp: u8,                 // 1 bit
    pub enable_ref_frame_mvs: u8,            // 1 bit
    pub seq_force_screen_content_tools: u8, // 1 bit
    pub seq_force_integer_mv: u8,           // 1 bit
    pub order_hint_bits: u8,                 // 3 bits
    pub enable_superres: u8,                 // 1 bit
    pub enable_cdef: u8,                     // 1 bit
    pub enable_restoration: u8,              // 1 bit
    pub color_config: Color_Config,
    pub film_grain_params_present: u8,       // 1 bit
}

// 5.5.3 Timing info syntax
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Timing_Info {
    pub num_units_in_display_tick: u32,                 // 32 bits
    pub time_scale: u32,                        // 32 bits
    pub equal_picture_interval: u8,             //  1 bit
    pub num_ticks_per_picture_minus_1: Option<uvlc>,   // UVLC
}

// 5.5.4 Decoder model info
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Decoder_Model_Info {
    pub buffer_delay_length_minus_1: u8,   // 5 bits
    pub num_units_in_decoding_tick: u32,   // 32 bits
    pub buffer_removal_delay_length_minus_1: u8, // 5 bits
    pub frame_presentation_delay_length_minus_1: u8, // 5 bits
}

// 5.5.5 Operating parameters info
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Operating_Parameters_Info {
    pub decoder_buffer_delay: Vec<u32>, // 2**5=32 bits max
    pub encoder_buffer_delay: Vec<u32>, // 2**5=32 bits max
    pub low_delay_mode_flag: Vec<u8>,   // 1 bit
}

// 5.5.2 Color Config
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Color_Config {
    pub bit_depth: u8,
    pub mono_chrome: u8,
    pub num_planes: u8,
    pub color_primaries: consts::COLOR_PRIMARIES,
    pub transfer_characteristics: consts::TRANSFER_CHARACTERISTICS,
    pub matrix_coefficients: consts::MATRIX_COEFFICIENTS,
    pub color_range: u8,
    pub subsampling_x: u8,
    pub subsampling_y: u8,
    pub chroma_sample_position: consts::CHROMA_SAMPLE_POSITION,
    pub separate_uv_delta_q: u8,
}
