use std::error::Error;
use async_trait::async_trait;
use djinn_core_lib::data::packets::DataPacketGenerator;
use djinn_core_lib::data::packets::{ControlPacket, packet::Packet};
use djinn_core_lib::jobs::{JobStatus, JobType};
use tokio::io::{BufWriter, AsyncWriteExt};

use crate::{connectivity::Connection, CONFIG};

use super::ControlCommand;

pub struct TransferStartCommand {}

#[async_trait]
impl ControlCommand for TransferStartCommand {
    async fn execute(&self, connection: &mut Connection, packet: &ControlPacket) -> Result<(), Box<dyn Error>> {
        // Get the file path from the packet
        let job_id = packet.params.get("job_id").unwrap().parse::<u32>().unwrap();

        // Get the job from the connection
        let sync_job_arc = connection.get_job(job_id).await.unwrap().clone();
        let job = sync_job_arc.lock().await;

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
        let packet_generator = DataPacketGenerator::new(job_id, full_path);
        let iterator = packet_generator.iter();

        // Get connection read_stream
        let write_stream_arc = connection.get_write_stream().await;
        let mut write_stream = write_stream_arc.lock().await;
        // let mut writer = BufWriter::new(&mut *write_stream);
        // TODO: check speed difference between BufWriter and and native stream

        for packet in iterator {
            let buffer = &packet.to_buffer();
            write_stream.write_all(&buffer).await?;
            //Log first 4 bytes
            // debug!("Packet length: {}", u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]));
            // debug!("Actual Length: {}", buffer.len());

            // debug!("sent: {:?}", String::from_utf8(packet.data.clone()));
        }

        write_stream.flush().await?;

        debug!("Done sending data");
        return Ok(());
    }
}
