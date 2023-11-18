use aya::maps::MapData;
use aya::programs::TracePoint;
use aya::BpfError;
use aya::{include_bytes_aligned, maps::RingBuf, Bpf};
use block_io_viz_common::BlockIOEvent;
use log::{debug, error, info};
use std::ffi::CStr;
use std::ptr;
use tokio::io::unix::AsyncFd;
use tokio::signal;

fn increase_mem_limit() {
    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }
}

fn load_bpf() -> Result<Bpf, BpfError> {
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

async fn handle_ringbuf_ready<T>(
    async_fd: &mut AsyncFd<RingBuf<&mut MapData>>,
    handle_fn: impl Fn(&T) -> Result<(), anyhow::Error>,
) -> Result<(), anyhow::Error> {
    let mut guard = async_fd.readable_mut().await?;
    let inner = guard.get_inner_mut();
    while let Some(event) = inner.next() {
        debug!("event: {:?}", event);
        let event = unsafe { ptr::read_unaligned::<T>(event.as_ptr() as *const T) };
        handle_fn(&event)?;
    }
    guard.clear_ready();
    Ok(())
}

fn handle_fn(event: &BlockIOEvent) -> Result<(), anyhow::Error> {
    info!("sector: {:?}", event.sector);
    info!("nr_sector: {:?}", event.nr_sector);
    let rwbs_str = CStr::from_bytes_until_nul(&event.rwbs).unwrap();
    info!("rwbs: {:?}", rwbs_str);
    Ok(())
}

async fn do_poll_loop(bpf: &mut Bpf) -> Result<(), anyhow::Error> {
    let ring_buf = RingBuf::try_from(bpf.map_mut("RING_BUF").unwrap())?;
    let mut async_fd = AsyncFd::new(ring_buf)?;

    loop {
        tokio::select! {
            ret = handle_ringbuf_ready(&mut async_fd, handle_fn) => {
                if ret.is_err() {
                    error!("error while reading ringbuf: {:?}", ret);
                    return ret;
                }
            },
            _ = signal::ctrl_c() => {
                return Ok(());
            }
        }
    }
}

const PROG_NAME: &str = "block_io_viz";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    increase_mem_limit();

    let mut bpf = load_bpf()?;

    let trace_id = {
        let program: &mut TracePoint = bpf.program_mut(PROG_NAME).unwrap().try_into()?;
        program.load()?;
        match program.attach("block", "block_rq_complete") {
            Err(e) => {
                error!("failed to attach tracepoint: {}", e);
                return Err(e.into());
            }
            Ok(trace_id) => trace_id,
        }
    };

    if let Err(e) = do_poll_loop(&mut bpf).await {
        error!("error in poll loop: {:?}", e);
    }

    let program: &mut TracePoint = bpf.program_mut(PROG_NAME).unwrap().try_into()?;
    program.detach(trace_id)?;

    info!("Exiting...");

    Ok(())
}
