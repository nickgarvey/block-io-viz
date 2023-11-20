use block_io_viz_common::BlockIOEvent;
use log::{debug, error, info};
use tokio_tungstenite::tungstenite::Message;

use tokio::signal;
use tokio::sync::broadcast;

mod bpf;
mod static_webserver;
mod websocket_server;

const ADDR: &str = "0.0.0.0:2828";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut bpf = bpf::load_bpf_obj()?;
    let trace_id = bpf::program_load_and_attach(&mut bpf)?;

    let (tx, _) = broadcast::channel(1024);

    let webserver = websocket_server::WebSocketServer::new(&ADDR, tx.clone())
        .await
        .expect("Can't listen");

    let tx_clone = tx.clone();
    let push_event = |event: BlockIOEvent| -> Result<(), anyhow::Error> {
        let buf = rmp_serde::to_vec(&event).unwrap();
        match tx_clone.send(Message::Binary(buf)) {
            Ok(_) => Ok(()),
            Err(e) => {
                debug!("No receivers, dropping event: {:?}", e);
                Ok(())
            }
        }
    };

    tokio::select! {
        Err(e) = bpf::do_bpf_poll_loop(&mut bpf, &push_event) => {
            error!("error in poll loop: {:?}", e);
        },
        Err(e) = webserver.run() => {
            error!("error in webserver: {:?}", e);
        },
        Err(e) = static_webserver::bind_and_serve() => {
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
