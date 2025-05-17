use bitstream_io::FromBitStream;

use crate::{
    consts::{
        FRAME_TYPE, NUM_REF_FRAMES, PRIMARY_REF_NONE, REFS_PER_FRAME, SELECT_INTEGER_MV, SELECT_SCREEN_CONTENT_TOOLS
    },
    obu::{context, sequence_header::TimingInfo},
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
        let sequence_header = ctx.last_sequence_header.clone().ok_or_else(|| {
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
        let mut error_resilient_mode: Option<u8> = None;

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
                let ref_frame_type = ctx.ref_frame_type;
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
                error_resilient_mode = Some(1);
            } else {
                error_resilient_mode = Some(r.read::<1, u8>()?);
            }
        } else {
            show_existing_frame = 0;
            frame_type = FRAME_TYPE::KEY_FRAME;
            show_frame = 1;
            showable_frame = 0;
            frame_is_intra = 1;
        }

        let disable_cdf_update: u8;
        let allow_screen_content_tools: u8;
        let mut force_integer_mv: u8;
        let frame_size_override_flag: u8;
        let order_hint: u8;
        let primary_ref_frame: u8;  // 3 bits
        let buffer_removal_time_present: u8;
        let mut buffer_removal_time: Option<Vec<u32>>;

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
        disable_cdf_update = r.read::<1, u8>()?;
        if sequence_header.seq_force_screen_content_tools == SELECT_SCREEN_CONTENT_TOOLS {
            allow_screen_content_tools = r.read::<1, u8>()?;
        } else {
            allow_screen_content_tools = sequence_header.seq_force_screen_content_tools;
        }
        if allow_screen_content_tools != 0 {
            if sequence_header.seq_force_integer_mv == SELECT_INTEGER_MV {
                force_integer_mv = r.read::<1, u8>()?;
            } else {
                force_integer_mv = sequence_header.seq_force_integer_mv;
            }
        } else {
            force_integer_mv = 0;
        }

        if frame_is_intra != 0 {
            force_integer_mv = 1;
        }

        if sequence_header.frame_id_numbers_present_flag != 0 {
            ctx.prev_frame_id = ctx.current_frame_id;
            ctx.current_frame_id = Some(r.read_var(id_len.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Id len not present")
            })? as u32)?);
            ctx.mark_ref_frames(id_len.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Id len not present")
            })?)?;
        } else {
            ctx.current_frame_id = Some(0);
        }

        if frame_type == FRAME_TYPE::SWITCH_FRAME {
            frame_size_override_flag = 0;
        } else {
            frame_size_override_flag = r.read::<1, u8>()?;
        }

        order_hint = r.read_var(sequence_header.order_hint_bits as u32)?;
        ctx.order_hint = order_hint;

        if frame_is_intra != 0 || (error_resilient_mode.is_some() && error_resilient_mode.unwrap() != 0){
            primary_ref_frame = PRIMARY_REF_NONE;
        } else {
            primary_ref_frame = r.read::<3, u8>()?;
        }

        if sequence_header.decoder_model_info_present_flag != 0 {
            buffer_removal_time_present = r.read::<1, u8>()?;
            if buffer_removal_time_present != 0 {
                /*for ( opNum = 0; opNum <= operating_points_cnt_minus_1; opNum++ ) {	 
                if ( decoder_model_present_for_this_op[ opNum ] ) {	 
                    opPtIdc = operating_point_idc[ opNum ]	 
                    inTemporalLayer = ( opPtIdc >> temporal_id ) & 1	 
                    inSpatialLayer = ( opPtIdc >> ( spatial_id + 8 ) ) & 1	 
                    if ( opPtIdc == 0 || ( inTemporalLayer && inSpatialLayer ) ) {	 
                        n = buffer_removal_time_length_minus_1 + 1	 
                        buffer_removal_time[ opNum ]	f(n)
                    }	 
                }	 
            } */
                for op_num in 0..sequence_header.operating_points_cnt as usize {
                    if sequence_header.decoder_model_present_for_this_op.len() > op_num && sequence_header.operating_point_idc.len() > op_num && sequence_header.decoder_model_present_for_this_op[op_num] != 0 {
                        let op_pt_idc = sequence_header.operating_point_idc[op_num];

                        // temporal_id specifies the temporal level of the data contained in the OBU. 
                        // In layer-specific OBUs, when temporal_id is not present it is inferred to be equal to 0.
                        let temporal_id = if let Some(obu_header) = ctx.obu_header.clone() {
                            if let Some(extension_header) = obu_header.extension_header {
                                extension_header.temporal_id
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        let in_temporal_layer = (op_pt_idc >> temporal_id) & 1;

                        // spatial_id specifies the spatial level of the data contained in the OBU. 
                        // In layer-specific OBUs, when spatial_id is not present it is inferred to be equal to 0.
                        let spatial_id = if let Some(obu_header) = ctx.obu_header.clone() {
                            if let Some(extension_header) = obu_header.extension_header {
                                extension_header.spatial_id
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        let in_spatial_layer = (op_pt_idc >> (spatial_id + 8)) & 1;

                        if op_pt_idc == 0 || (in_temporal_layer != 0 && in_spatial_layer != 0) {
                            let n = sequence_header.decoder_model_info.clone().ok_or_else(|| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Decoder model info not present",
                                )
                            })?.buffer_removal_delay_length_minus_1 + 1;
                            
                            // next impl buffer_removal_time
                        }
                    }
                }
                    
            }
        }

        todo!()
    }
}
