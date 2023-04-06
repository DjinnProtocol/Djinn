pub struct FsPoller {
    pub path: String
}

impl FsPoller {
    pub fn new(path: String) -> FsPoller {
        FsPoller {
            path
        }
    }

    pub async fn poll(&mut self, connection: &Connection, sync_handler: &SyncHandler) -> Result<(), Box<dyn Error>> {
        let mut index_manager = IndexManager::new(self.path.clone());
        index_manager.build().await;

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            //Check if the index has changed
            let mut new_index_manager = IndexManager::new(self.path.clone());
            new_index_manager.build().await;

            if index_manager.index != new_index_manager.index {
                //Send new index response
                let mut params = HashMap::new();
                let index = index_manager.index;

                //Stringify the timestamps
                for (key, value) in index.iter() {
                    params.insert(key.clone(), value.to_string());
                }

                let mut packet = ControlPacket::new(ControlPacketType::SyncIndexResponse, params);

                packet.job_id = self.job_id;

                connection.send_packet(packet).await.unwrap();
            }
        }
    }
}
