use crate::{jobs::Job, processing::PacketHandler};
use async_std::{
    io::{prelude::BufReadExt, BufReader, ReadExt, WriteExt},
    net::TcpStream,
};
use djinn_core_lib::data::packets::{
    packet::{self, deserialize_packet, get_packet_length, Packet},
    PacketReader,
};
use uuid::Uuid;

pub struct Connection {
    pub stream: TcpStream,
    pub thread_id: usize,
    pub uuid: Uuid,
    pub jobs: Vec<Job>,
    pub new_job_id: u32,
}

impl Connection {
    pub fn new(stream: TcpStream, thread_id: usize) -> Connection {
        Connection {
            stream,
            thread_id,
            uuid: Uuid::new_v4(),
            jobs: vec![],
            new_job_id: 0,
        }
    }

    pub async fn send_packet(
        &mut self,
        packet: impl Packet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = packet.to_buffer();

        self.stream.write_all(&buffer).await.unwrap();

        Ok(())
    }

    pub fn new_job_id(&mut self) -> u32 {
        self.new_job_id += 1;
        self.new_job_id
    }

    pub fn get_job(&mut self, job_id: u32) -> Option<&mut Job> {
        for job in &mut self.jobs {
            if job.id == job_id {
                return Some(job);
            }
        }

        None
    }

    pub async fn listen(&mut self) {
        // Handle incoming streams
        loop {
            let mut packet_reader = PacketReader::new();
            let mut packets = vec![];
            let amount_read = packet_reader.read(&mut self.stream, &mut packets).await;

            if amount_read == 0 {
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
}
