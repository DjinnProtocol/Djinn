use async_std::{net::TcpStream, io::{BufReader, prelude::BufReadExt}};
use djinn_core_lib::data::packets::packet::deserialize_packet;
use uuid::Uuid;
use crate::{processing::PacketHandler, jobs::Job};

pub struct Connection {
    pub stream: TcpStream,
    pub thread_id: usize,
    pub uuid: Uuid,
    pub jobs: Vec<Job>,
    pub new_job_id: u32
}

impl Connection {
    pub fn new(stream: TcpStream, thread_id: usize) -> Connection {
        Connection {
            stream,
            thread_id,
            uuid: Uuid::new_v4(),
            jobs: vec![],
            new_job_id: 0
        }
    }

    pub fn new_job_id(&mut self) -> u32 {
        self.new_job_id += 1;
        self.new_job_id
    }

    pub async fn listen(&mut self) {
        //Read using bufreader and until function
        loop {
            let mut buffer = vec![];
            let mut reader = BufReader::new(&mut self.stream);
            reader.read_until(b'\n', &mut buffer).await.unwrap();


            if buffer.len() == 0 {
                continue;
            }

            println!("Buffer: {:?}", String::from_utf8(buffer.clone()));

            let packet = deserialize_packet(&buffer);

            let packet_handler = PacketHandler {};
            packet_handler.handle_packet(&packet, self).await;
        }
    }
}
