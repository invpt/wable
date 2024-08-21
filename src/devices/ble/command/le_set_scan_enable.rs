use crate::devices::ble::{CompleteCommand, HciCommand, Ogf, Opcode, RawHciCommand, RawParameters};

use super::StatusCodeReturnParameters;

const OPCODE: Opcode = Opcode::new(Ogf::LE_CONTROLLER, 0x000C);

#[derive(Debug)]
pub struct LeSetScanEnable {
    pub le_scan_enable: u8,
    pub filter_duplicates: u8,
}

impl HciCommand for LeSetScanEnable {
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

impl CompleteCommand for LeSetScanEnable {
    type ReturnParameters = StatusCodeReturnParameters;

    fn match_opcode(opcode: Opcode) -> bool {
        opcode == OPCODE
    }
}
