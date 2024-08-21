use crate::devices::ble::{BoundedBytes, CompleteCommand, HciCommand, RawHciCommand};

use super::{Ogf, Opcode, StatusCodeReturnParameters};

const OPCODE: Opcode = Opcode::new(Ogf::CONTROLLER_BASEBAND, 0x0003);

pub struct Reset {   
}

impl HciCommand for Reset {
    fn raw(self) -> crate::devices::ble::RawHciCommand {
        RawHciCommand {
            opcode: OPCODE,
            parameters: BoundedBytes::new(&[]),
        }
    }
}

impl CompleteCommand for Reset {
    type ReturnParameters = StatusCodeReturnParameters;

    fn match_opcode(opcode: Opcode) -> bool {
        opcode == OPCODE
    }
}
