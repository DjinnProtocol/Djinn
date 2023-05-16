use std::{collections::HashMap, error::Error, sync::Arc};

use djinn_core_lib::data::{
    packets::{packet::Packet, ControlPacket, ControlPacketType, PacketType},
    syncing::IndexManager,
};
use tokio::{
    io::{BufReader, ReadHalf},
    net::TcpStream,
    sync::Mutex,
    time::sleep,
};

use crate::{connectivity::Connection, commands::GetCommand};

pub struct SyncHandler {
    pub path: String,
    pub target: String,
    pub job_id: Option<u32>,
    pub polling_ready: bool,
    pub jobs: Vec<Job>,
}

impl SyncHandler {
    pub fn new(path: String, target: String) -> SyncHandler {
        SyncHandler {
            path,
            target,
            job_id: None,
            polling_ready: false,
        }
    }

    pub async fn start(&mut self, connection: &mut Connection) -> Result<(), Box<dyn Error>> {
        //Ask the server to start syncing
        debug!("Sending sync request");

        let mut params = HashMap::new();
        params.insert("path".to_string(), self.path.clone());

        let packet = ControlPacket::new(ControlPacketType::SyncRequest, params);
        connection.send_packet(packet).await?;

        //Start listening to the server for updates and commands
        debug!("Starting to listen for updates");

        let reader_arc = connection.reader.clone();
        self.listen(reader_arc, connection).await?;
        Ok(())
    }

    async fn listen(
        &mut self,
        reader: Arc<Mutex<Option<BufReader<ReadHalf<TcpStream>>>>>,
        connection: &mut Connection,
    ) -> Result<(), Box<dyn Error>> {
        // Handle incoming streams
        let mut reader_option = reader.lock().await;
        let reader = reader_option.as_mut().unwrap();

        debug!("Actually started listening");

        loop {
            let packets = connection.packet_reader.read2(reader, None).await;

            if packets.len() == 0 {
                // Connection closed
                debug!("Connection closed");
                break;
            }

            for packet in packets {
                self.handle_boxed_packet(packet, &connection).await;
            }
        }

        Ok(())
    }

    pub async fn handle_boxed_packet<'a>(
        &mut self,
        boxed_packet: Box<dyn Packet + 'a>,
        connection: &Connection,
    ) {
        let packet_ref: &dyn Packet = boxed_packet.as_ref();

        match packet_ref.get_packet_type() {
            PacketType::Control => {
                info!("Control packet received");
                let control_packet = packet_ref.as_any().downcast_ref::<ControlPacket>().unwrap();

                self.handle_control_packet(&control_packet, connection)
                    .await;
            }
            PacketType::Data => {
                // Throw error
                panic!("Data packets are not supported yet")
            }
        }
    }

    pub async fn handle_control_packet(&mut self, packet: &ControlPacket, connection: &Connection) {
        match packet.control_packet_type {
            ControlPacketType::SyncIndexRequest => {
                info!("Sync index request received");

                let full_path = self.target.clone();
                let mut index_manager = IndexManager::new(full_path);
                index_manager.build().await;

                let mut params = HashMap::new();
                let index = index_manager.index;

                //Stringify the timestamps
                for (key, value) in index.iter() {
                    params.insert(key.clone(), value.to_string());
                }

                let mut packet = ControlPacket::new(ControlPacketType::SyncIndexResponse, params);

                packet.job_id = self.job_id;

                connection.send_packet(packet).await.unwrap();
            }
            ControlPacketType::SyncUpdate => {
                info!("Sync update received");
                self.handle_sync_update(packet).await;
            }
            ControlPacketType::SyncAck => {
                info!("Sync ack received");

                let job_id = packet.params.get("job_id").unwrap().parse::<u32>().unwrap();
                self.job_id = Some(job_id);
            }
            ControlPacketType::SyncDeny => {
                info!("Sync deny received");

                let reason = packet.params.get("reason").unwrap();
                panic!("Sync denied: {}", reason)
            }
            _ => {
                //Log type
                debug!(
                    "Unknown control packet type: {:?}",
                    packet.control_packet_type as u8
                );
                // Throw error
                panic!("Unknown control packet type")
            }
        }
    }

    pub async fn handle_sync_update(&mut self, packet: &ControlPacket) {
        info!("Sync update received");
        // Loop through hashmap params
        for (key, value) in packet.params.iter() {
            let key = key.clone();
            let value = value.clone();

            //TODO: improve
            let getString = "GET".to_string();
            let deleteString = "DELETE".to_string();
            let putString = "PUT".to_string();

            match value {
                getString => {
                    // Get the file from the client
                    info!("Getting file {}", key);

                deleteString => {
                    // Delete the file from the client
                    info!("Deleting file {}", key);
                }
                putString => {
                    // Put the file on the client
                    info!("Putting file {}", key);
                }
            }
        }
    }

    pub async fn poll_file_changes(&self, connection: &Connection) -> Result<(), Box<dyn Error>> {
        let mut index_manager = IndexManager::new(self.target.clone());

        loop {
            index_manager.build().await;

            let mut params = HashMap::new();
            let index = &index_manager.index;

            //Stringify the timestamps
            for (key, value) in index.iter() {
                params.insert(key.clone(), value.to_string());
            }

            let mut packet = ControlPacket::new(ControlPacketType::SyncUpdate, params);

            packet.job_id = self.job_id;

            connection.send_packet(packet).await.unwrap();

            sleep(std::time::Duration::from_secs(1)).await;
        }

        Ok(())
    }
}
