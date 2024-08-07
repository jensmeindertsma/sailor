mod app;

pub enum Command {
    Applications { subcommand: app::Command },
}
