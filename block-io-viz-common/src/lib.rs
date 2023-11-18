#![no_std]

#[repr(C)]
pub struct BlockIOEvent {
    // It's not clear if these are the right integer sizes for non 64 bit archs
    pub sector: u64,
    pub nr_sector: u32,
    pub rwbs: [u8; 8],
}
