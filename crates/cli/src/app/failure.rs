use super::controller;

#[derive(Debug)]
pub enum Failure {
    ControllerError(controller::Error),
    MissingCommand,
    UnknownCommand(String),
}
