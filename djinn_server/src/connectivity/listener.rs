use std::sync::Arc;

use tokio::{net::{TcpListener, TcpStream}, sync::Mutex};
use crate::CONFIG;

use super::{ConnectionData, Connection};

pub struct Listener {
    connections: Vec<Arc<Mutex<ConnectionData>>>,
}

impl Listener {
    pub fn new() -> Listener {
        Listener {
            connections: vec![],
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

    async fn handle_new_connection(&mut self, stream: TcpStream) {
        let mut connection_data = ConnectionData::new(stream);
        let packed_connection_data = Arc::new(Mutex::new(connection_data));
        self.connections.push(packed_connection_data.clone());

        tokio::spawn(async move {
            let mut connection = Connection::new(packed_connection_data);
            connection.listen().await;
        });
    }
}
