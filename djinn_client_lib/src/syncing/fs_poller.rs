use std::{error::Error, time::Duration, collections::HashMap, sync::Arc};

use djinn_core_lib::data::{syncing::IndexManager, packets::{ControlPacket, ControlPacketType, packet::Packet}};
use tokio::{time::sleep, sync::Mutex, io::{WriteHalf, AsyncWriteExt}, net::TcpStream};

pub struct FsPoller {
    pub path: String,
    pub job_id: u32,
    pub was_just_syncing: bool
}

impl FsPoller {
    pub fn new(path: String, job_id: u32) -> FsPoller {
        FsPoller {
            path,
            job_id,
            was_just_syncing: false
        }
    }

    pub async fn poll(&mut self, write_stream_arc: Arc<Mutex<Option<WriteHalf<TcpStream>>>>, is_syncing_arc: Arc<Mutex<bool>>) -> Result<(), Box<dyn Error>> {
        let mut index_manager = IndexManager::new(self.path.clone());
        index_manager.build().await;

        loop {
            sleep(Duration::from_secs(1)).await;
            //Check if we are syncing
            let is_syncing = is_syncing_arc.lock().await;
            if *is_syncing {
                self.was_just_syncing = true;
                continue;
            }

            //Check if the index has changed
            let mut new_index_manager = IndexManager::new(self.path.clone());
            new_index_manager.build().await;

            // Remove timestamps from the index
            let mut index_without_timestamps = index_manager.index.clone();
            let mut new_index_without_timestamps = new_index_manager.index.clone();
            index_without_timestamps.remove("#timestamp");
            new_index_without_timestamps.remove("#timestamp");

            if index_without_timestamps != new_index_without_timestamps || self.was_just_syncing {
                debug!("Index has changed, sending new index response");
                //Send new index response
                let mut params = HashMap::new();
                let mut index = new_index_manager.index.clone();

                // Add deleted files with timestamp 0 by looping through the old index
                for (key, _) in index_without_timestamps.iter() {
                    if !index.contains_key(key) {
                       index.insert(key.clone(), 0);
                    }
                }

                //Stringify the timestamps
                for (key, value) in index.iter() {
                    params.insert(key.clone(), value.to_string());
                }

                let mut packet = ControlPacket::new(ControlPacketType::SyncIndexUpdate, params);

                packet.job_id = Some(self.job_id);

                let mut write_stream_option = write_stream_arc.lock().await;

                if write_stream_option.is_none() {
                    debug!("Write stream is none");
                    continue;
                }

                let write_stream = write_stream_option.as_mut().unwrap();

                write_stream.write_all(packet.to_buffer().as_slice()).await?;
                write_stream.flush().await?;

                //Update index manager
                if !self.was_just_syncing {
                    index_manager = new_index_manager;
                }
            }

            if self.was_just_syncing {
                self.was_just_syncing = false;
            }
        }
    }
}
