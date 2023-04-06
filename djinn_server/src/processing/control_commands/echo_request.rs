use std::{collections::HashMap, error::Error};

use async_std::io::WriteExt;
use async_trait::async_trait;
use djinn_core_lib::data::packets::{ControlPacket, PacketType, ControlPacketType, packet::Packet};

use crate::connectivity::Connection;

use super::ControlCommand;

pub struct EchoRequestCommand {}

#[async_trait]
impl ControlCommand for EchoRequestCommand {
    async fn execute(&self, connection: &mut Connection, _packet: &ControlPacket) -> Result<(), Box<dyn Error>> {
        let response = ControlPacket {
            packet_type: PacketType::Control,
            control_packet_type: ControlPacketType::EchoReply,
            job_id: None,
            params: HashMap::new(),
        };

        debug!("Sending echo reply");

        connection.stream.write(&response.to_buffer()).await.unwrap();
        Ok(())
    }
}
