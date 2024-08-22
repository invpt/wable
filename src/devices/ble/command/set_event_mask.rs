use crate::devices::ble::{event::command_complete::CommandWithCompleteEvent, BoundedBytes, HciCommand};

use super::{Ogf, Opcode, RawHciCommand, StatusCodeReturnParameters};

const OPCODE: Opcode = Opcode::new(Ogf::CONTROLLER_BASEBAND, 0x0001);

pub struct SetEventMask {
    pub mask: u64
}

impl HciCommand for SetEventMask {
    fn match_opcode(opcode: Opcode) -> bool {
        opcode == OPCODE
    }

    fn raw(self) -> RawHciCommand {
        RawHciCommand {
            opcode: OPCODE,
            parameters: BoundedBytes::new(&self.mask.to_le_bytes()),
        }
    }
}

impl CommandWithCompleteEvent for SetEventMask {
    type ReturnParameters = StatusCodeReturnParameters;
}
