use bitstream_io::FromBitStream;

use crate::{consts::{self, OBU_TYPE}, generics::uvlc, leb_128};

use super::{handlers::choose_operating_point, Color_Config, Decoder_Model_Info, OBU_Extension_Header, OBU_Header, OBU_Sequence_Header, Operating_Parameters_Info, Timing_Info, OBU};

impl OBU {
    
    pub fn open_bitstream_unit<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        sz: u64,
    ) -> Result<OBU, std::io::Error> {
        let header = OBU_Header::from_reader(r)?;
        let obu_size = if header.obu_has_size_field == 1 {
            leb_128::from_reader(r)?
        } else {
            leb_128::new(sz - 1 - header.obu_extension_flag as u64)
        };
        let operating_point_idc: u8 = 0u8;

        //[!] OperatingPointIdc
        if header.obu_type != OBU_TYPE::OBU_SEQUENCE_HEADER
            && header.obu_type != OBU_TYPE::OBU_TEMPORAL_DELIMITER
            && operating_point_idc != 0
            && header.obu_extension_flag == 1
            && header.obu_extension_header.is_some()
        {
            let extra_headers = header.obu_extension_header.clone().unwrap();
            let in_temporal_layer = (operating_point_idc >> extra_headers.temporal_id) & 1u8;
            let in_spatial_layer = (operating_point_idc >> (extra_headers.spatial_id + 8u8)) & 1u8;

            if in_temporal_layer != 0u8 || in_spatial_layer == 0u8 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Layer filtering: not in temporal or spatial layer",
                ));
            }
        }

        let obu_sequence_header = if header.obu_type == OBU_TYPE::OBU_SEQUENCE_HEADER {
            Some(OBU_Sequence_Header::sequence_header_obu(r)?)
        } else {
            None
        };

        Ok(OBU { obu_size, obu_header: header })
    }
}

impl FromBitStream for OBU {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let obu_header = OBU_Header::from_reader(r)?;
        let obu_size = leb_128::from_reader(r)?;

        Ok(Self {
            obu_size,
            obu_header,
        })
    }
}

impl FromBitStream for OBU_Header {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let obu_forbidden_bit = r.read::<1, u8>()?;
        let obu_type = OBU_TYPE::from_reader(r)?;
        let obu_extension_flag = r.read::<1, u8>()?;
        let obu_has_size_field = r.read::<1, u8>()?;
        let obu_reserved_1bit = r.read::<1, u8>()?;
        let obu_extension_header = if obu_extension_flag == 1 {
            Some(OBU_Extension_Header::from_reader(r)?)
        } else {
            None
        };

        Ok(Self {
            obu_forbidden_bit,
            obu_type,
            obu_extension_flag,
            obu_has_size_field,
            obu_reserved_1bit,
            obu_extension_header,
        })
    }
}

impl FromBitStream for OBU_Extension_Header {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            temporal_id: r.read::<3, u8>()?,
            spatial_id: r.read::<2, u8>()?,
            extension_header_reserved_3bits: r.read::<3, u8>()?,
        })
    }
}


impl OBU_Sequence_Header {
    
