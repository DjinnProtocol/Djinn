use async_trait::async_trait;
use djinn_core_lib::data::packets::{DataPacketGenerator, ControlPacketType};
use djinn_core_lib::data::packets::{packet::Packet, ControlPacket};
use djinn_core_lib::jobs::{Job, JobStatus, JobType};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt};
use tokio::sync::Mutex;

use crate::{connectivity::Connection, CONFIG};

use super::ControlCommand;

pub struct TransferStartCommand {}

#[async_trait]
impl ControlCommand for TransferStartCommand {
    async fn execute(
        &self,
        connection: &mut Connection,
        packet: &ControlPacket,
    ) -> Result<(), Box<dyn Error>> {
        // Get job
        let arc_job_result = self.get_linked_job(connection, packet).await;

        if arc_job_result.is_err() {
            return Ok(()); //TODO: Handle error
        }

        let unwrapped_arc_job = arc_job_result.unwrap();
        let job = unwrapped_arc_job.lock().await;
        let file_path = job.params.get("file_path").unwrap().clone();

        // If the job is not in the pending state, return an error
        if !matches!(job.status, JobStatus::Pending) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Job is not in the pending state",
            )));
        }

        drop(job);

        // Transfer the file
        let push_result = self.push_file(connection, unwrapped_arc_job).await;

        if push_result.is_err() {
            return Ok(()); //TODO: Handle error
        }

        info!("server -> {}: {}", connection.uuid, file_path);

        return Ok(());
    }
}

impl TransferStartCommand {
    async fn get_linked_job(
        &self,
        connection: &mut Connection,
        packet: &ControlPacket,
    ) -> Result<Arc<Mutex<Job>>, Box<dyn Error + Send + Sync>> {
        // Get the file path from the packet
        let job_id = packet.params.get("job_id").unwrap().parse::<u32>().unwrap();

        // Get the job from the connection
        let option_sync_job_arc: Option<Arc<Mutex<Job>>> = connection.get_job(job_id).await;

        if option_sync_job_arc.is_none() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Job does not exist",
            )));
        }

        let job_arc = option_sync_job_arc.unwrap();
        let job = job_arc.lock().await;

        // If the job is not a transfer job, return an error
        if !matches!(job.job_type, JobType::Transfer) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Job is not a transfer job",
            )));
        }

        Ok(job_arc.clone())
    }

    async fn push_file(
        &self,
        connection: &mut Connection,
        arc_job: Arc<Mutex<Job>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut job = arc_job.lock().await;
        let job_id = job.id;
        // Set the job status to running
        job.status = JobStatus::Running;
        // Get the file path from the job
        let file_path = job.params.get("file_path").unwrap().clone();
        let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + &file_path;

        drop(job);

        // Open da file
        let packet_generator = DataPacketGenerator::new(job_id, full_path);
        let iterator = packet_generator.iter();

        // Get connection read_stream
        let write_stream_arc = connection.get_write_stream().await;
        let mut write_stream = write_stream_arc.lock().await;

        let mut canceled = false;

        for packet in iterator {
            // Write packet
            let buffer = &packet.to_buffer();
            write_stream.write_all(buffer).await?;

            // Check if the job has been canceled every 5 packets
            if packet.packet_number % 5 == 0 {
                let job = arc_job.lock().await;

                if matches!(job.status, JobStatus::Canceled) {
                    canceled = true;
                    break;
                }
            }
        }
        if canceled {
           // Send transfer cancel packet
            let mut params = HashMap::new();
            params.insert("job_id".to_string(), job_id.to_string());
            let packet = ControlPacket::new(ControlPacketType::TransferCancel, params);
            let buffer = &packet.to_buffer();

            write_stream.write_all(buffer).await?;
        } else {
            // Set the job status to complete
            let mut job = arc_job.lock().await;
            job.status = JobStatus::Finished;
        }

        // Flush the write stream
        write_stream.flush().await?;

        Ok(())
    }

}
