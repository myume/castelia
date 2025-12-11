#[derive(Debug)]
pub enum CommandMessage {
    Command,
    Data,
    SharedObject,
    Audio,
    Video,
    Aggregate,
}
