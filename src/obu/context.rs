use crate::{consts::{FRAME_TYPE, NUM_REF_FRAMES, REFS_PER_FRAME}, obu::sequence_header::SequenceHeader};

use super::frame_header::FrameHeader;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DecoderContext {
    pub last_sequence_header: Option<SequenceHeader>,
    pub last_frame_header: Option<FrameHeader>,

    // Reference Frame Management
    pub ref_valid: [u8; NUM_REF_FRAMES as usize],
    pub ref_order_hint: [u8; NUM_REF_FRAMES as usize],
    pub order_hints: Vec<u8>,
    pub last_frame_index: Option<usize>,
}

impl DecoderContext {
    pub fn new() -> Self {
        Self {
            last_sequence_header: None,
            last_frame_header: None,

            ref_valid: [0; NUM_REF_FRAMES as usize],
            ref_order_hint: [0; NUM_REF_FRAMES as usize],
            order_hints: [0; REFS_PER_FRAME as usize].to_vec(),
            last_frame_index: None,
        }
    }
}


#[allow(dead_code)]
pub fn choose_operating_point() -> Result<usize,std::io::Error> {
    log::debug!("obu->handlers->choose_operating_point()");
    Ok(0usize)
}

// 7.4 Decode frame wrapup process
pub fn decode_frame_wrapup() -> Result<(),std::io::Error> {
    log::debug!("obu->handlers->decode_frame_wrapup()");
    Ok(())
}


pub fn RefFrameType() -> Result<Vec<FRAME_TYPE>,std::io::Error> {
    log::debug!("obu->handlers->RefFrameType()");
    Ok(vec![FRAME_TYPE::KEY_FRAME])
}

pub fn load_grain_params(frame_to_show_map_idx: u8) -> Result<(),std::io::Error> {
    log::debug!("obu->handlers->load_grain_params()");
    Ok(())
}