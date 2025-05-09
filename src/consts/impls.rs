use bitstream_io::{FromBitStream, ToBitStream};

use super::{CHROMA_SAMPLE_POSITION, COLOR_PRIMARIES, MATRIX_COEFFICIENTS, OBU_TYPE, TRANSFER_CHARACTERISTICS};

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

impl FromBitStream for COLOR_PRIMARIES {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized {
        match r.read::<8,u8>()? {
            1 => Ok(Self::CP_BT_709),
            2 => Ok(Self::CP_UNSPECIFIED),
            4 => Ok(Self::CP_BT_470_M),
            5 => Ok(Self::CP_BT_470_B_G),
            6 => Ok(Self::CP_BT_601),
            7 => Ok(Self::CP_SMPTE_240),
            8 => Ok(Self::CP_GENERIC_FILM),
            9 => Ok(Self::CP_BT_2020),
            10 => Ok(Self::CP_XYZ),
            11 => Ok(Self::CP_SMPTE_431),
            12 => Ok(Self::CP_SMPTE_432),
            22 => Ok(Self::CP_EBU_3213),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid COLOR_PRIMARIES"))
        }
    }
}

impl FromBitStream for TRANSFER_CHARACTERISTICS {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized {
        match r.read::<8,u8>()? {
            0 => Ok(Self::TC_RESERVED_0),
            1 => Ok(Self::TC_BT_709),
            2 => Ok(Self::TC_UNSPECIFIED),
            3 => Ok(Self::TC_RESERVED_3),
            4 => Ok(Self::TC_BT_470_M),
            5 => Ok(Self::TC_BT_470_B_G),
            6 => Ok(Self::TC_BT_601),
            7 => Ok(Self::TC_SMPTE_240),
            8 => Ok(Self::TC_LINEAR),
            9 => Ok(Self::TC_LOG_100),
            10 => Ok(Self::TC_LOG_100_SQRT10),
            11 => Ok(Self::TC_IEC_61966),
            12 => Ok(Self::TC_BT_1361),
            13 => Ok(Self::TC_SRGB),
            14 => Ok(Self::TC_BT_2020_10_BIT),
            15 => Ok(Self::TC_BT_2020_12_BIT),
            16 => Ok(Self::TC_SMPTE_2084),
            17 => Ok(Self::TC_SMPTE_428),
            18 => Ok(Self::TC_HLG),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid TRANSFER_CHARACTERISTICS"))
        }
    }
}

impl FromBitStream for MATRIX_COEFFICIENTS {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized {
        match r.read::<8,u8>()? {
            0 => Ok(Self::MC_IDENTITY),
            1 => Ok(Self::MC_BT_709),
            2 => Ok(Self::MC_UNSPECIFIED),
            3 => Ok(Self::MC_RESERVED_3),
            4 => Ok(Self::MC_FCC),
            5 => Ok(Self::MC_BT_470_B_G),
            6 => Ok(Self::MC_BT_601),
            7 => Ok(Self::MC_SMPTE_240),
            8 => Ok(Self::MC_SMPTE_YCGCO),
            9 => Ok(Self::MC_BT_2020_NCL),
            10 => Ok(Self::MC_BT_2020_CL),
            11 => Ok(Self::MC_SMPTE_2085),
            12 => Ok(Self::MC_CHROMAT_NCL),
            13 => Ok(Self::MC_CHROMAT_CL),
            14 => Ok(Self::MC_ICTCP),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid MATRIX_COEFFICIENTS"))
        }
    }
}

impl FromBitStream for CHROMA_SAMPLE_POSITION {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized {
        match r.read::<2,u8>()? {
            0 => Ok(Self::CSP_UNKNOWN),
            1 => Ok(Self::CSP_VERTICAL),
            2 => Ok(Self::CSP_COLOCATED),
            3 => Ok(Self::CSP_RESERVED),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid CHROMA_SAMPLE_POSITION"))
        }
    }
}