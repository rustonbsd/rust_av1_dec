use std::u32;

use bitstream_io::FromBitStream;

// 4.10.3 UVLC
pub struct uvlc {
    value: u32,
}

impl uvlc {
    pub fn new(value: u32) -> Self {
        Self {
            value
        }
    }
}

impl FromBitStream for uvlc {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized {
        
        let mut leading_zeros = 0u32;
        while r.read::<1,u8>()? == 0u8 && leading_zeros < 32 {
            leading_zeros += 1;
        }

        if leading_zeros >= 32 {
            return Ok(Self { value: u32::MAX})
        }
        r.read_var(leading_zeros) as Result<u32, std::io::Error>
        
    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct leb_128 {
    value: u64,
}

impl leb_128 {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

impl FromBitStream for leb_128 {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut value: u64 = 0;
        let mut buf: [u8; 1] = [0u8; 1];
        for i in 0..8 {
            r.read_bytes(&mut buf)?;
            value |= ((buf[0] & 0x7f) as u64) << (i * 7);

            if buf[0] & 0x80 == 0 {
                return Ok(Self { value: value });
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "leb128 exeeded 8 bytes",
        ))
    }
}
