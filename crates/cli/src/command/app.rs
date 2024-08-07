pub enum Command {
    Create { name: String },
    Delete { name: String },
    List,
    Status,
}
