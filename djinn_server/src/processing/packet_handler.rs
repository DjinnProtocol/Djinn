use djinn_core_lib::{data::packets::{packet::{Packet}, PacketType, ControlPacketType, ControlPacket, DataPacket}, jobs::JobStatus};
use filetime::{FileTime, set_file_mtime};
use tokio::fs::{File, rename};
use tokio::io::AsyncWriteExt;

use crate::{connectivity::{Connection, ConnectionUpdate}, CONFIG, syncing::SourceOfTruth};

use super::control_commands::{EchoRequestCommand, ControlCommand, TransferRequestCommand, TransferStartCommand, SyncIndexUpdateCommand, SyncRequestCommand};



pub struct PacketHandler {

}

impl PacketHandler {
    pub async fn handle_boxed_packet<'a>(&self, boxed_packet: Box<dyn Packet + 'a>, connection: &mut Connection) {
        let packet_ref: &dyn Packet = boxed_packet.as_ref();

        match packet_ref.get_packet_type() {
            PacketType::Control => {
                debug!("Control packet received");
                let control_packet = packet_ref.as_any().downcast_ref::<ControlPacket>().unwrap();
                self.handle_control_packet(&control_packet, connection).await;
            },
            PacketType::Data => {
                // Throw error
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
                let command = SyncIndexUpdateCommand {
                    source_of_truth: SourceOfTruth::Server
                };
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
                let command = SyncIndexUpdateCommand {
                    source_of_truth: SourceOfTruth::Client
                };
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

        // If Job is still pending, create file
        if matches!(job.status, JobStatus::Pending) {
            // Throw error
            let file_path = job.params.get("file_path").unwrap();
            let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + file_path;

            // Open da file
            job.open_file = Some(File::create(full_path + ".djinn_temp").await.unwrap());

            job.status = JobStatus::Running;
        }

        // If Job is running, write to file
        if matches!(job.status, JobStatus::Running) {
            let file = job.open_file.as_mut().unwrap();
            if packet.has_data {
                file.write_all(&packet.data).await.unwrap();
            } else {
                // Close file
                file.flush().await.unwrap();
                job.status = JobStatus::Finished;

                // Rename file
                let file_path = job.params.get("file_path").unwrap();
                let full_path = CONFIG.serving_directory.clone().unwrap() + "/" + file_path;

                rename(full_path.clone() + ".djinn_temp", &full_path)
                    .await
                    .unwrap();

                // Set file mtime
                let modified_time = job.params.get("modified_time").unwrap();
                let modified_time = modified_time.parse::<u64>().unwrap();
                let file_time = FileTime::from_unix_time(modified_time as i64, 0);
                set_file_mtime(&full_path, file_time).unwrap();

                // Send connection update to all connections
                let data = connection.data.lock().await;
                let sender = &data.connections_broadcast_sender.lock().await;
                sender.send(ConnectionUpdate::new(data.uuid.clone())).expect("Failed to send connection update");

                // Log
                info!("{} -> server: {}", connection.uuid, file_path);
            }
        }
    }
}
