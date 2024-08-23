use core::{fmt::Debug, marker::PhantomData, ops::Deref};

use command::{AnyCommand, HciCommand, RawHciCommand};
use embedded_io::{Read, ReadExactError, Write};
use esp_hal::delay::Delay;
use esp_hal::prelude::*;
use event::{command_complete::CommandComplete, command_status::CommandStatus, EventCode, HciEvent, RawHciEvent};

pub mod command;
pub mod event;

pub struct Ble<H> {
    /// The most recent num_hci_command_packets value received from the controller, decremented
    /// whenever a command is sent. If this field is 0, no commands can be sent.
    num_hci_command_packets: usize,
    queued_command: Option<RawHciCommand>,
    hci: H,
    delay: Delay,
}

pub type RawParameters = BoundedBytes<255>;

#[derive(Debug, Clone)]
pub struct BoundedBytes<const N: usize> {
    len: u8,
    data: [u8; N],
}

impl<const N: usize> BoundedBytes<N> {
    pub fn new(data: &[u8]) -> BoundedBytes<N> {
        assert!(data.len() < N, "data length cannot exceed {}", N - 1);

        let len = data.len() as u8;
        let mut data_arr = [0; N];
        data_arr[..data.len()].copy_from_slice(data);

        BoundedBytes {
            len,
            data: data_arr,
        }
    }
}

impl<const N: usize> Deref for BoundedBytes<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len as usize]
    }
}

#[derive(Debug)]
pub enum BleError<E> {
    WouldBlock,
    UnexpectedEof,
    ParseError,
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

impl<E> From<ParseError> for BleError<E> {
    fn from(_value: ParseError) -> Self {
        Self::ParseError
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

mod private {
    pub trait CommandReceiptIndicator<C> {}
}

impl<C> QueueLock<C> {
    pub fn release_with<E: private::CommandReceiptIndicator<C>>(self, _event: &E) -> QueueSlot {
        self.qslot
    }
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
                hci,
                delay,
            },
            QueueSlot,
        )
    }

    pub fn queue<C: HciCommand>(
        &mut self,
        qslot: QueueSlot,
        command: C,
    ) -> Result<QueueLock<C>, BleError<E>> {
        if self.num_hci_command_packets > 0 {
            self.try_issue(command.raw())?
        } else {
            let None = self.queued_command else {
                panic!("Invalid state: Queue is full")
            };

            self.queued_command = Some(command.raw());
        }

        Ok(QueueLock {
            _phantom: PhantomData,
            qslot,
        })
    }

    /// Tries to issue `command`, returning `Err(BleError::WouldBlock)` if the controller currently
    /// cannot accept more commands.
    pub fn try_issue<C: HciCommand>(&mut self, command: C) -> Result<(), BleError<E>> {
        if self.num_hci_command_packets == 0 {
            return Err(BleError::WouldBlock);
        }

        let raw = command.raw();

        self.hci.write_all(&[0x01])?;
        self.hci.write_all(&raw.opcode.0.to_le_bytes())?;
        self.hci.write_all(&[raw.parameters.len])?;
        self.hci.write_all(&raw.parameters)?;
        self.hci.flush()?;

        Ok(())
    }

    pub fn poll(&mut self) -> Result<RawHciEvent, BleError<E>> {
        loop {
            match self.try_poll() {
                Ok(ev) => return Ok(ev),
                Err(BleError::WouldBlock) => {
                    self.delay.delay(10.millis());
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn try_poll(&mut self) -> Result<RawHciEvent, BleError<E>> {
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

        let raw = RawHciEvent {
            code: EventCode(event_code),
            parameters: RawParameters::new(event_parameters),
        };

        if let Some(event) = CommandComplete::<AnyCommand>::match_parse(&raw)? {
            self.num_hci_command_packets = event.num_hci_command_packets as usize;
        } else if let Some(event) = CommandStatus::<AnyCommand>::match_parse(&raw)? {
            self.num_hci_command_packets = event.num_hci_command_packets as usize;
        }

        if self.num_hci_command_packets > 0 {
            if let Some(queued_command) = self.queued_command.take() {
                self.try_issue(queued_command)?
            }
        }

        Ok(raw)
    }
}
