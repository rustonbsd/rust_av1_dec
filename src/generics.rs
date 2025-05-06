use bitstream_io::FromBitStream;

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
