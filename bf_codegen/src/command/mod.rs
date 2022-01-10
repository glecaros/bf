mod command_parser;
mod generator;

use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Read, Result},
    path::Path,
};

use codegen::Module;
use serde::{de::Visitor, Deserialize, Deserializer};

use self::{
    command_parser::CommandDetails,
    generator::{
        generate_execute_fn, generate_group_definition, generate_group_impl,
        generate_item_definition, generate_item_impl, generate_parse_item, generate_parse_items,
        generate_parse_task, generate_task_impl, generate_task_struct,
    },
};

pub use generator::generate_parse_input;
pub use generator::generate_task_enum;
pub use generator::generate_task_enum_impl;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Path,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupSetting {
    None,
    Inherit,
    Prefix,
    InheritPrefix,
}

#[derive(Debug, Deserialize)]
pub struct ParameterDescriptor {
    name: String,
    #[serde(rename = "type")]
    parameter_type: ParameterType,
    allow_group: GroupSetting,
    defaults_to: Option<String>,
    required: bool,
}

#[macro_export]
macro_rules! invalid {
    ($msg:literal) => {
        || {
            use std::io::{Error, ErrorKind};
            Error::new(ErrorKind::InvalidData, $msg)
        }
    };
}

struct CommandDetailsVisitor;

impl<'de> Visitor<'de> for CommandDetailsVisitor {
    type Value = CommandDetails;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A string representing a command")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CommandDetails::new(&v).map_err(serde::de::Error::custom)
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CommandDetails::new(&v).map_err(serde::de::Error::custom)
    }
}

fn parse_command<'de, D>(deserializer: D) -> std::result::Result<CommandDetails, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(CommandDetailsVisitor)
}

#[derive(Debug, Deserialize)]
pub struct CommandLineDescriptor {
    #[serde(deserialize_with = "parse_command")]
    linux: CommandDetails,
    #[serde(deserialize_with = "parse_command")]
    windows: CommandDetails,
    #[serde(deserialize_with = "parse_command")]
    osx: CommandDetails,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Snippet(String),
    CommandLine(CommandLineDescriptor),
}

#[derive(Debug, Deserialize)]
pub struct ElementDescriptor {
    attributes: Vec<ParameterDescriptor>,
}

impl ElementDescriptor {
    fn uses_groups(&self) -> bool {
        let allows_group = |parameter_descriptor: &ParameterDescriptor| -> bool {
            match parameter_descriptor.allow_group {
                GroupSetting::None => false,
                _ => true,
            }
        };
        let attr_allows = self
            .attributes
            .iter()
            .map(allows_group)
            .find(|predicate| *predicate);
        attr_allows.unwrap_or(false)
    }
}

#[derive(Debug, Deserialize)]
pub struct TaskDescriptor {
    name: String,
    command: Command,
    element: ElementDescriptor,
}

impl TaskDescriptor {
    fn from_reader<R: Read>(reader: R) -> Result<TaskDescriptor> {
        serde_yaml::from_reader(reader).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }

    pub fn load_from_directory(tasks_path: &Path) -> Result<Vec<TaskDescriptor>> {
        fs::read_dir(tasks_path)?
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
                TaskDescriptor::from_reader(file)
            })
            .collect()
    }

    pub fn generate(&self) -> Module {
        let group_struct = generate_group_definition(&self.element);
        let group_impl = generate_group_impl(&self.element);
        let item_struct = generate_item_definition(&self.element);
        let item_impl = generate_item_impl(&self.element);
        let task_struct = generate_task_struct();
        let task_impl = generate_task_impl(&self);
        let execute_fn = generate_execute_fn(&self);
        let parse_item_fn = generate_parse_item();
        let parse_items_fn = generate_parse_items();
        let parse_task_fn = generate_parse_task();
        Module::new(&self.name)
            .import("std::path", "PathBuf")
            .import("minidom", "Element")
            .import("crate::runtime", "Runtime")
            .import("crate::error", "Error")
            .import("crate::util", "interpolate_attribute")
            .import("crate::util", "ApplyPrefix")
            .import("crate::util", "evaluate_condition_from_element")
            .import("crate", "internal_error")
            .push_struct(group_struct)
            .push_impl(group_impl)
            .push_struct(item_struct)
            .push_impl(item_impl)
            .push_struct(task_struct)
            .push_impl(task_impl)
            .push_fn(execute_fn)
            .push_fn(parse_item_fn)
            .push_fn(parse_items_fn)
            .push_fn(parse_task_fn)
            .to_owned()
    }
}
