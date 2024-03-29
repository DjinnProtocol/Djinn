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
    TransferCancel,
    SyncRequest,
    SyncAck,
    SyncDeny,
    SyncUpdate,
    SyncIndexRequest,
    SyncIndexResponse,
    SyncIndexUpdate,
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
            6 => ControlPacketType::TransferCancel,
            7 => ControlPacketType::SyncRequest,
            8 => ControlPacketType::SyncAck,
            9 => ControlPacketType::SyncDeny,
            10 => ControlPacketType::SyncUpdate,
            11 => ControlPacketType::SyncIndexRequest,
            12 => ControlPacketType::SyncIndexResponse,
            13 => ControlPacketType::SyncIndexUpdate,
            14 => ControlPacketType::None,
            _ => panic!("Invalid control packet type"),
        }
    }
}

pub enum TransferDenyReason {
    FileNotFound,
    FileWriteLock,
    FileReadLock
}

impl TransferDenyReason {
    pub fn from_string(reason: &str) -> TransferDenyReason {
        match reason {
            "FileNotFound" => TransferDenyReason::FileNotFound,
            "FileWriteLock" => TransferDenyReason::FileWriteLock,
            "FileReadLock" => TransferDenyReason::FileReadLock,
            _ => panic!("Invalid transfer deny reason"),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TransferDenyReason::FileNotFound => "FileNotFound".to_string(),
            TransferDenyReason::FileWriteLock => "FileWriteLock".to_string(),
            TransferDenyReason::FileReadLock => "FileReadLock".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ControlPacket {
    pub packet_type: PacketType,
    pub control_packet_type: ControlPacketType,
    pub job_id: Option<u32>,
    pub params: HashMap<String, String>,
}

impl ControlPacket {
    pub fn new(control_packet_type: ControlPacketType, params: HashMap<String, String>) -> ControlPacket {
        ControlPacket {
            packet_type: PacketType::Control,
            job_id: None,
            control_packet_type,
            params
        }
    }
}

impl Packet for ControlPacket {
    fn fill_from_buffer(&mut self, buffer: &Vec<u8>) {
        self.control_packet_type = ControlPacketType::from_byte(buffer[5]);
        let job_id = u32::from_be_bytes([buffer[6], buffer[7], buffer[8], buffer[9]]);

        if job_id != 0 {
            self.job_id = Some(job_id);
        }else {
            self.job_id = None;
        }


        self.params = HashMap::new();

        let buffer = &buffer[10..];

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
        buffer.extend((self.job_id.unwrap_or(0).to_be_bytes()).to_vec());

        for (key, value) in &self.params {
            buffer.extend(key.as_bytes());
            buffer.push(b'=');
            buffer.extend(value.as_bytes());
            buffer.push(b';');
        }

        buffer
    }

    fn calculate_packet_size(&self) -> u32 {
        let mut size: u32 = 10;

        for (key, value) in &self.params {
            size += (key.len() + value.len() + 2) as u32;
        }

        size
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
    fn test_control_packet() {
        let mut control_packet = ControlPacket {
            packet_type: PacketType::Control,
            control_packet_type: ControlPacketType::EchoRequest,
            job_id: Some(10),
            params: HashMap::new(),
        };

        control_packet.params.insert("a".to_string(), "b".to_string());

        let buffer = control_packet.to_buffer();

        let mut control_packet2 = ControlPacket {
            packet_type: PacketType::Control,
            control_packet_type: ControlPacketType::None,
            job_id: None,
            params: HashMap::new(),
        };
        control_packet2.fill_from_buffer(&buffer);


        assert!(matches!(control_packet2.packet_type, PacketType::Control));
        assert!(matches!(control_packet2.control_packet_type, ControlPacketType::EchoRequest));
        assert_eq!(control_packet2.job_id.unwrap(), 10);
        assert_eq!(control_packet2.params.get("a").unwrap(), "b");
    }
}
