use crate::devices::ble::{command::Opcode, EventCode, HciCommand, ParseError, QueueSlot, RawHciEvent, RawParameters};

use super::HciEvent;

pub trait CommandWithCompleteEvent: HciCommand {
    type ReturnParameters: ReturnParameters;
}

pub trait ReturnParameters: Sized {
    fn parse(raw: RawParameters) -> Result<Self, ParseError>;
}

#[derive(Debug)]
pub struct CommandComplete<C: CommandWithCompleteEvent> {
    pub num_hci_command_packets: u8,
    pub command_opcode: Opcode,
    pub return_parameters: C::ReturnParameters,
    pub qslot: QueueSlot,
}

impl<C, R> HciEvent for CommandComplete<C>
where
    C: CommandWithCompleteEvent<ReturnParameters = R>,
    R: ReturnParameters,
{
    fn match_parse(raw: &RawHciEvent) -> Result<Option<Self>, ParseError> {
        if raw.code != EventCode(0x0E) {
            return Ok(None)
        }

        let [num_hci_command_packets, command_opcode_0, command_opcode_1, parameters @ ..] =
            &*raw.parameters
        else {
            return Err(ParseError);
        };

        let command_opcode = Opcode(u16::from_le_bytes([*command_opcode_0, *command_opcode_1]));

        if !C::match_opcode(command_opcode) {
            return Ok(None)
        }

        Ok(Some(CommandComplete {
            num_hci_command_packets: *num_hci_command_packets,
            command_opcode,
            return_parameters: C::ReturnParameters::parse(RawParameters::new(parameters))?,
            qslot: QueueSlot(()),
        }))
    }
}
