use std::{collections::HashMap, sync::Arc};

use djinn_core_lib::data::packets::{
    packet::Packet, ControlPacket, ControlPacketType, DataPacketGenerator,
};
use tokio::{fs, io::AsyncWriteExt, sync::Mutex};

use crate::{connectivity::Connection, syncing::TransferDirection};

use super::{SyncManager, Transfer};

pub struct TransferHandler {}

impl TransferHandler {
    pub fn new() -> TransferHandler {
        TransferHandler {}
    }

    pub async fn start_get_file(
        &self,
        sync_manager: &mut SyncManager,
        path: String,
        connection: &Connection,
    ) {
        let transfer_id = sync_manager.next_transfer_id;
        sync_manager.next_transfer_id += 1;

        sync_manager
            .transfers
            .push(Arc::new(Mutex::new(Transfer::new(
                TransferDirection::ToClient,
                transfer_id,
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

    pub async fn start_put_file(
        &self,
        sync_manager: &mut SyncManager,
        path: String,
        connection: &Connection,
    ) {
        let transfer_id = sync_manager.next_transfer_id;
        sync_manager.next_transfer_id += 1;

        sync_manager
            .transfers
            .push(Arc::new(Mutex::new(Transfer::new(
                TransferDirection::ToServer,
                transfer_id,
                path.clone(),
            ))));

        let mut params = HashMap::new();
        params.insert("file_path".to_string(), path.clone());
        params.insert("transfer_id".to_string(), transfer_id.to_string());
        params.insert("direction".to_string(), "toServer".to_string());

        // Get modified time
        let full_path = sync_manager.target.clone() + "/" + &path;
        let modified_time = fs::metadata(&full_path)
            .await
            .expect("AAAA")
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        params.insert("modified_time".to_string(), modified_time.to_string());

        let packet = ControlPacket::new(ControlPacketType::TransferRequest, params);
        connection.send_packet(packet).await.unwrap();
    }

    pub async fn start_sending_file(
        &self,
        sync_manager: &mut SyncManager,
        transfer: &Transfer,
        connection: &Connection,
    ) {
        // Get the file path from the job
        let file_path = transfer.file_path.clone();
        let full_path = format!("{}/{}", sync_manager.target, file_path);

        // Open da file
        let packet_generator = DataPacketGenerator::new(transfer.job_id, full_path);
        let iterator = packet_generator.iter();

        // Get connection read_stream
        let write_stream_arc = connection.write_stream.clone();

        debug!("Sending file loop {}", file_path);

        for packet in iterator {
            let mut option_write_stream = write_stream_arc.lock().await;

            if option_write_stream.is_none() {
                panic!("Write stream is none");
            }

            let write_stream = option_write_stream.as_mut().unwrap();

            let buffer = &packet.to_buffer();
            write_stream
                .write_all(buffer)
                .await
                .expect("Failed to write to stream")
        }

        connection
            .flush()
            .await
            .expect("Failed to flush connection");

        // Update sync
        sync_manager
            .write_off_sync_update_checklist(transfer.file_path.clone())
            .await;
    }
}
