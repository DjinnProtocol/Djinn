use std::{
    collections::HashMap,
    sync::{Arc}, time::{SystemTime, UNIX_EPOCH}, error::Error,
};

use djinn_core_lib::{data::{packets::{ControlPacket, ControlPacketType}, syncing::IndexManager}, jobs::Job};
use tokio::{fs, sync::Mutex};

use crate::{connectivity::{Connection, ConnectionUpdate}, CONFIG, SERVER_DELETES};

use super::{SourceOfTruth, IndexComparer};

pub struct ClientIndexHandler {
    client_index: HashMap<String, usize>,
    arc_sync_job: Arc<Mutex<Job>>,
    source_of_truth: SourceOfTruth
}

impl ClientIndexHandler {
    pub fn new(client_index: HashMap<String, usize>, arc_sync_job: Arc<Mutex<Job>>, source_of_truth: SourceOfTruth) -> Self {
        Self {
            client_index,
            arc_sync_job,
            source_of_truth
        }
    }

    pub async fn handle(&self, connection: &mut Connection) {
        // Generate changes
        let changes_result = self.generate_changes().await;

        if changes_result.is_err() {
            return; //TODO: Handle error
        }

        let changes = changes_result.unwrap();

        // Process changes
        let client_changes = self.process_changes(&changes, connection).await;

        if client_changes.is_err() {
            return; //TODO: Handle error
        }

        let client_changes = client_changes.unwrap();

        // Send changes to client
        let send_result = self.send_sync_update(connection, &client_changes).await;

        if send_result.is_err() {
            return; //TODO: Handle error
        }

        // Update client index
        self.save_last_index(connection).await;
    }

    async fn generate_changes(
        &self
    ) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>> {
        // Get server index
        let sync_job = self.arc_sync_job.lock().await;
        let path = sync_job.params.get("path").unwrap();
        let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + path;

        let mut server_index_manager = IndexManager::new(full_path.clone());
        server_index_manager.build().await;

        // Get index comparer
        let index_comparer = IndexComparer::new(
            self.client_index.clone(),
            server_index_manager.index,
            self.source_of_truth,
            SERVER_DELETES.lock().await.clone(),
        );

        Ok(index_comparer.compare())
    }

    async fn process_changes(
        &self,
        changes: &HashMap<String, String>,
        connection: &mut Connection
    ) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>> {
        let mut changes_for_client = changes.clone();
        let sync_job = self.arc_sync_job.lock().await;
        let path = sync_job.params.get("path").unwrap();
        let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + path;

        // First proces self deletes
        for (path, type_change) in changes.iter() {
            if type_change == "SELF_DELETE" {
                // Delete file
                let mut full_file_path = full_path.clone() + "/" + path;
                full_file_path = full_file_path.replace("//", "/");

                info!("{} -> server: DEL {}", connection.uuid, path);

                fs::remove_file(full_file_path).await.unwrap();

                // Remove from index
                changes_for_client.remove(path);

                // Add to global deletes
                let mut server_deletes = SERVER_DELETES.lock().await;

                // Let get current unix timestamp
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Failed to get system time");

                let unix_timestamp = current_time.as_secs();
                server_deletes.insert(path.clone(), unix_timestamp as usize);

                // Broadcast delete to all clients
                let data = connection.data.lock().await;
                let sender = &data.connections_broadcast_sender.lock().await;
                sender.send(ConnectionUpdate::new(data.uuid)).expect("Failed to send connection update");
            }
        }

        Ok(changes_for_client)
    }

    async fn send_sync_update(&self, connection: &mut Connection, changes: &HashMap<String, String>) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Build packet
        let mut response = ControlPacket::new(ControlPacketType::SyncUpdate, changes.clone());
        let sync_job = self.arc_sync_job.lock().await;
        response.job_id = Some(sync_job.id);
        // Send packet
        connection.send_packet(response).await.unwrap();
        connection.flush().await;

        Ok(())
    }

    async fn save_last_index(&self, connection: &mut Connection) {
        // Save in last index in connection data
        let mut data = connection.data.lock().await;
        data.last_index = self.client_index.clone();
        drop(data);
    }
}
