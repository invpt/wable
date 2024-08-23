use core::fmt::Debug;

use super::{ParseError, RawParameters};

pub mod command_complete;
pub mod command_status;
pub mod le_advertising_report;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventCode(pub u8);

pub trait HciEvent: Sized {
    fn match_parse(raw: &RawHciEvent) -> Result<Option<Self>, ParseError>;
}

#[derive(Debug, Clone)]
pub struct RawHciEvent {
    pub code: EventCode,
    pub parameters: RawParameters,
}
