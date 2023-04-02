use std::{collections::HashMap, any::Any};

use super::{PacketType, packet::Packet};

#[derive(Copy, Clone)]
pub enum ControlPacketType {
    EchoRequest,
    EchoReply,
    TransferRequest,
    TransferAck,
    TransferDeny,
    TransferStart,
    None
}

impl ControlPacketType {
    fn from_byte(byte: u8) -> ControlPacketType {
        match byte {
            0 => ControlPacketType::EchoRequest,
            1 => ControlPacketType::EchoReply,
            2 => ControlPacketType::TransferRequest,
            3 => ControlPacketType::TransferAck,
            4 => ControlPacketType::TransferDeny,
            5 => ControlPacketType::TransferStart,
            6 => ControlPacketType::None,
            _ => panic!("Invalid control packet type"),
        }
    }
}

pub enum TransferDenyReason {
    FileNotFound
}

impl TransferDenyReason {
    pub fn from_string(reason: &str) -> TransferDenyReason {
        match reason {
            "FileNotFound" => TransferDenyReason::FileNotFound,
            _ => panic!("Invalid transfer deny reason"),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TransferDenyReason::FileNotFound => "FileNotFound".to_string(),
        }
    }
}

pub struct ControlPacket {
    pub packet_type: PacketType,
    pub control_packet_type: ControlPacketType,
    pub params: HashMap<String, String>,
}

impl ControlPacket {
    pub fn new(control_packet_type: ControlPacketType, params: HashMap<String, String>) -> ControlPacket {
        return ControlPacket {
            packet_type: PacketType::Control,
            control_packet_type,
            params
        };
    }
}

impl Packet for ControlPacket {
    fn fill_from_buffer(&mut self, buffer: &Vec<u8>) {
        self.control_packet_type = ControlPacketType::from_byte(buffer[5]);
        self.params = HashMap::new();

        let buffer = &buffer[6..];

        if buffer.len() > 2 {
            let params_string = String::from_utf8(buffer.to_vec()).unwrap();
            let params_string_split = params_string.split(';');

            for param in params_string_split {
                if param.len() < 3 {
                    continue;
                }
                let param_split = param.split('=');
                let param_split_vec: Vec<&str> = param_split.collect();
                self.params.insert(param_split_vec[0].to_string(), param_split_vec[1].to_string());
            }
        }
    }

    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        //Add packet size
        buffer.extend((self.calculate_packet_size().to_be_bytes()).to_vec());
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

    fn calculate_packet_size(&self) -> u32 {
        let mut size: u32 = 6;

        for (key, value) in &self.params {
            size += (key.len() + value.len() + 2) as u32;
        }

        return size
    }

    fn get_packet_type(&self) -> PacketType {
        return self.packet_type;
    }

    fn as_any(&self) -> &dyn Any {
        return self;
    }
}


#[cfg(test)]
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

        let mut control_packet2 = ControlPacket {
            packet_type: PacketType::Control,
            control_packet_type: ControlPacketType::None,
            params: HashMap::new(),
        };
        control_packet2.fill_from_buffer(&buffer);


        assert!(matches!(control_packet2.packet_type, PacketType::Control));
        assert!(matches!(control_packet2.control_packet_type, ControlPacketType::EchoRequest));
        assert_eq!(control_packet2.params.get("a").unwrap(), "b");
    }
}
