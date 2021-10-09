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
