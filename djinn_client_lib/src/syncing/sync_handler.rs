use std::{collections::HashMap, error::Error, path::Path, sync::Arc};

use djinn_core_lib::{
    data::{
        packets::{packet::Packet, ControlPacket, ControlPacketType, DataPacket, PacketType, DataPacketGenerator},
        syncing::IndexManager,
    }
};
use filetime::{set_file_mtime, FileTime};
use tokio::{
    fs::{create_dir_all, rename, File, remove_file, self},
    io::{AsyncWriteExt, BufReader, ReadHalf},
    net::TcpStream,
    sync::Mutex,
    time::sleep,
};

use crate::{
    connectivity::Connection,
    syncing::{
        fs_poller::{self, FsPoller},
        TransferStatus,
    },
};

use super::{Transfer, TransferDirection, transfer};

pub struct SyncHandler {
    pub path: String,
    pub target: String,
    pub job_id: Option<u32>,
    pub polling_ready: bool,
    pub transfers: Vec<Arc<Mutex<Transfer>>>,
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
                let data_packet: &DataPacket =
                    packet_ref.as_any().downcast_ref::<DataPacket>().unwrap();

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

                // Spawn fs poller
                let new_target = self.target.clone();
                let new_job_id = self.job_id.unwrap().clone();
                let write_stream_arc = connection.write_stream.clone();

