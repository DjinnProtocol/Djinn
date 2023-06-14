use std::{path::Path, collections::HashMap};

use djinn_core_lib::data::{packets::{packet::Packet, PacketType, ControlPacket, DataPacket, ControlPacketType}, syncing::IndexManager};
use filetime::{FileTime, set_file_mtime};
use tokio::{fs::{create_dir_all, File, rename}, io::AsyncWriteExt};

use crate::{connectivity::Connection, syncing::{TransferStatus, fs_poller::FsPoller, TransferDirection, TransferHandler}};

use super::SyncManager;

pub struct PacketHandler {}

impl PacketHandler {
    pub fn new() -> PacketHandler {
        PacketHandler {}
    }
    pub async fn handle_boxed_packet<'a>(
        &self,
        sync_manager: &mut SyncManager,
        boxed_packet: Box<dyn Packet + 'a>,
        connection: &Connection,
    ) {
        let packet_ref: &dyn Packet = boxed_packet.as_ref();

        match packet_ref.get_packet_type() {
            PacketType::Control => {
                let control_packet = packet_ref.as_any().downcast_ref::<ControlPacket>().unwrap();
                debug!("Control packet received");

                self.handle_control_packet(sync_manager, &control_packet, connection)
                    .await;
            }
            PacketType::Data => {
                // Throw error
                let data_packet: &DataPacket =
                    packet_ref.as_any().downcast_ref::<DataPacket>().unwrap();

                self.handle_data_packet(sync_manager, &data_packet).await;
            }
        }
    }

    pub async fn handle_control_packet(&self, sync_manager: &mut SyncManager, packet: &ControlPacket, connection: &Connection) {
        match packet.control_packet_type {
            ControlPacketType::SyncIndexRequest => {
                let full_path = sync_manager.target.clone();
                let mut index_manager = IndexManager::new(full_path);
                index_manager.build().await;


                let mut params = HashMap::new();
                let index = index_manager.index;

                //Stringify the timestamps
                for (key, value) in index.iter() {
                    params.insert(key.clone(), value.to_string());
                }

                // Send sync index response
                let mut packet = ControlPacket::new(ControlPacketType::SyncIndexResponse, params);
                packet.job_id = sync_manager.job_id;
                connection.send_packet(packet).await.unwrap();
            }
            ControlPacketType::SyncUpdate => {
                debug!("Sync update received");
                sync_manager.handle_sync_update(packet, connection).await;
            }
            ControlPacketType::SyncAck => {
                debug!("Sync ack received");
                let job_id = packet.params.get("job_id").unwrap().parse::<u32>().unwrap();
                sync_manager.job_id = Some(job_id);

                // Spawn fs poller
                let new_target = sync_manager.target.clone();
                let new_job_id = sync_manager.job_id.unwrap().clone();
                let write_stream_arc = connection.write_stream.clone();
                let new_is_syncing = sync_manager.is_syncing.clone();

                tokio::spawn(async move {
                    let mut fs_poller = FsPoller::new(new_target, new_job_id);
                    fs_poller.poll(write_stream_arc, new_is_syncing).await.unwrap();
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

                let mut option_arc_transfer = sync_manager.get_transfer_by_id(transfer_id).await;
                let transfer_arc = option_arc_transfer.as_mut().unwrap();
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
                    let transfer_handler = TransferHandler::new();
                    transfer_handler.start_sending_file(sync_manager, &transfer, connection).await;
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

                let mut option_arc_transfer = sync_manager.get_transfer_by_id(transfer_id).await;
                let transfer_arc = option_arc_transfer.as_mut().unwrap();
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

    pub async fn handle_data_packet(&self, sync_manager: &mut SyncManager, packet: &DataPacket) {
        // Get the transfer by job id
        let job_id = packet.job_id;
        let mut option_arc_transfer = sync_manager.get_transfer_by_job_id(job_id).await;
        let transfer_arc = option_arc_transfer.as_mut().unwrap();
        let mut transfer = transfer_arc.lock().await;

        // Start transfer if accepted
        if matches!(transfer.status, TransferStatus::Accepted) {
            // Get the file
            let mut full_path = sync_manager.target.clone() + "/" + &transfer.file_path;
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
                let full_path = sync_manager.target.clone() + "/" + &transfer.file_path;

                let file_time = FileTime::from_unix_time(transfer.original_modified_time as i64, 0);
                set_file_mtime(full_path.clone() + ".djinn_temp", file_time).unwrap();

                rename(full_path.clone() + ".djinn_temp", &full_path)
                    .await
                    .unwrap();

                // Update checklist
                sync_manager.write_off_sync_update_checklist(transfer.file_path.clone()).await;
            }
        } else {
            panic!("Transfer data received for transfer that is not in progress")
        }
    }
}
