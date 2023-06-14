use async_trait::async_trait;
use djinn_core_lib::{
    data::packets::{ControlPacket, ControlPacketType, TransferDenyReason},
    jobs::{Job, JobStatus, JobType},
};
use std::{collections::HashMap, error::Error};
use tokio::{fs, time::sleep};

use crate::{connectivity::Connection, CONFIG};

use super::ControlCommand;

pub struct SyncRequestCommand {}

#[async_trait]
impl ControlCommand for SyncRequestCommand {
    async fn execute(
        &self,
        connection: &mut Connection,
        packet: &ControlPacket,
    ) -> Result<(), Box<dyn Error>> {
        let path = packet.params.get("path").unwrap();
        let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + path;

        //Check if file exists if download request
        if !fs::metadata(full_path).await.is_ok() {
            let mut params = HashMap::new();
            params.insert(
                "reason".to_string(),
                TransferDenyReason::FileNotFound.to_string(),
            );

            let response_packet = ControlPacket::new(ControlPacketType::SyncDeny, params);
            connection.send_packet(response_packet).await?;
            connection.flush().await;

            return Ok(());
        }

        let job_id = connection.new_job_id().await;
        //Create job
        let job = Job {
            id: job_id.clone(),
            job_type: JobType::Sync,
            status: JobStatus::Pending,
            params: packet.params.clone(),
            open_file: None,
        };

        connection.add_job(job).await;

        //Send response
        let mut response = ControlPacket::new(ControlPacketType::SyncAck, HashMap::new());
        response
            .params
            .insert("job_id".to_string(), job_id.to_string());
        connection.send_packet(response).await?;
        connection.flush().await;

        //Also send index request packet
        let mut index_request_packet =
            ControlPacket::new(ControlPacketType::SyncIndexRequest, HashMap::new());
        index_request_packet.job_id = Some(job_id);
        connection.send_packet(index_request_packet).await?;
        connection.flush().await;

        sleep(std::time::Duration::from_millis(1000)).await;

        debug!("Sent index request packet for sync job {}", job_id);

        return Ok(());
    }
}
