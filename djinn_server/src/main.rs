extern crate pretty_env_logger;
use std::collections::HashMap;

use configuration::application_config::ApplicationConfig;
use connectivity::ConnectionManager;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

mod connectivity;
mod configuration;
mod processing;
mod syncing;

#[macro_use] extern crate log;


lazy_static! {
    static ref CONFIG: ApplicationConfig = ApplicationConfig::build();
    static ref SERVER_DELETES: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
}


#[tokio::main]
async fn main(){
    pretty_env_logger::init();
    let mut listener = ConnectionManager::new();
    listener.listen_for_connections().await;
}
