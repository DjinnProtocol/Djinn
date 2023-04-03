use async_std::{
    io::{BufReader, ReadExt},
    net::TcpStream,
    path::Iter,
};

use crate::data::packets::packet::{deserialize_packet, get_packet_length};

use super::packet::Packet;

pub struct PacketReader {
    buffer: Vec<u8>
}

impl PacketReader {
    pub fn new() -> PacketReader {
        return PacketReader {
            buffer: Vec::with_capacity(131072)
        };
    }

    pub async fn read(&mut self, reader: &mut BufReader<&mut TcpStream>) -> Vec<Box<dyn Packet>> {
        let mut packets: Vec<Box<dyn Packet>> = Vec::with_capacity(10);

        while packets.len() == 0 {
            let mut temp_buffer = [0; 65536];
            let bytes_read = reader.read(&mut temp_buffer).await.unwrap();

            if bytes_read == 0 {
                return packets;
            }

            // let received = String::from_utf8(temp_buffer[0..bytes_read].to_vec()).unwrap();
            // debug!("Reveived: {:?}", received);

            //Add to self buffer
            self.buffer.extend_from_slice(&temp_buffer[0..bytes_read]);

            while self.buffer.len() > 0 {
                //Check if packet is complete
                let packet_length = get_packet_length(&self.buffer) as usize;

                if packet_length <= self.buffer.len() {
                    // Deserialize packet
                    let packet_vec = self.buffer[0..packet_length].to_vec();
                    let packet = deserialize_packet(&packet_vec);

                    //Remove packet from buffer
                    self.buffer.drain(0..packet_length);

                    packets.push(packet);
                } else {
                    break;
                }
            }
        }

        return packets;
    }
}
