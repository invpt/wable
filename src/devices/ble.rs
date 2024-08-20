use core::{fmt::Debug, ops::Deref};

use embedded_io::{Read, Write};

mod command;
mod event;

pub use command::*;
pub use event::*;

pub struct Ble<H> {
    hci: H,
}

#[derive(Debug, Clone)]
pub struct RawParameters {
    len: u8,
    data: [u8; 255],
}

impl RawParameters {
    pub fn new(data: &[u8]) -> RawParameters {
        assert!(data.len() < 256, "data length cannot exceed 255");

        let len = data.len() as u8;
        let mut data_arr = [0; 255];
        data_arr.copy_from_slice(data);

        RawParameters {
            len,
            data: data_arr,
        }
    }
}

impl Deref for RawParameters {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len as usize]
    }
}

#[derive(Debug)]
pub enum BleError<E> {
    UnexpectedEof,
    Io(E),
}

impl<E> From<E> for BleError<E>
where
    E: embedded_io::Error,
{
    fn from(value: E) -> Self {
        Self::Io(value)
    }
}

impl<E> From<embedded_io::ReadExactError<E>> for BleError<E>
where
    E: embedded_io::Error,
{
    fn from(value: embedded_io::ReadExactError<E>) -> Self {
        match value {
            embedded_io::ReadExactError::UnexpectedEof => Self::UnexpectedEof,
            embedded_io::ReadExactError::Other(e) => Self::Io(e),
        }
    }
}

pub trait HciCommand {
    fn raw(self) -> RawHciCommand;
}

#[derive(Debug, Clone)]
pub struct RawHciCommand {
    pub opcode: Opcode,
    pub parameters: RawParameters,
}

impl HciCommand for RawHciCommand {
    fn raw(self) -> RawHciCommand {
        self
    }
}

pub trait HciEvent: Sized {
    fn parse(raw: RawHciEvent) -> Result<Self, ParseError>;
}

#[derive(Debug, Clone)]
pub struct RawHciEvent {
    pub code: EventCode,
    pub parameters: RawParameters,
}

pub enum ParseError {
    WrongCode,
    BadFormat,
}

impl<E, H> Ble<H>
where
    H: Read<Error = E> + Write<Error = E>,
    E: embedded_io::Error,
{
    pub fn new(hci: H) -> Self {
        Self { hci }
    }

    pub fn issue(&mut self, command: impl HciCommand) -> Result<(), BleError<E>> {
        let raw = command.raw();

        self.hci.write_all(&[0x01])?;
        self.hci.write_all(&raw.opcode.0.to_le_bytes())?;
        self.hci.write_all(&[raw.parameters.len])?;
        self.hci.write_all(&raw.parameters)?;
        self.hci.flush()?;

        Ok(())
    }

    pub fn receive(&mut self) -> Result<RawHciEvent, BleError<E>> {
        let mut header_buf = [0; 2];
        self.hci.read_exact(&mut header_buf)?;
        let [event_code, parameter_length] = header_buf;

        let mut event_parameters_buf = [0; 255];
        let event_parameters = &mut event_parameters_buf[..parameter_length as usize];
        self.hci.read_exact(event_parameters)?;

        Ok(RawHciEvent {
            code: EventCode(event_code),
            parameters: RawParameters::new(event_parameters),
        })
    }
}
