use bitstream_io::FromBitStream;

use crate::{consts::OBU_TYPE, leb_128};

use super::OBU_Sequence_Header;

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

        let timing_info_present_flag: u8;
        let decoder_model_info_present_flag: u8;
        let initial_display_delay_present_flag: u8;
        let operating_points_cnt_minus_1: u8;
        let seq_level_idx: Vec<u8>;
        let seq_tier: Vec<u8>;
        let decoder_model_present_for_this_op: Vec<u8>;
        let initial_display_delay_present_for_this_op: Vec<u8>;

        if reduced_still_picture_header == 1 {
            timing_info_present_flag = 0;
            decoder_model_info_present_flag = 0;
            initial_display_delay_present_flag = 0;
            operating_points_cnt_minus_1 = 0;
            seq_level_idx = vec![r.read::<5, u8>()?];
            seq_tier = vec![0];
            decoder_model_present_for_this_op = vec![0];
            initial_display_delay_present_for_this_op = vec![0];
        } else {
            timing_info_present_flag = r.read::<1,u8>()?;
            if timing_info_present_flag == 1 {
                decoder_model_info_present_flag = r.read::<1, u8>()?;
                initial_display_delay_present_flag = r.read::<1, u8>()?;
            }
        }
        

        Ok(Self {
            seq_profile: r.read::<3, u8>()?,
            still_picture: r.read::<1, u8>()?,
            reduced_still_picture_header: r.read::<1, u8>()?,
        })
    }

}