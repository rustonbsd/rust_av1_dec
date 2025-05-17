mod frame_header;
mod context;
mod sequence_header;

use bitstream_io::FromBitStream;
use context::DecoderContext;
use frame_header::FrameHeader;
use sequence_header::SequenceHeader;

use crate::{Leb128, consts::OBU_TYPE};


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OBU {
    pub size: Leb128,                            // leb128
    pub header: ObuHeader,                       // 16 bits
    pub sequence_header: Option<SequenceHeader>, // Add this field
    pub temporal_delimiter: Option<TemporalDelimiter>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ObuHeader {
    pub forbidden_bit: u8,                            // 1 bit
    pub obu_type: OBU_TYPE,                           // 4 bits
    pub extension_flag: u8,                           // 1 bit
    pub has_size_field: u8,                           // 1 bit
    pub reserved_1bit: u8,                            // 1 bit
    pub extension_header: Option<ObuExtensionHeader>, // 8 bits
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ObuExtensionHeader {
    pub temporal_id: u8,                     // 3 bits
    pub spatial_id: u8,                      // 2 bits
    pub extension_header_reserved_3bits: u8, // 3 bits
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TemporalDelimiter {
    seen_frame_header: u32,
}

impl OBU {
    pub fn open_bitstream_unit<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        sz: u64,
        ctx: &mut DecoderContext,
    ) -> Result<OBU, std::io::Error> {
        let header = ObuHeader::from_reader(r)?;
        let obu_size = if header.has_size_field != 0 {
            Leb128::from_reader(r)?
        } else {
            Leb128::new(sz - 1 - header.extension_flag as u64)
        };

        let sequence_header = if header.obu_type == OBU_TYPE::OBU_SEQUENCE_HEADER {
            Some(SequenceHeader::sequence_header_obu(r)?)
        } else {
            None
        };
        let temporal_delimiter = if header.obu_type == OBU_TYPE::OBU_TEMPORAL_DELIMITER {
            Some(TemporalDelimiter {
                seen_frame_header: 0,
            })
        } else {
            None
        };

        /*
            If obu_type is equal to OBU_FRAME_HEADER or obu_type is equal to OBU_FRAME, 
            it is a requirement of bitstream conformance that SeenFrameHeader is equal to 0.
            
            If obu_type is equal to OBU_REDUNDANT_FRAME_HEADER, 
            it is a requirement of bitstream conformance that SeenFrameHeader is equal to 1. 
        */
        let frame_header = if header.obu_type == OBU_TYPE::OBU_REDUNDANT_FRAME_HEADER {
            if ctx.last_frame_header.is_none() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "No last frame header",
                ));
            }
            Some(FrameHeader::frame_header_obu(r,ctx)?)
        } else {
            None
        };

        Ok(OBU {
            size: obu_size,
            header,
            sequence_header,
            temporal_delimiter,
        })
    }
}

impl FromBitStream for ObuHeader {
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
        let obu_extension_header = if obu_extension_flag != 0 {
            Some(ObuExtensionHeader::from_reader(r)?)
        } else {
            None
        };

        Ok(Self {
            forbidden_bit: obu_forbidden_bit,
            obu_type,
            extension_flag: obu_extension_flag,
            has_size_field: obu_has_size_field,
            reserved_1bit: obu_reserved_1bit,
            extension_header: obu_extension_header,
        })
    }
}

impl FromBitStream for ObuExtensionHeader {
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

impl std::fmt::Display for OBU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OBU {{ obu_size: {}, obu_header: {}, obu_sequence_header: {:?} }}",
            self.size, self.header, self.sequence_header
        )
    }
}

impl std::fmt::Display for ObuHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OBU_Header {{ obu_forbidden_bit: {}, obu_type: {:?}, obu_extension_flag: {}, obu_has_size_field: {}, obu_reserved_1bit: {}, obu_extension_header: {:?} }}",
            self.forbidden_bit,
            self.obu_type,
            self.extension_flag,
            self.has_size_field,
            self.reserved_1bit,
            self.extension_header
        )
    }
}

impl std::fmt::Display for ObuExtensionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OBU_Extension_Header {{ temporal_id: {}, spatial_id: {}, extension_header_reserved_3bits: {} }}",
            self.temporal_id, self.spatial_id, self.extension_header_reserved_3bits
        )
    }
}
