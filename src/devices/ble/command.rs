use super::{RawParameters, ParseError, ReturnParameters};

mod le_set_scan_enable;
mod le_set_scan_parameters;
mod reset;
mod set_event_mask;

pub use le_set_scan_enable::*;
pub use le_set_scan_parameters::*;
pub use reset::*;
pub use set_event_mask::*;

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

#[derive(Debug)]
pub struct StatusCode(pub u8);

impl StatusCode {
    pub fn is_successful(self) -> bool {
        self.0 == 0x00
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
