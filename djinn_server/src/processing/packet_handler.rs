use djinn_core_lib::data::packets::{packet::{Packet, self}, PacketType, ControlPacketType, ControlPacket};

use crate::connectivity::Connection;

use super::control_commands::{EchoRequestCommand, ControlCommand, TransferRequestCommand, TransferStartCommand, SyncIndexResponseCommand};



pub struct PacketHandler {

}

impl PacketHandler {
    pub async fn handle_boxed_packet<'a>(&self, boxed_packet: Box<dyn Packet + 'a>, connection: &mut Connection) {
        let packet_ref: &dyn Packet = boxed_packet.as_ref();

        match packet_ref.get_packet_type() {
            PacketType::Control => {
                info!("Control packet received");
                let control_packet = packet_ref.as_any().downcast_ref::<ControlPacket>().unwrap();

                self.handle_control_packet(&control_packet, connection).await;
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
                //Transfer reverse for server -> client
            },
            ControlPacketType::SyncIndexResponse => {
                let command = SyncIndexResponseCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::TransferStart => {
                let command = TransferStartCommand {};
                command.execute(connection, packet).await.unwrap();
            },
            ControlPacketType::SyncRequest => {
                // Throw error
                panic!("Syncing is not supported yet")
            },
            _ => {
                // Throw error
                panic!("Unknown control packet type")
            }
        }
    }
}
