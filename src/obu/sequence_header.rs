use bitstream_io::FromBitStream;

use crate::{consts, generics::Uvlc};

use super::context::DecoderContext;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SequenceHeader {
    pub seq_profile: u8,                    // 3 bits
    pub still_picture: u8,                  // 1 bit
    pub timing_info: Option<TimingInfo>,
    pub decoder_model_info: Option<DecoderModelInfo>,
    pub operating_points_cnt: u8,           // 5 bits
    pub operating_point_idc: Vec<u16>,      // 12 bits
    pub seq_level_idx: Vec<u8>,                  // 5 bits
    pub seq_tier: Vec<u8>,                       // 1 bit
    pub decoder_model_present_for_this_op: Vec<u8>, // 1 bit
    pub operating_parameters_info: Option<OperatingParametersInfo>,
    pub initial_display_delay_present_for_this_op: Vec<u8>, // 1 bit
    pub initial_display_delay_minus_1: Option<Vec<u8>>,    // 4 bits
    pub c_operating_point_idc: u16,         // 12 bits
    pub max_frame_width_minus_one: u16,     //  2**frame_width_bits_minus_1+1 
    pub max_frame_height_minus_one: u16,    //  2**frame_height_bits_minus_1+1
    pub frame_width_bits: u8,                // 4 bits
    pub frame_height_bits: u8,               // 4 bits
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
    pub enable_superres: u8,                 // 1 bit
    pub enable_cdef: u8,                     // 1 bit
    pub enable_restoration: u8,              // 1 bit
    pub color_config: ColorConfig,
    pub film_grain_params_present: u8,       // 1 bit
    pub frame_id_numbers_present_flag: u8,   // 1 bit
    pub reduced_still_picture_header: u8,    // 1 bit
    pub decoder_model_info_present_flag: u8, // 1 bit
}

// 5.5.3 Timing info syntax
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TimingInfo {
    pub num_units_in_display_tick: u32,                 // 32 bits
    pub time_scale: u32,                        // 32 bits
    pub equal_picture_interval: u8,             //  1 bit
    pub num_ticks_per_picture_minus_1: Option<Uvlc>,   // UVLC
}

// 5.5.4 Decoder model info
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DecoderModelInfo {
    pub buffer_delay_length_minus_1: u8,   // 5 bits
    pub num_units_in_decoding_tick: u32,   // 32 bits
    pub buffer_removal_delay_length_minus_1: u8, // 5 bits
    pub frame_presentation_delay_length_minus_1: u8, // 5 bits
}

// 5.5.5 Operating parameters info
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OperatingParametersInfo {
    pub decoder_buffer_delay: Vec<u32>, // 2**5=32 bits max
    pub encoder_buffer_delay: Vec<u32>, // 2**5=32 bits max
    pub low_delay_mode_flag: Vec<u8>,   // 1 bit
}

