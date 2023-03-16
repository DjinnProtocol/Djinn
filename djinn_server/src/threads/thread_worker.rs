use async_std::{channel::Receiver, net::TcpStream, task};
use crate::connectivity::Connection;

pub struct ThreadWorker {
    pub id: usize,
    receiver: Receiver<TcpStream>
}

impl ThreadWorker {
    pub fn new(id: usize, receiver: Receiver<TcpStream>) -> ThreadWorker {
        ThreadWorker { id, receiver }
    }

    pub async fn run(&mut self) {
        self.listen_incoming_streams().await;
    }

    pub async fn listen_incoming_streams(&mut self) {
        while let Ok(stream) = self.receiver.recv().await {
            self.handle_incoming_stream(stream).await;
        }
    }

    async fn handle_incoming_stream(&mut self, stream: TcpStream) {
        let thread_id: usize = self.id;
        task::spawn(async move {
            let mut connection = Connection::new(stream, thread_id);
            connection.listen().await;
        });
    }
}
