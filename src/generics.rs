use std::u32;

use bitstream_io::FromBitStream;

// 4.10.3 UVLC
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Uvlc {
    pub value: u32,
}

impl Uvlc {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl FromBitStream for Uvlc {
    type Error = std::io::Error;

    fn from_reader<R: bitstream_io::BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut leading_zeros = 0u32;
        while r.read::<1, u8>()? == 0u8 && leading_zeros < 32 {
            leading_zeros += 1;
        }

        if leading_zeros >= 32 {
            return Ok(Self { value: u32::MAX });
        }
        r.read_var(leading_zeros).map(|a| Self { value: a })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Leb128 {
    pub value: u64,
}

impl Leb128 {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn size_in_bytes(value: u64) -> usize {
        let mut size = 1;
        let mut val = value >> 7;
        while val > 0 {
            size += 1;
            val >>= 7;
        }
        size
    }
}

impl FromBitStream for Leb128 {
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ns {
    pub value: u32,
}

/*
ns( n ) {	Type
    w = FloorLog2(n) + 1
    m = (1 << w) - n
    v	f(w - 1)
    if ( v < m )
        return v
    extra_bit	f(1)
    return (v << 1) - m + extra_bit
} */
impl Ns {
    pub fn ns<R: bitstream_io::BitRead + ?Sized>(r: &mut R, n: u32) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let w = floor_log2(n) + 1;
        let m = (1 << w) - n;
        let v = r.read_var::<u32>(w - 1)?;
        if v < m {
            return Ok(Self { value: v });
        }

        // extra_bit 
        Ok(Self {
            value: (v << 1) - m + r.read::<1, u32>()?,
        })
    }
}

/*
su(n) {	Type
    value	f(n)
    signMask = 1 << (n - 1)	 
    if ( value & signMask )	 
        value = value - 2 * signMask	 
    return value	 
}
*/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Su {
    pub value: u32,
}

impl Su {
    pub fn su<R: bitstream_io::BitRead + ?Sized>(r: &mut R, n: u32) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let value = r.read_var::<u32>(n)?;
        let sign_mask = 1 << (n-1);
        if value & sign_mask != 0 {
            Ok(Self {
                value: value - 2 * sign_mask,
            })
        } else {
            Ok(Self { value })
        }
    }
}

impl Default for Su {
    fn default() -> Self {
        Self { value: 0 }
    }
}


pub fn floor_log2(x: u32) -> u32 {
    let mut s = 0;
    let mut mut_x = x;
    while mut_x != 0 {
        mut_x >>= 1;
        s += 1;
    }
    s - 1
}

pub fn clip3(min: i16,max: i16, x: i16) -> i16 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

impl std::fmt::Display for Uvlc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uvlc({})", self.value)
    }
}

impl std::fmt::Display for Leb128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "leb_128({})", self.value)
    }
}