    // 5.5.1 General sequence header OBU syntax
    fn sequence_header_obu<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, std::io::Error> 
    where
        Self: Sized,
    {
        let seq_profile = r.read::<3,u8>()?;
        let still_picture = r.read::<1, u8>()?;
        let reduced_still_picture_header = r.read::<1, u8>()?;

        let mut timing_info_present_flag: u8 = 0u8;
        let mut decoder_model_info_present_flag: u8 = 0u8;
        let mut decoder_model_info: Option<Decoder_Model_Info> = None;
        let mut initial_display_delay_present_flag: u8 = 0u8;
        let mut operating_points_cnt_minus_1: u8 = 0u8;
        let mut operating_point_idc: Vec<u16> = vec![0u16];

        let mut seq_level_idx: Vec<u8> = vec![r.read::<5, u8>()?];

        let mut seq_tier: Vec<u8> = vec![0u8];
        let mut decoder_model_present_for_this_op: Vec<u8> = vec![0u8];
        let mut operating_parameters_info: Option<Operating_Parameters_Info> = None;
        let mut initial_display_delay_present_for_this_op: Vec<u8> = vec![0u8];
        let mut initial_display_delay_minus_1: Option<Vec<u8>> = None;
        let mut timing_info: Option<Timing_Info> = None;

        if reduced_still_picture_header == 0{

            timing_info_present_flag = r.read::<1,u8>()?;

            // Timing_Info
            if timing_info_present_flag == 1 {
                timing_info = Some(Timing_Info::from_reader(r)?);
            }

            // Decoder_Model_Info
            let decoder_model_info_present_flag: u8 = if timing_info_present_flag == 1 {
                r.read::<1, u8>()?
            } else {
                0u8
            };

            let decoder_model_info: Option<Decoder_Model_Info> = if decoder_model_info_present_flag != 0u8 {
                Some(Decoder_Model_Info::from_reader(r)?)
            } else {
                None
            };
            
            // Operating_point_idc
            // seq_level_idx
            // seq_tier
            // Operating_Parameters_Info
            // initial_display_delay_minus_1
            initial_display_delay_present_flag = r.read::<1, u8>()?;
            operating_points_cnt_minus_1 = r.read::<5, u8>()?;
            for i in 0..operating_points_cnt_minus_1 as usize {
                operating_point_idc.push(r.read::<12,u16>()?);
                seq_level_idx.push(r.read::<5,u8>()?);

                // seq_tier
                if *seq_level_idx.last().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Seq level idx not present"))? > 7 {
                    seq_tier.push(r.read::<1, u8>()?);
                } else {
                    seq_tier.push(0);
                }

                // Operating_Parameters_Info
                if decoder_model_info_present_flag != 0u8  {
                    decoder_model_present_for_this_op.push(r.read::<1, u8>()?);
                    if *decoder_model_present_for_this_op.last().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Decoder model present for this op not present"))? != 0 {
                        let decoder_model_info = decoder_model_info.as_ref().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Decoder model info not present"))?;

                        if operating_parameters_info.is_none() {
                            operating_parameters_info = Some(Operating_Parameters_Info::new());
                        }
                        operating_parameters_info.as_mut().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Operating parameters info not present"))?.from_reader(r,decoder_model_info)?;
                    }
                } else {
                    decoder_model_present_for_this_op.push(0);
                }

                // initial_display_delay_minus_1
                if initial_display_delay_present_flag != 0 {
                    initial_display_delay_present_for_this_op.push(r.read::<1, u8>()?);
                    if initial_display_delay_present_for_this_op.last().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Initial display delay present for this op not present"))?.to_owned() != 0u8 {
                        if initial_display_delay_minus_1.is_none() {
                            initial_display_delay_minus_1 = Some(Vec::new());
                        }
                        initial_display_delay_minus_1.as_mut().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Initial display delay minus 1 not present"))?.push(r.read::<4, u8>()?);
                    }
                }
            }
        }

        // Operating point
        let operating_point_index = choose_operating_point()?;
        if operating_point_index >= operating_point_idc.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Operating point index out of bounds"));
        }
        let c_operating_point_idc = operating_point_idc[operating_point_index];
        let frame_width_bits_minus_1 = r.read::<4, u8>()?;
        let frame_height_bits_minus_1 = r.read::<4, u8>()?;
        let max_frame_width_minus_one: u16 = r.read_var(frame_width_bits_minus_1 as u32 + 1u32)?;  // 2**4 = 16 bits
        let max_frame_height_minus_one: u16 = r.read_var(frame_height_bits_minus_1 as u32 + 1u32)?; // 2**4 = 16 bits
        let frame_id_numbers_present_flag = if reduced_still_picture_header != 0 {
            0u8
        } else {
            r.read::<1, u8>()?
        };
        let delta_frame_id_length_minus_2: Option<u8> = if frame_id_numbers_present_flag != 0 {
            Some(r.read::<4, u8>()?)
        } else {
            None
        };
        let additional_frame_id_length_minus_1: Option<u8> = if frame_id_numbers_present_flag != 0 {
            Some(r.read::<3, u8>()?)
        } else {
            None
        };

        // Flags
        let use_128x128_superblock = r.read::<1, u8>()?;
        let enable_filter_intra = r.read::<1,u8>()?;
        let enable_intra_edge_filter = r.read::<1,u8>()?;

        let mut enable_interintra_compound: u8 = 0u8;
        let mut enable_masked_compound: u8 = 0u8;
        let mut enable_warped_motion: u8 = 0u8;
        let mut enable_dual_filter: u8 = 0u8;
        let mut enable_order_hint: u8 = 0u8;
        let mut enable_jnt_comp: u8 = 0u8;
        let mut enable_ref_frame_mvs: u8 = 0u8;
        let mut seq_force_screen_content_tools: u8 = consts::SELECT_SCREEN_CONTENT_TOOLS;
        let mut seq_force_integer_mv: u8 = consts::SELECT_INTEGER_MV;
        let mut order_hint_bits: u8 = 0u8;

        if reduced_still_picture_header != 0 {
            enable_interintra_compound = r.read::<1,u8>()?;
            enable_masked_compound = r.read::<1,u8>()?;
            enable_warped_motion = r.read::<1,u8>()?;
            enable_dual_filter = r.read::<1,u8>()?;
            enable_order_hint = r.read::<1,u8>()?;

            if enable_order_hint != 0 {
                enable_jnt_comp = r.read::<1,u8>()?;
                enable_ref_frame_mvs = r.read::<1,u8>()?;
            }

            let seq_choose_screen_content_tools = r.read::<1,u8>()?;
            if seq_choose_screen_content_tools == 0 {
                seq_force_screen_content_tools = r.read::<1,u8>()?;
            }

            if seq_force_screen_content_tools > 0u8 {
                let seq_choose_integer_mv = r.read::<1,u8>()?;
                if seq_choose_integer_mv == 0 {
                    seq_force_integer_mv = r.read::<1,u8>()?;
                }
            }

            if enable_order_hint != 0 {
                order_hint_bits = r.read::<3,u8>()? + 1u8;
            }
        }

        let enable_superres = r.read::<1,u8>()?;
        let enable_cdef = r.read::<1,u8>()?;
        let enable_restoration = r.read::<1,u8>()?;

        // Color config
        let color_config = Color_Config::from_reader(r,seq_profile)?;
        let film_grain_params_present = r.read::<1,u8>()?;

        Ok(Self {
            seq_profile,
            still_picture,
            timing_info,
            decoder_model_info,
            operating_point_idc,
            seq_level_idx,
            seq_tier,
            decoder_model_present_for_this_op,
            operating_parameters_info,
            initial_display_delay_present_for_this_op,
            initial_display_delay_minus_1,
            c_operating_point_idc,
            max_frame_width_minus_one,
            max_frame_height_minus_one,
            delta_frame_id_length_minus_2,
            additional_frame_id_length_minus_1,
            use_128x128_superblock,
            enable_filter_intra,
            enable_intra_edge_filter,
            enable_interintra_compound,
            enable_masked_compound,
            enable_warped_motion,
            enable_dual_filter,
            enable_order_hint,
            enable_jnt_comp,
            enable_ref_frame_mvs,
            seq_force_screen_content_tools,
            seq_force_integer_mv,
            order_hint_bits,
            enable_superres,
            enable_cdef,
            enable_restoration,
            color_config,
            film_grain_params_present,
        })

    }

}

