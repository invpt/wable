use crate::devices::ble::{BoundedBytes, HciEvent, ParseError, RawHciEvent, RawParameters};

use super::EventCode;

pub struct LeAdvertisingReport {
    num_reports: u8,
    data: BoundedBytes<253>,
}

impl HciEvent for LeAdvertisingReport {
    fn match_parse(raw: &RawHciEvent) -> Result<Option<Self>, ParseError> {
        if raw.code != EventCode(0x3E) || raw.parameters.len() < 1 {
            return Ok(None);
        }

        let [_subevent_code @ 0x02, rest @ ..] = &*raw.parameters else {
            return Ok(None);
        };

        let [num_reports, rest @ ..] = rest else {
            return Err(ParseError);
        };

        Ok(Some(LeAdvertisingReport {
            num_reports: *num_reports,
            data: BoundedBytes::new(rest),
        }))
    }
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
    pub data: BoundedBytes<50>,
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
            data: BoundedBytes::new(data),
            rssi: *rssi,
        }))
    }
}
