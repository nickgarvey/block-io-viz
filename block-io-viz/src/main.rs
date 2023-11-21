use block::BlockDeviceInfo;
use block_io_viz_common::BlockIOEvent;
use caps::{CapSet, Capability};
use clap::Parser;
use log::{debug, error, info};
use std::collections::HashSet;
use std::net::SocketAddr;
use tokio::signal;
use tokio::sync::broadcast::{self, Sender};
use tokio_tungstenite::tungstenite::Message;

use crate::static_webserver::InitializationData;

mod block;
mod bpf;
mod static_webserver;
mod websocket_server;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    block_dev_path: String,

    #[arg(short = 'p', long, default_value_t = 80)]
    http_server_port: u16,

    #[arg(short, long, default_value_t = 23422)]
    websocket_port: u16,
}

fn push_event(
    tx: &Sender<Message>,
    traced: &BlockDeviceInfo,
    event: BlockIOEvent,
) -> Result<(), anyhow::Error> {
    debug!("push_event: {:?}", event);
    debug!("traced: {:?}", traced);
    if traced.major != event.major || traced.minor != event.minor {
        return Ok(());
    }

    let buf = rmp_serde::to_vec(&event).unwrap();
    match tx.send(Message::Binary(buf)) {
        Ok(_) => Ok(()),
        Err(e) => {
            debug!("No receivers, dropping event: {:?}", e);
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let args = Args::parse();
    let block_dev = block::block_dev_info(&args.block_dev_path).unwrap();

    info!("args: {:?}", args);

    let mut bpf = bpf::load_bpf_obj()?;
    let trace_id = bpf::program_load_and_attach(&mut bpf)?;

    // Once all the eBPF programs are loaded, drop everything
    // except what we need to bind the webservers.
    caps::set(
        None,
        CapSet::Effective,
        &HashSet::from([Capability::CAP_NET_BIND_SERVICE]),
    )?;

    let (tx, _) = broadcast::channel(1024);

    let websocket_addr: SocketAddr = format!("0.0.0.0:{:?}", args.websocket_port).parse()?;
    let webserver = websocket_server::WebSocketServer::new(&websocket_addr, tx.clone())
        .await
        .expect("Can't listen");

    let push_event = |event: BlockIOEvent| push_event(&tx, &block_dev, event);

    let static_addr: SocketAddr = format!("0.0.0.0:{:?}", args.http_server_port).parse()?;
    tokio::select! {
        Err(e) = bpf::do_bpf_poll_loop(&mut bpf, &push_event) => {
            error!("error in poll loop: {:?}", e);
        },
        Err(e) = webserver.run() => {
            error!("error in webserver: {:?}", e);
        },
        Err(e) = static_webserver::bind_and_serve(static_addr, InitializationData{
            name: block_dev.path.clone(),
            size_sectors: block_dev.size_sectors,
            websocket_port: args.websocket_port,
        }) => {
            error!("error in static webserver: {:?}", e);
        },
        _ = signal::ctrl_c() => {
            info!("Ctrl-c received, detaching tracepoint...");
        }
    }

    bpf::program_detach(&mut bpf, trace_id)?;

    info!("Exiting...");

    Ok(())
}
