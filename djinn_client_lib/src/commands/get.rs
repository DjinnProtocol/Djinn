use std::{collections::HashMap, error::Error};

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
        return Ok("Jemoeder".to_string())
    }
}
