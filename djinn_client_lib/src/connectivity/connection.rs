use std::error::Error;

use async_std::io::BufReader;
use async_std::io::WriteExt;
use async_std::net::TcpStream;
use djinn_core_lib::data::packets::packet::Packet;
use djinn_core_lib::data::packets::packet::deserialize_packet;
use async_std::io::prelude::BufReadExt;

pub struct Connection {
    stream: Option<TcpStream>,
    pub active: bool,
    pub host: String,
    pub port: usize,
}

impl Connection {
    pub fn new(host: String, port: usize) -> Connection {
        Connection {
            stream: None,
            active: false,
            host,
            port,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect(addr).await?;
        self.stream = Some(stream);
        self.active = true;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        //Close stream
        if let Some(stream) = &self.stream {
            stream.shutdown(std::net::Shutdown::Both)?;
        }

        //Set stream to None
        self.stream = None;
        self.active = false;
        Ok(())
    }

    pub async fn send_packet(&mut self, packet: impl Packet) -> Result<(), Box<dyn Error>> {
        if self.stream.is_some() {
            // Convert packet to buffer
            let mut buffer = packet.to_buffer();
            buffer.push(b'\n');
            // Write buffer to stream
            self.stream.as_mut().unwrap().write(&buffer).await?;
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream is not connected",
            )));
        }
        Ok(())
    }

    pub async fn read_next_packet(&mut self) -> Result<impl Packet, Box<dyn Error>> {
        if self.stream.is_some() {
            let mut buffer = vec![];
            let stream = self.stream.as_mut().unwrap();
            let mut reader = BufReader::new(stream);
            reader.read_until(b'\n', &mut buffer).await.unwrap();

            println!("Buffer: {:?}", String::from_utf8(buffer.clone()));
            // Convert buffer to packet
            let packet = deserialize_packet(&buffer);
            Ok(packet)
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream is not connected",
            )));
        }
    }
}
