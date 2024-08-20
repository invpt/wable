use core::fmt::Debug;


mod command_complete;

pub use command_complete::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventCode(pub u8);
