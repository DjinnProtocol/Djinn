use std::{sync::{Arc}, collections::HashMap};

use djinn_core_lib::jobs::Job;
use tokio::{net::TcpStream, io::{ReadHalf, WriteHalf}, sync::{Mutex, broadcast::{Receiver, Sender}}};
use uuid::Uuid;

use super::ConnectionUpdate;

pub struct ConnectionData {
    pub read_stream: Arc<Mutex<ReadHalf<TcpStream>>>,
    pub write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub uuid: Uuid,
    pub jobs: Vec<Arc<Mutex<Job>>>,
    pub connections_broadcast_receiver: Arc<Mutex<Receiver<ConnectionUpdate>>>,
    pub connections_broadcast_sender: Arc<Mutex<Sender<ConnectionUpdate>>>,
    pub new_job_id: u32,
    pub last_index: HashMap<String, usize>
}

impl ConnectionData {
    pub fn new(stream: TcpStream, connections_broadcast_receiver: Receiver<ConnectionUpdate>, connections_broadcast_sender: Sender<ConnectionUpdate>) -> ConnectionData {
        let (read_stream, write_stream) = tokio::io::split(stream);
        ConnectionData {
            read_stream: Arc::new(Mutex::new(read_stream)),
            write_stream: Arc::new(Mutex::new(write_stream)),
            uuid: Uuid::new_v4(),
            jobs: vec![],
            connections_broadcast_receiver: Arc::new(Mutex::new(connections_broadcast_receiver)),
            connections_broadcast_sender: Arc::new(Mutex::new(connections_broadcast_sender)),
            new_job_id: 0,
            last_index: HashMap::new()
        }
    }
}
