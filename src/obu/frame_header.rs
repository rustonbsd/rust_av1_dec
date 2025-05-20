use bitstream_io::FromBitStream;

use crate::{
    consts::{
        FRAME_TYPE, INTERPOLATION_FILTER, MAX_TILE_AREA, MAX_TILE_COLS, MAX_TILE_ROWS,
        MAX_TILE_WIDTH, NUM_REF_FRAMES, PRIMARY_REF_NONE, REFS_PER_FRAME, SELECT_INTEGER_MV,
        SELECT_SCREEN_CONTENT_TOOLS, SUPERRES_DENOM_BITS, SUPERRES_DENOM_MIN, SUPERRES_NUM,
    },
    generics::Ns,
    obu::{context, sequence_header::TimingInfo},
};

use super::{context::DecoderContext, sequence_header::SequenceHeader};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FrameHeader {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FrameSize {
    pub frame_width: u16,    // up tp 16 bits
    pub frame_height: u16,   // up to 16 bits
    pub use_superres: u8,    // 1 bit
    pub coded_denom: u8,     // 3 bit
    pub superres_denom: u8,  // 3 bit
    pub upscaled_width: u16, // up to 16 bits
    pub mi_cols: u16,        // up to 16 bits
    pub mi_rows: u16,        // up to 16 bits
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RenderSize {
    pub render_and_frame_size_different: u8, // 1 bit
    pub render_width: u16,                   // up to 16 bits
    pub render_height: u16,                  // up to 16 bits
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TileInfo {
    pub tile_cols: u16,
    pub tile_rows: u16,
    pub tile_cols_log2: u16,
    pub tile_rows_log2: u16,
    pub mi_col_starts: Vec<u16>,
    pub mi_row_starts: Vec<u16>,
    pub tile_size_bytes: Option<u8>,
    pub context_update_tile_id: u8,
}

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
        let primary_ref_frame: u8; // 3 bits
        let buffer_removal_time_present: u8;
        let mut buffer_removal_time: Option<Vec<u32>> = None;

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

        if frame_is_intra != 0
            || (error_resilient_mode.is_some() && error_resilient_mode.unwrap() != 0)
        {
            primary_ref_frame = PRIMARY_REF_NONE;
        } else {
            primary_ref_frame = r.read::<3, u8>()?;
        }

        if sequence_header.decoder_model_info_present_flag != 0 {
            buffer_removal_time_present = r.read::<1, u8>()?;
            if buffer_removal_time_present != 0 {
                buffer_removal_time = Some(Vec::new());
                for op_num in 0..sequence_header.operating_points_cnt as usize {
                    buffer_removal_time.as_mut().unwrap().push(0);

                    if sequence_header.decoder_model_present_for_this_op.len() > op_num
                        && sequence_header.operating_point_idc.len() > op_num
                        && sequence_header.decoder_model_present_for_this_op[op_num] != 0
                    {
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
                            let n = sequence_header
                                .decoder_model_info
                                .clone()
                                .ok_or_else(|| {
                                    std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Decoder model info not present",
                                    )
                                })?
                                .buffer_removal_delay_length_minus_1
                                + 1;

                            buffer_removal_time.as_mut().unwrap()[op_num] = r.read_var(n as u32)?;
                        }
                    }
                }
            }
        }

        let mut allow_high_precision_mv = 0;
        let mut use_ref_frame_mvs: u8 = 0;
        let mut allow_intrabc = 0;
        let mut ref_order_hint: [u8; NUM_REF_FRAMES as usize] = [0; NUM_REF_FRAMES as usize];
        let frame_refs_short_signaling: u8;
        let last_frame_index: u8;
        let gold_frame_index: u8;
        let mut ref_frame_index: [u8; REFS_PER_FRAME as usize] = [0; REFS_PER_FRAME as usize];
        let mut expected_frame_id: [u16; NUM_REF_FRAMES as usize] = [0; NUM_REF_FRAMES as usize];
        let frame_size: FrameSize;
        let render_size: RenderSize;
        let interpolation_filter: INTERPOLATION_FILTER;
        let is_motion_mode_switchable: u8;

        if frame_type == FRAME_TYPE::SWITCH_FRAME
            || (frame_type == FRAME_TYPE::KEY_FRAME && show_frame != 0)
        {
            refresh_frame_flag = all_frames;
        } else {
            refresh_frame_flag = r.read::<8, u8>()?;
        }

        if frame_is_intra == 0 || refresh_frame_flag != all_frames {
            if error_resilient_mode.is_some()
                && error_resilient_mode.unwrap() != 0
                && sequence_header.enable_order_hint != 0
            {
                for i in 0..NUM_REF_FRAMES as usize {
                    ref_order_hint[i] = r.read_var(sequence_header.order_hint_bits as u32)?;
                    if ref_order_hint[i] != ctx.ref_order_hint[i] {
                        ctx.ref_valid[i] = 0;
                    }
                }
            }
        }

        if frame_is_intra != 0 {
            frame_size = FrameSize::frame_size(r, frame_size_override_flag, ctx)?;
            render_size = RenderSize::render_size(r, &frame_size)?;
            if allow_screen_content_tools != 0
                && frame_size.upscaled_width == frame_size.frame_width
            {
                allow_intrabc = r.read::<1, u8>()?;
            }
        } else {
            if sequence_header.enable_order_hint == 0 {
                frame_refs_short_signaling = 0;
            } else {
                frame_refs_short_signaling = r.read::<1, u8>()?;
                if frame_refs_short_signaling != 0 {
                    last_frame_index = r.read::<3, u8>()?;
                    gold_frame_index = r.read::<3, u8>()?;
                    context::set_frame_refs()?;
                }
            }

            let _current_frame_id = ctx.current_frame_id.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "No current frame id")
            })?;
            let _id_len = id_len.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Id len not present")
            })?;
            for i in 0..REFS_PER_FRAME as usize {
                if frame_refs_short_signaling == 0 {
                    ref_frame_index[i] = r.read::<3, u8>()?;
                }
                if sequence_header.frame_id_numbers_present_flag != 0 {
                    let n = sequence_header
                        .delta_frame_id_length_minus_2
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Delta frame id length minus 2 not present",
                            )
                        })?
                        + 2;
                    let _delta_frame_id = r.read_var::<u16>(n as u32)? + 1;
                    expected_frame_id[i] = ((_current_frame_id as u16) + (1 << _id_len as u16)
                        - _delta_frame_id)
                        % (1 << _id_len as u16);
                }
            }

            if frame_size_override_flag != 0
                && (error_resilient_mode.is_none() || error_resilient_mode.unwrap() == 0)
            {
                let (_frame_size, _render_size) =
                    frame_size_with_refs(r, ref_frame_index, frame_size_override_flag, ctx)?;
                frame_size = _frame_size;
                render_size = _render_size;
            } else {
                frame_size = FrameSize::frame_size(r, frame_size_override_flag, ctx)?;
                render_size = RenderSize::render_size(r, &frame_size)?;
            }

            allow_high_precision_mv = if force_integer_mv != 0 {
                0
            } else {
                r.read::<1, u8>()?
            };

            // read_interpolation_filter( )
            interpolation_filter = read_interpolation_filter(r)?;

            is_motion_mode_switchable = r.read::<1, u8>()?;
            use_ref_frame_mvs = if error_resilient_mode.is_some()
                && error_resilient_mode.unwrap() != 0
                && sequence_header.enable_ref_frame_mvs == 0
            {
                0
            } else {
                r.read::<1, u8>()?
            };

            /*
            for ( i = 0; i < REFS_PER_FRAME; i++ ) {
                refFrame = LAST_FRAME + i
                hint = RefOrderHint[ ref_frame_idx[ i ] ]
                OrderHints[ refFrame ] = hint
                if ( !enable_order_hint ) {
                    RefFrameSignBias[ refFrame ] = 0
                } else {
                    RefFrameSignBias[ refFrame ] = get_relative_dist( hint, OrderHint) > 0
                }
            }
            */
            for i in 0..REFS_PER_FRAME as usize {
                let ref_frame = ctx.current_frame_id.unwrap_or_default() + i as u8;
                let hint = ctx.ref_order_hint[ref_frame_index[i] as usize];
                ctx.order_hints[ref_frame as usize] = hint;
                if sequence_header.enable_order_hint == 0 {
                    ctx.ref_frame_sign_bias[ref_frame as usize] = 0;
                } else {
                    // get_relative_dist( hint, OrderHint) > 0
                    ctx.ref_frame_sign_bias[ref_frame as usize] =
                        if get_relative_dist(hint, order_hint, ctx)? > 0 {
                            1
                        } else {
                            0
                        };
                }
            }
        }

        let disable_frame_end_update_cdf: u8;

        if sequence_header.reduced_still_picture_header != 0 || disable_cdf_update != 0 {
            disable_frame_end_update_cdf = 1;
        } else {
            disable_frame_end_update_cdf = r.read::<1, u8>()?;
        }

        if primary_ref_frame == PRIMARY_REF_NONE {
            context::init_non_coeff_cdfs()?;
            context::setup_past_independence()?;
        } else {
            context::load_cdfs(ref_frame_index[primary_ref_frame as usize])?;
            context::load_previous()?;
        }

        if use_ref_frame_mvs == 1 {
            context::motion_field_estimation()?;
        }

        // NEXT UP NEXT [!] quantization_params( )
        let tile_info = TileInfo::tile_info(r, &frame_size, ctx)?;

        /*
            tile_info( )
            quantization_params( )
            segmentation_params( )
            delta_q_params( )
            delta_lf_params( )
            if ( primary_ref_frame == PRIMARY_REF_NONE ) {
                init_coeff_cdfs( )
            } else {
                load_previous_segment_ids( )
            }
        */

        todo!()
    }
}

