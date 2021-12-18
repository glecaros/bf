mod generator;

use std::{fs::{self, File}, io::{Error, ErrorKind, Read, Result}, path::Path};

use serde::Deserialize;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Path,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
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
pub struct CommandDescriptor {
    linux: String,
    windows: String,
    osx: String,
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

    fn generate(&self) -> Result<String> {
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
        todo!()
    }
}
