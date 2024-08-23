use crate::devices::ble::{event::command_complete::CommandWithCompleteEvent, HciCommand, RawParameters};

use super::{Ogf, Opcode, RawHciCommand, StatusCodeReturnParameters};

const OPCODE: Opcode = Opcode::new(Ogf::LE_CONTROLLER, 0x000C);

#[derive(Debug)]
pub struct LeSetScanEnable {
    pub le_scan_enable: u8,
    pub filter_duplicates: u8,
}

impl HciCommand for LeSetScanEnable {
    fn match_opcode(opcode: Opcode) -> bool {
        opcode == OPCODE
    }

    fn raw(self) -> RawHciCommand {
        RawHciCommand {
            opcode: OPCODE,
            parameters: RawParameters::new(&[
                self.le_scan_enable,
                self.filter_duplicates,
            ])
        }
    }
}

impl CommandWithCompleteEvent for LeSetScanEnable {
    type ReturnParameters = StatusCodeReturnParameters;
}
