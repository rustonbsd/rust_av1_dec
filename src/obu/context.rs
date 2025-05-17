use crate::{
    consts::{FRAME_TYPE, NUM_REF_FRAMES, REFS_PER_FRAME},
    obu::sequence_header::SequenceHeader,
};

use super::{frame_header::FrameHeader, ObuHeader};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DecoderContext {
    pub obu_header: Option<ObuHeader>,
    pub last_sequence_header: Option<SequenceHeader>,
    pub last_frame_header: Option<FrameHeader>,

    // Reference Frame Management
    pub ref_frame_type: [FRAME_TYPE; NUM_REF_FRAMES as usize],
    pub ref_valid: [u8; NUM_REF_FRAMES as usize],
    pub ref_order_hint: [u8; NUM_REF_FRAMES as usize],
    pub order_hints: Vec<u8>,
    pub order_hint: u8,
    pub ref_frame_id: [u8; NUM_REF_FRAMES as usize],

    pub last_frame_index: Option<usize>,
    pub prev_frame_id: Option<u8>,
    pub current_frame_id: Option<u8>,
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
            order_hints: [0; REFS_PER_FRAME as usize].to_vec(),
            order_hint: 0,
            ref_frame_id: [0; NUM_REF_FRAMES as usize],

            last_frame_index: None,
            prev_frame_id: None,
            current_frame_id: None,
        }
    }

    pub fn mark_ref_frames(&mut self, id_len: u8) -> Result<(), std::io::Error> {
        let diff_len = self
            .last_sequence_header
            .clone()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "No last sequence header")
            })?
            .delta_frame_id_length_minus_2
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "No delta frame id length minus 2",
                )
            })?
            + 2;
        let shifted_diff_len = 1u8 << diff_len;
        let current_frame_id = self.current_frame_id.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "No current frame id")
        })?;
        
        for i in 0..NUM_REF_FRAMES {
            if current_frame_id > shifted_diff_len {
                if self.ref_frame_id[i as usize] > current_frame_id
                    || self.ref_frame_id[i as usize] < current_frame_id - shifted_diff_len
                {
                    self.ref_valid[i as usize] = 0;
                }
            } else {
                if self.ref_frame_id[i as usize] > current_frame_id
                    && self.ref_frame_id[i as usize] < ((1u8 << id_len) + current_frame_id
                        - shifted_diff_len)
                {
                    self.ref_valid[i as usize] = 0;
                }
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub fn choose_operating_point() -> Result<usize, std::io::Error> {
    log::debug!("obu->handlers->choose_operating_point()");
    Ok(0usize)
}

// 7.4 Decode frame wrapup process
pub fn decode_frame_wrapup() -> Result<(), std::io::Error> {
    log::debug!("obu->handlers->decode_frame_wrapup()");
    Ok(())
}

pub fn load_grain_params(frame_to_show_map_idx: u8) -> Result<(), std::io::Error> {
    log::debug!("obu->handlers->load_grain_params()");
    Ok(())
}
