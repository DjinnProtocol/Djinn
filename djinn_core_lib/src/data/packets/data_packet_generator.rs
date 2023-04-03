use std::{
    fs::File,
    io::{BufReader, Bytes, Read},
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
        let bytes_iterator = BufReader::new(file).bytes();
        DataPacketGeneratorIterator::new(self.job_id, bytes_iterator)
    }
}


pub struct DataPacketGeneratorIterator {
    job_id: u32,
    bytes_iterator: Bytes<BufReader<File>>,
    buffer: Vec<u8>,
    packet_count: usize,
    byte_count: usize,
    ended: bool,
}

impl DataPacketGeneratorIterator {
    pub fn new(job_id: u32, bytes_iterator: Bytes<BufReader<File>>) -> Self {
        DataPacketGeneratorIterator {
            job_id,
            buffer: vec![0; 60000],
            bytes_iterator,
            packet_count: 0,
            byte_count: 0,
            ended: false,
        }
    }
}

impl Iterator for DataPacketGeneratorIterator {
    type Item = DataPacket;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(byte) = self.bytes_iterator.next() {
            let byte = byte.unwrap();
            self.buffer[self.byte_count] = byte;

            self.byte_count += 1;

            if self.byte_count >= 60000 {
                return self.generate_packet();
            }
        }

        // Send the last packet if there is one
        if self.byte_count > 0 {
            return self.generate_packet();
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
            self.buffer[0..self.byte_count].to_vec(),
            (self.packet_count + 1) as u32,
        );
        self.packet_count += 1;
        self.byte_count = 0;
        return Some(packet);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        io::{Read, Write},
    };

    use super::*;

    #[test]
    fn test_data_packet_generator() {
        //Creat temporary file
        let dir = env::temp_dir();
        let file_path = dir.join("test_file.txt");
        let mut file = File::create(file_path.clone()).unwrap();

        for (_i, _) in (0..60000).enumerate() {
            file.write_all(&[1 as u8]).unwrap();
        }

        for (_i, _) in (0..60000).enumerate() {
            file.write_all(&[2 as u8]).unwrap();
        }

        // Read file
        let file = File::open(file_path.clone()).unwrap();

        let bytes_iterator = BufReader::new(file).bytes();
        let mut data_packet_generator = DataPacketGeneratorIterator::new(1, bytes_iterator);

        let mut packet_count = 0;

        // Test packets
        while let Some(packet) = data_packet_generator.next() {
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
