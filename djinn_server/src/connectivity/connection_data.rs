use std::sync::Arc;

use djinn_core_lib::jobs::Job;
use tokio::{net::TcpStream, io::{ReadHalf, WriteHalf}, sync::Mutex};
use uuid::Uuid;

pub struct ConnectionData {
    pub read_stream: Arc<Mutex<ReadHalf<TcpStream>>>,
    pub write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub uuid: Uuid,
    pub jobs: Vec<Arc<Mutex<Job>>>,
    pub new_job_id: u32
}

impl ConnectionData {
    pub fn new(stream: TcpStream) -> ConnectionData {
        let (read_stream, write_stream) = tokio::io::split(stream);
        ConnectionData {
            read_stream: Arc::new(Mutex::new(read_stream)),
            write_stream: Arc::new(Mutex::new(write_stream)),
            uuid: Uuid::new_v4(),
            jobs: vec![],
            new_job_id: 0,
        }
    }
}
