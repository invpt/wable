use super::data::{opcode::Opcode, Buffer, Encode, Encoder, EncoderFull};

pub mod le_create_connection;
pub mod le_set_scan_enable;
pub mod le_set_scan_parameters;
pub mod reset;
pub mod set_event_mask;

pub struct AnyCommand;

pub trait CommandParameters: Encode {
    const OPCODE: Opcode;
}

pub struct EncodedCommand {
    pub opcode: Opcode,
    pub parameters: Buffer<255>,
}

impl EncodedCommand {
    pub fn encode<C: Encode + HasOpcode>(command: C) -> Result<EncodedCommand, EncoderFull> {
        let opcode = command.opcode();
        let mut parameters = Buffer::new();
        command.encode(&mut parameters)?;
        Ok(EncodedCommand { opcode, parameters })
    }
}

pub trait HasOpcode {
    fn opcode(&self) -> Opcode;
}

impl<C: CommandParameters + ?Sized> HasOpcode for C {
    fn opcode(&self) -> Opcode {
        C::OPCODE
    }
}

impl HasOpcode for EncodedCommand {
    fn opcode(&self) -> Opcode {
        self.opcode
    }
}

pub trait MatchOpcode {
    fn match_opcode(opcode: Opcode) -> bool;
}

impl<C: CommandParameters + ?Sized> MatchOpcode for C {
    fn match_opcode(opcode: Opcode) -> bool {
        opcode == Self::OPCODE
    }
}

impl MatchOpcode for AnyCommand {
    fn match_opcode(_opcode: Opcode) -> bool {
        true
    }
}
