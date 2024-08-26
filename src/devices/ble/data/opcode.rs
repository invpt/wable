use super::{Decode, DecodeError, Decoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ogf(pub u8);

impl Ogf {
    pub const CONTROLLER_BASEBAND: Ogf = Ogf(0x03);
    pub const LE_CONTROLLER: Ogf = Ogf(0x08);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode(pub u16);

impl Opcode {
    pub const fn new(ogf: Ogf, ocf: u16) -> Opcode {
        Opcode(((ogf.0 as u16) << 10) | ocf)
    }
}

impl Decode for Opcode {
    fn decode<D>(d: &mut D) -> Result<Self, DecodeError>
    where
        D: Decoder + ?Sized,
    {
        Ok(Self(d.decode()?))
    }
}
