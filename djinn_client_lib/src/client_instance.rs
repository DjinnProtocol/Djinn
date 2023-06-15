use std::error::Error;

use crate::{connectivity::Connection, commands::{EchoCommand, GetCommand}, syncing::{SyncManager, UserMonkey}};

pub struct ClientInstance {
  connection: Connection,
}

impl ClientInstance {
  pub async fn new(host: String, port: usize) -> Result<ClientInstance, Box<dyn Error>> {
    let mut connection = Connection::new(host, port);
    connection.connect().await?;

    Ok(ClientInstance {
      connection,
    })
  }

  pub async fn echo(&mut self){
    let command = EchoCommand::new();
    command.execute(&mut self.connection).await.expect("AAA");
  }

  pub async fn get_as_iterator(&mut self, file_path: String) {
    let command = GetCommand::new(file_path);
    let str = command.execute(&mut self.connection).await.expect("Failed to get file");
    println!("{}", str);
  }

  pub async fn put(&mut self, _file_path: String, _destination: String) {
    // let command = PutCommand::new(file_path, destination);
    // command.execute(&mut self.connection).await.expect("AAA");
    info!("Put command not implemented")
  }

  pub async fn sync_internal(&mut self, path: String, target: String) {
    let mut handler = SyncManager::new(path, target);
    handler.start(&mut self.connection).await.expect("Sync handler failed");
  }

  pub async fn monkey_internal(&mut self, path: String, target: String) {
    let mut monkey = UserMonkey::new(
        path.clone()
      );

    monkey.run().await;
  }

  pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
    self.connection.disconnect().await
  }
}
