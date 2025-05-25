use crate::{
    consts::{
        FRAME_TYPE, MAX_SEGMENTS, NUM_REF_FRAMES, REF_FRAME, REF_FRAME_LIST, REFS_PER_FRAME,
        SEG_LVL_ALT_Q,
    },
    generics::clip3,
    obu::sequence_header::SequenceHeader,
};

use super::{
    ObuHeader,
    frame_header::{
        DeltaQParams, FrameHeader, FrameSize, QuantizationParams, RenderSize, SegmentationParams,
    },
};

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
                    && self.ref_frame_id[i as usize]
                        < ((1u8 << id_len) + current_frame_id - shifted_diff_len)
                {
                    self.ref_valid[i as usize] = 0;
                }
            }
        }

        Ok(())
    }
}

pub fn choose_operating_point() -> Result<usize, std::io::Error> {
    log::debug!("[-] obu->handlers->choose_operating_point() default: 0");
    Ok(0usize)
}

// 7.4 Decode frame wrapup process
pub fn decode_frame_wrapup() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->decode_frame_wrapup()");
    todo!("Implement decode_frame_wrapup");
    Ok(())
}

pub fn load_grain_params(frame_to_show_map_idx: u8) -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->load_grain_params()");
    todo!("Implement load_grain_params");
    Ok(())
}

/*
    Spec-Note: 
    
    Multiple reference frames can share the same value for OrderHint and care needs to be 
    taken to handle this case consistently. The reference implementation uses an equivalent 
    implementation based on sorting the reference frames based on their expected output order, 
    with ties broken based on the reference frame index. 
*/
pub fn set_frame_refs(
    last_frame_index: u8,
    gold_frame_index: u8,
    ctx: &mut DecoderContext,
) -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->set_frame_refs()");

    ctx.ref_frame_index = [0; REFS_PER_FRAME as usize];

    assert!(last_frame_index < REFS_PER_FRAME);
    assert!(gold_frame_index < REFS_PER_FRAME);
    assert!(gold_frame_index as i16 - last_frame_index as i16 >= 0);

    ctx.ref_frame_index[0] = last_frame_index as i8;
    ctx.ref_frame_index[(gold_frame_index - last_frame_index) as usize] = gold_frame_index as i8;

    ctx.used_frame = [0; NUM_REF_FRAMES as usize];
    ctx.used_frame[last_frame_index as usize] = 1;
    ctx.used_frame[gold_frame_index as usize] = 1;

    // IMPL VARS INTO CONTEXT THAT ARE LOCAL [!]
    let cur_frame_hint = 1 << (ctx.order_hint_bits - 1);
    let mut shifted_order_hints = [0; NUM_REF_FRAMES as usize];

    for i in 0..NUM_REF_FRAMES as usize {
        shifted_order_hints[i] =
            cur_frame_hint + get_relative_dist(ctx.ref_order_hint[i], ctx.order_hint, ctx)?;
    }

    let last_order_hint = shifted_order_hints[last_frame_index as usize];
    // [c] It is a requirement of bitstream conformance that lastOrderHint is strictly less than curFrameHint.
    assert!(last_order_hint < cur_frame_hint);

    let gold_order_hint = shifted_order_hints[gold_frame_index as usize];
    // [c] It is a requirement of bitstream conformance that goldOrderHint is strictly less than curFrameHint.
    assert!(gold_order_hint < cur_frame_hint);

    // find_latest_backwards( )
    fn find_latest_backwards(
        shifted_order_hints: &[u16; NUM_REF_FRAMES as usize],
        cur_frame_hint: u16,
        ctx: &mut DecoderContext,
    ) -> i8 {
        let candidate: Option<(u16, i8)> = (0..NUM_REF_FRAMES as usize)
            .filter_map(|i| {
                if ctx.used_frame[i] == 0 && shifted_order_hints[i] >= cur_frame_hint {
                    Some((shifted_order_hints[i], i as i8))
                } else {
                    None
                }
            })
            .max_by_key(|(hint, index)| (*hint, *index));

        match candidate {
            Some((_, index)) => index,
            None => -1,
        }
    }

    let _ref = find_latest_backwards(&shifted_order_hints, cur_frame_hint, ctx);
    if _ref >= 0 {
        let _index = REF_FRAME::ALTREF_FRAME as u8 - REF_FRAME::LAST_FRAME as u8;
        ctx.ref_frame_index[_index as usize] = _ref as i8;
        ctx.used_frame[_ref as usize] = 1;
    }

    fn find_earliest_backward(
        shifted_order_hints: &[u16; NUM_REF_FRAMES as usize],
        cur_frame_hint: u16,
        ctx: &mut DecoderContext,
    ) -> i8 {
        let candidate: Option<(u16, i8)> = (0..NUM_REF_FRAMES as usize)
            .filter_map(|i| {
                if ctx.used_frame[i] == 0 && shifted_order_hints[i] >= cur_frame_hint {
                    Some((shifted_order_hints[i], i as i8))
                } else {
                    None
                }
            })
            .min_by_key(|(hint, index)| (*hint, *index));

        match candidate {
            Some((_, index)) => index,
            None => -1,
        }
    }

    let _ref = find_earliest_backward(&shifted_order_hints, cur_frame_hint, ctx);
    if _ref >= 0 {
        let _index = REF_FRAME::BWDREF_FRAME as u8 - REF_FRAME::LAST_FRAME as u8;
        ctx.ref_frame_index[_index as usize] = _ref as i8;
        ctx.used_frame[_ref as usize] = 1;
    }

    let _ref = find_earliest_backward(&shifted_order_hints, cur_frame_hint, ctx);
    if _ref >= 0 {
        let _index = REF_FRAME::ALTREF2_FRAME as u8 - REF_FRAME::LAST_FRAME as u8;
        ctx.ref_frame_index[_index as usize] = _ref as i8;
        ctx.used_frame[_ref as usize] = 1;
    }

    fn find_latest_forward(
        shifted_order_hints: &[u16; NUM_REF_FRAMES as usize],
        cur_frame_hint: u16,
        ctx: &mut DecoderContext,
    ) -> i8 {
        let candidate: Option<(u16, i8)> = (0..NUM_REF_FRAMES as usize)
            .filter_map(|i| {
                if ctx.used_frame[i] == 0 && shifted_order_hints[i] < cur_frame_hint {
                    Some((shifted_order_hints[i], i as i8))
                } else {
                    None
                }
            })
            .max_by_key(|(hint, index)| (*hint, *index));

        match candidate {
            Some((_, index)) => index,
            None => -1,
        }
    }

    let mut ref_frame;
    for i in 0..(REFS_PER_FRAME - 2) as usize {
        ref_frame = REF_FRAME_LIST[i];
        if ctx.ref_frame_index[REF_FRAME_LIST[i] as usize - REF_FRAME::LAST_FRAME as usize] < 0 {
            let _ref = find_latest_forward(&shifted_order_hints, cur_frame_hint, ctx);
            if _ref >= 0 {
                ctx.ref_frame_index[ref_frame as usize - REF_FRAME::LAST_FRAME as usize] =
                    _ref as i8;
                ctx.used_frame[_ref as usize] = 1;
            }
        }
    }

    let mut _ref: i8 = -1;
    let mut earliest_order_hint: u16 = 0;
    for i in 0..NUM_REF_FRAMES as usize {
        let hint = shifted_order_hints[i];
        if _ref < 0 || hint < earliest_order_hint {
            _ref = i as i8;
            earliest_order_hint = hint;
        }
    }

    let candidate: Option<(u16, i8)> = (0..NUM_REF_FRAMES as usize)
            .map(|i| 
                (shifted_order_hints[i],i as i8)
            )
            .max_by_key(|(hint, index)| (*hint, *index));

    let _ref = match candidate {
        Some((_, index)) => index,
        None => -1,
    };
    
    ctx.ref_frame_index
        .iter_mut()
        .map(|rf| if *rf < 0 { _ref } else { *rf });

    Ok(())
}

