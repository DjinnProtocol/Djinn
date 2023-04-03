#[derive(Copy, Clone)]
pub enum PacketType {
  Control,
  Data,
}

impl PacketType {
  pub fn from_byte(byte: u8) -> PacketType {
    match byte {
      0 => PacketType::Control,
      1 => PacketType::Data,
      _ => panic!("Invalid packet type"),
    }
  }
}

                                 