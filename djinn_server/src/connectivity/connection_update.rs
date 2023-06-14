use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum ConnectionUpdateType {
    ServerIndexUpdated
}

//Implement clone
#[derive(Clone, Debug)]
pub struct ConnectionUpdate {
    pub update_type: ConnectionUpdateType,
    pub connection_uuid: Uuid
}

impl ConnectionUpdate {
    pub fn new(connection_uuid: Uuid) -> Self {
        Self {
            connection_uuid,
            update_type: ConnectionUpdateType::ServerIndexUpdated
        }
    }
}