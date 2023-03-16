use std::{net::TcpStream, io::Write, collections::HashMap, thread, time::Duration};

use djinn_core_lib::data::packets::{ControlPacket, ControlPacketType, PacketType, packet::Packet};

pub fn echo() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7777")?;

    for _ in 0..10 {
        let mut buffer = vec![];
        buffer.append(&mut ControlPacket {
            packet_type: PacketType::Control,
            control_packet_type: ControlPacketType::EchoRequest,
            params: HashMap::new()
        }.to_buffer());
        //Add newline to buffer
        buffer.push(b'\n');

        //Print buffer as string
        println!("Buffer: {:?}", String::from_utf8(buffer.clone()));
        
        stream.write(&buffer)?;

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
} 