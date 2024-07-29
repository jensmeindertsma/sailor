mod app;

use app::controller;
use app::Failure;
use std::{
    env,
    process::{ExitCode, Termination},
};

const SOCKET_PATH: &str = "/run/sail.socket";

fn main() -> impl Termination {
    let arguments: Vec<String> = env::args().skip(1).collect();

    if let Err(failure) = app::run(arguments) {
        match failure {
            Failure::ControllerError(error) => match error {
                controller::Error::ConnectionFailure(io_error_kind) => {
                    eprintln!("ERROR: controller failure: {:?}", io_error_kind)
                }
            },
            Failure::MissingCommand => {
                eprintln!("ERROR: missing command")
            }
            Failure::UnknownCommand(command) => {
                eprintln!("ERROR: unknown command `{command:?}`")
            }
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
