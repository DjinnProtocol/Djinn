use std::any::Any;

use super::{PacketType, ControlPacket, DataPacket};

pub trait Packet {
    fn from_buffer(buffer: &Vec<u8>) -> Self;
    fn to_buffer(&self) -> Vec<u8>;
    fn get_packet_type(&self) -> PacketType;
    fn as_any(&self) -> &dyn Any;
}

pub fn deserialize_packet(buffer: &Vec<u8>) -> Box<dyn Packet> {
    let packet_type_byte = buffer[0];
    let packet_type = PacketType::from_byte(packet_type_byte);

    let temporary_packet: Box<dyn Packet> = match packet_type {
        PacketType::Control => {
            Box::new(ControlPacket::from_buffer(buffer))
        },
        PacketType::Data => {
            Box::new(DataPacket::from_buffer(buffer))
        }
    };

    return temporary_packet;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_control_packet() {
        let buffer: Vec<u8> = vec![0, 0];
        let packet = deserialize_packet(&buffer);

        assert!(!matches!(packet.get_packet_type(), PacketType::Data));
    }
}
