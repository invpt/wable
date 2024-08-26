use core::marker::PhantomData;

use crate::devices::ble::{
    command::{AnyCommand, MatchOpcode},
    data::{opcode::Opcode, DecodeError, MaybeDecode, MaybeDecoder},
    private::Internal,
    CommandReceiptIndicator,
};

use super::{EventCode, EventParameters};

pub trait CommandWithStatusEvent: MatchOpcode {}

impl CommandWithStatusEvent for AnyCommand {}

impl<C> Internal for CommandStatus<C> {}
impl<C: CommandWithStatusEvent> CommandReceiptIndicator<C> for CommandStatus<C> {}

#[derive(Debug)]
pub struct CommandStatus<C> {
    _phantom: PhantomData<C>,
    pub status: u8,
    pub num_hci_command_packets: u8,
    pub command_opcode: Opcode,
}

impl<C> MaybeDecode for CommandStatus<C>
where
    C: CommandWithStatusEvent,
{
    fn maybe_decode<D>(d: &mut D) -> Result<Option<Self>, DecodeError>
    where
        D: MaybeDecoder + ?Sized,
    {
        let status = d.decode()?;
        let num_hci_command_packets = d.decode()?;
        let command_opcode = d.decode()?;
        if !C::match_opcode(command_opcode) {
            return Ok(None);
        }

        Ok(Some(Self {
            _phantom: PhantomData,
            status,
            num_hci_command_packets,
            command_opcode,
        }))
    }
}

impl<C: CommandWithStatusEvent> EventParameters for CommandStatus<C> {
    const EVENT_CODE: EventCode = EventCode(0x0F);
}
