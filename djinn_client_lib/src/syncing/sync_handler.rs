use std::{error::Error, collections::HashMap};

use djinn_core_lib::data::packets::{ControlPacketType, ControlPacket};

use crate::connectivity::Connection;

pub struct SyncHandler {
    pub directory: String,
}

impl SyncHandler {
    pub fn new(directory: String) -> SyncHandler {
        SyncHandler {
            directory,
        }
    }

    pub async fn start(&self, connection: &mut Connection) -> Result<(), Box<dyn Error>> {
        //Ask the server to start syncing
        debug!("Sending sync request");

        let mut params = HashMap::new();
        params.insert("directory".to_string(), self.directory.clone());

        let packet = ControlPacket::new(ControlPacketType::SyncRequest, params);

        connection.send_packet(packet).await?;

        Ok(())

    }
}
