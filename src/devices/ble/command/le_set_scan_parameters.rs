use crate::devices::ble::{
    data::{
        opcode::{Ogf, Opcode},
        status_code::StatusCode,
        Encode, Encoder, EncoderFull,
    },
    event::command_complete::CommandWithCompleteEvent,
};

use super::CommandParameters;

#[derive(Debug)]
pub struct LeSetScanParameters {
    pub le_scan_type: u8,
    pub le_scan_interval: u16,
    pub le_scan_window: u16,
    pub own_address_type: u8,
    pub scanning_filter_policy: u8,
}

impl Encode for LeSetScanParameters {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.encode(&self.le_scan_type)?;
        e.encode(&self.le_scan_interval)?;
        e.encode(&self.le_scan_window)?;
        e.encode(&self.own_address_type)?;
        e.encode(&self.scanning_filter_policy)?;

        Ok(())
    }
}

impl CommandParameters for LeSetScanParameters {
    const OPCODE: Opcode = Opcode::new(Ogf::LE_CONTROLLER, 0x000B);
}

impl CommandWithCompleteEvent for LeSetScanParameters {
    type ReturnParameters = StatusCode;
}
