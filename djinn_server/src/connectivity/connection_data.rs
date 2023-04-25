pub struct ConnectionData {
    pub stream: TcpStream,
    pub uuid: Uuid,
    pub jobs: Vec<Job>,
    pub new_job_id: u32
}

impl ConnectionData {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            uuid: Uuid::new_v4(),
            jobs: vec![],
            new_job_id: 0,
        }
    }
}
