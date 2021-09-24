mod copy;

use std::{convert::TryFrom, fs::File, io::Read, path::{Path, PathBuf}};

use log::debug;
use minidom::Element;

use crate::{error::Error, internal_error, runtime::Runtime, util::WorkingDirGuard};

pub use copy::CopyTask;

pub enum Task {
    Copy(CopyTask),
}

impl Task {
    pub fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
        match self {
            Task::Copy(copy_task) => copy_task.execute(runtime),
        }
    }
}

/* TODO: Explore if the following 2 functions can be implemented with a single macro */
fn combine_conditions(left: Option<&str>, right: Option<&str>) -> Option<String> {
    if let Some(left) = left {
        if let Some(right) = right {
            Some(format!("({}) && ({})", &left, &right))
        } else {
            Some(String::from(left))
        }
    } else {
        right.map(String::from)
    }
}

fn combine_paths(prefix: Option<&Path>, suffix: Option<&Path>) -> Option<PathBuf> {
    if let Some(prefix) = prefix {
        let mut prefix = prefix.to_string_lossy().to_string();
        if !prefix.ends_with(std::path::MAIN_SEPARATOR) {
            prefix.push(std::path::MAIN_SEPARATOR);
        }
        let prefix = PathBuf::from(prefix);
        if let Some(suffix) = suffix {
            Some(prefix.join(suffix))
        } else {
            Some(prefix)
        }
    } else {
        suffix.map(PathBuf::from)
    }
}

fn evaluate_condition(condition: &Option<String>, runtime: &Runtime) -> Result<bool, Error> {
    use eval::Expr;
    if let Some(condition) = condition {
        debug!("Evaluating condition {}", &condition);
        let expr = &runtime.variables.iter().fold(Expr::new(condition), |expr, (name, value)| {
            expr.value(name, value)
        });
        let value = expr.exec().map_err(|e| e.to_string())?;
        value.as_bool().ok_or(internal_error!("Expression {} does not evaluate to bool", &condition))
    } else {
        debug!("Empty condition");
        Ok(true)
    }

}

pub fn parse_input_file(runtime: &Runtime) -> Result<Vec<Task>, Error> {
    let input = File::open(&runtime.input).and_then(|mut file| {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    })?;
    let _guard = WorkingDirGuard::new(&runtime.working_directory)?;
    let xml_elements: Element = input.parse()?;
    xml_elements
        .children()
        .map(|task| {
            let task_name = task.name();
            match task_name {
                "copy" => {
                    let task = CopyTask::try_from(task)?;
                    Ok(Task::Copy(task))
                }
                _ => Err(internal_error!("Invalid task {}", task_name)),
            }
        })
        .collect()
}