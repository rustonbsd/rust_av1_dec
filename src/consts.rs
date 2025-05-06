use bitstream_io::{FromBitStream, ToBitStream};

/*
obu_type specifies the type of data structure contained in the OBU payload:

obu_type	Name of obu_type	Layer-specific
0	Reserved	-
1	OBU_SEQUENCE_HEADER.	N
2	OBU_TEMPORAL_DELIMITER	N
3	OBU_FRAME_HEADER	Y
4	OBU_TILE_GROUP	Y
5	OBU_METADATA	See Table in Section 6.7.1
6	OBU_FRAME	Y
7	OBU_REDUNDANT_FRAME_HEADER	Y
8	OBU_TILE_LIST	N
9-14	Reserved	-
15	OBU_PADDING	Either
*/
// OBU_TYPE enum that resolves to the right byte code
#[derive(Debug, PartialEq, Eq)]
pub enum OBU_TYPE {
    OBU_SEQUENCE_HEADER ,
    OBU_TEMPORAL_DELIMITER,
    OBU_FRAME_HEADER,
    OBU_TILE_GROUP,
    OBU_METADATA,
    OBU_FRAME,
    OBU_REDUNDANT_FRAME_HEADER,
    OBU_TILE_LIST,
    OBU_PADDING,
}

impl ToBitStream for OBU_TYPE {
    type Error = std::io::Error;

    fn to_writer<W: bitstream_io::BitWrite + ?Sized>(&self, w: &mut W) -> Result<(), Self::Error>
    where
        Self: Sized {
        let val = match self {
            Self::OBU_SEQUENCE_HEADER => 1u8,
            Self::OBU_TEMPORAL_DELIMITER => 2u8,
            Self::OBU_FRAME_HEADER => 3u8,
            Self::OBU_TILE_GROUP => 4u8,
            Self::OBU_METADATA => 5u8,
            Self::OBU_FRAME => 6u8,
            Self::OBU_REDUNDANT_FRAME_HEADER => 7u8,
            Self::OBU_TILE_LIST => 8u8,
            Self::OBU_PADDING => 15u8,
        };
        w.write::<4,u8>(val)?;
        Ok(())
    }
}

impl FromBitStream for OBU_TYPE {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized {
            match r.read::<4,u8>()? {
                1 => Ok(Self::OBU_SEQUENCE_HEADER),
                2 => Ok(Self::OBU_TEMPORAL_DELIMITER),
                3 => Ok(Self::OBU_FRAME_HEADER),
                4 => Ok(Self::OBU_TILE_GROUP),
                5 => Ok(Self::OBU_METADATA),
                6 => Ok(Self::OBU_FRAME),
                7 => Ok(Self::OBU_REDUNDANT_FRAME_HEADER),
                8 => Ok(Self::OBU_TILE_LIST),
                15 => Ok(Self::OBU_PADDING),
                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid OBU_TYPE"))
            }
        }
}