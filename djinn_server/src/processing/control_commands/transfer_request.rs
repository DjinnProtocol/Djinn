use std::{collections::HashMap, error::Error};
use async_std::fs;

use async_std::io::WriteExt;
use async_trait::async_trait;
use djinn_core_lib::data::packets::{ControlPacket, PacketType, ControlPacketType, packet::Packet, TransferDenyReason};

use crate::connectivity::Connection;

use super::ControlCommand;

pub struct TransferRequestCommand {}

#[async_trait]
impl ControlCommand for TransferRequestCommand {
    async fn execute(&self, connection: &mut Connection, packet: &ControlPacket) -> Result<(), Box<dyn Error>> {
        //Check if file exists if download request
        let path = packet.params.get("path").unwrap();

        if !fs::metadata(path).await.is_ok() {
            let params = HashMap::new();
            params.insert("reason", TransferDenyReason::FileNotFound);
            let response = ControlPacket::new(ControlPacketType::TransferDeny, params);

            connection.stream.write(&response.to_buffer()).await.unwrap();

            return Ok(());
        }

        return Ok(());
    }
}
