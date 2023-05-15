use clap::{arg, Command};
use djinn_client_lib::DjinnClient;

#[macro_use]
extern crate log;


#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    // let mut djinn_client = DjinnClient::new("127.0.0.1".to_string(), 7777).await.unwrap();

    // return djinn_client.sync_internal("/".to_string(), "./files".to_string()).await;

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
        )
        .subcommand(
            Command::new("put")
                .about("Put a file on the host")
                .arg(arg!( --file -f [FILE] "The file to put").required(true))
                .arg(arg!( --destination -d [DISTINATION] "The destination").required(true)),
        )
        .subcommand(
            Command::new("sync")
                .about("Sync a directory")
                .arg(arg!( --path -p [PATH] "The path to sync").required(true))
                .arg(arg!( --target -t [TARGET] "The target to sync to").required(true)),
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
        Some(("put", matches)) => {
            debug!("Put command called");
            let file_arg = matches.get_one::<String>("file").unwrap();
            let file = file_arg.to_owned();
            debug!("File: {}", file);
            let destination_arg = matches.get_one::<String>("destination").unwrap();
            let destination = destination_arg.to_owned();
            debug!("Destination: {}", destination);
            // djinn_client.put(file, destination).await;
        }
        Some(("sync", matches)) => {
            debug!("Sync command called");
            let path_arg = matches.get_one::<String>("path").unwrap();
            let path = path_arg.to_owned();
            debug!("Path: {}", path);
            let target_arg = matches.get_one::<String>("target").unwrap();
            let target = target_arg.to_owned();

            djinn_client.sync_internal(path, target).await;
        }
        _ => unreachable!(),
    }

    djinn_client
        .disconnect()
        .await
        .expect("Failed to disconnect");
}
