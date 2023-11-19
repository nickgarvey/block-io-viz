#![cfg_attr(not(feature = "user"), no_std)]
#[cfg(feature = "user")]
use serde::Serialize;

#[cfg_attr(feature = "user", derive(Serialize))]
#[derive(Debug, Clone)]
#[repr(C)]
pub struct BlockIOEvent {
    // It's not clear if these are the right integer sizes for non 64 bit archs
    pub sector: u64,
    pub nr_sector: u32,
    pub rwbs: [u8; 8],
}
