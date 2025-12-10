use std::io;

use tokio::net::{TcpListener, TcpStream};
use tracing::{error, trace};

use crate::handshake::handshake;

pub struct RTMPSever {
    listener: TcpListener,
}

impl RTMPSever {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub async fn run(&self) -> io::Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            trace!("Accepted connection from {addr}");

            tokio::spawn(async move {
                if let Err(e) = process(socket).await {
                    error!("Failed to process rtmp connection: {e}");
                }
            });
        }
    }
}

async fn process(mut socket: TcpStream) -> io::Result<()> {
    handshake(&mut socket).await?;

    Ok(())
}
