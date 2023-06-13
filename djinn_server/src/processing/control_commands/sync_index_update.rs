use std::{
    collections::HashMap,
    error::Error,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use djinn_core_lib::{
    data::{
        packets::{ControlPacket, ControlPacketType},
        syncing::IndexManager,
    },
    jobs::{Job, JobType},
};
use tokio::{fs, sync::Mutex};

use crate::{
    connectivity::{self, Connection, ConnectionUpdate},
    syncing::{IndexComparer, SourceOfTruth},
};

use super::ControlCommand;

use crate::CONFIG;
use crate::SERVER_DELETES;

pub struct SyncIndexUpdateCommand {}

#[async_trait]
impl ControlCommand for SyncIndexUpdateCommand {
    async fn execute(
        &self,
        connection: &mut Connection,
        packet: &ControlPacket,
    ) -> Result<(), Box<dyn Error>> {
        //Check if job id exists
        if packet.job_id.is_none() {
            error!("SyncIndexUpdate packet does not contain a job id");
            return Ok(());
        }

        //Check if sync exists
        let possible_sync_job = connection.get_job(packet.job_id.unwrap()).await;

        if possible_sync_job.is_none() {
            error!("SyncIndexUpdate packet does not contain a valid job id");
            return Ok(());
        }

        //Check if sync is a sync job
        let sync_job_arc = possible_sync_job.unwrap().clone();
        let sync_job = sync_job_arc.lock().await;

        if !matches!(sync_job.job_type, JobType::Sync) {
            error!("SyncIndexUpdate packet job id does not belong to a sync job");
            return Ok(());
        }

        // Get client index
        let mut client_index = HashMap::new();

        for (key, value) in packet.params.iter() {
            client_index.insert(key.clone(), value.parse::<usize>().unwrap());
        }

        drop(sync_job);

        process_client_index(connection, client_index, sync_job_arc.clone(), SourceOfTruth::Client).await;

        Ok(())
    }
}

pub async fn process_client_index(
    connection: &mut Connection,
    client_index: HashMap<String, usize>,
    arc_sync_job: Arc<Mutex<Job>>,
    source_of_truth: SourceOfTruth,
) {
    // Get server index
    let sync_job = arc_sync_job.lock().await;
    let path = sync_job.params.get("path").unwrap();
    let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + path;

    let mut server_index_manager = IndexManager::new(full_path.clone());
    server_index_manager.build().await;

    // Get index comparer
    debug!(
        "Server: {:?}. Client: {:?}",
        server_index_manager.index, client_index
    );
    let index_comparer = IndexComparer::new(
        client_index.clone(),
        server_index_manager.index,
        source_of_truth,
        SERVER_DELETES.lock().await.clone(),
    );
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

            // Add to global deletes
            let mut server_deletes = SERVER_DELETES.lock().await;
            // Let get current unix timestamp
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Failed to get system time");

            let unix_timestamp = current_time.as_secs();
            server_deletes.insert(key.clone(), unix_timestamp as usize);

            // Broadcast delete to all clients
            let data = connection.data.lock().await;
            let sender = &data.connections_broadcast_sender.lock().await;
            sender.send(ConnectionUpdate::new(data.uuid.clone())).expect("Failed to send connection update");
        }
    }

    // Save in last index in connection data
    let mut data = connection.data.lock().await;
    data.last_index = client_index.clone();
    drop(data);

    // Send changes to client
    let mut response = ControlPacket::new(ControlPacketType::SyncUpdate, changes);
    response.job_id = Some(sync_job.id);

    connection.send_packet(response).await.unwrap();
    connection.flush().await;
}
