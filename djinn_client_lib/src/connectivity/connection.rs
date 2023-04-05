use std::error::Error;

use async_std::io::BufReader;
use async_std::io::ReadExt;
use async_std::io::WriteExt;
use async_std::net::TcpStream;
use djinn_core_lib::data::packets::PacketReader;
use djinn_core_lib::data::packets::packet::Packet;
use djinn_core_lib::data::packets::packet::deserialize_packet;
use async_std::io::prelude::BufReadExt;

pub struct Connection {
    pub stream: Option<TcpStream>,
    pub active: bool,
    pub host: String,
    pub port: usize,
    pub packet_reader: PacketReader,
}

impl Connection {
    pub fn new(host: String, port: usize) -> Connection {
        Connection {
            stream: None,
            active: false,
            host,
            port,
            packet_reader: PacketReader::new(),
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
            let buffer = packet.to_buffer();
            // Write buffer to stream
            let mut total_bytes_written = 0;

            while total_bytes_written < buffer.len() {
                let bytes_written = self.stream.as_mut().unwrap().write(&buffer).await?;
                total_bytes_written += bytes_written;
            }

        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream is not connected",
            )));
        }
        Ok(())
    }

    pub async fn read_next_packet(&mut self) -> Result<Box<dyn Packet>, Box<dyn Error>> {
        if self.stream.is_some() {
            let stream = self.stream.as_mut().unwrap();
            let mut reader = BufReader::new(stream);
            let packets = self.packet_reader.read(&mut reader, Some(1)).await;

            if packets.is_empty() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Connection closed",
                )));
            }

            Ok(packets[0].clone())
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream is not connected",
            )));
        }
    }
}
