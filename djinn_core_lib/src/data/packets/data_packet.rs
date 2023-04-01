use std::{collections::HashMap, any::Any};

use super::{PacketType, packet::Packet};

pub struct DataPacket {
    pub packet_type: PacketType,
    pub job_id: u32,
    pub data: Vec<u8>,
}

impl DataPacket {
    pub fn new(job_id: u32, data: Vec<u8>) -> DataPacket {
        return DataPacket {
            packet_type: PacketType::Data,
            job_id,
            data
        };
    }
}

impl Packet for DataPacket {
    fn from_buffer(buffer: &Vec<u8>) -> DataPacket {
        let job_id = u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);
        let data = buffer[5..].to_vec();

        return DataPacket {
            packet_type: PacketType::Data,
            job_id,
            data,
        };
    }

    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.push(self.packet_type as u8);
        buffer.extend(self.job_id.to_be_bytes().to_vec());
        buffer.extend(self.data.to_vec());

        return buffer;
    }

    fn get_packet_type(&self) -> PacketType {
        return self.packet_type;
    }

    fn as_any(&self) -> &dyn Any {
        return self;
    }
}
