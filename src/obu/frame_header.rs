use bitstream_io::FromBitStream;

use crate::{
    consts::{FRAME_TYPE, NUM_REF_FRAMES, REFS_PER_FRAME},
    obu::context,
};

use super::{context::DecoderContext, sequence_header::SequenceHeader};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FrameHeader {}

impl FrameHeader {
    pub fn frame_header_obu<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        ctx: &mut DecoderContext,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let sequence_header = ctx.last_sequence_header.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "No last sequence header")
        })?;

        let seen_frame_header = 1u8;
        let id_len = if sequence_header.frame_id_numbers_present_flag != 0 {
            Some(
                sequence_header
                    .additional_frame_id_length_minus_1
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Additional frame id length minus 1 not present",
                        )
                    })?
                    + sequence_header
                        .delta_frame_id_length_minus_2
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Delta frame id length minus 2 not present",
                            )
                        })?
                    + 3,
            )
        } else {
            None
        };

        let all_frames = (1u8 << NUM_REF_FRAMES) - 1;
        let show_existing_frame: u8;
        let frame_type: FRAME_TYPE;
        let frame_is_intra: u8;
        let show_frame: u8;
        let showable_frame: u8;
        let frame_to_show_map_index: u8;
        let mut refresh_frame_flag: u8;
        let error_resilient_mode: u8;

        // It is a requirement of bitstream conformance
        // that the number of bits needed to read display_frame_id does not exceed 16.
        // This is equivalent to the constraint that idLen <= 16.
        let display_frame_id: u16;
        let frame_presentation_time: u32;

        if sequence_header.reduced_still_picture_header == 0 {
            show_existing_frame = r.read::<1, u8>()?;

            // Repeat same frame
            if show_existing_frame == 1 {
                frame_to_show_map_index = r.read::<3, u8>()?;
                if sequence_header.decoder_model_info_present_flag != 0
                    && sequence_header
                        .timing_info
                        .as_ref()
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Timing info not present",
                            )
                        })?
                        .equal_picture_interval
                        == 0
                {
                    let frame_presentation_time_bits: u32 = sequence_header
                        .decoder_model_info
                        .clone()
                        .unwrap()
                        .frame_presentation_delay_length_minus_1
                        as u32
                        + 1;
                    frame_presentation_time = r.read_var(frame_presentation_time_bits)?;
                }

                refresh_frame_flag = 0;
                if sequence_header.frame_id_numbers_present_flag != 0 {
                    display_frame_id = r.read_var(id_len.ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Id len not present")
                    })? as u32)?;
                }
                let ref_frame_type = context::RefFrameType()?;
                frame_type = ref_frame_type[frame_to_show_map_index as usize];

                if frame_type == FRAME_TYPE::KEY_FRAME {
                    refresh_frame_flag = all_frames;
                }
                if sequence_header.film_grain_params_present != 0 {
                    context::load_grain_params(frame_to_show_map_index)?;
                }

                return Ok(Self {});
            }

            frame_type = FRAME_TYPE::from_reader(r)?;
            frame_is_intra = (frame_type.eq(&FRAME_TYPE::INTRA_ONLY_FRAME)
                || frame_type.eq(&FRAME_TYPE::KEY_FRAME)) as u8;

            show_frame = r.read::<1, u8>()?;
            if show_frame != 0
                && sequence_header.decoder_model_info_present_flag != 0
                && sequence_header
                    .timing_info
                    .clone()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Timing info not present",
                        )
                    })?
                    .equal_picture_interval
                    == 0
            {
                let frame_presentation_time_bits: u32 = sequence_header
                    .decoder_model_info
                    .clone()
                    .unwrap()
                    .frame_presentation_delay_length_minus_1
                    as u32
                    + 1;
                frame_presentation_time = r.read_var(frame_presentation_time_bits)?;
            }
            if show_frame != 0 {
                showable_frame = (frame_type != FRAME_TYPE::KEY_FRAME) as u8;
            } else {
                showable_frame = r.read::<1, u8>()?;
            }
            if frame_type == FRAME_TYPE::SWITCH_FRAME
                || (frame_type == FRAME_TYPE::KEY_FRAME && show_frame != 0)
            {
                error_resilient_mode = 1;
            } else {
                error_resilient_mode = r.read::<1, u8>()?;
            }
        } else {
            show_existing_frame = 0;
            frame_type = FRAME_TYPE::KEY_FRAME;
            show_frame = 1;
            showable_frame = 0;
            frame_is_intra = 1;
        }

        // RefValid[ i ] should be set equal to 0 for i = 0..NUM_REF_FRAMES-1 before the decoding process begins
        let mut ref_valid: [u8; NUM_REF_FRAMES as usize] = [0; NUM_REF_FRAMES as usize];

        // The decoding process for the frame does not use values in RefOrderHint before they have been written
        // (written either by the decoding process for the current frame, or written when a previous frame was processed)
        let mut ref_order_hint: [u8; NUM_REF_FRAMES as usize] = [0; NUM_REF_FRAMES as usize];

        if frame_type == FRAME_TYPE::KEY_FRAME && showable_frame != 0 {
            ctx.ref_valid = [0; NUM_REF_FRAMES as usize];
            ctx.ref_order_hint = [0; NUM_REF_FRAMES as usize];
            for i in (ctx.last_frame_index.unwrap_or_default() + 1)..REFS_PER_FRAME as usize {
                if ctx.order_hints.len() <= i {
                    ctx.order_hints.push(0);
                } else {
                    ctx.order_hints[i as usize] = 0;
                }
            }
        }

        // next up disable_cdf_update #1

        todo!()
    }
}
