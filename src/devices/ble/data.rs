use core::ops::Deref;

pub mod address;
pub mod opcode;
pub mod status_code;

pub type Buffer<const MAX: usize> = _Buffer<[u8; MAX]>;

pub type Buf = _Buffer<[u8]>;

mod private {
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct _Buffer<D: ?Sized> {
        pub(super) len: usize,
        pub(super) data: D,
    }
}

use private::_Buffer;

impl<const MAX: usize> Buffer<MAX> {
    pub fn new() -> Buffer<MAX> {
        Buffer {
            len: 0,
            data: [0; MAX],
        }
    }
}

impl Deref for Buf {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len]
    }
}

impl<const MAX: usize> Deref for Buffer<MAX> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len]
    }
}

impl<'a, const MAX: usize> From<&'a [u8]> for Buffer<MAX> {
    fn from(value: &'a [u8]) -> Self {
        let len = value.len();
        if len > MAX {
            panic!(
                "cannot create buffer from oversized source slice (len was {len}, max was {MAX})"
            )
        }
        let mut data = [0; MAX];
        data[..len].copy_from_slice(value);
        Buffer { len, data }
    }
}

#[derive(Debug)]
pub struct EncoderFull;

pub trait Encode {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized;
}

impl Encode for u8 {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.write(&[*self])
    }
}

impl Encode for u16 {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.write(&self.to_le_bytes())
    }
}

impl Encode for u64 {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.write(&self.to_le_bytes())
    }
}

impl Encode for [u8] {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.write(self)
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.write(self)
    }
}

pub trait Encoder {
    fn write(&mut self, data: &[u8]) -> Result<(), EncoderFull>;

    fn encode<E>(&mut self, encodable: &E) -> Result<(), EncoderFull>
    where
        E: Encode + ?Sized,
    {
        encodable.encode(self)
    }
}

impl Encoder for Buf {
    fn write(&mut self, data: &[u8]) -> Result<(), EncoderFull> {
        let Some(new_len) = self.len.checked_add(data.len()) else {
            return Err(EncoderFull);
        };

        if new_len > self.data.len() {
            return Err(EncoderFull);
        }

        self.data[self.len..new_len].copy_from_slice(data);
        self.len = new_len;

        Ok(())
    }
}

impl<const MAX: usize> Encoder for Buffer<MAX> {
    fn write(&mut self, data: &[u8]) -> Result<(), EncoderFull> {
        (self as &mut Buf).write(data)
    }
}

#[derive(Debug)]
pub enum DecodeError {
    Empty,
    Malformed(&'static str),
}

pub trait Decode: Sized {
    fn decode<D>(d: &mut D) -> Result<Self, DecodeError>
    where
        D: Decoder + ?Sized;
}

impl Decode for () {
    fn decode<D>(d: &mut D) -> Result<Self, DecodeError>
    where
        D: Decoder + ?Sized,
    {
        Ok(())
    }
}

impl Decode for u8 {
    fn decode<D>(d: &mut D) -> Result<Self, DecodeError>
    where
        D: Decoder + ?Sized,
    {
        Ok(d.decode::<[u8; 1]>()?[0])
    }
}

impl Decode for u16 {
    fn decode<D>(d: &mut D) -> Result<Self, DecodeError>
    where
        D: Decoder + ?Sized,
    {
        Ok(u16::from_le_bytes(d.decode::<[u8; 2]>()?))
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode<D>(d: &mut D) -> Result<Self, DecodeError>
    where
        D: Decoder + ?Sized,
    {
        let mut buf = [0; N];
        d.read(&mut buf)?;
        Ok(buf)
    }
}

impl<const MAX: usize> Decode for Buffer<MAX> {
    fn decode<D>(d: &mut D) -> Result<Self, DecodeError>
    where
        D: Decoder + ?Sized,
    {
        if d.available() > MAX {
            return Err(DecodeError::Malformed(
                "there are more bytes available than can fit in the target buffer",
            ));
        }

        let mut buffer = Self::new();
        buffer.len = d.available();
        d.read(&mut buffer.data[..d.available()])?;

        Ok(buffer)
    }
}

pub trait MaybeDecode: Sized {
    fn maybe_decode<D>(d: &mut D) -> Result<Option<Self>, DecodeError>
    where
        D: MaybeDecoder + ?Sized;
}

impl<T> MaybeDecode for T
where
    T: Decode,
{
    fn maybe_decode<D>(d: &mut D) -> Result<Option<Self>, DecodeError>
    where
        D: MaybeDecoder + ?Sized,
    {
        Ok(Some(T::decode(d)?))
    }
}

pub trait Decoder {
    fn read(&mut self, buf: &mut [u8]) -> Result<(), DecodeError>;
    fn available(&self) -> usize;

    fn decode<D>(&mut self) -> Result<D, DecodeError>
    where
        D: Decode + ?Sized,
    {
        D::decode(self)
    }
}

pub trait MaybeDecoder: Decoder {
    fn maybe_decode<D>(&mut self) -> Result<Option<D>, DecodeError>
    where
        Self: Sized,
        D: MaybeDecode + ?Sized;
}

impl<T: Decoder + Copy> MaybeDecoder for T {
    fn maybe_decode<D>(&mut self) -> Result<Option<D>, DecodeError>
    where
        Self: Sized,
        D: MaybeDecode + ?Sized,
    {
        let mut copy = *self;
        if let Some(result) = D::maybe_decode(&mut copy)? {
            *self = copy;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

impl<'a> Decoder for &'a [u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        if self.len() < buf.len() {
            return Err(DecodeError::Empty);
        }

        buf.copy_from_slice(&self[0..buf.len()]);
        *self = &self[buf.len()..];

        Ok(())
    }

    fn available(&self) -> usize {
        self.len()
    }
}
