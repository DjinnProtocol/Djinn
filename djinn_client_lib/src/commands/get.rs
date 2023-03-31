use std::{error::Error, collections::HashMap};

use djinn_core_lib::data::packets::{packet::Packet, ControlPacket, ControlPacketType, PacketType};

use crate::connectivity::Connection;

pub struct GetCommand {
    file_path: String
}

impl GetCommand {
    pub fn new(file_path: String) -> Self {
        GetCommand {
            file_path
        }
    }

    pub async fn execute(&self, connection: &mut Connection) -> Result<String, Box<dyn Error>> {
        //Ask for the file from the server
        debug!("Sending transfer request");
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), self.file_path.clone());
        let packet = ControlPacket::new(ControlPacketType::TransferRequest, params);
        connection.send_packet(packet).await?;

        debug!("Sent transfer request");

        //Wait for the server to acknowledge the request
        let response_packet = connection.read_next_packet().await?;

        debug!("Received transfer ack");

        Ok("Hello".to_string())
    }
}
