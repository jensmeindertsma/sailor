use std::{env, process::Termination};

const SOCKET_PATH: &str = "/run/sail.socket";

fn main() -> impl Termination {
    let arguments: Vec<String> = env::args().skip(1).collect();

    println!("Arguments = {arguments:?}");

    todo!("connect to socket at `{SOCKET_PATH}`")
}