pub fn init_non_coeff_cdfs() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->init_non_coeff_cdfs()");
    todo!("Implement init_non_coeff_cdfs");
    Ok(())
}

pub fn setup_past_independence() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->setup_past_independence()");
    todo!("Implement setup_past_independence");
    Ok(())
}

pub fn load_cdfs(frame_to_show_map_idx: u8) -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->load_cdfs()");
    todo!("Implement load_cdfs");
    Ok(())
}

pub fn load_previous() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->load_previous()");
    todo!("Implement load_previous");
    Ok(())
}

pub fn motion_field_estimation() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->motion_field_estimation()");
    todo!("Implement motion_field_estimation");
    Ok(())
}

pub fn tile_info() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->tile_info()");
    todo!("Implement tile_info");
    Ok(())
}

pub fn quantization_params() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->quantization_params()");
    todo!("Implement quantization_params");
    Ok(())
}

pub fn init_coeff_cdfs() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->init_coeff_cdfs()");
    todo!("Implement init_coeff_cdfs");
    Ok(())
}

pub fn load_previous_segment_ids() -> Result<(), std::io::Error> {
    log::debug!("[] obu->handlers->load_previous_segment_ids()");
    todo!("Implement load_previous_segment_ids");
    Ok(())
}

pub fn seg_feature_active_idx(idx: u8, feature: u8, seg_params: &SegmentationParams) -> u8 {
    if seg_params.segmentation_enabled != 0
        && seg_params.feature_enabled[idx as usize][feature as usize] != 0
    {
        1u8
    } else {
        0u8
    }
}

pub fn get_qindex(
    ignore_delta_q: u8,
    segment_id: u8,
    seg_params: &SegmentationParams,
    quantization_params: &QuantizationParams,
    delta_q_params: &DeltaQParams,
    ctx: &mut DecoderContext,
) -> Result<u8, std::io::Error> {
    log::debug!("[x] obu->handlers->get_qindex()");
    if seg_feature_active_idx(segment_id, SEG_LVL_ALT_Q, seg_params) == 1 {
        let data = seg_params.feature_data[segment_id as usize][SEG_LVL_ALT_Q as usize];
        let mut q_index = quantization_params.base_q_index as i16 + data;

        if ignore_delta_q == 0 && delta_q_params.delta_q_present == 1 {
            q_index = ctx.current_q_index as i16 + data;
        }

        Ok(clip3(0, 255, q_index) as u8)
    } else {
        if ignore_delta_q == 0 && delta_q_params.delta_q_present == 1 {
            Ok(ctx.current_q_index)
        } else {
            Ok(quantization_params.base_q_index)
        }
    }
}

pub fn get_relative_dist(a: u8, b: u8, ctx: &mut DecoderContext) -> Result<u16, std::io::Error> {
    let sequence_header = ctx.last_sequence_header.clone().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "No last sequence header")
    })?;

    if sequence_header.enable_order_hint == 0 {
        return Ok(0);
    }

    let diff: u16 = a as u16 - b as u16;
    let m: u16 = 1u16 << (ctx.order_hint_bits as u16 - 1u16);
    Ok((diff & (m - 1)) - (diff & m))
}
