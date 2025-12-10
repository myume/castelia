use tokio::{io::BufReader, net::TcpStream};

pub enum RTMPMessage {}

impl RTMPMessage {
    pub async fn read_message(reader: &mut BufReader<&mut TcpStream>) -> Self {
        todo!()
    }
}
