use std::collections::HashMap;

use tokio::fs::File;


pub enum JobType {
    Transfer,
    Sync
}


pub enum JobStatus {
    Pending,
    Running,
    Finished,
    Canceled
}

pub struct Job {
    pub id: u32,
    pub job_type: JobType,
    pub status: JobStatus,
    pub params: HashMap<String, String>,
    pub open_file: Option<File>
}