impl FromBitStream for Timing_Info {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized {
        let num_units_in_display_tick = r.read::<32,u32>()?;
        let time_scale = r.read::<32,u32>()?;
        let equal_picture_interval = r.read::<1, u8>()?;
        let num_ticks_per_picture_minus_1 = if equal_picture_interval == 1 {
            Some(uvlc::from_reader(r)?)
        } else {
            None
        };

        Ok(Self {
            num_units_in_display_tick,
            time_scale,
            equal_picture_interval,
            num_ticks_per_picture_minus_1,
        })
    }
}

impl FromBitStream for Decoder_Model_Info {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized {
            let buffer_delay_length_minus_1 = r.read::<5, u8>()?;
            let num_units_in_decoding_tick = r.read::<32, u32>()?;
            let buffer_removal_delay_length_minus_1 = r.read::<5, u8>()?;
            let frame_presentation_delay_length_minus_1 = r.read::<5, u8>()?;

            Ok(Self {
                buffer_delay_length_minus_1,
                num_units_in_decoding_tick,
                buffer_removal_delay_length_minus_1,
                frame_presentation_delay_length_minus_1,
            })
        }
}

impl Operating_Parameters_Info {

    pub fn new() -> Self {
        Self {
            decoder_buffer_delay: Vec::new(),
            encoder_buffer_delay: Vec::new(),
            low_delay_mode_flag: Vec::new(),
        }
    }

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(&mut self, r: &mut R, decoder_model_info: &Decoder_Model_Info) -> Result<(), std::io::Error>
    where
        Self: Sized {
            let n = decoder_model_info.buffer_delay_length_minus_1 as u32 + 1; // max 32 = 2**5+1
            self.decoder_buffer_delay.push(r.read_var(n)?);
            self.encoder_buffer_delay.push(r.read_var(n)?);
            self.low_delay_mode_flag.push(r.read::<1, u8>()?);
            Ok(())
    }
}

