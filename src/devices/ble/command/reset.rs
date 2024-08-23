use crate::devices::ble::{event::command_complete::CommandWithCompleteEvent, BoundedBytes, HciCommand};

use super::{Ogf, Opcode, RawHciCommand, StatusCodeReturnParameters};

const OPCODE: Opcode = Opcode::new(Ogf::CONTROLLER_BASEBAND, 0x0003);

pub struct Reset {   
}

impl HciCommand for Reset {
    fn match_opcode(opcode: Opcode) -> bool {
        opcode == OPCODE
    }

    fn raw(self) -> RawHciCommand {
        RawHciCommand {
            opcode: OPCODE,
            parameters: BoundedBytes::new(&[]),
        }
    }
}

impl CommandWithCompleteEvent for Reset {
    type ReturnParameters = StatusCodeReturnParameters;
}
