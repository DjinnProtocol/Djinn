use std::{error::Error, time::Duration, collections::HashMap, sync::Arc};

use djinn_core_lib::data::{syncing::IndexManager, packets::{ControlPacket, ControlPacketType, packet::Packet}};
use tokio::{time::sleep, sync::Mutex, io::{WriteHalf, BufWriter, AsyncWriteExt}, net::TcpStream};

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

    pub async fn poll(&mut self, write_stream_arc: Arc<Mutex<Option<WriteHalf<TcpStream>>>>) -> Result<(), Box<dyn Error>> {
        let mut index_manager = IndexManager::new(self.path.clone());
        index_manager.build().await;

        loop {
            debug!("Checking");
            sleep(Duration::from_secs(1)).await;
            //Check if the index has changed
            let mut new_index_manager = IndexManager::new(self.path.clone());
            new_index_manager.build().await;

            if index_manager.index != new_index_manager.index {
                debug!("Index has changed, sending new index response");
                //Send new index response
                let mut params = HashMap::new();
                let index = new_index_manager.index.clone();

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
                index_manager = new_index_manager;
            }

            debug!("Done")
        }
    }
}