impl FrameSize {
    pub fn frame_size<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        frame_size_override_flag: u8,
        ctx: &mut DecoderContext,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let sequence_header = ctx.last_sequence_header.clone().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "No last sequence header")
        })?;
        let mut frame_width: u16;
        let frame_height: u16;

        if frame_size_override_flag != 0 {
            frame_width = r.read_var::<u16>(sequence_header.frame_width_bits as u32)? + 1;
            frame_height = r.read_var::<u16>(sequence_header.frame_height_bits as u32)? + 1;
        } else {
            frame_width = sequence_header.max_frame_width_minus_one + 1;
            frame_height = sequence_header.max_frame_height_minus_one + 1;
        }

        // superres_params( )
        let (use_superres, coded_denom, superres_denom, upscaled_width) =
            superres_params(r, sequence_header, &mut frame_width)?;

        // compute_image_size( )
        let (mi_cols, mi_rows) = compute_image_size(frame_width, frame_height);

        Ok(Self {
            frame_width,
            frame_height,
            use_superres,
            coded_denom: coded_denom.unwrap_or_default(),
            superres_denom,
            upscaled_width,
            mi_cols,
            mi_rows,
        })
    }
}

impl RenderSize {
    fn render_size<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        frame_size: &FrameSize,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let render_and_frame_size_different = r.read::<1, u8>()?;
        let (render_width, render_height) = if render_and_frame_size_different == 1 {
            (r.read_var::<u16>(16)? + 1, r.read_var::<u16>(16)? + 1)
        } else {
            (frame_size.upscaled_width, frame_size.frame_height)
        };
        Ok(Self {
            render_and_frame_size_different,
            render_width,
            render_height,
        })
    }
}

