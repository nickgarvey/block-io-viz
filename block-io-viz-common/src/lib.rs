#![cfg_attr(not(feature = "user"), no_std)]
#[cfg(feature = "user")]
use serde::Serialize;

#[cfg_attr(feature = "user", derive(Serialize))]
#[derive(Debug, Clone)]
#[repr(C)]
pub struct BlockIOEvent {
    pub sector: u64,
    pub nr_sector: u32,
    pub rwbs: [u8; 8],
    pub major: u32,
    pub minor: u32,
}
