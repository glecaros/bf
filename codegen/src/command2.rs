
struct CommandPart<'a> {
    tokens: Vec<&'a str>,
    dependencies: Vec<&'a str>,
    optional: bool,
}

pub struct CommandDetails<'a> {
    command_name: &'a str,
    parts: Vec<CommandPart<'a>>,
}

impl<'a> CommandDetails<'a> {
    fn new(command: &'a str) -> Result<CommandDetails<'a>> {
        let (command, arguments) = command
            .split_once(" ").ok_or_else(invalid!("Malformed command definition"))?;
        let mut arguments = arguments.trim();
        let mut parts = Vec::new();
        while !arguments.is_empty() {
            if arguments.starts_with("[") {
                let index = arguments.find("]").ok_or_else(invalid!("Invalid command definition (mismatched '[')"))?;
                let part = &arguments[1..index];
                let tokens: Vec<&str> = part.trim().split(" ").collect();
                let dependencies: Vec<&str> = tokens.iter().filter_map(|&token| {
                    if token.starts_with("$") {
                        Some(&token[1..])
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
                let tokens: Vec<&str> = part.trim().split(" ").collect();
                let dependencies: Vec<&str> = tokens.iter().filter_map(|&token| {
                    if token.starts_with("$") {
                        Some(&token[1..])
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
            command_name: command,
            parts: parts
        })
    }
    
    
    // pub write_execute_function()
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
