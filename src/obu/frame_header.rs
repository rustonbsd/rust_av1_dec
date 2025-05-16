use bitstream_io::FromBitStream;

use crate::consts::{FRAME_TYPE, NUM_REF_FRAMES};

use super::sequence_header::SequenceHeader;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FrameHeader {}

impl FrameHeader {
    pub fn frame_header_obu<R: bitstream_io::BitRead + ?Sized>(
        r: &mut R,
        sequence_header: &SequenceHeader,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
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
        let mut show_existing_frame: u8 = 0;
        let frame_type: FRAME_TYPE;
        let frame_is_intra: u8;
        let show_frame: u8;
        let showable_frame: u8;
        let frame_to_show_map_index: u8;
        let mut refresh_frame_flag: u8;

        // It is a requirement of bitstream conformance 
        // that the number of bits needed to read display_frame_id does not exceed 16. 
        // This is equivalent to the constraint that idLen <= 16.
        let display_frame_id: u16;

        if sequence_header.reduced_still_picture_header == 0 {
            show_existing_frame = r.read::<1, u8>()?;
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
                    // temporal_point_info()
                }
                refresh_frame_flag = 0;
                if sequence_header.frame_id_numbers_present_flag != 0 {
                    display_frame_id = r.read_var(id_len.ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Id len not present")
                    })? as u32)?;
                }
            }
        }

        todo!()
    }
}
