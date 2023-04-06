use std::{collections::HashMap, error::Error};

use async_std::io::WriteExt;
use async_trait::async_trait;
use djinn_core_lib::data::packets::{ControlPacket, PacketType, ControlPacketType, packet::Packet};

use crate::connectivity::Connection;

use super::ControlCommand;

pub struct SyncIndexResponseCommand {}

#[async_trait]
impl ControlCommand for SyncIndexResponseCommand {
    async fn execute(&self, connection: &mut Connection, _packet: &ControlPacket) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
