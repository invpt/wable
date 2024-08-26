use crate::devices::ble::data::{address::Address, status_code::StatusCode, DecodeError, MaybeDecode, MaybeDecoder};

use super::{EventParameters, EventCode};

pub struct LeConnectionComplete {
    pub status: StatusCode,
    pub connection_handle: u16,
    pub role: u8,
    pub peer_address_type: u8,
    pub peer_address: Address,
    pub connection_interval: u16,
    pub peripheral_latency: u16,
    pub supervision_timeout: u16,
    pub central_clock_accuracy: u8,
}

impl MaybeDecode for LeConnectionComplete {
    fn maybe_decode<D>(d: &mut D) -> Result<Option<Self>, DecodeError>
    where
        D: MaybeDecoder + ?Sized,
    {
        let 0x01u8 = d.decode()? else { return Ok(None) };
        
        Ok(Some(Self {
            status: d.decode()?,
            connection_handle: d.decode()?,
            role: d.decode()?,
            peer_address_type: d.decode()?,
            peer_address: d.decode()?,
            connection_interval: d.decode()?,
            peripheral_latency: d.decode()?,
            supervision_timeout: d.decode()?,
            central_clock_accuracy: d.decode()?,
        }))
    }
}

impl EventParameters for LeConnectionComplete {
    const EVENT_CODE: super::EventCode = EventCode(0x3E);
}
