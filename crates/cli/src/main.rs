mod command;
mod socket;

use std::{
    env,
    process::{ExitCode, Termination},
};

use sail_core::Request;
use socket::Socket;

const SOCKET_PATH: &str = "/run/sail.socket";

fn main() -> impl Termination {
    let arguments: Vec<String> = env::args().skip(1).collect();

    println!("Arguments = {arguments:?}");

    let mut socket = Socket::connect(SOCKET_PATH).unwrap();

    println!("Sending greeting ...");

    let response = socket.send_request(Request::Greeting);

    println!("Response to greeting = {response:?}");

    ExitCode::SUCCESS
}
