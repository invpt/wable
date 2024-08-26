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
pub struct LeSetScanEnable {
    pub le_scan_enable: u8,
    pub filter_duplicates: u8,
}

impl Encode for LeSetScanEnable {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.encode(&self.le_scan_enable)?;
        e.encode(&self.filter_duplicates)?;

        Ok(())
    }
}

impl CommandParameters for LeSetScanEnable {
    const OPCODE: Opcode = Opcode::new(Ogf::LE_CONTROLLER, 0x000C);
}

impl CommandWithCompleteEvent for LeSetScanEnable {
    type ReturnParameters = StatusCode;
}
