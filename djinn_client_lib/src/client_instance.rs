use std::error::Error;

use crate::{connectivity::Connection, commands::{EchoCommand, GetCommand, SyncCommand}, syncing::SyncHandler};

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
    let str = command.execute(&mut self.connection).await.expect("AAA");
    println!("{}", str);
  }

  pub async fn put(&mut self, file_path: String, destination: String) {
    // let command = PutCommand::new(file_path, destination);
    // command.execute(&mut self.connection).await.expect("AAA");
  }

  pub async fn syncInternal(&mut self, directory: String) {
    let handler = SyncHandler::new(directory);
    handler.start(&mut self.connection).await.expect("AAA");
  }

  pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
    self.connection.disconnect().await
  }
}
