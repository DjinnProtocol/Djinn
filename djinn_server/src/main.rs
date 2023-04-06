extern crate pretty_env_logger;
use configuration::application_config::ApplicationConfig;
use connectivity::Listener;
use futures::executor::block_on;
use lazy_static::lazy_static;

mod threads;
mod connectivity;
mod configuration;
mod processing;
mod jobs;
mod syncing;

#[macro_use] extern crate log;


lazy_static! {
    static ref CONFIG: ApplicationConfig = ApplicationConfig::build();
}


fn main() {
    pretty_env_logger::init();
    block_on(async_main())
}

async fn async_main(){
    let mut listener = Listener::new();
    listener.listen_for_connections().await;
}
