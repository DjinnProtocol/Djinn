use tokio::{io::{ReadHalf, BufReader, AsyncReadExt, AsyncRead}, net::TcpStream};

use crate::data::packets::packet::{deserialize_packet, get_packet_length};

use super::packet::Packet;

pub struct PacketReader {
    packets_processed: usize,
    buffer: Vec<u8>
}

impl PacketReader {
    pub fn new() -> PacketReader {
        PacketReader {
            packets_processed: 0,
            buffer: Vec::with_capacity(131072)
        }
    }

    pub async fn read<T>(&mut self, reader: &mut BufReader<T>, max_packets: Option<usize>) -> Vec<Box<dyn Packet>>
    where
        T: AsyncRead + Unpin,
    {
        let mut packets: Vec<Box<dyn Packet>> = Vec::with_capacity(10);

        while packets.is_empty() {
            let mut temp_buffer = [0; 65536];
            let bytes_read = reader.read(&mut temp_buffer).await.unwrap();

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

                    self.packets_processed += 1;

                    //Add packet to packets
                    packets.push(packet);
                } else {
                    break;
                }
            }
        }

        packets
    }
}