impl Color_Config {
    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R, seq_profile: u8) -> Result<Self, std::io::Error>
    where
        Self: Sized {
            let high_bit_depth: u8 = r.read::<1,u8>()?;

            let bit_depth = if seq_profile == 2u8 && high_bit_depth != 0u8 {
                12u8
            } else {
                10u8
            };

            let mono_chrome = if seq_profile == 1u8 {
                0u8
            } else {
                r.read::<1, u8>()?
            };

            let num_planes = if mono_chrome == 0u8 {
                3u8
            } else {
                1u8
            };

        let mut color_primaries = consts::COLOR_PRIMARIES::CP_UNSPECIFIED;
        let mut transfer_characteristics = consts::TRANSFER_CHARACTERISTICS::TC_UNSPECIFIED;
        let mut matrix_coefficients = consts::MATRIX_COEFFICIENTS::MC_UNSPECIFIED;

        // color_description_present_flag
        if r.read::<1,u8>()? != 0u8 {
            color_primaries = consts::COLOR_PRIMARIES::from_reader(r)?;
            transfer_characteristics = consts::TRANSFER_CHARACTERISTICS::from_reader(r)?;
            matrix_coefficients = consts::MATRIX_COEFFICIENTS::from_reader(r)?;
        }

        let color_range: u8;
        let subsampling_x: u8;
        let subsampling_y: u8;
        let mut chroma_sample_position: consts::CHROMA_SAMPLE_POSITION = consts::CHROMA_SAMPLE_POSITION::CSP_UNKNOWN;
        let separate_uv_delta_q: u8;

        if mono_chrome != 0 {
            color_range = r.read::<1, u8>()?;
            subsampling_x = 1u8;
            subsampling_y = 1u8;
            separate_uv_delta_q = 0u8;

            return Ok(Self {
                bit_depth,
                mono_chrome,
                num_planes,
                color_primaries,
                transfer_characteristics,
                matrix_coefficients,
                color_range,
                subsampling_x,
                subsampling_y,
                chroma_sample_position,
                separate_uv_delta_q,
            });
        } else if color_primaries == consts::COLOR_PRIMARIES::CP_BT_709 
            && transfer_characteristics == consts::TRANSFER_CHARACTERISTICS::TC_SRGB 
            && matrix_coefficients == consts::MATRIX_COEFFICIENTS::MC_IDENTITY {
            color_range = 1u8;
            subsampling_x = 0u8;
            subsampling_y = 0u8;
        } else {
            color_range = r.read::<1, u8>()?;
            
            match seq_profile {
                0 => {
                    subsampling_x = 1u8;
                    subsampling_y = 1u8;
                },
                1 => {
                    subsampling_x = r.read::<1, u8>()?;
                    subsampling_y = r.read::<1, u8>()?;
                },
                _ => {
                    if bit_depth == 12 {
                        subsampling_x = r.read::<1, u8>()?;
                        if subsampling_x != 0 {
                            subsampling_y = r.read::<1, u8>()?;
                        } else {
                            subsampling_y = 0u8;
                        }
                    } else {
                        subsampling_x = 1u8;
                        subsampling_y = 0u8;
                    }
                }
            }

            if subsampling_x != 0 && subsampling_y != 0 {
                chroma_sample_position = consts::CHROMA_SAMPLE_POSITION::from_reader(r)?;
            }
        }

        separate_uv_delta_q = r.read::<1, u8>()?;

        Ok(Self {
            bit_depth,
            mono_chrome,
            num_planes,
            color_primaries,
            transfer_characteristics,
            matrix_coefficients,
            color_range,
            subsampling_x,
            subsampling_y,
            chroma_sample_position,
            separate_uv_delta_q,
        })

        }
}