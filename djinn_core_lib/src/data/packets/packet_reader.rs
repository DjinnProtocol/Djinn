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

    pub async fn read(&mut self, reader: &mut BufReader<&mut TcpStream>, max_packets: Option<usize>) -> Vec<Box<dyn Packet>> {
        let mut packets: Vec<Box<dyn Packet>> = Vec::with_capacity(10);

        while packets.len() == 0 {
            let mut temp_buffer = [0; 65536];
            let bytes_read = reader.read(&mut temp_buffer).await.unwrap();

            debug!("Bytes read: {}", bytes_read);

            if bytes_read == 0 {
                return packets;
            }

            //Add to self buffer
            self.buffer.extend_from_slice(&temp_buffer[0..bytes_read]);

            //If buffer has more than 4 bytes and more packets need to be read
            while self.buffer.len() > 4 && (max_packets.is_none() || packets.len() < max_packets.unwrap()) {
                //Check if packet is complete
                let packet_length = get_packet_length(&self.buffer) as usize;

                //Check if packet can be extracted
                if packet_length <= self.buffer.len() {
                    // Deserialize packet
                    let packet_vec = self.buffer[0..packet_length].to_vec();
                    let packet = deserialize_packet(&packet_vec);

                    //Remove packet from buffer
                    self.buffer.drain(0..packet_length);

                    //Add packet to packets
                    packets.push(packet);
                } else {
                    break;
                }
            }
        }

        return packets;
    }
}
