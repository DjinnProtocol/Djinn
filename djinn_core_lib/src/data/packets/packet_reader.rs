use async_std::{
    io::{BufReader, ReadExt},
    net::TcpStream,
};

use crate::data::packets::packet::{deserialize_packet, get_packet_length};

use super::packet::Packet;

pub struct PacketReader {
    pre_buffer: Vec<u8>,
}

impl PacketReader {
    pub fn new() -> PacketReader {
        return PacketReader {
            pre_buffer: Vec::new(),
        };
    }

    pub async fn read(
        &mut self,
        stream: &mut TcpStream,
        packets: &mut Vec<Box<dyn Packet>>,
    ) -> usize {
        let mut packets_count = 0;
        let mut reader = BufReader::new(stream);

        loop {
            let mut buffer = vec![0; 65535];
            debug!("Listening for packets...");
            let bytes_read = reader.read(&mut buffer).await.unwrap();
            let mut filled_buffer = buffer[..bytes_read].to_vec();
            // debug!("Buffer received: {:?}", String::from_utf8(filled_buffer.clone()));

            if bytes_read == 0 {
                return packets_count;
            }

            //Loop through possible packets
            loop {
                //Combine pre_buffer and buffer
                let full_buffer = self
                    .pre_buffer
                    .clone()
                    .into_iter()
                    .chain(filled_buffer.to_vec().into_iter())
                    .collect::<Vec<u8>>();
                filled_buffer.clear();
                //Get the length of the packet
                let packet_length = get_packet_length(&full_buffer);

                if full_buffer.len() < packet_length.try_into().unwrap() {
                    //Not enough data to read packet
                    self.pre_buffer = full_buffer;
                    break;
                } else {
                    //Read the packet
                    let packet_buffer = full_buffer[..packet_length.try_into().unwrap()].to_vec();

                    //Deserialize the packet
                    let boxed_packet = deserialize_packet(&packet_buffer);
                    packets.push(boxed_packet);
                    packets_count += 1;

                    //Remove the packet from the buffer
                    self.pre_buffer = full_buffer[packet_length.try_into().unwrap()..].to_vec();

                    if self.pre_buffer.len() == 0 {
                        break;
                    }
                }
            }

            if packets_count > 0 || self.pre_buffer.len() == 0 {
                break;
            }
        }
        packets_count
    }
}
