use crate::{
    consts::{
        FRAME_TYPE, MAX_SEGMENTS, NUM_REF_FRAMES, REFS_PER_FRAME, REF_FRAME, REF_FRAME_LIST, SEG_LVL_ALT_Q
    },
    generics::clip3,
    obu::{frame_header::{DeltaQParams, FrameHeader, FrameSize, QuantizationParams, RenderSize, SegmentationParams}, ObuHeader, SequenceHeader},
};

pub mod cdf;
pub mod sequence_header;
pub mod frame_header;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DecoderContext {
    pub obu_header: Option<ObuHeader>,
    pub last_sequence_header: Option<SequenceHeader>,
    pub last_frame_header: Option<FrameHeader>,

    // Reference Frame Management
    pub ref_frame_type: [FRAME_TYPE; NUM_REF_FRAMES as usize],
    pub ref_valid: [u8; NUM_REF_FRAMES as usize],
    pub ref_order_hint: [u8; NUM_REF_FRAMES as usize],
    pub ref_frame_id: [u8; NUM_REF_FRAMES as usize],
    pub ref_frame_sign_bias: [u8; NUM_REF_FRAMES as usize],
    pub ref_frame_sizes: [FrameSize; NUM_REF_FRAMES as usize],
    pub ref_frame_render_sizes: [RenderSize; NUM_REF_FRAMES as usize],
    pub used_frame: [u8; NUM_REF_FRAMES as usize],

    pub ref_frame_index: [i8; REFS_PER_FRAME as usize],

    pub order_hints: Vec<u8>,
    pub order_hint: u8,
    pub order_hint_bits: u8,
    pub shifted_order_hints: [u16; NUM_REF_FRAMES as usize],

    pub last_frame_index: Option<u8>,
    pub prev_frame_id: Option<u8>,
    pub current_frame_id: Option<u8>,

    pub current_q_index: u8,

    pub lossless_array: [u8; MAX_SEGMENTS as usize],
    pub seg_qm_level: [[u8; MAX_SEGMENTS as usize]; 3],

    pub seen_frame_header: u8,
    pub tile_num: u16,
}

impl DecoderContext {
    pub fn new() -> Self {
        Self {
            obu_header: None,
            last_sequence_header: None,
            last_frame_header: None,

            ref_frame_type: [FRAME_TYPE::KEY_FRAME; NUM_REF_FRAMES as usize],
            ref_valid: [0; NUM_REF_FRAMES as usize],
            ref_order_hint: [0; NUM_REF_FRAMES as usize],
            ref_frame_id: [0; NUM_REF_FRAMES as usize],
            ref_frame_sign_bias: [0; NUM_REF_FRAMES as usize],
            ref_frame_sizes: [FrameSize::default(); NUM_REF_FRAMES as usize],
            ref_frame_render_sizes: [RenderSize::default(); NUM_REF_FRAMES as usize],
            used_frame: [0; NUM_REF_FRAMES as usize],

            ref_frame_index: [-1; REFS_PER_FRAME as usize],

            order_hints: [0; REFS_PER_FRAME as usize].to_vec(),
            order_hint: 0,
            order_hint_bits: 0,
            shifted_order_hints: [0; NUM_REF_FRAMES as usize],

            last_frame_index: None,
            prev_frame_id: None,
            current_frame_id: None,

            current_q_index: 0,
            lossless_array: [0; MAX_SEGMENTS as usize],
            seg_qm_level: [[0; MAX_SEGMENTS as usize]; 3],

            seen_frame_header: 0,
            tile_num: 0,
        }
    }
}
