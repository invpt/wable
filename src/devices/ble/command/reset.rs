use crate::devices::ble::{
    data::{
        opcode::{Ogf, Opcode},
        status_code::StatusCode,
        Encode, Encoder, EncoderFull,
    },
    event::command_complete::CommandWithCompleteEvent,
};

use super::CommandParameters;

pub struct Reset {}

impl Encode for Reset {
    fn encode<E>(&self, _e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        Ok(())
    }
}

impl CommandParameters for Reset {
    const OPCODE: Opcode = Opcode::new(Ogf::CONTROLLER_BASEBAND, 0x0003);
}

impl CommandWithCompleteEvent for Reset {
    type ReturnParameters = StatusCode;
}
