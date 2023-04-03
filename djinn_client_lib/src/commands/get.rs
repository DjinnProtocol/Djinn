use std::{collections::HashMap, error::Error};

use async_std::{fs::File, io::{WriteExt, BufReader}};
use djinn_core_lib::data::packets::{packet::Packet, ControlPacket, ControlPacketType, PacketType, DataPacket, PacketReader};

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
        params.insert("type".to_string(), "get".to_string());

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

        //Open file
        let mut file = File::create(self.file_path.clone()).await?;

        //Wait for the server to send the file
        let mut reader = BufReader::new(connection.stream.as_mut().unwrap());
        let mut packet_reader = PacketReader::new();

        loop {
            let packet = packet_reader.read(&mut reader).await;

            if packet.is_none() {
                debug!("Connection closed");
                break;
            }

            let data_packet = packet.as_ref().unwrap()
                .as_any()
                .downcast_ref::<DataPacket>()
                .unwrap();

            debug!("Reveived: {:?}", String::from_utf8(data_packet.data.clone()));

            if data_packet.data.len() == 0 {
                break;
            }

            file.write_all(&data_packet.data).await?;
        }

        debug!("Transfer complete");

        file.flush().await?;

        Ok("Transfer complete".to_string())
    }
}
