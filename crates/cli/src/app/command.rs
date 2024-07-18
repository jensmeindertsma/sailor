#[derive(Clone, Copy, Debug)]
pub enum Command {
    Help,
    Status,
    Application,
}

impl std::str::FromStr for Command {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "app" => Ok(Self::Application),
            "help" => Ok(Self::Help),
            "status" => Ok(Self::Status),
            other => Err(other.to_string()),
        }
    }
}
