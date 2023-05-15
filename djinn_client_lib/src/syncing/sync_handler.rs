use std::{collections::HashMap, error::Error, sync::Arc};

use djinn_core_lib::data::{
    packets::{packet::Packet, ControlPacket, ControlPacketType, PacketType},
    syncing::IndexManager,
};
use tokio::{time::sleep, io::{ReadHalf, BufReader}, net::TcpStream, sync::Mutex};

use crate::{connectivity::Connection};

pub struct SyncHandler {
    pub path: String,
    pub target: String,
    pub job_id: Option<u32>,
    pub polling_ready: bool,
}

impl SyncHandler {
    pub fn new(path: String, target: String) -> SyncHandler {
        SyncHandler {
            path,
            target,
            job_id: None,
            polling_ready: false,
        }
    }

    pub async fn start(&mut self, connection: &mut Connection) -> Result<(), Box<dyn Error>> {
        //Ask the server to start syncing
        debug!("Sending sync request");

        let mut params = HashMap::new();
        params.insert("path".to_string(), self.path.clone());

        let packet = ControlPacket::new(ControlPacketType::SyncRequest, params);
        connection.send_packet(packet).await?;

        //Wait for the server to send the ack
        debug!("Waiting for sync ack");
        let boxed_packet = connection.read_next_packet().await?.unwrap();

        if !matches!(boxed_packet.get_packet_type(), PacketType::Control) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected packet type",
            )));
        }

        let control_packet = boxed_packet
            .as_any()
            .downcast_ref::<ControlPacket>()
            .unwrap();

        if matches!(
            control_packet.control_packet_type,
            ControlPacketType::SyncDeny
        ) {
            let reason = control_packet.params.get("reason").unwrap();
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Transfer denied: {}", reason),
            )));
        }

        if !matches!(
            control_packet.control_packet_type,
            ControlPacketType::SyncAck
        ) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected control packet type",
            )));
        }

        let job_id = control_packet
            .params
            .get("job_id")
            .unwrap()
            .parse::<u32>()
            .unwrap();
        self.job_id = Some(job_id);

        debug!("Received sync ack");

        //Start listening to the server for updates and commands
        debug!("Starting to listen for updates");

        let reader_arc = connection.reader.clone();
        let write_stream_arc = connection.write_stream.clone();
        let new_target = self.target.clone();
        let new_job_id = self.job_id.clone().unwrap();

        // tokio::spawn (async move {
        //     let mut fs_poller = FsPoller::new(new_target, new_job_id);
        //     fs_poller.poll(write_stream_arc).await.unwrap();
        // });

        self.listen(reader_arc, connection).await?;
        Ok(())
    }

    async fn listen(&self, reader: Arc<Mutex<Option<BufReader<ReadHalf<TcpStream>>>>>, connection: &mut Connection) -> Result<(), Box<dyn Error>> {
        // Handle incoming streams
        let mut reader_option = reader.lock().await;
        let reader = reader_option.as_mut().unwrap();

        loop {
            let packets = connection.packet_reader.read2(reader, None).await;

            if packets.len() == 0 {
                // Connection closed
                debug!("Connection closed");
                break;
            }

            for packet in packets {
                self.handle_boxed_packet(packet, &connection).await;
            }
        }

        Ok(())
    }

    pub async fn handle_boxed_packet<'a>(
        &self,
        boxed_packet: Box<dyn Packet + 'a>,
        connection: &Connection,
    ) {
        let packet_ref: &dyn Packet = boxed_packet.as_ref();

        match packet_ref.get_packet_type() {
            PacketType::Control => {
                info!("Control packet received");
                let control_packet = packet_ref.as_any().downcast_ref::<ControlPacket>().unwrap();

                self.handle_control_packet(&control_packet, connection)
                    .await;
            }
            PacketType::Data => {
                // Throw error
                panic!("Data packets are not supported yet")
            }
        }
    }

    pub async fn handle_control_packet(&self, packet: &ControlPacket, connection: &Connection) {
        match packet.control_packet_type {
            ControlPacketType::SyncIndexRequest => {
                info!("Sync index request received");

                let full_path = self.target.clone();
                let mut index_manager = IndexManager::new(full_path);
                index_manager.build().await;

                let mut params = HashMap::new();
                let index = index_manager.index;

                //Stringify the timestamps
                for (key, value) in index.iter() {
                    params.insert(key.clone(), value.to_string());
                }

                let mut packet = ControlPacket::new(ControlPacketType::SyncIndexResponse, params);

                packet.job_id = self.job_id;

                connection.send_packet(packet).await.unwrap();
            }
            ControlPacketType::SyncUpdate => {
                info!("Sync update received");
            }
            _ => {
                //Log type
                debug!(
                    "Unknown control packet type: {:?}",
                    packet.control_packet_type as u8
                );
                // Throw error
                panic!("Unknown control packet type")
            }
        }
    }

    pub async fn poll_file_changes(&self, connection: &Connection) -> Result<(), Box<dyn Error>> {
        let mut index_manager = IndexManager::new(self.target.clone());

        loop {
            index_manager.build().await;

            let mut params = HashMap::new();
            let index = &index_manager.index;

            //Stringify the timestamps
            for (key, value) in index.iter() {
                params.insert(key.clone(), value.to_string());
            }

            let mut packet = ControlPacket::new(ControlPacketType::SyncUpdate, params);

            packet.job_id = self.job_id;

            connection.send_packet(packet).await.unwrap();

            sleep(std::time::Duration::from_secs(1)).await;
        }

        Ok(())
    }
}
