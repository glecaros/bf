use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use log::debug;
use minidom::Element;

use crate::{error::Error, internal_error, interpolation::interpolate, runtime::Runtime};

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
        Ok(WorkingDirGuard {
            original_dir: current_dir,
        })
    }
}

impl Drop for WorkingDirGuard {
    fn drop(&mut self) {
        debug!(
            "Restoring working directory to {}",
            self.original_dir.to_string_lossy()
        );
        env::set_current_dir(&self.original_dir).unwrap();
    }
}

pub const ATTR_CONDITION: &str = "condition";

pub fn interpolate_attribute(
    name: &str,
    element: &Element,
    runtime: &Runtime,
) -> Result<Option<String>, Error> {
    element
        .attr(name)
        .map(|v| {
            let variables = runtime.variables.clone().into_iter().collect();
            interpolate(v, &variables)
        })
        .transpose()
}

pub fn evaluate_condition(condition: Option<&str>, runtime: &Runtime) -> Result<bool, Error> {
    use eval::Expr;
    let variables: HashMap<String, String> = runtime.variables.clone().into_iter().collect();
    debug!("variables: {:?}", variables);
    if let Some(condition) = condition {
        debug!("Evaluating condition {}", &condition);
        let expr = &variables
            .into_iter()
            .fold(Expr::new(condition), |expr, (name, value)| {
                expr.value(name, value)
            });
        let value = expr.exec().map_err(|e| e.to_string())?;
        value.as_bool().ok_or(internal_error!(
            "Expression {} does not evaluate to bool",
            &condition
        ))
    } else {
        debug!("Empty condition");
        Ok(true)
    }
}

pub fn evaluate_condition_from_element(
    runtime: &Runtime,
    element: &Element,
) -> Result<bool, Error> {
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

#[cfg(test)]
mod test {
    use crate::{runtime::Runtime, util::evaluate_condition};

    #[test]
    fn evaluate_condition_no_condition() {
        let runtime = Runtime::default();
        let result = evaluate_condition(None, &runtime);
        assert!(matches!(result, Ok(_)));
        let result = result.unwrap();
        assert!(result);
    }

    #[test]
    fn evaluate_condition_test_single_variable_present() {
        let mut runtime = Runtime::default();
        runtime
            .variables
            .push((String::from("var"), String::from("value")));
        const CONDITION: &str = "var == 'value'";
        let result = evaluate_condition(Some(CONDITION), &runtime);
        assert!(matches!(result, Ok(_)));
        let result = result.unwrap();
        assert!(result);
    }

    #[test]
    fn evaluate_condition_test_single_variable_not_present() {
        let runtime = Runtime::default();
        const CONDITION: &str = "var == 'value'";
        let result = evaluate_condition(Some(CONDITION), &runtime);
        assert!(matches!(result, Ok(_)));
        let result = result.unwrap();
        assert!(!result);
    }

    #[test]
    fn evaluate_condition_test_single_variable_wrong() {
        let mut runtime = Runtime::default();
        runtime
            .variables
            .push((String::from("var"), String::from("wrong")));
        const CONDITION: &str = "var == 'value'";
        let result = evaluate_condition(Some(CONDITION), &runtime);
        assert!(matches!(result, Ok(_)));
        let result = result.unwrap();
        assert!(!result);
    }

    #[test]
    fn evaluate_condition_test_not_bool() {
        let runtime = Runtime::default();
        const CONDITION: &str = "3 + 1'";
        let result = evaluate_condition(Some(CONDITION), &runtime);
        assert!(matches!(result, Err(_)));
    }
}
