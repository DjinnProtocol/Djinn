use std::{collections::HashMap, error::Error};

use djinn_core_lib::data::packets::{packet::Packet, ControlPacket, ControlPacketType, PacketType, DataPacket};

use crate::connectivity::Connection;

pub struct GetCommand {
    file_path: String,
}

impl GetCommand {
    pub fn new(file_path: String) -> Self {
        GetCommand { file_path }
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

        //Send start transfer packet
        let mut packet = ControlPacket::new(ControlPacketType::TransferStart, HashMap::new());
        packet.params.insert("job_id".to_string(), job_id.clone());
        connection.send_packet(packet).await?;

        debug!("Sent transfer start");

        //Wait for the server to send the file
        while let Ok(packet) = connection.read_next_packet().await {
            if !matches!(packet.get_packet_type(), PacketType::Data) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unexpected packet type",
                )));
            }

            let data_packet = packet.as_any().downcast_ref::<DataPacket>().unwrap();

            if data_packet.job_id != job_id.parse::<u32>().unwrap() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unexpected job id",
                )));
            }

            debug!("Received data packet");
        }

        Ok("Hello".to_string())
    }
}
