use log::{debug, error, info, warn};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::{error::RecvError, Receiver, Sender},
};
pub struct WebServer {
    listener: TcpListener,
    tx: Sender<Message>,
}
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

impl WebServer {
    pub async fn new(addr: &str, tx: Sender<Message>) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { listener, tx })
    }

    async fn handle_connection(
        socket: TcpStream,
        mut rx: Receiver<Message>,
        addr: std::net::SocketAddr,
    ) -> Result<(), anyhow::Error> {
        debug!("Spawned task for {:?}", addr);

        let mut ws_stream = tokio_tungstenite::accept_async(socket).await?;

        loop {
            // Ignore errors
            let event = rx.recv().await;
            match event {
                Ok(event) => {
                    info!("Sending to {:?}: {:?}", addr, event);
                    ws_stream.send(event).await?;
                }
                Err(RecvError::Lagged(_)) => {
                    warn!("Lagging, skipping oldest value");
                    continue;
                }
                Err(RecvError::Closed) => {
                    error!("Sender closed, exiting");
                    return Ok(());
                }
            }
        }
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        loop {
            let (socket, addr) = match self.listener.accept().await {
                Ok((socket, _v)) => (socket, _v),
                Err(e) => {
                    error!("Error accepting connection: {:?}", e);
                    break;
                }
            };
            debug!("Accepted connection from {:?}", addr);
            let rx = self.tx.subscribe();
            tokio::spawn(async move {
                debug!("In async");
                Self::handle_connection(socket, rx, addr).await
            });
        }

        Ok(())
    }
}
