mod command;
pub mod controller;
mod failure;
mod modules;

use crate::SOCKET_PATH;
use command::Command;
use controller::Controller;
pub use failure::Failure;

pub fn run(arguments: impl IntoIterator<Item = String>) -> Result<(), Failure> {
    let mut arguments = arguments.into_iter();

    let command: Command = arguments
        .next()
        .ok_or(Failure::MissingCommand)?
        .parse()
        .map_err(Failure::UnknownCommand)?;

    let mut controller = Controller::connect(SOCKET_PATH).map_err(Failure::ControllerError)?;

    match command {
        Command::Application => modules::application(&mut controller, arguments)?,
        Command::Help => modules::help(),
        Command::Status => modules::status(&mut controller),
    };

    Ok(())
}
