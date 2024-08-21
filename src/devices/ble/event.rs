use core::fmt::Debug;


mod command_complete;
mod le_advertising_report;

pub use command_complete::*;
pub use le_advertising_report::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventCode(pub u8);
