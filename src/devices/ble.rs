use core::{fmt::Debug, marker::PhantomData};

use command::{AnyCommand, CommandParameters, EncodedCommand, HasOpcode};
use data::{Buffer, DecodeError, Encode, EncoderFull};
use embedded_io::{Read, ReadExactError, Write};
use esp_hal::delay::Delay;
use esp_hal::prelude::*;
use event::{command_complete::{CommandComplete, CommandWithCompleteEvent}, command_status::CommandStatus, EncodedEvent, EventCode, EventParameters};

pub mod command;
pub mod data;
pub mod event;

mod private {
    pub trait Internal {}
}

use private::Internal;

pub struct Ble<H> {
    /// The most recent num_hci_command_packets value received from the controller, decremented
    /// whenever a command is sent. If this field is 0, no commands can be sent.
    num_hci_command_packets: usize,
    queued_command: Option<EncodedCommand>,
    queued_event: Option<EncodedEvent>,
    hci: H,
    delay: Delay,
}

#[derive(Debug)]
pub enum BleError<E> {
    WouldBlock,
    UnexpectedEof,
    UnexpectedEvent,
    Encode(EncoderFull),
    Decode(DecodeError),
    Io(E),
}

impl<E> From<E> for BleError<E>
where
    E: embedded_io::Error,
{
    fn from(value: E) -> Self {
        Self::Io(value)
    }
}

impl<E> From<embedded_io::ReadExactError<E>> for BleError<E>
where
    E: embedded_io::Error,
{
    fn from(value: embedded_io::ReadExactError<E>) -> Self {
        match value {
            embedded_io::ReadExactError::UnexpectedEof => Self::UnexpectedEof,
            embedded_io::ReadExactError::Other(e) => Self::Io(e),
        }
    }
}

impl<E> From<EncoderFull> for BleError<E> {
    fn from(value: EncoderFull) -> Self {
        Self::Encode(value)
    }
}

impl<E> From<DecodeError> for BleError<E> {
    fn from(value: DecodeError) -> Self {
        Self::Decode(value)
    }
}

#[derive(Debug)]
pub struct ParseError;

#[derive(Debug)]
#[non_exhaustive]
pub struct QueueSlot;

#[derive(Debug)]
pub struct QueueLock<C> {
    _phantom: PhantomData<C>,
    qslot: QueueSlot,
}

pub trait CommandReceiptIndicator<C>: Internal {}

impl<C> QueueLock<C> {
    pub fn release_with<E: CommandReceiptIndicator<C>>(self, _event: &E) -> QueueSlot {
        self.qslot
    }
}

pub enum PollBehavior {
    Strict,
    Filter,
}

