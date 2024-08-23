use core::num::NonZeroU8;

use super::{event::command_complete::ReturnParameters, ParseError, RawParameters};

pub mod le_set_scan_enable;
pub mod le_set_scan_parameters;
pub mod reset;
pub mod set_event_mask;

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

pub struct AnyCommand;

pub trait HciCommand {
    fn match_opcode(opcode: Opcode) -> bool;
    fn raw(self) -> RawHciCommand;
}

impl HciCommand for AnyCommand {
    fn match_opcode(_opcode: Opcode) -> bool {
        true
    }

    fn raw(self) -> RawHciCommand {
        panic!("AnyCommand cannot be encoded")
    }
}

#[derive(Debug, Clone)]
pub struct RawHciCommand {
    pub opcode: Opcode,
    pub parameters: RawParameters,
}

impl HciCommand for RawHciCommand {
    fn match_opcode(_opcode: Opcode) -> bool {
        true
    }

    fn raw(self) -> RawHciCommand {
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatusCode(pub u8);

#[derive(Debug, Clone, Copy)]
pub struct StatusError(pub NonZeroU8);

impl StatusCode {
    pub fn is_successful(self) -> bool {
        self.0 == 0x00
    }

    pub fn assert(self) -> Result<(), StatusError> {
        match NonZeroU8::new(self.0) {
            Some(nz) => Err(StatusError(nz)),
            None => Ok(())
        }
    }
}

#[derive(Debug)]
pub struct StatusCodeReturnParameters {
    pub status: StatusCode,
}

impl ReturnParameters for StatusCodeReturnParameters {
    fn parse(raw: RawParameters) -> Result<Self, ParseError> {
        let &[status] = &*raw else {
            return Err(ParseError);
        };

        Ok(StatusCodeReturnParameters {
            status: StatusCode(status),
        })
    }
}
