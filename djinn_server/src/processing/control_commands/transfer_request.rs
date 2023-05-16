use std::{collections::HashMap, error::Error};
use async_trait::async_trait;
use djinn_core_lib::{data::packets::{ControlPacket, ControlPacketType, TransferDenyReason}, jobs::{Job, JobType, JobStatus}};
use tokio::fs;

use crate::{connectivity::Connection, CONFIG};

use super::ControlCommand;

pub struct TransferRequestCommand {}

#[async_trait]
impl ControlCommand for TransferRequestCommand {
    async fn execute(&self, connection: &mut Connection, packet: &ControlPacket) -> Result<(), Box<dyn Error>> {
        let path = packet.params.get("file_path").unwrap();
        let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + path;


        //Check if file exists if download request
        if !fs::metadata(&full_path).await.is_ok() {
            let mut params = HashMap::new();
            params.insert("reason".to_string(), TransferDenyReason::FileNotFound.to_string());

            let response = ControlPacket::new(ControlPacketType::TransferDeny, params);

            connection.send_packet(response).await?;

            return Ok(());
        }

        //Create job
        let job_id = connection.new_job_id().await;
        let job = Job {
            id: job_id.clone(),
            job_type: JobType::Transfer,
            status: JobStatus::Pending,
            params: packet.params.clone()
        };

        connection.add_job(job).await;

        // Get modified time
        let modified_time = fs::metadata(&full_path).await?.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        //Send response
        let mut response = ControlPacket::new(ControlPacketType::TransferAck, HashMap::new());
        response.params.insert("job_id".to_string(), job_id.to_string());
        response.params.insert("transfer_id".to_string(), packet.params.get("transfer_id").unwrap().clone());
        response.params.insert("modified_time".to_string(), modified_time.to_string());

        connection.send_packet(response).await?;

        connection.flush().await;

        return Ok(());
    }
}
