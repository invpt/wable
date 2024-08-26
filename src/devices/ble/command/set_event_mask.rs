use crate::devices::ble::{
    data::{
        opcode::{Ogf, Opcode},
        status_code::StatusCode,
        Encode, Encoder, EncoderFull,
    },
    event::command_complete::CommandWithCompleteEvent,
};

use super::CommandParameters;

pub struct SetEventMask {
    pub mask: u64,
}

impl Encode for SetEventMask {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.encode(&self.mask)?;

        Ok(())
    }
}

impl CommandParameters for SetEventMask {
    const OPCODE: Opcode = Opcode::new(Ogf::CONTROLLER_BASEBAND, 0x0001);
}

impl CommandWithCompleteEvent for SetEventMask {
    type ReturnParameters = StatusCode;
}