// 5.5.2 Color Config
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColorConfig {
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


impl SequenceHeader {
    // 5.5.1 General sequence header OBU syntax
    pub fn sequence_header_obu<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        ctx: &mut DecoderContext,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let seq_profile = r.read::<3, u8>()?; // 3
        let still_picture = r.read::<1, u8>()?; // 4
        let reduced_still_picture_header = r.read::<1, u8>()?; // 5

        let timing_info_present_flag: u8;
        let mut decoder_model_info_present_flag: u8 = 0;
        let decoder_model_info: Option<DecoderModelInfo> = None;
        let initial_display_delay_present_flag: u8;
        let mut operating_points_cnt: u8 = 0;
        let mut operating_point_idc: Vec<u16> = vec![0u16];

        let mut seq_level_idx: Vec<u8> = vec![];

        let mut seq_tier: Vec<u8> = vec![0u8];
        let mut decoder_model_present_for_this_op: Vec<u8> = vec![0u8];
        let mut operating_parameters_info: Option<OperatingParametersInfo> = None;
        let mut initial_display_delay_present_for_this_op: Vec<u8> = vec![0u8];
        let mut initial_display_delay_minus_1: Option<Vec<u8>> = None;
        let mut timing_info: Option<TimingInfo> = None;

        if reduced_still_picture_header != 0 {
            seq_level_idx.push(r.read::<5, u8>()?); // 10
        } else if reduced_still_picture_header == 0 {
            timing_info_present_flag = r.read::<1, u8>()?; // 11

            // Timing_Info
            if timing_info_present_flag == 1 {
                timing_info = Some(TimingInfo::from_reader(r)?);
            }

            // Decoder_Model_Info
            decoder_model_info_present_flag = if timing_info_present_flag == 1 {
                r.read::<1, u8>()?
            } else {
                0u8
            };

            let decoder_model_info: Option<DecoderModelInfo> =
                if decoder_model_info_present_flag != 0u8 {
                    Some(DecoderModelInfo::from_reader(r)?)
                } else {
                    None
                };

            // Operating_point_idc
            // seq_level_idx
            // seq_tier
            // Operating_Parameters_Info
            // initial_display_delay_minus_1
            initial_display_delay_present_flag = r.read::<1, u8>()?;
            operating_points_cnt = r.read::<5, u8>()? + 1;
            for _ in 0..operating_points_cnt as usize {
                operating_point_idc.push(r.read::<12, u16>()?);
                seq_level_idx.push(r.read::<5, u8>()?);

                // seq_tier
                if *seq_level_idx.last().ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Seq level idx not present",
                    )
                })? > 7
                {
                    seq_tier.push(r.read::<1, u8>()?); // 35
                } else {
                    seq_tier.push(0);
                }

                // Operating_Parameters_Info
                if decoder_model_info_present_flag != 0u8 {
                    decoder_model_present_for_this_op.push(r.read::<1, u8>()?);
                    if *decoder_model_present_for_this_op.last().ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Decoder model present for this op not present",
                        )
                    })? != 0
                    {
                        let decoder_model_info = decoder_model_info.as_ref().ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Decoder model info not present",
                            )
                        })?;

                        if operating_parameters_info.is_none() {
                            operating_parameters_info = Some(OperatingParametersInfo::new());
                        }
                        operating_parameters_info
                            .as_mut()
                            .ok_or_else(|| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Operating parameters info not present",
                                )
                            })?
                            .from_reader(r, decoder_model_info)?;
                    }
                } else {
                    decoder_model_present_for_this_op.push(0);
                }

                // initial_display_delay_minus_1
                if initial_display_delay_present_flag != 0 {
                    initial_display_delay_present_for_this_op.push(r.read::<1, u8>()?);
                    if initial_display_delay_present_for_this_op
                        .last()
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Initial display delay present for this op not present",
                            )
                        })?
                        .to_owned()
                        != 0u8
                    {
                        if initial_display_delay_minus_1.is_none() {
                            initial_display_delay_minus_1 = Some(Vec::new());
                        }
                        initial_display_delay_minus_1
                            .as_mut()
                            .ok_or_else(|| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Initial display delay minus 1 not present",
                                )
                            })?
                            .push(r.read::<4, u8>()?);
                    }
                }
            }
        }

        // Operating point
        //let operating_point_index = choose_operating_point()?;
        //if operating_point_index >= operating_point_idc.len() {
        //    return Err(std::io::Error::new(
        //        std::io::ErrorKind::InvalidData,
        //        "Operating point index out of bounds",
        //    ));
        //}
        //
        let c_operating_point_idc = 0u16; //operating_point_idc[operating_point_index];
        let frame_width_bits = r.read_unsigned::<4, u8>()? + 1;
        let frame_height_bits = r.read_unsigned::<4, u8>()? + 1;

        let max_frame_width: u16 = r.read_unsigned_var::<u16>(frame_width_bits as u32)? + 1u16;
        let max_frame_height: u16 = r.read_unsigned_var::<u16>(frame_height_bits as u32)? + 1u16;

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
        let enable_filter_intra = r.read::<1, u8>()?;
        let enable_intra_edge_filter = r.read::<1, u8>()?;

        let mut enable_interintra_compound: u8 = 0u8;
        let mut enable_masked_compound: u8 = 0u8;
        let mut enable_warped_motion: u8 = 0u8;
        let mut enable_dual_filter: u8 = 0u8;
        let mut enable_order_hint: u8 = 0u8;
        let mut enable_jnt_comp: u8 = 0u8;
        let mut enable_ref_frame_mvs: u8 = 0u8;
        let mut seq_force_screen_content_tools: u8 = consts::SELECT_SCREEN_CONTENT_TOOLS;
        let mut seq_force_integer_mv: u8 = consts::SELECT_INTEGER_MV;

        if reduced_still_picture_header == 0 {
            enable_interintra_compound = r.read::<1, u8>()?;
            enable_masked_compound = r.read::<1, u8>()?;
            enable_warped_motion = r.read::<1, u8>()?;
            enable_dual_filter = r.read::<1, u8>()?;
            enable_order_hint = r.read::<1, u8>()?;

            if enable_order_hint != 0 {
                enable_jnt_comp = r.read::<1, u8>()?;
                enable_ref_frame_mvs = r.read::<1, u8>()?;
            }

            let seq_choose_screen_content_tools = r.read::<1, u8>()?;
            if seq_choose_screen_content_tools == 0 {
                seq_force_screen_content_tools = r.read::<1, u8>()?;
            }

            if seq_force_screen_content_tools > 0u8 {
                let seq_choose_integer_mv = r.read::<1, u8>()?;
                if seq_choose_integer_mv == 0 {
                    seq_force_integer_mv = r.read::<1, u8>()?;
                }
            }

            if enable_order_hint != 0 {
                ctx.order_hint_bits = r.read::<3, u8>()? + 1u8;
            }
        }

        let enable_superres = r.read::<1, u8>()?;
        let enable_cdef = r.read::<1, u8>()?;
        let enable_restoration = r.read::<1, u8>()?;

        // Color config
        let color_config = ColorConfig::from_reader(r, seq_profile)?;

        let film_grain_params_present = r.read::<1, u8>()?;

        Ok(Self {
            seq_profile,
            still_picture,
            timing_info,
            decoder_model_info,
            operating_points_cnt,
            operating_point_idc,
            seq_level_idx,
            seq_tier,
            decoder_model_present_for_this_op,
            operating_parameters_info,
            initial_display_delay_present_for_this_op,
            initial_display_delay_minus_1,
            c_operating_point_idc,
            max_frame_width_minus_one: max_frame_width,
            max_frame_height_minus_one: max_frame_height,
            frame_width_bits,
            frame_height_bits,
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
            enable_superres,
            enable_cdef,
            enable_restoration,
            color_config,
            film_grain_params_present,
            frame_id_numbers_present_flag,
            reduced_still_picture_header,
            decoder_model_info_present_flag,
        })
    }
}

