use crate::devices::ble::{
    command::{AnyCommand, MatchOpcode},
    data::{opcode::Opcode, Decode, DecodeError, MaybeDecode, MaybeDecoder},
    private::Internal,
    CommandReceiptIndicator, EventCode,
};

use super::EventParameters;

pub trait CommandWithCompleteEvent: MatchOpcode {
    type ReturnParameters: Decode;
}

impl CommandWithCompleteEvent for AnyCommand {
    type ReturnParameters = ();
}

impl<C: CommandWithCompleteEvent> Internal for CommandComplete<C> {}
impl<C: CommandWithCompleteEvent> CommandReceiptIndicator<C> for CommandComplete<C> {}

#[derive(Debug)]
pub struct CommandComplete<C: CommandWithCompleteEvent> {
    pub num_hci_command_packets: u8,
    pub command_opcode: Opcode,
    pub return_parameters: C::ReturnParameters,
}

impl<C, R> MaybeDecode for CommandComplete<C>
where
    C: CommandWithCompleteEvent<ReturnParameters = R>,
    R: Decode,
{
    fn maybe_decode<D>(d: &mut D) -> Result<Option<Self>, DecodeError>
    where
        D: MaybeDecoder + ?Sized,
    {
        let num_hci_command_packets = d.decode()?;
        let command_opcode = d.decode()?;
        if !C::match_opcode(command_opcode) {
            return Ok(None);
        }

        Ok(Some(CommandComplete {
            num_hci_command_packets,
            command_opcode,
            return_parameters: d.decode()?,
        }))
    }
}

impl<C: CommandWithCompleteEvent> EventParameters for CommandComplete<C> {
    const EVENT_CODE: EventCode = EventCode(0x0E);
}
