use std::{collections::HashMap, any::Any};

use super::{PacketType, packet::Packet};

#[derive(Copy, Clone)]
pub enum ControlPacketType {
    EchoRequest,
    EchoReply,
    TransferRequest,
    TransferAck,
    TransferStart
}

impl ControlPacketType {
    fn from_byte(byte: u8) -> ControlPacketType {
        match byte {
            0 => ControlPacketType::EchoRequest,
            1 => ControlPacketType::EchoReply,
            _ => panic!("Invalid control packet type"),
        }
    }
}

pub struct ControlPacket {
    pub packet_type: PacketType,
    pub control_packet_type: ControlPacketType,
    pub params: HashMap<String, String>,
}

// impl ControlPacket {
//     pub fn new(packet_type: ControlPacketType) -> ControlPacket {
//         return ControlPacket {
//             packet_type,
//             params: HashMap::new(),
//         };
//     }
// }

impl Packet for ControlPacket {
    fn from_buffer(buffer: &Vec<u8>) -> ControlPacket {
        let mut params: HashMap<String, String> = HashMap::new();
        let packet_type = ControlPacketType::from_byte(buffer[1]);

        let buffer = &buffer[2..];

        if buffer.len() > 2 {
            let params_string = String::from_utf8(buffer.to_vec()).unwrap();
            let params_string_split = params_string.split(';');

            for param in params_string_split {
                if param.len() < 3 {
                    continue;
                }
                let param_split = param.split('=');
                let param_split_vec: Vec<&str> = param_split.collect();
                params.insert(param_split_vec[0].to_string(), param_split_vec[1].to_string());
            }
        }
        
        return ControlPacket {
            packet_type: PacketType::Control,
            control_packet_type: packet_type,
            params,
        };
    }

    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.push(self.packet_type as u8);
        buffer.push(self.control_packet_type as u8);

        for (key, value) in &self.params {
            buffer.extend(key.as_bytes());
            buffer.push(b'=');
            buffer.extend(value.as_bytes());
            buffer.push(b';');
        }

        return buffer;
    }

    fn get_packet_type(&self) -> PacketType {
        return self.packet_type;
    }

    fn as_any(&self) -> &dyn Any {
        return self;
    }
}


mod tests {
    use super::*;

    #[test]
    fn test_control_packet() {
        let mut control_packet = ControlPacket {
            packet_type: PacketType::Control,
            control_packet_type: ControlPacketType::EchoRequest,
            params: HashMap::new(),
        };

        control_packet.params.insert("a".to_string(), "b".to_string());

        let buffer = control_packet.to_buffer();
        let control_packet = ControlPacket::from_buffer(&buffer);

        assert!(matches!(control_packet.packet_type, PacketType::Control));
        assert!(matches!(control_packet.control_packet_type, ControlPacketType::EchoRequest));
        assert_eq!(control_packet.params.get("a").unwrap(), "b");
    }
}