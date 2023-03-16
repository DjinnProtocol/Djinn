use std::collections::HashMap;

use async_std::io::WriteExt;
use djinn_core_lib::data::packets::{packet::Packet, PacketType, ControlPacketType, ControlPacket};

use crate::connectivity::Connection;



pub struct PacketHandler {

}

impl PacketHandler {
    pub async fn handle_packet(&self, packet: &impl Packet, connection: &mut Connection) {
        match packet.get_packet_type() {
            PacketType::Control => {
                info!("Control packet received");
                let control_packet = packet.as_any()
                .downcast_ref::<ControlPacket>().unwrap();

                self.handle_control_packet(control_packet, connection).await;
            },
            PacketType::Data => {
                // Throw error
                panic!("Data packets are not supported yet")
            }
        }
    }

    pub async fn handle_control_packet(&self, packet: &ControlPacket, connection: &mut Connection) {
        match packet.control_packet_type {
            ControlPacketType::EchoRequest => {
                let response = ControlPacket {
                    packet_type: PacketType::Control,
                    control_packet_type: ControlPacketType::EchoReply,
                    params: HashMap::new(),
                };

                connection.stream.write(&response.to_buffer()).await.unwrap();

            },
            ControlPacketType::EchoReply => {
                info!("Echo reply received");
            },
            ControlPacketType::TransferRequest => {

            },
            ControlPacketType::TransferAck => {

            },
            ControlPacketType::TransferStart => {

            },
        }
    }
}
