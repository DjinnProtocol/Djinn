use std::{error::Error, sync::Arc};

use crate::{processing::PacketHandler};
use djinn_core_lib::{data::packets::{
    packet::Packet,
    PacketReader,
}, jobs::Job};
use tokio::{sync::Mutex, io::{BufReader, AsyncWriteExt, WriteHalf, ReadHalf}, net::TcpStream};
use super::ConnectionData;

pub struct Connection {
    pub data: Arc<Mutex<ConnectionData>>,
}

impl Connection {
    pub fn new(data: Arc<Mutex<ConnectionData>>) -> Connection {
        Connection {
            data
        }
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
                return Some(job.clone())
            }
        }

        None
    }

    pub async fn add_job(&mut self, job: Job) {
        let mut data = self.data.lock().await;
        data.jobs.push(Arc::new(Mutex::new(job)));
    }

    pub async fn listen(&mut self) {
        // Handle incoming streams
        let mut packet_reader = PacketReader::new();
        loop {
            let data = self.data.lock().await;
            let read_stream_arc = data.read_stream.clone();
            let mut read_stream = read_stream_arc.lock().await;
            let mut reader = BufReader::new(&mut *read_stream);
            let packets = packet_reader.read(&mut reader, None).await;
            std::mem::drop(data);

            if packets.len() == 0 {
                // Connection closed
                debug!("Connection closed");
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

    pub async fn get_read_stream(&mut self) -> Arc<Mutex<ReadHalf<TcpStream>>> {
        let data = self.data.lock().await;
        data.read_stream.clone()
    }
}
