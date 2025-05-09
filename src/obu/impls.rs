use bitstream_io::FromBitStream;

use crate::{consts::{self, OBU_TYPE}, generics::uvlc, leb_128};

use super::{handlers::choose_operating_point, Decoder_Model_Info, OBU_Sequence_Header, Operating_Parameters_Info, Timing_Info};

/* OBU syntax
open_bitstream_unit( sz ) {	Type
    obu_header()
    if ( obu_has_size_field ) {
        obu_size	leb128()
    } else {
        obu_size = sz - 1 - obu_extension_flag
    }
    startPosition = get_position( )
    if ( obu_type != OBU_SEQUENCE_HEADER &&
         obu_type != OBU_TEMPORAL_DELIMITER &&
         OperatingPointIdc != 0 &&
         obu_extension_flag == 1 )
    {
        inTemporalLayer = (OperatingPointIdc >> temporal_id ) & 1
        inSpatialLayer = (OperatingPointIdc >> ( spatial_id + 8 ) ) & 1
        if ( !inTemporalLayer || ! inSpatialLayer ) {
            drop_obu( )
            return
        }
    }
    if ( obu_type == OBU_SEQUENCE_HEADER )
        sequence_header_obu( )
    else if ( obu_type == OBU_TEMPORAL_DELIMITER )
        temporal_delimiter_obu( )
    else if ( obu_type == OBU_FRAME_HEADER )
        frame_header_obu( )
    else if ( obu_type == OBU_REDUNDANT_FRAME_HEADER )
        frame_header_obu( )
    else if ( obu_type == OBU_TILE_GROUP )
        tile_group_obu( obu_size )
    else if ( obu_type == OBU_METADATA )
        metadata_obu( )
    else if ( obu_type == OBU_FRAME )
        frame_obu( obu_size )
    else if ( obu_type == OBU_TILE_LIST )
        tile_list_obu( )
    else if ( obu_type == OBU_PADDING )
        padding_obu( )
    else
        reserved_obu( )
    currentPosition = get_position( )
    payloadBits = currentPosition - startPosition
    if ( obu_size > 0 && obu_type != OBU_TILE_GROUP &&
         obu_type != OBU_TILE_LIST &&
         obu_type != OBU_FRAME ) {
        trailing_bits( obu_size * 8 - payloadBits )
    }
}
*/
#[derive(Debug, PartialEq, Eq)]
pub struct OBU {
    obu_size: leb_128,
    obu_header: OBU_Header,
}

/*
    obu_header() {	Type
        obu_forbidden_bit	f(1)
        obu_type	f(4)
        obu_extension_flag	f(1)
        obu_has_size_field	f(1)
        obu_reserved_1bit	f(1)
        if ( obu_extension_flag == 1 )
            obu_extension_header()
    }
*/
#[derive(Debug, PartialEq, Eq)]
pub struct OBU_Header {
    obu_forbidden_bit: u8,                              // 1 bit
    obu_type: OBU_TYPE,                                 // 4 bits
    obu_extension_flag: u8,                             // 1 bit
    obu_has_size_field: u8,                             // 1 bit
    obu_reserved_1bit: u8,                              // 1 bit
    obu_extension_header: Option<OBU_Extension_Header>, // 8 bits
}

/* obu_extension_header() {	Type
        temporal_id	f(3)
        spatial_id	f(2)
        extension_header_reserved_3bits	f(3)
    }
*/
#[derive(Debug, PartialEq, Eq)]
pub struct OBU_Extension_Header {
    temporal_id: u8,                     // 3 bits
    spatial_id: u8,                      // 2 bits
    extension_header_reserved_3bits: u8, // 3 bits
}

impl OBU {
    
    pub fn open_bitstream_unit<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        sz: u64,
    ) -> Result<(), std::io::Error> {
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
            let extra_headers = header.obu_extension_header.unwrap();
            let in_temporal_layer = (operating_point_idc >> extra_headers.temporal_id) & 1u8;
            let in_spatial_layer = (operating_point_idc >> (extra_headers.spatial_id + 8u8)) & 1u8;

            if in_temporal_layer != 0u8 || in_spatial_layer == 0u8 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Layer filtering: not in temporal or spatial layer",
                ));
            }
        }

        match header.obu_type {
            OBU_TYPE::OBU_SEQUENCE_HEADER => 
        }
        Ok(())
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
    fn sequence_header_obu<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, std:io::Error> 
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

        if reduced_still_picture_header == 0{

            timing_info_present_flag = r.read::<1,u8>()?;

            // Timing_Info
            let timing_info = if timing_info_present_flag == 1 {
                Some(Timing_Info::from_reader(r)?)
            } else {
                None
            };

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
                if seq_level_idx.last()?.to_owned() > 7 {
                    seq_tier.push(r.read::<1, u8>()?);
                } else {
                    seq_tier.push(0);
                }

                // Operating_Parameters_Info
                if decoder_model_info_present_flag != 0u8  {
                    decoder_model_present_for_this_op.push(r.read::<1, u8>()?);
                    if decoder_model_present_for_this_op.last()?.to_owned() != 0 {
                        let decoder_model_info = decoder_model_info.as_ref().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Decoder model info not present"))?;

                        if operating_parameters_info.is_none() {
                            operating_parameters_info = Some(Operating_Parameters_Info::new());
                        }
                        operating_parameters_info.as_mut()?.from_reader(r,decoder_model_info)?;
                    }
                } else {
                    decoder_model_present_for_this_op.push(0);
                }

                // initial_display_delay_minus_1
                if initial_display_delay_present_flag != 0 {
                    initial_display_delay_present_for_this_op.push(r.read::<1, u8>()?);
                    if initial_display_delay_present_for_this_op.last()?.to_owned() != 0u8 {
                        if initial_display_delay_minus_1.is_none() {
                            initial_display_delay_minus_1 = Some(Vec::new());
                        }
                        initial_display_delay_minus_1.as_mut()?.push(r.read::<4, u8>()?);
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
        let enable_filter_infra = r.read::<1,u8>()?;
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
        todo!();

        let film_grain_params_present = r.read::<1,u8>()?;


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
