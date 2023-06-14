use std::{
    fs::File,
    io::{BufReader, Read},
};

use super::DataPacket;

pub struct DataPacketGenerator {
    job_id: u32,
    path: String,
}

impl DataPacketGenerator {
    pub fn new(job_id: u32, path: String) -> Self {
        DataPacketGenerator { job_id, path }
    }

    pub fn iter(&self) -> DataPacketGeneratorIterator {
        let file = File::open(self.path.clone()).unwrap();
        let buf_reader = BufReader::new(file);
        DataPacketGeneratorIterator::new(self.job_id, buf_reader)
    }
}

pub struct DataPacketGeneratorIterator {
    job_id: u32,
    buf_reader: BufReader<File>,
    buffer: Vec<u8>,
    packet_count: usize,
    ended: bool,
}

impl DataPacketGeneratorIterator {
    pub fn new(job_id: u32, buf_reader: BufReader<File>) -> Self {
        DataPacketGeneratorIterator {
            job_id,
            buffer: Vec::with_capacity(60000),
            buf_reader,
            packet_count: 0,
            ended: false,
        }
    }
}

impl Iterator for DataPacketGeneratorIterator {
    type Item = DataPacket;

    fn next(&mut self) -> Option<Self::Item> {
        while self.buffer.len() < 60000 {
            let mut temp_buffer = [0; 60000];
            let bytes_read = self.buf_reader.read(&mut temp_buffer).unwrap();

            if bytes_read == 0 {
                break;
            }

            self.buffer.extend_from_slice(&temp_buffer[0..bytes_read]);
        }

        // Send packet if buffer is full
        if !self.buffer.is_empty() {
            let packet = self.generate_packet();

            self.buffer.clear();
            self.packet_count += 1;

            return packet;
        }

        // Send end packet
        if !self.ended {
            self.ended = true;
            return Some(DataPacket::new(
                self.job_id,
                vec![],
                (self.packet_count + 1) as u32,
            ));
        }

        None
    }
}

impl DataPacketGeneratorIterator {
    fn generate_packet(&mut self) -> Option<DataPacket> {
        let packet = DataPacket::new(
            self.job_id,
            self.buffer.clone(),
            (self.packet_count + 1) as u32,
        );

        Some(packet)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, io::Write};

    use super::*;

    #[test]
    fn test_data_packet_generator() {
        //Creat temporary file
        let dir = env::temp_dir();
        let file_path = dir.join("test_file.txt");
        let mut file = File::create(file_path.clone()).unwrap();

        for (_i, _) in (0..60000).enumerate() {
            file.write_all(&[1_u8]).unwrap();
        }

        for (_i, _) in (0..60000).enumerate() {
            file.write_all(&[2_u8]).unwrap();
        }

        // Read file
        let file = File::open(file_path).unwrap();

        let buf_reader = BufReader::new(file);
        let data_packet_generator = DataPacketGeneratorIterator::new(1, buf_reader);

        let mut packet_count = 0;

        // Test packets
        for packet in data_packet_generator {
            packet_count += 1;

            if packet_count < 3 {
                assert_eq!(packet.job_id, 1);
                assert_eq!(packet.data.len(), 60000);
                assert_eq!(packet.packet_number, packet_count as u32);
                assert!(packet.data.iter().all(|&x| x == packet_count as u8));
            } else {
                assert_eq!(packet.job_id, 1);
                assert_eq!(packet.packet_number, 3);
                assert_eq!(packet.data.len(), 0);
            }
        }

        assert_eq!(packet_count, 3);
    }
}
