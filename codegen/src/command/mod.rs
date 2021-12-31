mod generator;

use std::{fs::{self, File}, io::{Error, ErrorKind, Read, Result, Write}, path::Path};

use codegen::Module;
use serde::Deserialize;

use self::generator::{generate_group_definition, generate_group_impl, generate_item_definition, generate_item_impl, generate_parse_item, generate_parse_items, generate_task_struct, generate_parse_task};


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
    required: bool,
}


macro_rules! invalid {
    ($msg:literal) => {
        || Error::new(ErrorKind::InvalidData, $msg)
    };
}


#[derive(Debug, Deserialize)]
pub struct CommandLineDescriptor {
    linux: String,
    windows: String,
    osx: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Snippet(String),
    CommandLine(CommandLineDescriptor)
}

#[derive(Debug, Deserialize)]
pub struct ElementDescriptor {
    tag: String,
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
        let attr_allows = self.attributes.iter().map(allows_group).find(|predicate| *predicate);
        attr_allows.unwrap_or(false)
    }
}

#[derive(Debug, Deserialize)]
pub struct PluginDescriptor {
    name: String,
    command: Command,
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

    pub fn generate(&self) -> Module {
        let group_struct = generate_group_definition(&self.element);
        let group_impl = generate_group_impl(&self.element);
        let item_struct = generate_item_definition(&self.element);
        let item_impl = generate_item_impl(&self.element);
        let task_struct = generate_task_struct();
        let parse_item_fn = generate_parse_item();
        let parse_items_fn = generate_parse_items();
        let parse_task_fn = generate_parse_task();
        Module::new(&self.name)
            .push_struct(group_struct)
            .push_impl(group_impl)
            .push_struct(item_struct)
            .push_impl(item_impl)
            .push_struct(task_struct)
            .push_fn(parse_item_fn)
            .push_fn(parse_items_fn)
            .push_fn(parse_task_fn)
            .to_owned()
    }
}