impl TileInfo {
    pub fn tile_info<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        frame_size: &FrameSize,
        ctx: &mut DecoderContext,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let sequence_header = ctx.last_sequence_header.clone().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Last sequence header not present",
            )
        })?;

        let sb_cols = if sequence_header.use_128x128_superblock != 0 {
            (frame_size.mi_cols + 31) >> 5
        } else {
            (frame_size.mi_cols + 15) >> 4
        };
        let sb_rows = if sequence_header.use_128x128_superblock != 0 {
            (frame_size.mi_rows + 31) >> 5
        } else {
            (frame_size.mi_rows + 15) >> 4
        };
        let sb_shift = if sequence_header.use_128x128_superblock != 0 {
            5u8
        } else {
            4u8
        };
        let sb_size = sb_shift + 2u8;

        let max_tile_width_sb = (MAX_TILE_WIDTH >> sb_size) as u16;
        let max_tile_height_sb: u16;
        let mut max_tile_area_sb = MAX_TILE_AREA >> (2 * sb_size);
        let min_log2_tile_cols = tile_log2(max_tile_width_sb as u32, sb_cols as u32);
        let max_log2_tile_cols = tile_log2(1, std::cmp::min(sb_cols as u32, MAX_TILE_COLS));
        let max_log2_tile_rows = tile_log2(1, std::cmp::min(sb_rows as u32, MAX_TILE_ROWS));
        let min_log2_tiles = std::cmp::max(
            min_log2_tile_cols,
            tile_log2(max_tile_area_sb, sb_rows as u32 * sb_cols as u32),
        );

        let mut mi_col_starts: Vec<u16> = vec![];
        let mut mi_row_starts: Vec<u16> = vec![];

        let mut tile_rows_log2: u16;
        let mut tile_cols_log2: u16;

        let tile_cols: u16;
        let tile_rows: u16;

        let uniform_tile_spacing_flag = r.read::<1, u8>()?;
        if uniform_tile_spacing_flag != 0 {
            // cols
            tile_cols_log2 = min_log2_tile_cols;
            while tile_cols_log2 < max_log2_tile_cols {
                if r.read::<1, u8>()? == 1 {
                    tile_cols_log2 += 1;
                } else {
                    break;
                }
            }

            let tile_width_sb = (sb_cols + (1 << tile_cols_log2) - 1) >> tile_cols_log2;

            let mut i = 0u16;
            let mut start_sb = 0u16;
            while start_sb < sb_cols {
                mi_col_starts.push(start_sb << sb_shift);
                i += 1;
                start_sb += tile_width_sb;
            }
            mi_col_starts.push(frame_size.mi_cols);
            tile_cols = i;

            // rows
            tile_rows_log2 = std::cmp::max(min_log2_tiles - tile_cols_log2, 0u16);
            while tile_rows_log2 < max_log2_tile_rows {
                if r.read::<1, u8>()? == 1 {
                    tile_rows_log2 += 1;
                } else {
                    break;
                }
            }
            let tile_height_sb = (sb_rows + (1 << tile_rows_log2) - 1) >> tile_rows_log2;

            let mut i = 0u16;
            let mut start_sb = 0;
            while start_sb < sb_rows {
                mi_row_starts.push(start_sb << sb_shift);
                i += 1;
                start_sb += tile_height_sb;
            }
            mi_row_starts.push(frame_size.mi_rows);
            tile_rows = i;
        } else {
            // cols
            let mut widest_tile_sb = 0u32;
            let mut start_sb = 0u16;
            let mut i = 0u16;

            while start_sb < sb_cols {
                mi_col_starts.push(start_sb << sb_shift);
                let max_width = std::cmp::min(sb_cols - start_sb, max_tile_width_sb);
                let size_sb = Ns::ns(r, max_width as u32)?.value + 1;

                widest_tile_sb = std::cmp::max(size_sb, widest_tile_sb as u32);
                start_sb += size_sb as u16;

                i += 1;
            }
            mi_col_starts.push(frame_size.mi_cols);
            tile_cols = i;
            tile_cols_log2 = tile_log2(1, tile_cols as u32);

            // rows
            if min_log2_tiles > 0 {
                max_tile_area_sb = (sb_rows as u32 * sb_cols as u32) >> (min_log2_tiles + 1);
            } else {
                max_tile_area_sb = sb_rows as u32 * sb_cols as u32;
            }
            max_tile_height_sb = std::cmp::max((max_tile_area_sb / widest_tile_sb) as u16, 1);

            let mut start_sb = 0u16;
            let mut i = 0u16;
            while start_sb < sb_rows {
                mi_row_starts.push(start_sb << sb_shift);
                let max_height = std::cmp::min(sb_rows - start_sb, max_tile_height_sb);
                let size_sb = Ns::ns(r, max_height as u32)?.value + 1;
                start_sb += size_sb as u16;
                i += 1
            }
            mi_row_starts.push(frame_size.mi_rows);
            tile_rows = i;
            tile_rows_log2 = tile_log2(1, tile_rows as u32);
        }

        let context_update_tile_id: u8;
        let tile_size_bytes: Option<u8>;
        if tile_cols_log2 > 0 || tile_rows_log2 > 0 {
            context_update_tile_id = r.read::<2,u8>()?;
            tile_size_bytes = Some(r.read::<2, u8>()? + 1);
        } else {
            context_update_tile_id = 0;
            tile_size_bytes = None;
        }

        Ok(Self {
            tile_cols,
            tile_rows,
            tile_cols_log2,
            tile_rows_log2,
            mi_col_starts,
            mi_row_starts,
            tile_size_bytes,
            context_update_tile_id,
        })

    }
}

