use std::io::Result;

use crate::invalid;

#[derive(Debug)]
pub struct CommandPart {
    pub tokens: Vec<String>,
    pub dependencies: Vec<String>,
    pub optional: bool,
}

#[derive(Debug)]
pub struct CommandDetails {
    pub command_name: String,
    pub parts: Vec<CommandPart>,
}

impl CommandDetails {
    pub fn new(command: &str) -> Result<CommandDetails> {
        let (command, arguments) = command
            .split_once(" ").ok_or_else(invalid!("Malformed command definition"))?;
        let mut arguments = arguments.trim();
        let mut parts = Vec::new();
        while !arguments.is_empty() {
            if arguments.starts_with("[") {
                let index = arguments.find("]").ok_or_else(invalid!("Invalid command definition (mismatched '[')"))?;
                let part = &arguments[1..index];
                let tokens: Vec<String> = part.trim().split(" ").map(String::from).collect();
                let dependencies: Vec<String> = tokens.iter().filter_map(|token| {
                    if token.starts_with("$") {
                        Some(String::from(&token[1..]))
                    } else {
                        None
                    }
                }).collect();
                parts.push(CommandPart {
                    tokens: tokens,
                    dependencies: dependencies,
                    optional: true
                });
                arguments = &arguments[index + 1..];
            } else {
                let index = arguments.find("[");
                let part = if let Some(index) = index {
                    &arguments[0..index]
                } else {
                    arguments
                };
                let tokens: Vec<String> = part.trim().split(" ").map(String::from).collect();
                let dependencies: Vec<String> = tokens.iter().filter_map(|token| {
                    if token.starts_with("$") {
                        Some(String::from(&token[1..]))
                    } else {
                        None
                    }
                }).collect();
                parts.push(CommandPart {
                    tokens: tokens,
                    dependencies: dependencies,
                    optional: false
                });
                arguments = if let Some(index) = index {
                    &arguments[index..]
                } else {
                    &arguments[0..0]
                }
            }
        }
        Ok(CommandDetails{
            command_name: String::from(command),
            parts: parts
        })
    }

}

#[cfg(test)]
mod test {
    use std::io::Result;

    use super::CommandDetails;

    #[test]
    fn new_command_with_single_optional() -> Result<()> {
        const INPUT: &str = "strip [-o $destination] $source";
        let details = CommandDetails::new(INPUT)?;
        assert_eq!("strip", details.command_name);
        assert_eq!(2, details.parts.len());
        let first = &details.parts[0];
        let second = &details.parts[1];
        assert!(first.optional);
        assert_eq!(&vec!["-o", "$destination"], &first.tokens);
        assert_eq!(&vec!["destination"], &first.dependencies);
        assert!(!second.optional);
        assert_eq!(&vec!["$source"], &second.tokens);
        assert_eq!(&vec!["source"], &second.dependencies);
        Ok(())
    }

    #[test]
    fn new_command_with_optional_in_the_middle() -> Result<()> {
        const INPUT: &str = "strip -v [-o $destination] $source";
        let details = CommandDetails::new(INPUT)?;
        assert_eq!("strip", details.command_name);
        assert_eq!(3, details.parts.len());
        let first = &details.parts[0];
        let second = &details.parts[1];
        let third = &details.parts[2];
        assert!(!first.optional);
        assert_eq!(&vec!["-v"], &first.tokens);
        assert!(first.dependencies.is_empty());
        assert!(second.optional);
        assert_eq!(&vec!["-o", "$destination"], &second.tokens);
        assert_eq!(&vec!["destination"], &second.dependencies);
        assert!(!third.optional);
        assert_eq!(&vec!["$source"], &third.tokens);
        assert_eq!(&vec!["source"], &third.dependencies);
        Ok(())
    }

    #[test]
    fn new_command_with_no_optional() -> Result<()> {
        const INPUT: &str = "strip -v $source";
        let details = CommandDetails::new(INPUT)?;
        assert_eq!("strip", details.command_name);
        assert_eq!(1, details.parts.len());
        let first = &details.parts[0];
        assert!(!first.optional);
        assert_eq!(&vec!["-v", "$source"], &first.tokens);
        assert_eq!(&vec!["source"], &first.dependencies);
        Ok(())
    }
}
