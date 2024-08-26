use crate::devices::ble::{
    data::opcode::{Ogf, Opcode},
    data::{address::Address, Encode, Encoder, EncoderFull},
    event::command_status::CommandWithStatusEvent,
};

use super::CommandParameters;

pub struct LeCreateConnection {
    pub le_scan_interval: u16,
    pub le_scan_window: u16,
    pub initiator_filter_policy: u8,
    pub peer_address_type: u8,
    pub peer_address: Address,
    pub own_address_type: u8,
    pub connection_interval_min: u16,
    pub connection_interval_max: u16,
    pub max_latency: u16,
    pub supervision_timeout: u16,
    pub min_ce_length: u16,
    pub max_ce_length: u16,
}

impl Encode for LeCreateConnection {
    fn encode<E>(&self, e: &mut E) -> Result<(), EncoderFull>
    where
        E: Encoder + ?Sized,
    {
        e.encode(&self.le_scan_interval)?;
        e.encode(&self.le_scan_window)?;
        e.encode(&self.initiator_filter_policy)?;
        e.encode(&self.peer_address_type)?;
        e.encode(&self.peer_address)?;
        e.encode(&self.own_address_type)?;
        e.encode(&self.connection_interval_min)?;
        e.encode(&self.connection_interval_max)?;
        e.encode(&self.max_latency)?;
        e.encode(&self.supervision_timeout)?;
        e.encode(&self.min_ce_length)?;
        e.encode(&self.max_ce_length)?;

        Ok(())
    }
}

impl CommandParameters for LeCreateConnection {
    const OPCODE: Opcode = Opcode::new(Ogf::LE_CONTROLLER, 0x000D);
}

impl CommandWithStatusEvent for LeCreateConnection {}
