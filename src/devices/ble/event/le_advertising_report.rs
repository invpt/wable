use crate::devices::ble::{
    data::{Buffer, DecodeError, MaybeDecode, MaybeDecoder},
    ParseError,
};

use super::{EventParameters, EventCode};

pub struct LeAdvertisingReport {
    num_reports: u8,
    data: Buffer<253>,
}

impl MaybeDecode for LeAdvertisingReport {
    fn maybe_decode<D>(d: &mut D) -> Result<Option<Self>, DecodeError>
    where
        D: MaybeDecoder + ?Sized,
    {
        let 0x02u8 = d.decode()? else { return Ok(None) };

        Ok(Some(LeAdvertisingReport {
            num_reports: d.decode()?,
            data: d.decode()?,
        }))
    }
}

impl EventParameters for LeAdvertisingReport {
    const EVENT_CODE: EventCode = EventCode(0x3E);
}

impl LeAdvertisingReport {
    pub fn items(&self) -> LeAdvertisingReportItems {
        LeAdvertisingReportItems {
            num_left: self.num_reports as usize,
            data: &self.data,
        }
    }
}

#[derive(Debug)]
pub struct LeAdvertisingReportItem {
    pub event_type: u8,
    pub address_type: u8,
    pub address: [u8; 6],
    pub data: Buffer<0x1F>,
    pub rssi: u8,
}

pub struct LeAdvertisingReportItems<'a> {
    num_left: usize,
    data: &'a [u8],
}

impl<'a> Iterator for LeAdvertisingReportItems<'a> {
    type Item = Result<LeAdvertisingReportItem, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_left == 0 {
            return None;
        }

        self.num_left -= 1;

        let [event_type, address_type, rest @ ..] = self.data else {
            return Some(Err(ParseError));
        };
        let [address0, address1, address2, address3, address4, address5, rest @ ..] = rest else {
            return Some(Err(ParseError));
        };
        let address = [
            *address0, *address1, *address2, *address3, *address4, *address5,
        ];
        let [data_length, rest @ ..] = rest else {
            return Some(Err(ParseError));
        };
        if rest.len() < *data_length as usize {
            return Some(Err(ParseError));
        }
        let data = &rest[..*data_length as usize];
        let rest = &rest[*data_length as usize..];
        let [rssi] = rest else {
            return Some(Err(ParseError));
        };

        Some(Ok(LeAdvertisingReportItem {
            event_type: *event_type,
            address_type: *address_type,
            address,
            data: Buffer::from(data),
            rssi: *rssi,
        }))
    }
}
