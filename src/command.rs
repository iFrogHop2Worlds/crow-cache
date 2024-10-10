pub enum Command {
    Get,
    Put,
    IsExpired,
    Invalid,
}

impl Command {
    pub fn get_command(str:  &String) -> Command {
        match str.as_bytes() {
            b"put" => Command::Put,
            b"get" => Command::Get,
            b"is_expired" => Command::IsExpired,
            _ => Command::Invalid
        }
    }
}