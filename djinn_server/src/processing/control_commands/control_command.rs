use std::error::Error;

use async_trait::async_trait;
use djinn_core_lib::data::packets::ControlPacket;

use crate::connectivity::Connection;

#[async_trait]
pub trait ControlCommand {
    async fn execute(&self, connection: &mut Connection, packet: &ControlPacket) -> Result<(), Box<dyn Error>>;
}
