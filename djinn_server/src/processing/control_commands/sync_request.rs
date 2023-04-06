use std::{collections::HashMap, error::Error};
use async_std::fs;

use async_std::io::WriteExt;
use async_trait::async_trait;
use djinn_core_lib::data::packets::{ControlPacket, ControlPacketType, packet::Packet, TransferDenyReason};

use crate::{connectivity::Connection, CONFIG, jobs::{Job, JobType, JobStatus}};

use super::ControlCommand;

pub struct SyncRequestCommand {}

#[async_trait]
impl ControlCommand for SyncRequestCommand {
    async fn execute(&self, connection: &mut Connection, packet: &ControlPacket) -> Result<(), Box<dyn Error>> {
        let path = packet.params.get("path").unwrap();
        let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + path;


        //Check if file exists if download request
        if !fs::metadata(full_path).await.is_ok() {
            let mut params = HashMap::new();
            params.insert("reason".to_string(), TransferDenyReason::FileNotFound.to_string());

            let response = ControlPacket::new(ControlPacketType::SyncDeny, params);
            connection.stream.write(&response.to_buffer()).await.unwrap();

            return Ok(());
        }

        //Create job
        let job = Job {
            id: connection.new_job_id(),
            job_type: JobType::Sync,
            status: JobStatus::Pending,
            params: packet.params.clone()
        };

        connection.jobs.push(job.clone());

        //Send response
        let mut response = ControlPacket::new(ControlPacketType::SyncAck, HashMap::new());
        response.params.insert("job_id".to_string(), job.id.to_string());

        connection.send_packet(response).await?;

        //Also send index request packet
        let mut index_request_packet = ControlPacket::new(ControlPacketType::SyncIndexRequest, HashMap::new());
        index_request_packet.job_id = Some(job.id);
        connection.send_packet(index_request_packet).await?;

        connection.stream.flush().await.unwrap();

        debug!("Sent index request packet for sync job {}", job.id);

        return Ok(());
    }
}
