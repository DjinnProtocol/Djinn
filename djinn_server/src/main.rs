extern crate pretty_env_logger;
use configuration::application_config::ApplicationConfig;
use connectivity::Listener;
use lazy_static::lazy_static;

mod connectivity;
mod configuration;
mod processing;
mod jobs;
mod syncing;

#[macro_use] extern crate log;


lazy_static! {
    static ref CONFIG: ApplicationConfig = ApplicationConfig::build();
}


#[tokio::main]
async fn main(){
    pretty_env_logger::init();
    let mut listener = Listener::new();
    listener.listen_for_connections().await;
}
