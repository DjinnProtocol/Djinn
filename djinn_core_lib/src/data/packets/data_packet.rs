use std::any::Any;

use super::{PacketType, packet::Packet};


#[derive(Clone)]
pub struct DataPacket {
    pub packet_type: PacketType,
    pub job_id: u32,
    pub packet_number: u32,
    pub data: Vec<u8>,
    pub has_data: bool
}

impl DataPacket {
    pub fn new(job_id: u32, data: Vec<u8>, packet_number: u32) -> DataPacket {
        let has_data = !data.is_empty();

        DataPacket {
            packet_type: PacketType::Data,
            job_id,
            packet_number,
            data,
            has_data
        }
    }
}

impl Packet for DataPacket {
    fn fill_from_buffer(&mut self, buffer: &Vec<u8>){
        self.job_id = u32::from_be_bytes([buffer[5], buffer[6], buffer[7], buffer[8]]);
        self.packet_number = u32::from_be_bytes([buffer[9], buffer[10], buffer[11], buffer[12]]);
        if buffer.len() > 13 {
            self.has_data = true;
            self.data = buffer[13..].to_vec();
        } else {
            self.data = vec![]
        }
    }

    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend(self.calculate_packet_size().to_be_bytes().to_vec());
        buffer.push(self.packet_type as u8);
        buffer.extend(self.job_id.to_be_bytes().to_vec());
        buffer.extend(self.packet_number.to_be_bytes().to_vec());
        buffer.extend(self.data.to_vec());

        buffer
    }

    fn calculate_packet_size(&self) -> u32 {
        13 + self.data.len() as u32
    }

    fn get_packet_type(&self) -> PacketType {
        self.packet_type
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_packet() {
        let mut data_packet = DataPacket::new(0, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 0);
        data_packet.packet_number = 99_u32;
        let buffer = data_packet.to_buffer();

        let mut data_packet2 = DataPacket::new(0, Vec::new(), 0);
        data_packet2.fill_from_buffer(&buffer);

        assert_eq!(data_packet.job_id, data_packet2.job_id);
        assert_eq!(data_packet.data, data_packet2.data);
        assert_eq!(data_packet.packet_number, data_packet2.packet_number);
    }
}
