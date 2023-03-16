trait ControlCommand {
    fn execute(&self, connection: &Connection, packet: &ControlPacket);
}
