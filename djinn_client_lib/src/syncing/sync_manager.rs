use std::{collections::HashMap, error::Error, sync::Arc};

use djinn_core_lib::data::packets::{ControlPacket, ControlPacketType};

use tokio::{
    fs::remove_file,
    io::{BufReader, ReadHalf},
    net::TcpStream,
    sync::Mutex,
};

use crate::connectivity::Connection;

use super::{PacketHandler, Transfer, TransferHandler};

pub struct SyncManager {
    pub path: String,
    pub target: String,
    pub job_id: Option<u32>,
    pub polling_ready: bool,
    pub transfers: Vec<Arc<Mutex<Transfer>>>,
    pub next_transfer_id: u32,
    pub is_syncing: Arc<Mutex<bool>>,
    pub current_sync_update_checklist: HashMap<String, bool>,
}

impl SyncManager {
    pub fn new(path: String, target: String) -> SyncManager {
        SyncManager {
            path,
            target,
            job_id: None,
            polling_ready: false,
            transfers: vec![],
            next_transfer_id: 0,
            is_syncing: Arc::new(Mutex::new(false)),
            current_sync_update_checklist: HashMap::new(),
        }
    }

    pub async fn start(&mut self, connection: &mut Connection) -> Result<(), Box<dyn Error>> {
        //Ask the server to start syncing
        info!("Asking server if we can sync");

        let mut params = HashMap::new();
        params.insert("path".to_string(), self.path.clone());

        let packet = ControlPacket::new(ControlPacketType::SyncRequest, params);
        connection.send_packet(packet).await?;

        //Start listening to the server for updates and commands
        info!("Listening for updates and commands");

        let reader_arc = connection.reader.clone();
        self.listen(reader_arc, connection).await?;

        Ok(())
    }

    async fn listen(
        &mut self,
        reader: Arc<Mutex<Option<BufReader<ReadHalf<TcpStream>>>>>,
        connection: &mut Connection,
    ) -> Result<(), Box<dyn Error>> {
        // Open reader
        let mut reader_option = reader.lock().await;
        let reader = reader_option.as_mut().unwrap();
        let packet_handler = PacketHandler::new();

        loop {
            // Read packets
            let packets = connection.packet_reader.read(reader, None).await;

            if packets.is_empty() {
                // Connection closed
                info!("Connection closed");
                break;
            }

            // Handle packets
            for packet in packets {
                packet_handler
                    .handle_boxed_packet(self, packet, connection)
                    .await;
            }
        }

        Ok(())
    }

    pub async fn handle_sync_update(&mut self, packet: &ControlPacket, connection: &Connection) {
        info!("Sync update received");
        debug!("Sync update packet: {:?}", packet.params);

        let transfer_handler = TransferHandler::new();

        // Deny update if already syncing
        let is_syncing = self.is_syncing.lock().await;
        if *is_syncing {
            return;
        }
        drop(is_syncing);

        self.create_sync_update_checklist(packet.params.clone())
            .await;

        // Loop through hashmap params
        for (key, value) in packet.params.iter() {
            let key = key.clone();
            let value = value.clone();

            if value == "GET" {
                // Get the file from the client
                info!("Getting file {}", key);
                transfer_handler
                    .start_get_file(self, key, connection)
                    .await;
            } else if value == "DELETE" {
                // Delete the file from the client
                info!("Deleting file {}", key);

                remove_file(self.target.clone() + "/" + &key)
                    .await
                    .expect("Failed to delete file");

                self.write_off_sync_update_checklist(key.clone()).await;
            } else if value == "PUT" {
                // Put the file on the client
                info!("Putting file {}", key);
                transfer_handler
                    .start_put_file(self, key, connection)
                    .await;
            } else {
                //Log type
                debug!("Unknown sync update type: {}", value);
            }
        }
    }

    pub async fn create_sync_update_checklist(&mut self, sync_update: HashMap<String, String>) {
        let mut new_hashmap: HashMap<String, bool> = HashMap::new();

        for (key, _) in sync_update.iter() {
            new_hashmap.insert(key.clone(), false);
        }

        self.current_sync_update_checklist = new_hashmap;

        debug!(
            "Created sync update checklist: {:?}",
            self.current_sync_update_checklist
        );

        // If list is not empty, set syncing to true
        if !self.current_sync_update_checklist.is_empty() {
            let is_syncing_arc = self.is_syncing.clone();
            let mut is_syncing = is_syncing_arc.lock().await;
            *is_syncing = true;
        }
    }

    pub async fn write_off_sync_update_checklist(&mut self, path: String) {
        self.current_sync_update_checklist.insert(path, true);

        // Check if all values are true
        let mut all_true = true;
        for (_, value) in self.current_sync_update_checklist.iter() {
            if !value {
                all_true = false;
            }
        }

        if all_true {
            // Log checklist
            debug!(
                "Sync update checklist: {:?}",
                self.current_sync_update_checklist
            );
            // Set sync update to false
            let is_syncing_arc = self.is_syncing.clone();
            let mut is_syncing = is_syncing_arc.lock().await;
            *is_syncing = false;

            self.current_sync_update_checklist = HashMap::new();
        }
    }

    pub async fn get_transfer_by_id(&mut self, transfer_id: u32) -> Option<Arc<Mutex<Transfer>>> {
        for transfer in &mut self.transfers {
            let unlocked_transfer = transfer.lock().await;
            if unlocked_transfer.id == transfer_id {
                return Some(transfer.clone());
            }
        }

        None
    }

    pub async fn get_transfer_by_job_id(&mut self, job_id: u32) -> Option<Arc<Mutex<Transfer>>> {
        for transfer in &mut self.transfers {
            let unlocked_transfer = transfer.lock().await;
            if unlocked_transfer.job_id == job_id {
                return Some(transfer.clone());
            }
        }

        None
    }
}
