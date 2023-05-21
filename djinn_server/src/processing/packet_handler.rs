use djinn_core_lib::{data::packets::{packet::{Packet, self}, PacketType, ControlPacketType, ControlPacket, DataPacket}, jobs::JobStatus};
use filetime::{FileTime, set_file_mtime};
use tokio::fs::{File, rename};
use tokio::io::AsyncWriteExt;

use crate::{connectivity::Connection, CONFIG};

use super::control_commands::{EchoRequestCommand, ControlCommand, TransferRequestCommand, TransferStartCommand, SyncIndexResponseCommand, SyncRequestCommand, SyncIndexUpdateCommand};



pub struct PacketHandler {

}

impl PacketHandler {
    pub async fn handle_boxed_packet<'a>(&self, boxed_packet: Box<dyn Packet + 'a>, connection: &mut Connection) {
        let packet_ref: &dyn Packet = boxed_packet.as_ref();

        match packet_ref.get_packet_type() {
            PacketType::Control => {
                info!("Control packet received");
                let control_packet = packet_ref.as_any().downcast_ref::<ControlPacket>().unwrap();

                self.handle_control_packet(&control_packet, connection).await;
            },
            PacketType::Data => {
                // Throw error
                debug!("data packet received");
                let data_packet = packet_ref.as_any().downcast_ref::<DataPacket>().unwrap();

                self.handle_data_packet(&data_packet, connection).await;
            }
        }
    }

    pub async fn handle_control_packet(&self, packet: &ControlPacket, connection: &mut Connection) {
        match packet.control_packet_type {
            ControlPacketType::EchoRequest => {
                let command = EchoRequestCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::TransferRequest => {
                let command = TransferRequestCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::TransferAck => {
                //Transfer reverse for server -> client
            },
            ControlPacketType::SyncIndexResponse => {
                let command = SyncIndexResponseCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::TransferStart => {
                let command = TransferStartCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::SyncRequest => {
                let command = SyncRequestCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::SyncIndexUpdate => {
                let command = SyncIndexUpdateCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            _ => {
                // Throw error
                panic!("Unknown control packet type")
            }
        }
    }

    pub async fn handle_data_packet(&self, packet: &DataPacket, connection: &mut Connection) {
        let job_id = packet.job_id.clone();

        let option_arc_job = connection.get_job(job_id).await;

        if option_arc_job.is_none() {
            // Throw error
            panic!("Job not found");
        }

        let arc_job = option_arc_job.unwrap();
        let mut job = arc_job.lock().await;

        if matches!(job.status, JobStatus::Pending) {
            // Throw error
            let file_path = job.params.get("file_path").unwrap();
            let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + file_path;

            // Open da file
            job.open_file = Some(File::create(full_path + ".djinn_temp").await.unwrap());

            job.status = JobStatus::Running;
        }

        if matches!(job.status, JobStatus::Running) {
            let file = job.open_file.as_mut().unwrap();
            if packet.has_data {
                file.write_all(&packet.data).await.unwrap();
            } else {
                file.flush().await.unwrap();
                job.status = JobStatus::Finished;
                // Move file and set modified time
                let file_path = job.params.get("file_path").unwrap();
                let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + file_path;

                rename(full_path.clone() + ".djinn_temp", &full_path)
                    .await
                    .unwrap();

                let modified_time = job.params.get("modified_time").unwrap();
                let modified_time = modified_time.parse::<u64>().unwrap();
                let file_time = FileTime::from_unix_time(modified_time as i64, 0);
                set_file_mtime(&full_path, file_time).unwrap();
            }
        }
    }
}
