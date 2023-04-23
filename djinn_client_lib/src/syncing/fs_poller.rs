use std::{error::Error, time::Duration, collections::HashMap, sync::Arc};

use async_std::{net::TcpStream, sync::Mutex, io::WriteExt, task};
use djinn_core_lib::data::{syncing::IndexManager, packets::{ControlPacket, ControlPacketType, packet::Packet}};

use crate::connectivity::Connection;

use super::SyncHandler;

pub struct FsPoller {
    pub path: String,
    pub job_id: u32
}

impl FsPoller {
    pub fn new(path: String, job_id: u32) -> FsPoller {
        FsPoller {
            path,
            job_id
        }
    }

    pub async fn poll(&mut self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let mut index_manager = IndexManager::new(self.path.clone());
        index_manager.build().await;

        loop {
            debug!("Checking");
            task::sleep(Duration::from_secs(1)).await;
            //Check if the index has changed
            let mut new_index_manager = IndexManager::new(self.path.clone());
            new_index_manager.build().await;

            //Check if hashmaps are equal
            let mut equal = false;

            if new_index_manager.index.len() != index_manager.index.len() {
                equal = false;
            }

            for(key, _) in index_manager.index.iter() {
                if !new_index_manager.index.contains_key(key) {
                    equal = false;
                    break;
                }
            }

            if !equal {
                debug!("Index has changed, sending new index response");
                //Send new index response
                let mut params = HashMap::new();
                let index = new_index_manager.index.clone();

                //Stringify the timestamps
                for (key, value) in index.iter() {
                    params.insert(key.clone(), value.to_string());
                }

                let mut packet = ControlPacket::new(ControlPacketType::SyncIndexResponse, params);

                packet.job_id = Some(self.job_id);

                stream.write_all(packet.to_buffer().as_slice()).await?;
                stream.flush().await?;

                //Update index manager
                index_manager = new_index_manager;
            }

            debug!("Done")
        }
    }
}
