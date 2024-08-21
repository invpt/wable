use crate::devices::ble::{BoundedBytes, CompleteCommand, HciCommand, RawHciCommand};

use super::{Ogf, Opcode, StatusCodeReturnParameters};

const OPCODE: Opcode = Opcode::new(Ogf::CONTROLLER_BASEBAND, 0x0001);

pub struct SetEventMask {
    pub mask: u64
}

impl HciCommand for SetEventMask {
    fn raw(self) -> RawHciCommand {
        RawHciCommand {
            opcode: OPCODE,
            parameters: BoundedBytes::new(&self.mask.to_le_bytes()),
        }
    }
}

impl CompleteCommand for SetEventMask {
    type ReturnParameters = StatusCodeReturnParameters;

    fn match_opcode(opcode: Opcode) -> bool {
        opcode == OPCODE
    }
}