use std::collections::HashMap;

use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum ConnectionUpdateType {
    ServerIndexUpdated,
}

//Implement clone
#[derive(Clone, Debug)]
pub struct ConnectionUpdate {
    pub update_type: ConnectionUpdateType,
    pub connection_uuid: Uuid,
    pub data: HashMap<String, usize>,
}

impl ConnectionUpdate {
    pub fn new(connection_uuid: Uuid, data: HashMap<String, usize>) -> Self {
        Self {
            connection_uuid,
            update_type: ConnectionUpdateType::ServerIndexUpdated,
            data,
        }
    }
}
