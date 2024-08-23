use core::marker::PhantomData;

use crate::devices::ble::{command::{AnyCommand, Opcode}, private::CommandReceiptIndicator, EventCode, HciCommand, ParseError, RawHciEvent};

use super::HciEvent;

pub trait CommandWithStatusEvent: HciCommand {}

impl CommandWithStatusEvent for AnyCommand {}

impl<C: CommandWithStatusEvent> CommandReceiptIndicator<C> for CommandStatus<C> {}

#[derive(Debug)]
pub struct CommandStatus<C> {
    _phantom: PhantomData<C>,
    pub status: u8,
    pub num_hci_command_packets: u8,
    pub command_opcode: Opcode,
}

impl<C> HciEvent for CommandStatus<C>
where
    C: CommandWithStatusEvent,
{
    fn match_parse(raw: &RawHciEvent) -> Result<Option<Self>, ParseError> {
        if raw.code != EventCode(0x0F) {
            return Ok(None);
        }

        let [status, num_hci_command_packets, command_opcode_0, command_opcode_1] =
            &*raw.parameters
        else {
            return Err(ParseError);
        };

        let command_opcode = Opcode(u16::from_le_bytes([*command_opcode_0, *command_opcode_1]));

        if !C::match_opcode(command_opcode) {
            return Ok(None);
        }

        Ok(Some(CommandStatus {
            _phantom: PhantomData,
            status: *status,
            num_hci_command_packets: *num_hci_command_packets,
            command_opcode,
        }))
    }
}
