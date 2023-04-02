use std::{collections::HashMap, error::Error};
use async_std::fs::{self, File};


use async_std::io::{WriteExt, BufReader, ReadExt};
use async_std::stream::StreamExt;
use async_trait::async_trait;
use djinn_core_lib::data::packets::DataPacket;
use djinn_core_lib::data::packets::{ControlPacket, PacketType, ControlPacketType, packet::Packet, TransferDenyReason};

use crate::{connectivity::Connection, CONFIG, jobs::{Job, JobType, JobStatus}};

use super::ControlCommand;

pub struct TransferStartCommand {}

#[async_trait]
impl ControlCommand for TransferStartCommand {
    async fn execute(&self, connection: &mut Connection, packet: &ControlPacket) -> Result<(), Box<dyn Error>> {
        // Get the file path from the packet
        let job_id = packet.params.get("job_id").unwrap().parse::<u32>().unwrap();

        // Get the job from the connection
        let job = connection.get_job(job_id).unwrap();

        // If the job is not a transfer job, return an error
        if !matches!(job.job_type, JobType::Transfer) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Job is not a transfer job",
            )));
        }

        // If the job is not in the pending state, return an error
        if !matches!(job.status, JobStatus::Pending) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Job is not in the pending state",
            )));
        }

        // Get the file path from the job
        let file_path = job.params.get("file_path").unwrap();
        let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + file_path;

        // Open da file
        let file = File::open(full_path).await?;
        let mut bytes_iterator = BufReader::new(file).bytes();
        let mut buffer = Vec::new();

        // Send the file to the client
        while let Some(byte) = bytes_iterator.next().await {
            let byte = byte?;
            buffer.push(byte);
            if buffer.len() == 60000 {
                debug!("Sending data packet normally");
                let packet = DataPacket::new(job_id, buffer.clone());
                connection.send_packet(packet).await?;
                debug!("Sent data packet");
                buffer.clear();
            }
        }

        // Send the last packet
        if buffer.len() > 0 {
            let packet = DataPacket::new(job_id, buffer.clone());
            connection.send_packet(packet).await?;
            debug!("Sent last data packet");
        }

        // Send the end packet
        let packet = DataPacket::new(job_id, vec![]);
        connection.send_packet(packet).await?;

        connection.stream.flush().await?;

        return Ok(());
    }
}