impl FromBitStream for TimingInfo {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let num_units_in_display_tick = r.read::<32, u32>()?;
        let time_scale = r.read::<32, u32>()?;
        let equal_picture_interval = r.read::<1, u8>()?;
        let num_ticks_per_picture_minus_1 = if equal_picture_interval == 1 {
            Some(Uvlc::from_reader(r)?);
            Some(Uvlc::new(0 + 1u32))
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

impl FromBitStream for DecoderModelInfo {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
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

impl OperatingParametersInfo {
    pub fn new() -> Self {
        Self {
            decoder_buffer_delay: Vec::new(),
            encoder_buffer_delay: Vec::new(),
            low_delay_mode_flag: Vec::new(),
        }
    }

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(
        &mut self,
        r: &mut R,
        decoder_model_info: &DecoderModelInfo,
    ) -> Result<(), std::io::Error>
    where
        Self: Sized,
    {
        let n = decoder_model_info.buffer_delay_length_minus_1 as u32 + 1; // max 32 = 2**5+1
        self.decoder_buffer_delay.push(r.read_var(n)?);
        self.encoder_buffer_delay.push(r.read_var(n)?);
        self.low_delay_mode_flag.push(r.read::<1, u8>()?);
        Ok(())
    }
}

impl ColorConfig {
    fn from_reader<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        seq_profile: u8,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let high_bit_depth: u8 = r.read::<1, u8>()?;

        let bit_depth = if seq_profile == 2u8 && high_bit_depth != 0u8 {
            let twelve_bits = r.read::<1, u8>()?;
            if twelve_bits != 0u8 { 12u8 } else { 10u8 }
        } else if seq_profile <= 2 {
            if high_bit_depth != 0u8 { 10u8 } else { 8u8 }
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid seq_profile",
            ));
        };

        let mono_chrome = if seq_profile == 1u8 {
            0u8
        } else {
            r.read::<1, u8>()?
        };

        let num_planes = if mono_chrome == 0u8 { 3u8 } else { 1u8 };

        let mut color_primaries = consts::COLOR_PRIMARIES::CP_UNSPECIFIED;
        let mut transfer_characteristics = consts::TRANSFER_CHARACTERISTICS::TC_UNSPECIFIED;
        let mut matrix_coefficients = consts::MATRIX_COEFFICIENTS::MC_UNSPECIFIED;

        // color_description_present_flag
        let color_description_present_flag = r.read::<1, u8>()?;
        if color_description_present_flag != 0u8 {
            color_primaries = consts::COLOR_PRIMARIES::from_reader(r)?;
            transfer_characteristics = consts::TRANSFER_CHARACTERISTICS::from_reader(r)?;
            matrix_coefficients = consts::MATRIX_COEFFICIENTS::from_reader(r)?;
        }

        let color_range: u8;
        let subsampling_x: u8;
        let subsampling_y: u8;
        let mut chroma_sample_position: consts::CHROMA_SAMPLE_POSITION =
            consts::CHROMA_SAMPLE_POSITION::CSP_UNKNOWN;
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
            && matrix_coefficients == consts::MATRIX_COEFFICIENTS::MC_IDENTITY
        {
            color_range = 1u8;
            subsampling_x = 0u8;
            subsampling_y = 0u8;
        } else {
            color_range = r.read::<1, u8>()?;

            match seq_profile {
                0 => {
                    subsampling_x = 1u8;
                    subsampling_y = 1u8;
                }
                1 => {
                    subsampling_x = r.read::<1, u8>()?;
                    subsampling_y = r.read::<1, u8>()?;
                }
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

impl std::fmt::Display for SequenceHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OBU_Sequence_Header {{ seq_profile: {}, still_picture: {}, timing_info: {:?}, decoder_model_info: {:?}, operating_point_idc: {:?}, seq_level_idx: {:?}, seq_tier: {:?}, decoder_model_present_for_this_op: {:?}, operating_parameters_info: {:?}, initial_display_delay_present_for_this_op: {:?}, initial_display_delay_minus_1: {:?}, c_operating_point_idc: {}, max_frame_width_minus_one: {}, max_frame_height_minus_one: {}, delta_frame_id_length_minus_2: {:?}, additional_frame_id_length_minus_1: {:?}, use_128x128_superblock: {}, enable_filter_intra: {}, enable_intra_edge_filter: {}, enable_interintra_compound: {}, enable_masked_compound: {}, enable_warped_motion: {}, enable_dual_filter: {}, enable_order_hint: {}, enable_jnt_comp: {}, enable_ref_frame_mvs: {}, seq_force_screen_content_tools: {}, seq_force_integer_mv: {}, enable_superres: {}, enable_cdef: {}, enable_restoration: {}, color_config: {:?}, film_grain_params_present: {}, frame_id_numbers_present_flag: {}, reduced_still_picture_header: {}, decoder_model_info_present_flag: {} }}",
            self.seq_profile,
            self.still_picture,
            self.timing_info,
            self.decoder_model_info,
            self.operating_point_idc,
            self.seq_level_idx,
            self.seq_tier,
            self.decoder_model_present_for_this_op,
            self.operating_parameters_info,
            self.initial_display_delay_present_for_this_op,
            self.initial_display_delay_minus_1,
            self.c_operating_point_idc,
            self.max_frame_width_minus_one,
            self.max_frame_height_minus_one,
            self.delta_frame_id_length_minus_2,
            self.additional_frame_id_length_minus_1,
            self.use_128x128_superblock,
            self.enable_filter_intra,
            self.enable_intra_edge_filter,
            self.enable_interintra_compound,
            self.enable_masked_compound,
            self.enable_warped_motion,
            self.enable_dual_filter,
            self.enable_order_hint,
            self.enable_jnt_comp,
            self.enable_ref_frame_mvs,
            self.seq_force_screen_content_tools,
            self.seq_force_integer_mv,
            self.enable_superres,
            self.enable_cdef,
            self.enable_restoration,
            self.color_config,
            self.film_grain_params_present,
            self.frame_id_numbers_present_flag,
            self.reduced_still_picture_header,
            self.decoder_model_info_present_flag,
        )
    }
}

impl std::fmt::Display for TimingInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Timing_Info {{ num_units_in_display_tick: {}, time_scale: {}, equal_picture_interval: {}, num_ticks_per_picture_minus_1: {:?} }}",
            self.num_units_in_display_tick,
            self.time_scale,
            self.equal_picture_interval,
            self.num_ticks_per_picture_minus_1
        )
    }
}

impl std::fmt::Display for DecoderModelInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Decoder_Model_Info {{ buffer_delay_length_minus_1: {}, num_units_in_decoding_tick: {}, buffer_removal_delay_length_minus_1: {}, frame_presentation_delay_length_minus_1: {} }}",
            self.buffer_delay_length_minus_1,
            self.num_units_in_decoding_tick,
            self.buffer_removal_delay_length_minus_1,
            self.frame_presentation_delay_length_minus_1
        )
    }
}

impl std::fmt::Display for OperatingParametersInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operating_Parameters_Info {{ decoder_buffer_delay: {:?}, encoder_buffer_delay: {:?}, low_delay_mode_flag: {:?} }}",
            self.decoder_buffer_delay, self.encoder_buffer_delay, self.low_delay_mode_flag
        )
    }
}

impl std::fmt::Display for ColorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Color_Config {{ bit_depth: {}, mono_chrome: {}, num_planes: {}, ... }}",
            self.bit_depth, self.mono_chrome, self.num_planes
        )
    }
}
