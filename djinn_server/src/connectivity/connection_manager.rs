use std::{sync::Arc};

use tokio::{net::{TcpListener, TcpStream}, sync::{Mutex, broadcast:: {Sender, Receiver}}};
use crate::CONFIG;

use super::{ConnectionData, Connection, ConnectionUpdate};

pub struct ConnectionManager {
    connections: Vec<Arc<Mutex<ConnectionData>>>,
    _connections_broadcast_receiver: Receiver<ConnectionUpdate>,
    connections_broadcast_sender: Sender<ConnectionUpdate>,
}

impl ConnectionManager {
    pub fn new() -> ConnectionManager {
        let (connection_broadcast_writer, connection_broadcast_reader) = tokio::sync::broadcast::channel(100);
        ConnectionManager {
            connections: vec![],
            connections_broadcast_sender: connection_broadcast_writer,
            _connections_broadcast_receiver: connection_broadcast_reader
        }
    }

    pub async fn listen_for_connections(&mut self) {
        let host = CONFIG.host.clone().unwrap();
        let port = CONFIG.port.unwrap();
        let listener = TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
        info!("Listening on {}:{}", host, port);
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            info!("New connection accepted from: {}", socket.peer_addr().unwrap());
            self.handle_new_connection(socket).await;
        }
    }

    async fn handle_new_connection(&mut self, stream: TcpStream) {
        let new_receiver = self.connections_broadcast_sender.subscribe();
        let connection_data = ConnectionData::new(stream, new_receiver, self.connections_broadcast_sender.clone());
        let connection_uuid = connection_data.uuid.clone();
        let packed_connection_data = Arc::new(Mutex::new(connection_data));
        self.connections.push(packed_connection_data.clone());

        tokio::spawn(async move {
            let mut connection = Connection::new(connection_uuid, packed_connection_data);
            connection.listen().await;
        });
    }
}
