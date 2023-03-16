use async_std::net::{TcpListener, TcpStream};
use crate::{threads::ThreadPool, CONFIG};

pub struct Listener {
    thread_pool: ThreadPool
}

impl Listener {
    pub fn new() -> Listener {
        Listener {
            thread_pool: ThreadPool::new(CONFIG.amount_of_threads.unwrap())
        }
    }

    pub async fn listen_for_connections(&mut self) {
        let host = CONFIG.host.clone().unwrap();
        let port = CONFIG.port.unwrap();
        let listener = TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
        info!("Listening on {}:{}", host, port);
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            debug!("New connection accepted from: {}", socket.peer_addr().unwrap());
            self.handle_new_connection(socket).await;
        }
    }

    async fn handle_new_connection(&mut self, socket: TcpStream) {
        self.thread_pool.distribute_stream(socket).await;
    }
}
