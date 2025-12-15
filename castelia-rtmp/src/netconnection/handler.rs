use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::Sender;

use crate::{
    messages::{protocol_control::ProtocolControlMessage, user_control::UserControlMessage},
    netconnection::{NetConnection, command::NetConnectionCommand},
    rtmp::{RTMPConnectionState, SendQueueMessage},
};

impl NetConnection {
    #[allow(clippy::unwrap_used)]
    pub async fn handle_protocol_message(
        &mut self,
        message: ProtocolControlMessage,
        connection_state: Arc<Mutex<RTMPConnectionState>>,
    ) {
        match message {
            ProtocolControlMessage::SetChunkSize(size) => {
                connection_state.lock().unwrap().max_chunk_size = size;
            }
            ProtocolControlMessage::Abort(chunk_stream_id) => {
                // figure out how to mutate chunk stream from here.
                connection_state
                    .lock()
                    .unwrap()
                    .abort_queue
                    .push(chunk_stream_id);
            }
            ProtocolControlMessage::Ack(sequence_number) => {
                connection_state.lock().unwrap().ack_seq_num = sequence_number;
            }
            ProtocolControlMessage::AckWindowSize(window_size) => {
                connection_state.lock().unwrap().ack_window_size = window_size;
            }
            ProtocolControlMessage::SetPeerBandwidth(peer_bandwidth) => {
                connection_state.lock().unwrap().peer_bandwidth = peer_bandwidth;
            }
        }
    }

    pub async fn handle_user_control_message(
        &mut self,
        message: UserControlMessage,
        send_queue: Sender<SendQueueMessage>,
    ) {
    }

    pub async fn handle_command<'a>(
        &mut self,
        command: NetConnectionCommand<'a>,
        send_queue: Sender<SendQueueMessage>,
    ) {
    }
}
