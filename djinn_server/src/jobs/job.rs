use std::collections::HashMap;

pub enum JobType {
    Transfer
}

pub struct Job {
    id: u32,
    job_type: JobType,
    args: HashMap<String, String>
}
