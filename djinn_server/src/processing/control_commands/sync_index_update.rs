use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use djinn_core_lib::{
    data::packets::ControlPacket,
    jobs::{Job, JobType},
};
use tokio::sync::Mutex;

use crate::{
    connectivity::Connection,
    syncing::{ClientIndexHandler, SourceOfTruth},
};

use super::ControlCommand;

pub struct SyncIndexUpdateCommand {
    pub source_of_truth: SourceOfTruth,
}

#[async_trait]
impl ControlCommand for SyncIndexUpdateCommand {
    async fn execute(
        &self,
        connection: &mut Connection,
        packet: &ControlPacket,
    ) -> Result<(), Box<dyn Error>> {
        // Get sync job
        let arc_sync_job_result = self.get_linked_sync_job(connection, packet).await;

        if arc_sync_job_result.is_err() {
            return Ok(()); //TODO: Handle error
        }

        let unwrapped_arc_sync_job = arc_sync_job_result.unwrap();

        // Unwrap client index
        let mut client_index = HashMap::new();

        for (key, value) in packet.params.iter() {
            client_index.insert(key.clone(), value.parse::<usize>().unwrap());
        }

        // Handle sync index response
        let client_index_handler = ClientIndexHandler::new(
            client_index,
            unwrapped_arc_sync_job.clone(),
            self.source_of_truth.clone(),
        );
        client_index_handler.handle(connection).await;

        Ok(())
    }
}

impl SyncIndexUpdateCommand {
    async fn get_linked_sync_job(
        &self,
        connection: &mut Connection,
        packet: &ControlPacket,
    ) -> Result<Arc<Mutex<Job>>, Box<dyn Error + Send + Sync>> {
        //Check if job id exists
        if packet.job_id.is_none() {
            return Err("SyncIndexResponse packet does not contain a job id".into());
        }

        //Check if sync exists
        let possible_sync_job = connection.get_job(packet.job_id.unwrap()).await;

        if possible_sync_job.is_none() {
            return Err("SyncIndexResponse packet does not contain a valid job id".into());
        }

        //Check if sync is a sync job
        let sync_job_arc = possible_sync_job.unwrap();
        let sync_job = sync_job_arc.lock().await;

        if !matches!(sync_job.job_type, JobType::Sync) {
            return Err("SyncIndexResponse packet does not contain a valid job id".into());
        }

        drop(sync_job);

        Ok(sync_job_arc)
    }
}