fn frame_size_with_refs<R: bitstream_io::BitRead + ?Sized>(
    r: &mut R,
    ref_frame_index: [u8; REFS_PER_FRAME as usize],
    frame_size_override_flag: u8,
    ctx: &mut DecoderContext,
) -> Result<(FrameSize, RenderSize), std::io::Error> {
    let upscaled_width: u16;
    let mut frame_width: u16;
    let frame_height: u16;
    let render_width: u16;
    let render_height: u16;

    for i in 0..REFS_PER_FRAME as usize {
        // found_ref
        if r.read::<1, u8>()? == 1 {
            upscaled_width = ctx.ref_frame_sizes[ref_frame_index[i] as usize].upscaled_width;
            frame_width = upscaled_width;
            frame_height = ctx.ref_frame_sizes[ref_frame_index[i] as usize].frame_height;
            render_width = ctx.ref_frame_render_sizes[ref_frame_index[i] as usize].render_width;
            render_height = ctx.ref_frame_render_sizes[ref_frame_index[i] as usize].render_height;

            // superres_params( )
            let (use_superres, coded_denom, superres_denom, upscaled_width) = superres_params(
                r,
                ctx.last_sequence_header.clone().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "No last sequence header")
                })?,
                &mut frame_width,
            )?;

            // compute_image_size( )
            let (mi_cols, mi_rows) = compute_image_size(frame_width, frame_height);

            return Ok((
                FrameSize {
                    frame_width,
                    frame_height,
                    use_superres,
                    coded_denom: coded_denom.unwrap_or_default(),
                    superres_denom,
                    upscaled_width,
                    mi_cols,
                    mi_rows,
                },
                RenderSize {
                    render_and_frame_size_different: 0,
                    render_width,
                    render_height,
                },
            ));
        }
    }
    let frame_size = FrameSize::frame_size(r, frame_size_override_flag, ctx)?;
    let render_size = RenderSize::render_size(r, &frame_size)?;
    Ok((frame_size, render_size))
}

