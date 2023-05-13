use std::{collections::HashMap, error::Error};

use djinn_core_lib::data::packets::{ControlPacket, ControlPacketType, PacketType};
use tokio::fs::File;

use crate::connectivity::Connection;

pub struct PutCommand {
    file_path: String,
}

impl PutCommand {
    pub fn new(file_path: String) -> Self {
        PutCommand { file_path }
    }

    pub async fn execute(&self, connection: &mut Connection) -> Result<String, Box<dyn Error>> {
        //Ask for the file from the server
        debug!("Sending transfer request");
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), self.file_path.clone());
        params.insert("type".to_string(), "put".to_string());


        let packet = ControlPacket::new(ControlPacketType::TransferRequest, params);
        connection.send_packet(packet).await?;

        debug!("Sent transfer request");

        //Wait for the server to acknowledge the request
        let response_packet = connection.read_next_packet().await?.unwrap();

        debug!("Received transfer ack");

        if !matches!(response_packet.get_packet_type(), PacketType::Control) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected packet type",
            )));
        }

        let control_packet = response_packet
            .as_any()
            .downcast_ref::<ControlPacket>()
            .unwrap();

        // If the server denies the request, return an error
        if matches!(
            control_packet.control_packet_type,
            ControlPacketType::TransferDeny
        ) {
            let reason = control_packet.params.get("reason").unwrap();
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Transfer denied: {}", reason),
            )));
        }

        if !matches!(
            control_packet.control_packet_type,
            ControlPacketType::TransferAck
        ) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected control packet type",
            )));
        }

        // If the server accepts the request, start receiving the file
        debug!("Transfer accepted, starting transfer");

        let job_id = control_packet.params.get("job_id").unwrap();
        let mut file = File::create(self.file_path.clone()).await?;

        //Send file parts



        Ok("Transfer complete".to_string())
    }
}
