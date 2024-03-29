mod command;
mod command_parser;
mod generator;

use std::{
    fs::File,
    io::{Error, ErrorKind, Result, Write},
    path::Path,
};

use codegen::{Module, Scope};
use command::TaskDescriptor;

use crate::generator::{generate_parse_input, generate_task_enum, generate_task_enum_impl};

fn load_tasks(base_path: &Path) -> Result<Vec<TaskDescriptor>> {
    if !base_path.is_dir() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Provided path is not a directory",
        ));
    }
    TaskDescriptor::load_from_directory(base_path)
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
    let tasks = load_tasks(source_path)?;
    let mut target_file = validate_and_open_target_file(target_file)?;
    let modules: Vec<Module> = tasks.iter().map(|command| command.generate()).collect();
    let task_enum = generate_task_enum(&tasks);
    let task_enum_impl = generate_task_enum_impl(&tasks);
    let parse_input = generate_parse_input(&tasks);
    let mut scope = Scope::new();
    for module in modules {
        scope.push_module(module);
    }
    scope.push_enum(task_enum);
    scope.push_impl(task_enum_impl);
    scope.push_fn(parse_input);
    writeln!(target_file, "{}", scope.to_string())
}
