use tokio::{io::{ReadHalf, BufReader, AsyncReadExt}, net::TcpStream};

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

    pub async fn read2(&mut self, reader: &mut BufReader<ReadHalf<TcpStream>>, max_packets: Option<usize>) -> Vec<Box<dyn Packet>> {
        let mut packets: Vec<Box<dyn Packet>> = Vec::with_capacity(10);

        while packets.is_empty() {
            let mut temp_buffer = [0; 65536];
            // debug!("Started reading");
            // debug!("Packet's processed: {}", self.packetsProcessed);
            // debug!("reader buffer size: {}", reader.buffer().len());
            let bytes_read = reader.read(&mut temp_buffer).await.unwrap();

            // debug!("JV Bytes read: {}", bytes_read);

            if bytes_read == 0 {
                return packets;
            }

            //Add to self buffer
            self.buffer.extend_from_slice(&temp_buffer[0..bytes_read]);

            //If buffer has more than 4 bytes and more packets need to be read
            // debug!("Hello world");
            while self.buffer.len() > 4 && (max_packets.is_none() || packets.len() < max_packets.unwrap()) {
                // debug!("Hello world 2");
                //Check if packet is complete
                let packet_length = get_packet_length(&self.buffer) as usize;
                // debug!("Packet length: {}", packet_length);

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

    pub async fn read(&mut self, reader: &mut BufReader<&mut ReadHalf<TcpStream>>, max_packets: Option<usize>) -> Vec<Box<dyn Packet>> {
        let mut packets: Vec<Box<dyn Packet>> = Vec::with_capacity(10);

        while packets.is_empty() {
            let mut temp_buffer = [0; 65536];
            let bytes_read = reader.read(&mut temp_buffer).await.unwrap();

            // debug!("Buffer size1: {}", self.buffer.len());
            // debug!("JM Bytes read: {}", bytes_read);

            if bytes_read == 0 {
                return packets;
            }



            //Add to self buffer
            self.buffer.extend_from_slice(&temp_buffer[0..bytes_read]);

            //If buffer has more than 4 bytes and more packets need to be read
            while self.buffer.len() > 4 && (max_packets.is_none() || packets.len() < max_packets.unwrap()) {
                //Check if packet is complete
                // debug!("Buffer size2: {}", self.buffer.len());
                let packet_length = get_packet_length(&self.buffer) as usize;
                // debug!("Packet length: {}", packet_length);

                            //TODO: Find out why the packet lenght is zero here eventhough the buffer is empty and something gets in

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

        packets
    }
}
