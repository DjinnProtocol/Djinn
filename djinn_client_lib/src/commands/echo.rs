use std::{collections::HashMap, error::Error};

use djinn_core_lib::data::packets::{ControlPacket, ControlPacketType, PacketType};

use crate::connectivity::Connection;

pub struct EchoCommand {}

impl EchoCommand {
    pub fn new() -> Self {
        EchoCommand {}
    }

    pub async fn execute(&self, connection: &mut Connection) -> Result<(), Box<dyn Error>> {
        // Create packet
        let packet = ControlPacket::new(ControlPacketType::EchoRequest, HashMap::new());

        // Send packet
        let start = std::time::Instant::now();
        connection.send_packet(packet).await?;
        connection.flush().await?;

        // Wait for response
        let response_packet = connection.read_next_packet().await?.unwrap();

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

        if !matches!(
            control_packet.control_packet_type,
            ControlPacketType::EchoReply
        ) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected control packet type",
            )));
        }

        let end = std::time::Instant::now();

        println!(
            "Server responded in {}ms",
            end.duration_since(start).as_millis()
        );

        Ok(())
    }
}
