use std::{collections::HashMap, error::Error, sync::Arc};

use djinn_core_lib::{data::{
    packets::{packet::Packet, ControlPacket, ControlPacketType, PacketType, DataPacket},
    syncing::IndexManager,
}, jobs::Job};
use filetime::{set_file_mtime, FileTime};
use tokio::{
    io::{BufReader, ReadHalf, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
    time::sleep, fs::{File, rename},
};

use crate::{connectivity::Connection, commands::GetCommand, syncing::TransferStatus};

use super::{Transfer, TransferDirection};

pub struct SyncHandler {
    pub path: String,
    pub target: String,
    pub job_id: Option<u32>,
    pub polling_ready: bool,
    pub transfers: Vec<Transfer>,
    pub next_transfer_id: u32,
}

impl SyncHandler {
    pub fn new(path: String, target: String) -> SyncHandler {
        SyncHandler {
            path,
            target,
            job_id: None,
            polling_ready: false,
            transfers: vec![],
            next_transfer_id: 0,
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
                let data_packet: &DataPacket = packet_ref.as_any().downcast_ref::<DataPacket>().unwrap();

                self.handle_data_packet(&data_packet, connection).await;
            }
        }
    }

    pub async fn handle_control_packet(&mut self, packet: &ControlPacket, connection: &Connection) {
        match packet.control_packet_type {
            ControlPacketType::SyncIndexRequest => {
                info!("Sync index request received");

                let full_path = self.target.clone();
                info!("Full path: {}", full_path);
                let mut index_manager = IndexManager::new(full_path);
                index_manager.build().await;

                info!("Client: {:?}", index_manager.index);

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
                self.handle_sync_update(packet, connection).await;
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
            ControlPacketType::TransferAck => {
                info!("Transfer ack received");

                // Update status of transfer
                let transfer_id = packet.params.get("transfer_id").unwrap().parse::<u32>().unwrap();
                let job_id = packet.params.get("job_id").unwrap().parse::<u32>().unwrap();
                let mut transfer = self.transfers.iter_mut().find(|transfer| transfer.id == transfer_id).unwrap();
                transfer.status = TransferStatus::Accepted;
                transfer.job_id = job_id;
                transfer.original_modified_time = packet.params.get("modified_time").unwrap().parse::<u64>().unwrap();

                // Start the transfer
                let mut packet = ControlPacket::new(ControlPacketType::TransferStart, HashMap::new());
                packet.params.insert("job_id".to_string(), job_id.clone().to_string());
                connection.send_packet(packet).await.expect("Failed to send transfer start packet");
            }
            ControlPacketType::TransferDeny => {
                info!("Transfer deny received");

                // Update status of transfer
                let transfer_id = packet.params.get("transfer_id").unwrap().parse::<u32>().unwrap();
                let mut transfer = self.transfers.iter_mut().find(|transfer| transfer.id == transfer_id).unwrap();
                transfer.status = TransferStatus::Denied;

                //TODO: handle deny reason
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


    pub async fn handle_data_packet(&mut self, packet: &DataPacket, connection: &Connection) {
        // Get the transfer by job id
        let job_id = packet.job_id;
        let transfer = self.transfers.iter_mut().find(|transfer| transfer.job_id == job_id).unwrap();

        // Start transfer if accepted
        if matches!(transfer.status, TransferStatus::Accepted) {
            // Get the file
            let full_path = self.target.clone() + "/" + &transfer.file_path;
            transfer.open_file = Some(File::create(full_path + ".djinn_temp").await.unwrap());
            transfer.status = TransferStatus::InProgress;
        }

        // Write the data to the file
        if matches!(transfer.status, TransferStatus::InProgress) {
            let file = transfer.open_file.as_mut().unwrap();
            if packet.has_data {
                file.write_all(&packet.data).await.unwrap();
            } else {
                file.flush().await.unwrap();
                transfer.status = TransferStatus::Completed;
                // Move file and set modified time
                let full_path = self.target.clone() + "/" + &transfer.file_path;
                rename(full_path.clone() + ".djinn_temp", &full_path).await.unwrap();
                let file_time = FileTime::from_unix_time(transfer.original_modified_time as i64, 0);
                set_file_mtime(&full_path, file_time).unwrap();
            }
        } else {
            panic!("Transfer data received for transfer that is not in progress")
        }
    }

    pub async fn handle_sync_update(&mut self, packet: &ControlPacket, connection: &Connection) {
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
                    self.start_get_file(key, &connection).await;
                }
                deleteString => {
                    // Delete the file from the client
                    info!("Deleting file {}", key);
                }
                putString => {
                    // Put the file on the client
                    info!("Putting file {}", key);
                }
                _ => {
                    //Log type
                    debug!("Unknown sync update type: {}", value);
                    // Throw error
                    panic!("Unknown sync update type")
                }
            }
        }
    }

    pub async fn start_get_file(&mut self, path: String, connection: &Connection) {
        let transfer_id = self.next_transfer_id;
        self.next_transfer_id += 1;

        self.transfers.push(Transfer::new(TransferDirection::ToClient, transfer_id.clone(), path.clone()));


        let mut params = HashMap::new();
        params.insert("file_path".to_string(), path);
        params.insert("transfer_id".to_string(), transfer_id.to_string());

        let packet = ControlPacket::new(ControlPacketType::TransferRequest, params);
        connection.send_packet(packet).await.unwrap();
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
