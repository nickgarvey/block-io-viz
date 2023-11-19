use block_io_viz_common::BlockIOEvent;
use log::{error, info};
use std::ffi::CStr;
use tokio::signal;

mod bpf;

fn print_event(event: &BlockIOEvent) -> Result<(), anyhow::Error> {
    info!("sector: {:?}", event.sector);
    info!("nr_sector: {:?}", event.nr_sector);
    let rwbs_str = CStr::from_bytes_until_nul(&event.rwbs).unwrap();
    info!("rwbs: {:?}", rwbs_str);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut bpf = bpf::load_bpf_obj()?;
    let trace_id = bpf::program_load_and_attach(&mut bpf)?;

    tokio::select! {
        Err(e) = bpf::do_bpf_poll_loop(&mut bpf, &print_event)=>{
            error!("error in poll loop: {:?}", e);
        },
        _ = signal::ctrl_c() => {
            info!("Ctrl-c received, detaching tracepoint...");
        }
    }

    bpf::program_detach(&mut bpf, trace_id)?;

    info!("Exiting...");

    Ok(())
}
