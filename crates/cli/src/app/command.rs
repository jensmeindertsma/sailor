#[derive(Clone, Copy, Debug)]
pub enum Command {
    Help,
    Status,
    Application,
}

// TODO: we want more here, a way to view docker logs, and a way to rotate upload secret

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
