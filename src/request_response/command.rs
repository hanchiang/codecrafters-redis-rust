#[derive(Debug, PartialEq)]
pub enum Command {
    PING,
    ECHO,
    GET,
}

impl Command {
    pub fn from(str: &str) -> Option<Command> {
        let mut command: Option<Command> = None;

        if str.to_lowercase() == "echo" {
            command = Some(Command::ECHO);
        } else if str.to_lowercase() == "ping" {
            command = Some(Command::PING);
        } else if str.to_lowercase() == "get" {
            command = Some(Command::GET);
        }

        command
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_none_if_no_command_match() {
        let result = Command::from("random");
        assert!(result.is_none());
    }

    #[test]
    fn return_echo_command() {
        let result = Command::from("echo");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Command::ECHO);
    }

    #[test]
    fn return_ping_command() {
        let result = Command::from("ping");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Command::PING);
    }

    #[test]
    fn return_get_command() {
        let result = Command::from("get");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Command::GET);
    }
}