use tokio::fs::File;

pub struct Transfer {
    pub direction: TransferDirection,
    pub status: TransferStatus,
    pub open_file: Option<File>,
    pub file_path: String,
    pub original_modified_time: u64,
    pub id: u32,
    pub job_id: u32
}

pub enum TransferDirection {
    ToServer,
    ToClient,
}

pub enum TransferStatus {
    Requested,
    Accepted,
    Denied,
    InProgress,
    Completed
}

impl Transfer {
    pub fn new(direction: TransferDirection, id: u32, file_path: String) -> Transfer {
        Transfer {
            direction,
            status: TransferStatus::Requested,
            open_file: None,
            file_path,
            original_modified_time: 0,
            id,
            job_id: 0
        }
    }
}
