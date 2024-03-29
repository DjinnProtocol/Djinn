use std::{collections::HashMap, error::Error};


use djinn_core_lib::data::packets::{ControlPacket, ControlPacketType, PacketType, DataPacket, PacketReader};
use tokio::{fs::File, io::{AsyncWriteExt}};

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

        //Send start transfer packet
        let mut packet = ControlPacket::new(ControlPacketType::TransferStart, HashMap::new());
        packet.params.insert("job_id".to_string(), job_id.clone());
        connection.send_packet(packet).await?;

        debug!("Sent transfer start");

        //Open file
        let mut file = File::create(self.file_path.clone()).await?;

        //Wait for the server to send the file
        let mut possible_reader = connection.reader.lock().await;

        if possible_reader.is_none() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Reader is none",
            )));
        }

        let reader = possible_reader.as_mut().unwrap();


        let mut packet_reader = PacketReader::new();
        let mut last_packet_received = false;

        while !last_packet_received {
            let packets = packet_reader.read(reader, None).await;

            if packets.is_empty() {
                debug!("Connection closed");
                break;
            }

            for packet in packets {
                let data_packet = packet
                    .as_any()
                    .downcast_ref::<DataPacket>()
                    .unwrap();

                if !data_packet.has_data {
                    last_packet_received = true;
                    break;
                }

                file.write_all(&data_packet.data).await?;
            }
        }

        debug!("Transfer complete");

        file.flush().await?;

        Ok("Transfer complete".to_string())
    }
}
