use std::{any::Any, collections::HashMap};

use super::{PacketType, ControlPacket, DataPacket, ControlPacketType};

pub trait Packet: Send + Sync {
    fn fill_from_buffer(&mut self, buffer: &Vec<u8>);
    fn to_buffer(&self) -> Vec<u8>;
    fn get_packet_type(&self) -> PacketType;
    fn calculate_packet_size(&self) -> u32;
    fn as_any(&self) -> &dyn Any;
}

pub fn get_packet_length(buffer: &Vec<u8>) -> u32 {
    u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]])
}

pub fn duplicate_packet(packet: &Box<dyn Packet>) -> Box<dyn Packet> {
    let packet_type = packet.get_packet_type();

    let temporary_packet: Box<dyn Packet> = match packet_type {
        PacketType::Control => {
            let control_packet = packet.as_any().downcast_ref::<ControlPacket>().unwrap();
            Box::new(ControlPacket::new(control_packet.control_packet_type, control_packet.params.clone()))
        },
        PacketType::Data => {
            let data_packet = packet.as_any().downcast_ref::<DataPacket>().unwrap();
            Box::new(DataPacket::new(data_packet.job_id, data_packet.data.clone(), data_packet.packet_number))
        }
    };

    temporary_packet
}

pub fn deserialize_packet(buffer: &Vec<u8>) -> Box<dyn Packet> {
    // debug!("Buffer length: {}", buffer.len());
    let packet_type_byte = buffer[4];
    let packet_type = PacketType::from_byte(packet_type_byte);

    let temporary_packet: Box<dyn Packet> = match packet_type {
        PacketType::Control => {
            let mut control_packet = ControlPacket::new(ControlPacketType::None, HashMap::new());
            control_packet.fill_from_buffer(buffer);
            Box::new(control_packet)
        },
        PacketType::Data => {
            let mut data_packet = DataPacket::new(0, Vec::new(), 0);
            data_packet.fill_from_buffer(buffer);
            Box::new(data_packet)
        }
    };

    temporary_packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_control_packet() {
        let buffer: Vec<u8> = DataPacket::new(0, vec![], 0).to_buffer();
        let boxed_packet = deserialize_packet(&buffer);
        let packet_ref: &dyn Packet = boxed_packet.as_ref();


        assert!(matches!(packet_ref.get_packet_type(), PacketType::Data));
    }
}
