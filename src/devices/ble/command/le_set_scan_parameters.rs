use crate::devices::ble::{CompleteCommand, HciCommand, Ogf, Opcode, RawHciCommand, RawParameters};

use super::StatusCodeReturnParameters;

const OPCODE: Opcode = Opcode::new(Ogf::LE_CONTROLLER, 0x000B);

#[derive(Debug)]
pub struct LeSetScanParameters {
    pub le_scan_type: u8,
    pub le_scan_interval: u16,
    pub le_scan_window: u16,
    pub own_address_type: u8,
    pub scanning_filter_policy: u8,
}

impl HciCommand for LeSetScanParameters {
    fn raw(self) -> RawHciCommand {
        let le_scan_interval = self.le_scan_interval.to_le_bytes();
        let le_scan_window = self.le_scan_window.to_le_bytes();

        RawHciCommand {
            opcode: OPCODE,
            parameters: RawParameters::new(&[
                self.le_scan_type,
                le_scan_interval[0],
                le_scan_interval[1],
                le_scan_window[0],
                le_scan_window[1],
                self.own_address_type,
                self.scanning_filter_policy,
            ]),
        }
    }
}

impl CompleteCommand for LeSetScanParameters {
    type ReturnParameters = StatusCodeReturnParameters;

    fn match_opcode(opcode: Opcode) -> bool {
        opcode == OPCODE
    }
}