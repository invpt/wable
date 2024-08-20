use crate::devices::ble::{EventCode, HciEvent, Opcode, ParseError, RawHciEvent, RawParameters};

pub trait CompleteCommand {
    type ReturnParameters: ReturnParameters;
}

pub trait ReturnParameters: Sized {
    fn parse(raw: RawParameters) -> Result<Self, ParseError>;
}

pub struct CommandCompleteEvent<C: CompleteCommand> {
    pub num_hci_command_packets: u8,
    pub command_opcode: Opcode,
    pub return_parameters: C::ReturnParameters,
}

impl<C, R> HciEvent for CommandCompleteEvent<C>
where
    C: CompleteCommand<ReturnParameters = R>,
    R: ReturnParameters,
{
    fn parse(raw: RawHciEvent) -> Result<Self, ParseError> {
        const CODE: EventCode = EventCode(0x0E);
        if raw.code != CODE {
            return Err(ParseError::WrongCode);
        };

        let [num_hci_command_packets, command_opcode_0, command_opcode_1, parameters @ ..] =
            &*raw.parameters
        else {
            return Err(ParseError::BadFormat);
        };

        Ok(CommandCompleteEvent {
            num_hci_command_packets: *num_hci_command_packets,
            command_opcode: Opcode(u16::from_le_bytes([*command_opcode_0, *command_opcode_1])),
            return_parameters: C::ReturnParameters::parse(RawParameters::new(parameters))?,
        })
    }
}
