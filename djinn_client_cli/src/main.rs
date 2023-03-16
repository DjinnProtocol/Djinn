use clap::{arg, command, value_parser, ArgAction, Command};

fn main() {
    let matches = Command::new("djinn")
        .about("Djinn client CLI")
        .version("1.0")
        .arg(
            arg!(
                -h --host <host> "The host to connect to"
            )
        )
        .arg(
            arg!(
                -p --port <port> "The port to connect to"
            )
        )
        .subcommand(Command::new("echo")
            .about("Ping the server"));


    let matches = matches.get_matches();

    match matches.subcommand() {
        Some(("echo", matches)) => {
            
            return;
        }
        _ => unreachable!(),
    }
}
