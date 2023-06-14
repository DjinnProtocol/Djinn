use std::{error::Error, sync::Arc};

use super::{ConnectionData, ConnectionUpdate, ConnectionUpdateType};
use crate::{
    processing::PacketHandler,
    syncing::{ClientIndexHandler, SourceOfTruth},
};
use djinn_core_lib::{
    data::packets::{packet::Packet, PacketReader},
    jobs::{Job, JobType},
};
use tokio::{
    io::{AsyncWriteExt, BufReader, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};
use uuid::Uuid;

pub struct Connection {
    pub uuid: Uuid,
    pub data: Arc<Mutex<ConnectionData>>,
}

impl Connection {
    pub fn new(uuid: Uuid, data: Arc<Mutex<ConnectionData>>) -> Connection {
        Connection { uuid, data }
    }

    pub async fn new_job_id(&mut self) -> u32 {
        let mut data = self.data.lock().await;
        data.new_job_id += 1;
        data.new_job_id
    }

    pub async fn get_job(&mut self, job_id: u32) -> Option<Arc<Mutex<Job>>> {
        let mut data = self.data.lock().await;

        for job in &mut data.jobs {
            let unlocked_job = job.lock().await;
            if unlocked_job.id == job_id {
                return Some(job.clone());
            }
        }

        None
    }

    pub async fn add_job(&mut self, job: Job) {
        let mut data = self.data.lock().await;
        data.jobs.push(Arc::new(Mutex::new(job)));
    }

    pub async fn listen(&mut self) {
        let data_arc = self.data.clone();
        let data = data_arc.lock().await;
        let connection_uuid = data.uuid;
        let read_stream_arc = data.read_stream.clone();
        drop(data);

        // Listen for broadcast in separate async task
        tokio::spawn(async move {
            let mut new_connection = Connection::new(connection_uuid, data_arc);
            new_connection.listen_for_broadcasts().await;
        });

        info!("Listening for packets on connection {}", connection_uuid);

        // Handle incoming streams
        let mut packet_reader = PacketReader::new();
        loop {
            let mut read_stream = read_stream_arc.lock().await;
            let mut reader = BufReader::new(&mut *read_stream);
            let packets = packet_reader.read(&mut reader, None).await;

            if packets.is_empty() {
                // Connection closed
                info!("Connection closed for {}", connection_uuid);
                break;
            }

            for packet in packets {
                let packet_handler = PacketHandler {};
                packet_handler.handle_boxed_packet(packet, self).await;
            }
        }
    }

    pub async fn flush(&mut self) {
        let data = self.data.lock().await;
        let write_stream_arc = data.write_stream.clone();
        let mut write_stream = write_stream_arc.lock().await;
        write_stream.flush().await.unwrap();
    }

    pub async fn send_packet(&mut self, packet: impl Packet) -> Result<(), Box<dyn Error>> {
        // Convert packet to buffer
        let buffer = packet.to_buffer();
        // Write buffer to stream
        let data = self.data.lock().await;
        let write_stream_arc = data.write_stream.clone();
        let mut write_stream = write_stream_arc.lock().await;
        write_stream.write_all(&buffer).await?;
        debug!("Written stuff");

        Ok(())
    }

    pub async fn get_write_stream(&mut self) -> Arc<Mutex<WriteHalf<TcpStream>>> {
        let data = self.data.lock().await;
        data.write_stream.clone()
    }

    pub async fn listen_for_broadcasts(&mut self) {
        // Lock data to get broadcast receiver
        let data = self.data.lock().await;
        let broadcast_receiver_arc = data.connections_broadcast_receiver.clone();
        drop(data);

        loop {
            debug!("Waiting for broadcast");
            let mut broadcast_receiver = broadcast_receiver_arc.lock().await;
            let broadcast = broadcast_receiver.recv().await;
            drop(broadcast_receiver);
            debug!("Received broadcast");

            match broadcast {
                Ok(broadcast) => {
                    self.handle_connection_update(broadcast).await;
                }
                Err(_) => {
                    // Connection closed
                    debug!("Connection closed");
                    break;
                }
            }
        }
    }

    pub async fn handle_connection_update(&mut self, connection_update: ConnectionUpdate) {
        match connection_update.update_type {
            ConnectionUpdateType::ServerIndexUpdated => {
                let mut data = self.data.lock().await;
                if data.uuid == connection_update.connection_uuid {
                    // Ignore own broadcast
                    return;
                }
                //Find active sync job
                let mut arc_sync_job = None;
                for job in &mut data.jobs {
                    let unlocked_job = job.lock().await;
                    if matches!(unlocked_job.job_type, JobType::Sync) {
                        arc_sync_job = Some(job.clone());
                        break;
                    }
                }

                let last_index = data.last_index.clone();
                drop(data);

                if arc_sync_job.is_some() {
                    debug!("Processing client index");
                    let client_index_handler = ClientIndexHandler::new(
                        last_index,
                        arc_sync_job.unwrap(),
                        SourceOfTruth::Server,
                    );
                    client_index_handler.handle(self).await;
                }
            }
        }
    }
}