                tokio::spawn(async move {
                    let mut fs_poller = FsPoller::new(new_target, new_job_id);
                    fs_poller.poll(write_stream_arc).await.unwrap();
                });
            }
            ControlPacketType::SyncDeny => {
                info!("Sync deny received");

                let reason = packet.params.get("reason").unwrap();
                panic!("Sync denied: {}", reason)
            }
            ControlPacketType::TransferAck => {
                info!("Transfer ack received");

                // Update status of transfer
                let transfer_id = packet
                    .params
                    .get("transfer_id")
                    .unwrap()
                    .parse::<u32>()
                    .unwrap();

                let job_id = packet.params.get("job_id").unwrap().parse::<u32>().unwrap();

                let mut option_arc_transfer = self.get_transfer_by_id(transfer_id).await;
                let mut transfer_arc = option_arc_transfer.as_mut().unwrap();
                let mut transfer = transfer_arc.lock().await;

                transfer.status = TransferStatus::Accepted;
                transfer.job_id = job_id;

                // Save modified date to transfer if it's a download
                if matches!(transfer.direction, TransferDirection::ToClient) {
                    transfer.original_modified_time = packet
                        .params
                        .get("modified_time")
                        .unwrap()
                        .parse::<u64>()
                        .unwrap();
                }

                // Start the transfer
                if matches!(transfer.direction, TransferDirection::ToClient) {
                    let mut packet =
                    ControlPacket::new(ControlPacketType::TransferStart, HashMap::new());
                packet
                    .params
                    .insert("job_id".to_string(), job_id.clone().to_string());
                connection
                    .send_packet(packet)
                    .await
                    .expect("Failed to send transfer start packet");
                } else {
                    self.start_sending_file(&transfer, connection).await;
                }
            }
            ControlPacketType::TransferDeny => {
                info!("Transfer deny received");

                // Update status of transfer
                let transfer_id = packet
                    .params
                    .get("transfer_id")
                    .unwrap()
                    .parse::<u32>()
                    .unwrap();

                let mut option_arc_transfer = self.get_transfer_by_id(transfer_id).await;
                let mut transfer_arc = option_arc_transfer.as_mut().unwrap();
                let mut transfer = transfer_arc.lock().await;

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

    pub async fn start_sending_file(&mut self, transfer: &Transfer, connection: &Connection) {
        // Get the file path from the job
        let file_path = transfer.file_path.clone();
        let full_path = format!("{}/{}", self.target, file_path);

        // Open da file
        let packet_generator = DataPacketGenerator::new(transfer.job_id.clone(), full_path);
        let iterator = packet_generator.iter();

        // Get connection read_stream
        let write_stream_arc = connection.write_stream.clone();
        // let mut writer = BufWriter::new(&mut *write_stream);
        // TODO: check speed difference between BufWriter and and native stream

        for packet in iterator {
            let mut option_write_stream = write_stream_arc.lock().await;

            if option_write_stream.is_none() {
                panic!("Write stream is none");
            }

            let write_stream = option_write_stream.as_mut().unwrap();

            let buffer = &packet.to_buffer();
            write_stream.write_all(&buffer).await.expect("Failed to write to stream")
        }

        connection.flush().await.expect("Failed to flush connection");
    }

    pub async fn handle_data_packet(&mut self, packet: &DataPacket, connection: &Connection) {
        // Get the transfer by job id
        let job_id = packet.job_id;
        let mut option_arc_transfer = self.get_transfer_by_job_id(job_id).await;
                let mut transfer_arc = option_arc_transfer.as_mut().unwrap();
                let mut transfer = transfer_arc.lock().await;

        // Start transfer if accepted
        if matches!(transfer.status, TransferStatus::Accepted) {
            // Get the file
            let mut full_path = self.target.clone() + "/" + &transfer.file_path;
            full_path = full_path.replace("//", "/");
            debug!("Full path: {}", full_path);
            // Create the directories if they don't exist
            create_dir_all(Path::new(&full_path).parent().unwrap())
                .await
                .unwrap();
            // Create the file
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
                rename(full_path.clone() + ".djinn_temp", &full_path)
                    .await
                    .unwrap();
                let file_time = FileTime::from_unix_time(transfer.original_modified_time as i64, 0);
                set_file_mtime(&full_path, file_time).unwrap();
            }
        } else {
            panic!("Transfer data received for transfer that is not in progress")
        }
    }

    pub async fn handle_sync_update(&mut self, packet: &ControlPacket, connection: &Connection) {
        info!("Sync update received");
        debug!("Sync update packet: {:?}", packet.params);
        // Loop through hashmap params
        for (key, value) in packet.params.iter() {
            let key = key.clone();
            let value = value.clone();

            if value == "GET" {
                // Get the file from the client
                info!("Getting file {}", key);
                self.start_get_file(key, &connection).await;
            } else if value == "DELETE" {
                // Delete the file from the client
                info!("Deleting file {}", key);

                remove_file(self.target.clone() + "/" + &key)
                    .await
                    .expect("Failed to delete file");
            } else if value == "PUT" {
                // Put the file on the client
                info!("Putting file {}", key);
                self.start_put_file(key, &connection).await;
            } else {
                //Log type
                debug!("Unknown sync update type: {}", value);
            }
        }
    }

    pub async fn start_get_file(&mut self, path: String, connection: &Connection) {
        let transfer_id = self.next_transfer_id;
        self.next_transfer_id += 1;

        self.transfers.push(Arc::new(Mutex::new(Transfer::new(
            TransferDirection::ToClient,
            transfer_id.clone(),
            path.clone(),
        ))));

        let mut params = HashMap::new();
        params.insert("file_path".to_string(), path.clone());
        params.insert("transfer_id".to_string(), transfer_id.to_string());
        params.insert("direction".to_string(), "toClient".to_string());

        debug!("Sending transfer request packet for {}", path);

        let packet = ControlPacket::new(ControlPacketType::TransferRequest, params);
        connection.send_packet(packet).await.unwrap();
    }

    pub async fn start_put_file(&mut self, path: String, connection: &Connection) {
        let transfer_id = self.next_transfer_id;
        self.next_transfer_id += 1;

        self.transfers.push(Arc::new(Mutex::new(Transfer::new(
            TransferDirection::ToServer,
            transfer_id.clone(),
            path.clone(),
        ))));

        let mut params = HashMap::new();
        params.insert("file_path".to_string(), path.clone());
        params.insert("transfer_id".to_string(), transfer_id.to_string());
        params.insert("direction".to_string(), "toServer".to_string());

        // Get modified time
        let full_path = self.target.clone() + "/" + &path;
        let modified_time = fs::metadata(&full_path).await.expect("AAAA").modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        params.insert("modified_time".to_string(), modified_time.to_string());

        let packet = ControlPacket::new(ControlPacketType::TransferRequest, params);
        connection.send_packet(packet).await.unwrap();
    }

    pub async fn get_transfer_by_id(&mut self, transfer_id: u32) -> Option<Arc<Mutex<Transfer>>> {
        for transfer in &mut self.transfers {
            let unlocked_transfer = transfer.lock().await;
            if unlocked_transfer.id == transfer_id {
                return Some(transfer.clone())
            }
        }

        None
    }

    pub async fn get_transfer_by_job_id(&mut self, job_id: u32) -> Option<Arc<Mutex<Transfer>>> {
        for transfer in &mut self.transfers {
            let unlocked_transfer = transfer.lock().await;
            if unlocked_transfer.job_id == job_id {
                return Some(transfer.clone())
            }
        }

        None
    }
}
