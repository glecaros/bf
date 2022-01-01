mod command;
mod dependency_tracker;
mod module;

use std::{
    fs::File,
    io::{Error, ErrorKind, Result, Write},
    path::Path,
};

use codegen::{Module, Scope};
use command::PluginDescriptor;

use crate::command::generate_task_enum;

fn load_commands(base_path: &Path) -> Result<Vec<PluginDescriptor>> {
    if !base_path.is_dir() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Provided path is not a directory",
        ));
    }
    PluginDescriptor::load_from_directory(base_path)
}

fn validate_and_open_target_file(target_file: &Path) -> Result<File> {
    use ErrorKind::*;
    if target_file.is_dir() {
        Err(Error::new(
            InvalidInput,
            "Invalid target file (should not be a directory)",
        ))?;
    }
    let parent = target_file
        .parent()
        .ok_or(Error::new(NotFound, "Invalid target directory"))?;
    if !parent.is_dir() {
        Err(Error::new(
            InvalidInput,
            "Invalid target file (parent is not a directory)",
        ))?;
    }
    File::create(target_file)
}

pub fn generate_from_path(source_path: &Path, target_file: &Path) -> Result<()> {
    let commands = load_commands(source_path)?;
    let mut target_file = validate_and_open_target_file(target_file)?;
    let modules: Vec<Module> = commands.iter().map(|command| command.generate()).collect();
    let mut scope = Scope::new();
    for module in modules {
        scope.push_module(module);
    }
    let task_enum = generate_task_enum(&commands);
    scope.push_enum(task_enum);
    writeln!(target_file, "{}", scope.to_string())
}