impl<E, H> Ble<H>
where
    H: Read<Error = E> + Write<Error = E>,
    E: embedded_io::Error,
{
    pub fn new(hci: H, delay: Delay) -> (Self, QueueSlot) {
        (
            Self {
                num_hci_command_packets: 1,
                queued_command: None,
                queued_event: None,
                hci,
                delay,
            },
            QueueSlot,
        )
    }

    pub fn run_until_complete<C: CommandParameters + CommandWithCompleteEvent>(
        &mut self,
        qslot: QueueSlot,
        poll_behavior: PollBehavior,
        command: C,
    ) -> Result<(C::ReturnParameters, QueueSlot), BleError<E>> {
        let qlock = self.queue(qslot, command)?;
        
        loop {
            if let Some(complete) = self.maybe_poll::<CommandComplete<C>>()? {
                let slot = qlock.release_with(&complete);
                return Ok((complete.return_parameters, slot))
            } else {
                match poll_behavior {
                    PollBehavior::Filter => self.queued_event = None,
                    PollBehavior::Strict => return Err(BleError::UnexpectedEvent),
                }
            }
        }
    }

    /// Queues `command`, turning `qslot` into a [QueueLock]. To queue more commands, poll for either a
    /// [CommandComplete](event::command_complete::CommandComplete) or a
    /// [CommandStatus](event::command_status::CommandStatus) event and call [QueueLock::release_with()].
    pub fn queue<C: CommandParameters>(
        &mut self,
        qslot: QueueSlot,
        command: C,
    ) -> Result<QueueLock<C>, BleError<E>> {
        let encoded = EncodedCommand::encode(command)?;

        if self.num_hci_command_packets > 0 {
            self.try_issue_raw(encoded)?
        } else {
            if self.queued_command.is_some() {
                panic!("Invalid state: Queue is full")
            }

            self.queued_command = Some(encoded);
        }

        Ok(QueueLock {
            _phantom: PhantomData,
            qslot,
        })
    }

    /// Tries to issue `command`, returning `Err(BleError::WouldBlock)` if the controller currently
    /// cannot accept more commands.
    pub fn try_issue<C: Encode + HasOpcode>(&mut self, command: C) -> Result<(), BleError<E>> {
        if self.num_hci_command_packets == 0 {
            return Err(BleError::WouldBlock);
        }

        self.try_issue_raw(EncodedCommand::encode(command)?)
    }

    /// Tries to issue `command`, returning `Err(BleError::WouldBlock)` if the controller currently
    /// cannot accept more commands.
    pub fn try_issue_raw(&mut self, command: EncodedCommand) -> Result<(), BleError<E>> {
        if self.num_hci_command_packets == 0 {
            return Err(BleError::WouldBlock);
        }

        self.hci.write_all(&[0x01])?;
        self.hci.write_all(&command.opcode().0.to_le_bytes())?;
        self.hci.write_all(&[command.parameters.len() as u8])?;
        self.hci.write_all(&command.parameters)?;
        self.hci.flush()?;

        Ok(())
    }

    /// Polls for events, decoding as the event type `Ev` and ignoring any that don't match.
    pub fn filter_poll<Ev: EventParameters>(&mut self) -> Result<Option<Ev>, BleError<E>> {
        Ok(self.poll_raw()?.decode()?)
    }

    /// Polls for events, decoding as the event type `Ev` and leaving unmatched events unprocessed.
    pub fn maybe_poll<Ev: EventParameters>(&mut self) -> Result<Option<Ev>, BleError<E>> {
        let encoded = self.poll_raw()?;

        let decoded = encoded.decode::<Ev>()?;

        if let Some(decoded) = decoded {
            Ok(Some(decoded))
        } else {
            self.queued_event = Some(encoded);
            Ok(None)
        }
    }

    pub fn poll_raw(&mut self) -> Result<EncodedEvent, BleError<E>> {
        if let Some(encoded) = self.queued_event.take() {
            return Ok(encoded)
        }

        loop {
            match self.try_poll_raw() {
                Ok(ev) => return Ok(ev),
                Err(BleError::WouldBlock) => {
                    self.delay.delay(10.millis());
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn try_poll_raw(&mut self) -> Result<EncodedEvent, BleError<E>> {
        loop {
            let mut packet_type_buf = [0; 1];
            match self.hci.read_exact(&mut packet_type_buf) {
                Ok(()) => (),
                Err(ReadExactError::UnexpectedEof) => return Err(BleError::WouldBlock),
                Err(ReadExactError::Other(e)) => return Err(BleError::Io(e)),
            }
            let packet_type = packet_type_buf[0];
            assert_eq!(packet_type, 4);

            let mut header_buf = [0; 2];
            self.hci.read_exact(&mut header_buf)?;
            let [event_code, parameter_length] = header_buf;

            let mut event_parameters_buf = [0; 255];
            let event_parameters = &mut event_parameters_buf[..parameter_length as usize];
            self.hci.read_exact(event_parameters)?;

            let encoded = EncodedEvent {
                code: EventCode(event_code),
                parameters: Buffer::from(&*event_parameters),
            };

            if let Some(event) = encoded.decode::<CommandComplete<AnyCommand>>()? {
                self.num_hci_command_packets = event.num_hci_command_packets as usize;
            } else if let Some(event) = encoded.decode::<CommandStatus<AnyCommand>>()? {
                self.num_hci_command_packets = event.num_hci_command_packets as usize;
            }

            if self.num_hci_command_packets > 0 {
                if let Some(queued_command) = self.queued_command.take() {
                    self.try_issue_raw(queued_command)?
                }
            }

            return Ok(encoded)
        }
    }
}
