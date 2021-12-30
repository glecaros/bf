use std::{env, path::{Path, PathBuf}};

use log::debug;
use minidom::Element;

use crate::{error::Error, interpolation::interpolate, runtime::Runtime, task::evaluate_condition};

pub struct WorkingDirGuard {
    original_dir: PathBuf,
}

impl WorkingDirGuard {
    pub fn new(dir: &Path) -> Result<WorkingDirGuard, Error> {
        debug!("Switching working directory");
        let current_dir = env::current_dir()?;
        debug!("  from: {}", current_dir.to_string_lossy());
        debug!("  to:   {}", dir.to_string_lossy());
        env::set_current_dir(dir)?;
        Ok(WorkingDirGuard{ original_dir: current_dir })
    }
}

impl Drop for WorkingDirGuard {
    fn drop(&mut self) {
        debug!("Restoring working directory to {}", self.original_dir.to_string_lossy());
        env::set_current_dir(&self.original_dir).unwrap();
    }
}

pub fn interpolate_text(element: &Element, runtime: &Runtime) -> Result<Option<String>, Error> {
    if element.children().count() > 0 {
        Err(Error::from("Malformed item"))
    } else {
        let text = element.text();
        if text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(interpolate(&text, &runtime.variables)?))
        }
    }
}

pub const ATTR_CONDITION: &str = "condition";

pub fn interpolate_attribute(name: &str, element: &Element, runtime: &Runtime) -> Result<Option<String>, Error> {
    element.attr(name).map(|v| {
        interpolate(v, &runtime.variables)
    }).transpose()
}

pub fn evaluate_condition_from_element(runtime: &Runtime, element: &Element) -> Result<bool, Error> {
    let condition = element.attr(ATTR_CONDITION);
    evaluate_condition(condition, runtime)
}

pub trait ApplyPrefix {
    fn apply_prefix(&self, prefix: &Self) -> Self;
}

impl ApplyPrefix for Option<PathBuf> {
    fn apply_prefix(&self, prefix: &Self) -> Self {
        if let Some(prefix) = &prefix {
            let mut prefix = prefix.to_string_lossy().to_string();
            if !prefix.ends_with(std::path::MAIN_SEPARATOR) {
                prefix.push(std::path::MAIN_SEPARATOR);
            }
            let prefix = PathBuf::from(prefix);
            if let Some(suffix) = &self {
                Some(prefix.join(suffix))
            } else {
                Some(prefix)
            }
        } else {
            self.as_ref().map(PathBuf::from)
        }        
    }
}
