use djinn_core_lib::data::packets::{packet::Packet, PacketType, ControlPacketType, ControlPacket};

use crate::connectivity::Connection;

use super::control_commands::{EchoRequestCommand, ControlCommand, TransferRequestCommand};



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
                let command = EchoRequestCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::TransferRequest => {
                let command = TransferRequestCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::TransferAck => {

            },
            ControlPacketType::TransferStart => {

            },
            _ => {
                // Throw error
                panic!("Unknown control packet type")
            }
        }
    }
}
