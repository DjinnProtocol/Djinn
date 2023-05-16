use std::collections::HashMap;

#[derive(Clone)]
pub enum JobType {
    Transfer,
    Sync
}

#[derive(Clone)]
pub enum JobStatus {
    Pending,
    Running,
    Finished
}

#[derive(Clone)]
pub struct Job {
    pub id: u32,
    pub job_type: JobType,
    pub status: JobStatus,
    pub params: HashMap<String, String>
}
