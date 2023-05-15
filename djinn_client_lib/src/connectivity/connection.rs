use std::error::Error;
use std::sync::Arc;


use djinn_core_lib::data::packets::PacketReader;
use djinn_core_lib::data::packets::packet::Packet;
use djinn_core_lib::data::packets::packet::duplicate_packet;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct Connection {
    pub reader: Arc<Mutex<Option<BufReader<ReadHalf<TcpStream>>>>>,
    pub write_stream: Arc<Mutex<Option<WriteHalf<TcpStream>>>>,
    pub active: bool,
    pub host: String,
    pub port: usize,
    pub packet_reader: PacketReader
}

impl Connection {
    pub fn new(host: String, port: usize) -> Connection {
        Connection {
            reader: Arc::new(Mutex::new(None)),
            write_stream: Arc::new(Mutex::new(None)),
            active: false,
            host,
            port,
            packet_reader: PacketReader::new()
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect(addr).await?;
        let (read_stream, write_stream) = tokio::io::split(stream);

        let mut internal_write_stream = self.write_stream.lock().await;
        *internal_write_stream = Some(write_stream);

        //Create reader
        let mut internal_reader = self.reader.lock().await;
        *internal_reader = Some(BufReader::new(read_stream));

        self.active = true;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        //Drop halves
        let mut reader = self.reader.lock().await;
        *reader = None;
        let mut write_stream = self.write_stream.lock().await;
        *write_stream = None;

        self.active = false;
        Ok(())
    }

    pub async fn send_packet(&self, packet: impl Packet) -> Result<(), Box<dyn Error>> {
        let mut stream = self.write_stream.lock().await;
        if stream.is_some() {
            // Convert packet to buffer
            let buffer = packet.to_buffer();
            // Write buffer to stream with writer
            let writer_stream = stream.as_mut().unwrap();

            writer_stream.write_all(&buffer).await?;
            writer_stream.flush().await?;
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream is not connected",
            )));
        }
        Ok(())
    }

    pub async fn flush(&self) -> Result<(), Box<dyn Error>> {
        let mut write_stream = self.write_stream.lock().await;
        if write_stream.is_some() {
            write_stream.as_mut().unwrap().flush().await?;
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream is not connected",
            )));
        }
        Ok(())
    }

    pub async fn read_next_packet(&mut self) -> Result<Option<Box<dyn Packet>>, Box<dyn Error>> {
        debug!("Waiting for lock");
        let mut possible_reader = self.reader.lock().await;
        debug!("Lock acquired");


        if possible_reader.is_some() {
            let reader = possible_reader.as_mut().unwrap();

            debug!("Waiting for packet");
            let packets = self.packet_reader.read2(reader, Some(1)).await;
            debug!("Packet received");

            if packets.is_empty() {
                return Ok(None);
            }

            return Ok(Some(duplicate_packet(&packets[0])));
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream is not connected",
            )));
        }
    }
}
