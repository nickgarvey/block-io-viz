use std::collections::HashSet;
use std::net::SocketAddr;

use block_io_viz_common::BlockIOEvent;
use caps::{CapSet, Capability};
use log::{debug, error, info};
use tokio_tungstenite::tungstenite::Message;

use tokio::signal;
use tokio::sync::broadcast::{self, Sender};

use crate::static_webserver::InitializationData;

mod bpf;
mod static_webserver;
mod websocket_server;

const WEBSOCKET_PORT: u16 = 2828;
const WEBSERVER_PORT: u16 = 80;
const BLOCK_DEV_SECTORS: u64 = 68719476736 / 512;
const BLOCK_DEV_NAME: &str = "/dev/sda";
const BLOCK_DEV_MAJOR: u32 = 8;
const BLOCK_DEV_MINOR: u32 = 0;

fn is_correct_block_device(event: &BlockIOEvent) -> bool {
    if event.dev_t >> 20 != BLOCK_DEV_MAJOR {
        false
    } else if event.dev_t & 0x000FFFFF != BLOCK_DEV_MINOR {
        false
    } else {
        true
    }
}

fn push_event(tx: &Sender<Message>, event: BlockIOEvent) -> Result<(), anyhow::Error> {
    if !is_correct_block_device(&event) {
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

    let websocket_addr: SocketAddr = format!("0.0.0.0:{:?}", WEBSOCKET_PORT).parse()?;
    let webserver = websocket_server::WebSocketServer::new(&websocket_addr, tx.clone())
        .await
        .expect("Can't listen");

    let push_event = |event: BlockIOEvent| push_event(&tx, event);

    let static_addr: SocketAddr = format!("0.0.0.0:{:?}", WEBSERVER_PORT).parse()?;
    tokio::select! {
        Err(e) = bpf::do_bpf_poll_loop(&mut bpf, &push_event) => {
            error!("error in poll loop: {:?}", e);
        },
        Err(e) = webserver.run() => {
            error!("error in webserver: {:?}", e);
        },
        Err(e) = static_webserver::bind_and_serve(static_addr, InitializationData{
            name: BLOCK_DEV_NAME.to_string(),
            size_sectors: BLOCK_DEV_SECTORS,
            websocket_port: WEBSOCKET_PORT,
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
