#![no_std]
#![no_main]

use aya_bpf::{
    macros::{map, tracepoint},
    maps::RingBuf,
    programs::TracePointContext,
};
use block_io_viz_common::BlockIOEvent;

#[tracepoint]
pub fn block_io_viz(ctx: TracePointContext) -> i64 {
    match try_block_io_viz(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[map]
static RING_BUF: RingBuf = RingBuf::with_byte_size(1024 * 1024, 0);

// from /sys/kernel/debug/tracing/events/block/block_rq_complete/format
const DEV_T_OFFSET: usize = 8;
const SECTOR_OFFSET: usize = 16;
const NR_SECTOR_OFFSET: usize = 24;
const RWBS_OFFSET: usize = 32;
const RWBS_LEN: usize = 8;

fn try_block_io_viz(ctx: TracePointContext) -> Result<i64, i64> {
    let dev = unsafe { ctx.read_at::<u32>(DEV_T_OFFSET)? };
    let event = unsafe {
        BlockIOEvent {
            sector: ctx.read_at::<u64>(SECTOR_OFFSET)?,
            nr_sector: ctx.read_at::<u32>(NR_SECTOR_OFFSET)?,
            rwbs: ctx.read_at::<[u8; RWBS_LEN]>(RWBS_OFFSET)?,
            major: {
                // major is the upper 20 bits of dev
                dev >> 20
            },
            minor: {
                // minor is the lower 12 bits of dev
                dev & 0xfff
            },
        }
    };

    // We don't care about I/O that doesn't actually
    // read or write anything.
    if event.nr_sector == 0 {
        return Ok(0);
    }

    match { RING_BUF.reserve::<BlockIOEvent>(0) } {
        Some(mut entry) => {
            entry.write(event);
            entry.submit(0);
        }
        None => {
            return Err(0);
        }
    }
    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
