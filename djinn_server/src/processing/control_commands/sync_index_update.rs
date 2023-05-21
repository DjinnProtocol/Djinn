use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use djinn_core_lib::{data::{packets::{ControlPacket, ControlPacketType}, syncing::IndexManager}, jobs::JobType};
use tokio::fs;

use crate::{connectivity::Connection, syncing::{IndexComparer, SourceOfTruth}};

use super::ControlCommand;

use crate::CONFIG;

pub struct SyncIndexUpdateCommand {}

#[async_trait]
impl ControlCommand for SyncIndexUpdateCommand {
    async fn execute(&self, connection: &mut Connection, packet: &ControlPacket) -> Result<(), Box<dyn Error>> {
        //Check if job id exists
        if packet.job_id.is_none() {
            error!("SyncIndexUpdate packet does not contain a job id");
            return Ok(())
        }

        //Check if sync exists
        let possible_sync_job = connection.get_job(packet.job_id.unwrap()).await;


        if possible_sync_job.is_none() {
            error!("SyncIndexUpdate packet does not contain a valid job id");
            return Ok(())
        }

        //Check if sync is a sync job
        let sync_job_arc = possible_sync_job.unwrap().clone();
        let sync_job = sync_job_arc.lock().await;

        if !matches!(sync_job.job_type, JobType::Sync) {
            error!("SyncIndexUpdate packet job id does not belong to a sync job");
            return Ok(())
        }

        // Get server index
        let path = sync_job.params.get("path").unwrap();
        let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + path;


        let mut server_index_manager = IndexManager::new(full_path.clone());
        server_index_manager.build().await;

        // Get client index
        let mut client_index = HashMap::new();

        for (key, value) in packet.params.iter() {
            client_index.insert(key.clone(), value.parse::<usize>().unwrap());
        }

        // Get index comparer
        debug!("Server: {:?}. Client: {:?}", server_index_manager.index, client_index);
        let index_comparer = IndexComparer::new(client_index, server_index_manager.index, SourceOfTruth::Client);
        let mut changes = index_comparer.compare();
        debug!("Changes: {:?}", changes);
        let immutable_changes = changes.clone();

        // First proces self deletes
        for (key, value) in immutable_changes.iter() {
            if value == "SELF_DELETE" {
                // Delete file
                let mut full_file_path = full_path.clone() + "/" + key;
                full_file_path = full_file_path.replace("//", "/");

                debug!("Deleting file: {}", full_file_path);

                fs::remove_file(full_file_path).await.unwrap();
                // Remove from index
                changes.remove(key);
            }
        }

        // Send changes to client
        let mut response = ControlPacket::new(ControlPacketType::SyncUpdate, changes);
        response.job_id = Some(sync_job.id);

        connection.send_packet(response).await.unwrap();

        connection.flush().await;

        Ok(())
    }
}