fn compute_image_size(frame_width: u16, frame_height: u16) -> (u16, u16) {
    let mi_cols = 2 * ((frame_width + 7) >> 3);
    let mi_rows = 2 * ((frame_height + 7) >> 3);
    (mi_cols, mi_rows)
}

fn superres_params<R: bitstream_io::BitRead + ?Sized>(
    r: &mut R,
    sequence_header: SequenceHeader,
    frame_width: &mut u16,
) -> Result<(u8, Option<u8>, u8, u16), std::io::Error> {
    let use_superres: u8 = if sequence_header.enable_superres != 0 {
        r.read::<1, u8>()?
    } else {
        0
    };
    let coded_denom: Option<u8>;
    let superres_denom: u8;
    let upscaled_width: u16;
    if use_superres != 0 {
        coded_denom = Some(r.read_var::<u8>(SUPERRES_DENOM_BITS as u32)?);
        superres_denom = coded_denom.unwrap() + SUPERRES_DENOM_MIN;
    } else {
        coded_denom = None;
        superres_denom = SUPERRES_NUM;
    }
    upscaled_width = *frame_width;
    *frame_width = (upscaled_width * SUPERRES_NUM as u16 + (superres_denom as u16 / 2))
        / superres_denom as u16;
    Ok((use_superres, coded_denom, superres_denom, upscaled_width))
}

fn read_interpolation_filter<R: bitstream_io::BitRead + ?Sized>(
    r: &mut R,
) -> Result<INTERPOLATION_FILTER, std::io::Error> {
    let is_filter_switchable = r.read::<1, u8>()?;
    if is_filter_switchable == 1 {
        Ok(INTERPOLATION_FILTER::SWITCHABLE)
    } else {
        Ok(INTERPOLATION_FILTER::from_reader(r)?)
    }
}

fn tile_log2(blk_size: u32, target: u32) -> u16 {
    let mut k = 0;
    while (blk_size << k) < target {
        k += 1;
    }
    k
}

fn get_relative_dist(a: u8, b: u8, ctx: &mut DecoderContext) -> Result<i16, std::io::Error> {
    let sequence_header = ctx.last_sequence_header.clone().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "No last sequence header")
    })?;

    if sequence_header.enable_order_hint == 0 {
        return Ok(0);
    }

    let diff: i16 = a as i16 - b as i16;
    let m: i16 = 1i16 << (sequence_header.order_hint_bits as i16 - 1i16);
    Ok((diff & (m - 1)) - (diff & m))
}

impl Default for FrameSize {
    fn default() -> Self {
        Self {
            frame_width: 0,
            frame_height: 0,
            use_superres: 0,
            coded_denom: 0,
            superres_denom: 0,
            upscaled_width: 0,
            mi_cols: 0,
            mi_rows: 0,
        }
    }
}

impl Default for RenderSize {
    fn default() -> Self {
        Self {
            render_and_frame_size_different: 0,
            render_width: 0,
            render_height: 0,
        }
    }
}
