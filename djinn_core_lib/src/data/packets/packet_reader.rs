use async_std::{
    io::{BufReader, ReadExt},
    net::TcpStream,
    path::Iter,
};

use crate::data::packets::packet::{deserialize_packet, get_packet_length};

use super::packet::Packet;

pub struct PacketReader {
    buffer: Vec<u8>,
    byte_count: usize
}

impl PacketReader {
    pub fn new() -> PacketReader {
        return PacketReader {
            buffer: vec![0; 65536],
            byte_count: 0
        };
    }

    pub async fn read(&mut self, reader: &mut BufReader<&mut TcpStream>) -> Option<Box<dyn Packet>> {
        loop {
            let mut temp_buffer = vec![0; 65536];
            let bytes_read = reader.read(&mut temp_buffer).await.unwrap();

            if bytes_read == 0 {
                return None;
            }

            //Add to self buffer
            for i in 0..bytes_read {
                self.buffer[self.byte_count] = temp_buffer[i];
                self.byte_count += 1;
            }

            //Check if packet is complete
            let packet_length = get_packet_length(&self.buffer) as usize;

            if packet_length <= self.byte_count {
                // Deserialize packet
                let packet_vec = self.buffer[0..packet_length].to_vec();
                let packet = deserialize_packet(&packet_vec);

                //Remove packet from buffer
                for i in 0..(self.byte_count - packet_length) {
                    self.buffer[i] = self.buffer[i + packet_length];
                }

                //Update byte count
                self.byte_count -= packet_length;

                return Some(packet);
            }
        }
    }
}
