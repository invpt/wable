use core::num::NonZeroU8;

use super::{Decode, DecodeError, Decoder, Encode, Encoder, EncoderFull};

#[derive(Debug, Clone, Copy)]
pub struct StatusCode(pub u8);

impl Encode for StatusCode {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.encode(&self.0)
    }
}

impl Decode for StatusCode {
    fn decode<D>(d: &mut D) -> Result<Self, DecodeError>
    where
        D: Decoder + ?Sized,
    {
        Ok(Self(d.decode()?))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatusError(pub NonZeroU8);

impl StatusCode {
    pub fn is_successful(self) -> bool {
        self.0 == 0x00
    }

    pub fn assert(self) -> Result<(), StatusError> {
        match NonZeroU8::new(self.0) {
            Some(nz) => Err(StatusError(nz)),
            None => Ok(()),
        }
    }
}
