use std::io;

use tokio::net::{TcpListener, TcpStream};
use tracing::trace;

use crate::handshake;

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
                process(socket).await;
            });
        }
    }
}

async fn process(socket: TcpStream) {
    handshake::handshake(socket).await;
}
