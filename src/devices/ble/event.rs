use core::fmt::Debug;

use super::data::{Buffer, DecodeError, MaybeDecode};

pub mod command_complete;
pub mod command_status;
pub mod le_advertising_report;
pub mod le_connection_complete;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventCode(pub u8);

pub trait EventParameters: MaybeDecode {
    const EVENT_CODE: EventCode;
}

pub struct EncodedEvent {
    pub code: EventCode,
    pub parameters: Buffer<255>,
}

impl EncodedEvent {
    pub fn decode<E: EventParameters>(&self) -> Result<Option<E>, DecodeError> {
        if self.code != E::EVENT_CODE {
            return Ok(None);
        }

        E::maybe_decode(&mut &*self.parameters)
    }
}
