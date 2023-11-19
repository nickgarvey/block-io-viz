use aya::programs::trace_point::TracePointLinkId;
use aya::programs::ProgramError;
use aya::programs::TracePoint;
use aya::BpfError;
use aya::{include_bytes_aligned, maps::RingBuf, Bpf};
use log::debug;
use std::ptr;
use tokio::io::unix::AsyncFd;

const PROG_NAME: &str = "block_io_viz";

pub fn load_bpf_obj() -> Result<Bpf, BpfError> {
    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    return Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/block-io-viz"
    ));
    #[cfg(not(debug_assertions))]
    return Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/block-io-viz"
    ));
}

pub fn program_load_and_attach(bpf: &mut Bpf) -> Result<TracePointLinkId, ProgramError> {
    let program: &mut TracePoint = bpf.program_mut(PROG_NAME).unwrap().try_into()?;
    program.load()?;
    program.attach("block", "block_rq_complete")
}

pub fn program_detach(bpf: &mut Bpf, trace_id: TracePointLinkId) -> Result<(), ProgramError> {
    let program: &mut TracePoint = bpf.program_mut(PROG_NAME).unwrap().try_into()?;
    program.detach(trace_id)
}

pub async fn do_bpf_poll_loop<T>(
    bpf: &mut Bpf,
    handle_fn: &impl Fn(&T) -> Result<(), anyhow::Error>,
) -> Result<(), anyhow::Error> {
    let ring_buf = RingBuf::try_from(bpf.map_mut("RING_BUF").unwrap())?;
    let mut async_fd = AsyncFd::new(ring_buf)?;

    loop {
        let mut guard = async_fd.readable_mut().await?;
        let inner = guard.get_inner_mut();
        while let Some(event) = inner.next() {
            debug!("event: {:?}", event);
            let event = unsafe { ptr::read_unaligned::<T>(event.as_ptr() as *const T) };
            handle_fn(&event)?;
        }
        guard.clear_ready();
    }
}
