use clap::{arg, Command};
use djinn_client_lib::DjinnClient;
use futures::executor::block_on;

#[macro_use] extern crate log;


fn main() {
    pretty_env_logger::init();
    block_on(async_main());
}

async fn async_main() {
    let matches = Command::new("djinn")
        .about("Djinn client CLI")
        .subcommand_required(true)
        .version("1.0")
        .arg(arg!( --host -h [HOST] "The host to connect to").required(true))
        .arg(arg!( --port -p [PORT] "The port to connect to").required(true))
        .subcommand(Command::new("echo").about("Ping the host"))
        .subcommand(
            Command::new("get")
                .about("Get a file from the host")
                .arg(arg!( --file -f [FILE] "The file to get").required(true))
                .arg(arg!( --destination -d [DISTINATION] "The destination").required(true)),
        );

    let matches = matches.get_matches();

    //Connect to the host
    let host_arg = matches.get_one::<String>("host").unwrap();
    let host = host_arg.to_owned();
    let port_arg = matches.get_one::<String>("port").unwrap();
    let port = port_arg.to_owned().parse::<usize>().unwrap();

    let mut djinn_client = DjinnClient::new(host, port).await.unwrap();

    match matches.subcommand() {
        Some(("echo", _matches)) => {
            djinn_client.echo().await;
        }
        Some(("get", matches)) => {
            debug!("Get command called");
            let file_arg = matches.get_one::<String>("file").unwrap();
            let file = file_arg.to_owned();
            debug!("File: {}", file);
            djinn_client.get_as_iterator(file).await;
        }
        _ => unreachable!(),
    }

    djinn_client
        .disconnect()
        .await
        .expect("Failed to disconnect");
}
