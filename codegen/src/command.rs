use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Read, Result},
    path::Path,
};

use serde::Deserialize;

use crate::module::Module;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Path,
}

#[derive(Debug, Deserialize)]
pub struct ParameterDescriptor {
    name: String,
    #[serde(rename = "type")]
    parameter_type: ParameterType,
    required: bool,
}

#[derive(Debug, Deserialize)]
pub struct CommandDescriptor {
    linux: String,
    windows: String,
    osx: String,
}

#[derive(Debug, Deserialize)]
pub struct ElementDescriptor {
    tag: String,
    value: ParameterDescriptor,
    attributes: Vec<ParameterDescriptor>,
}

#[derive(Debug, Deserialize)]
pub struct PluginDescriptor {
    name: String,
    command: CommandDescriptor,
    element: ElementDescriptor,
}

impl PluginDescriptor {
    fn from_reader<R: Read>(reader: R) -> Result<PluginDescriptor> {
        serde_yaml::from_reader(reader).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }

    pub fn load_from_directory(plugin_path: &Path) -> Result<Vec<PluginDescriptor>> {
        fs::read_dir(plugin_path)?
            .map(|entry| entry.map(|e| e.path()))
            .filter_map(|entry| match entry {
                Ok(entry) => {
                    let extension = entry.extension();
                    if let Some(_) = extension {
                        Some(Ok(entry))
                    } else {
                        None
                    }
                }
                Err(err) => Some(Err(err)),
            })
            .map(|path| {
                let path = path?;
                let file = File::open(path)?;
                PluginDescriptor::from_reader(file)
            })
            .collect()
    }

    pub fn write_command(&self, target_module: Module) -> Result<()> {
        let command = if cfg!(target_os = "windows") {
            Ok(&self.command.windows)
        } else if cfg!(target_os = "linux") {
            Ok(&self.command.linux)
        } else if cfg!(target_os = "macos") {
            Ok(&self.command.osx)
        } else {
            Err(Error::new(
                ErrorKind::Unsupported,
                "Target OS not supported",
            ))
        }?;
        Ok(())
    }
}

struct CommandPart<'a> {
    tokens: Vec<&'a str>,
    dependencies: Vec<&'a str>,
    optional: bool,
}

struct CommandDetails<'a> {
    command_name: &'a str,
    parts: Vec<CommandPart<'a>>,
}

macro_rules! invalid {
    ($msg:literal) => {
        || Error::new(ErrorKind::InvalidData, $msg)
    };
}

fn split_command_parts<'a>(command: &'a str) -> Result<CommandDetails<'a>> {
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

#[cfg(test)]
mod test {
    use std::io::Result;

    use super::split_command_parts;

    #[test]
    fn split_command_parts_single_optional() -> Result<()> {
        const INPUT: &str = "strip [-o $destination] $source";
        let details = split_command_parts(INPUT)?;
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
    fn split_command_parts_optional_in_the_middle() -> Result<()> {
        const INPUT: &str = "strip -v [-o $destination] $source";
        let details = split_command_parts(INPUT)?;
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
    fn split_command_parts_no_optional() -> Result<()> {
        const INPUT: &str = "strip -v $source";
        let details = split_command_parts(INPUT)?;
        assert_eq!("strip", details.command_name);
        assert_eq!(1, details.parts.len());
        let first = &details.parts[0];
        assert!(!first.optional);
        assert_eq!(&vec!["-v", "$source"], &first.tokens);
        assert_eq!(&vec!["source"], &first.dependencies);
        Ok(())
    }
}
