use std::{error::Error, collections::HashMap};

use djinn_core_lib::data::packets::{ControlPacketType, ControlPacket, PacketType, packet::Packet};

use crate::connectivity::Connection;

pub struct SyncHandler {
    pub directory: String,
    pub job_id: Option<u32>
}

impl SyncHandler {
    pub fn new(directory: String) -> SyncHandler {
        SyncHandler {
            directory,
            job_id: None
        }
    }

    pub async fn start(&mut self, connection: &mut Connection) -> Result<(), Box<dyn Error>> {
        //Ask the server to start syncing
        debug!("Sending sync request");

        let mut params = HashMap::new();
        params.insert("directory".to_string(), self.directory.clone());

        let packet = ControlPacket::new(ControlPacketType::SyncRequest, params);
        connection.send_packet(packet).await?;

        //Wait for the server to send the ack
        debug!("Waiting for sync ack");
        let boxed_packet = connection.read_next_packet().await?;

        if !matches!(boxed_packet.get_packet_type(), PacketType::Control) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected packet type",
            )));
        }

        let control_packet = boxed_packet
            .as_any()
            .downcast_ref::<ControlPacket>()
            .unwrap();

            if matches!(
                control_packet.control_packet_type,
                ControlPacketType::SyncDeny
            ) {
                let reason = control_packet.params.get("reason").unwrap();
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Transfer denied: {}", reason),
                )));
            }

            if !matches!(
                control_packet.control_packet_type,
                ControlPacketType::SyncAck
            ) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unexpected control packet type",
                )));
            }

        let job_id = control_packet.params.get("job_id").unwrap().parse::<u32>().unwrap();
        self.job_id = Some(job_id);



        debug!("Received sync ack");

        //Start listening to the server for updates and commands
        debug!("Starting to listen for updates");
        self.listen(connection).await?;

        Ok(())

    }

    async fn listen(&self, connection: &mut Connection) -> Result<(), Box<dyn Error>> {
        loop {
            let boxed_packet = connection.read_next_packet().await?;

            if !matches!(boxed_packet.get_packet_type(), PacketType::Control) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unexpected packet type",
                )));
            }

            let control_packet = boxed_packet
                .as_any()
                .downcast_ref::<ControlPacket>()
                .unwrap();

            break;
        }

        Ok(())
    }

    pub async fn handle_boxed_packet<'a>(&self, boxed_packet: Box<dyn Packet + 'a>, connection: &mut Connection) {
        let packet_ref: &dyn Packet = boxed_packet.as_ref();

        match packet_ref.get_packet_type() {
            PacketType::Control => {
                info!("Control packet received");
                let control_packet = packet_ref.as_any().downcast_ref::<ControlPacket>().unwrap();

                self.handle_control_packet(&control_packet, connection).await;
            },
            PacketType::Data => {
                // Throw error
                panic!("Data packets are not supported yet")
            }
        }
    }

    pub async fn handle_control_packet(&self, packet: &ControlPacket, connection: &mut Connection) {
        match packet.control_packet_type {
            ControlPacketType::SyncIndexRequest => {
                info!("Sync index request received");

                connection.index_manager.build(self.directory.clone()).await;

                let mut params = HashMap::new();
                let index = &connection.index_manager.index;

                //Stringify the timestamps
                for (key, value) in index.iter() {
                    params.insert(key.clone(), value.to_string());
                }

                let mut packet = ControlPacket::new(ControlPacketType::SyncIndexResponse, params);

                packet.job_id = self.job_id;

                connection.send_packet(packet).await.unwrap();
            },
            _ => {
                // Throw error
                panic!("Unknown control packet type")
            }
        }
    }

}
