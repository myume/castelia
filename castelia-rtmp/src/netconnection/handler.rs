use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::Sender;
use tracing::{error, warn};

use crate::{
    amf::AMF0Value,
    chunks::header::MessageHeader,
    messages::{
        command::command_message_type, protocol_control::ProtocolControlMessage,
        router::MessageStream, user_control::UserControlMessage,
    },
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

    async fn handle_connect<'a>(
        &self,
        command: NetConnectionCommand<'a>,
        send_queue: Sender<SendQueueMessage>,
    ) {
    }

    async fn handle_create_stream<'a>(
        &self,
        sender_stream_id: &u32,
        command: NetConnectionCommand<'a>,
        streams: &mut MessageStream,
        send_queue: Sender<SendQueueMessage>,
    ) {
        let created_stream_id = streams.create_stream();
        let response = [
            AMF0Value::String("_result"),
            AMF0Value::Number(command.transaction_id),
            AMF0Value::Null,
            AMF0Value::Number(created_stream_id.into()),
        ]
        .map(|val| val.serialize())
        .concat();

        let header = MessageHeader::Type0 {
            timestamp: 0,
            message_length: response.len() as u32,
            message_type_id: command_message_type::COMMAND_AMF0,
            message_stream_id: *sender_stream_id,
        };

        if let Err(e) = send_queue.send((header, response.into())).await {
            error!("Failed to send create stream response {e}");
            streams.delete_stream(&created_stream_id);
        };
    }

    pub async fn handle_command<'a>(
        &mut self,
        sender_stream_id: &u32,
        command: NetConnectionCommand<'a>,
        send_queue: Sender<SendQueueMessage>,
        streams: &mut MessageStream,
    ) {
        match command.command_type {
            super::command::NetConnectionCommandType::Connect => {
                self.handle_connect(command, send_queue).await;
            }
            super::command::NetConnectionCommandType::Call(procedure) => {
                warn!("call command is unsupported, attempt to call procedure {procedure}");
            }
            super::command::NetConnectionCommandType::Close => todo!(),
            super::command::NetConnectionCommandType::CreateStream => {
                self.handle_create_stream(sender_stream_id, command, streams, send_queue)
                    .await;
            }
        }
    }
}
