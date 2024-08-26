use super::{Decode, Encode};

pub struct Address(pub [u8; 6]);

impl Encode for Address {
    fn encode<E>(&self, e: &mut E) -> Result<(), super::EncoderFull>
    where
        E: super::Encoder + ?Sized,
    {
        e.encode(&self.0)
    }
}

impl Decode for Address {
    fn decode<D>(d: &mut D) -> Result<Self, super::DecodeError>
    where
        D: super::Decoder + ?Sized,
    {
        Ok(Self(d.decode()?))
    }
}
