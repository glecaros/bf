mod copy;

use std::{
    convert::TryFrom,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

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

fn evaluate_condition(condition: Option<&str>, runtime: &Runtime) -> Result<bool, Error> {
    use eval::Expr;
    if let Some(condition) = condition {
        debug!("Evaluating condition {}", &condition);
        let expr = &runtime
            .variables
            .iter()
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

fn parse_input(runtime: &Runtime, input: &str) -> Result<Vec<Task>, Error> {
    let _guard = WorkingDirGuard::new(&runtime.working_directory)?;
    let xml_elements: Element = input.parse()?;
    xml_elements
        .children()
        .map(|task| {
            let task_name = task.name();
            let condition = task.attr("condition");
            let condition = evaluate_condition(condition, &runtime)?;
            if condition {
                match task_name {
                    "copy" => {
                        let task = CopyTask::try_from(task)?;
                        Ok(Some(Task::Copy(task)))
                    }
                    _ => Err(internal_error!("Invalid task {}", task_name)),
                }
            } else {
                Ok(None)
            }
        })
        .filter_map(|x| match x {
            Ok(v) => v.map(|task| Ok(task)),
            Err(e) => Some(Err(e)),
        })
        .collect()
}

pub fn parse_input_file(runtime: &Runtime) -> Result<Vec<Task>, Error> {
    let input = File::open(&runtime.input).and_then(|mut file| {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    })?;
    parse_input(runtime, &input)
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::PathBuf};

    use crate::{
        runtime::Runtime,
        task::{parse_input, Task},
    };

    use super::evaluate_condition;

    fn new_runtime() -> Runtime {
        Runtime {
            input: PathBuf::new(),
            working_directory: PathBuf::new(),
            variables: HashMap::new(),
            dry_run: true,
            source_base: None,
            destination_base: None,
        }
    }

    #[test]
    fn evaluate_condition_no_condition() {
        let runtime = new_runtime();
        let result = evaluate_condition(None, &runtime);
        assert!(matches!(result, Ok(_)));
        let result = result.unwrap();
        assert!(result);
    }

    #[test]
    fn evaluate_condition_test_single_variable_present() {
        let mut runtime = new_runtime();
        runtime.variables.insert(String::from("var"), String::from("value"));
        const CONDITION: &str = "var == 'value'";
        let result = evaluate_condition(Some(CONDITION), &runtime);
        assert!(matches!(result, Ok(_)));
        let result = result.unwrap();
        assert!(result); 
    }


    #[test]
    fn single_task_failed_condition_variable_present() {
        const INPUT: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <tasks xmlsns="https://github.com/glecaros/bf">
            <copy condition="var == 'value'">
                <item>item.file</item>
            </copy>
        </tasks>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("var"), String::from("wrong_value"));
        let tasks = parse_input(&runtime, INPUT);
        assert!(matches!(tasks, Ok(_)));
        let tasks = tasks.unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn single_task_with_single_item() {
        const INPUT: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <tasks xmlsns="https://github.com/glecaros/bf">
            <copy>
                <item>item.file</item>
            </copy>
        </tasks>
        "#;
        let runtime = new_runtime();
        let tasks = super::parse_input(&runtime, INPUT);
        assert!(matches!(tasks, Ok(_)));
        let tasks = tasks.unwrap();
        assert_eq!(tasks.len(), 1);
        let task = tasks.first().unwrap();
        assert!(matches!(task, Task::Copy(_)));
        let Task::Copy(task) = task;
        assert_eq!(task.item_count(), 1);
    }

    // fn single_task_with_multiple_items() {
    //     const input: &str = r#"
    //     <?xml version="1.0" encoding="UTF-8"?>
    //     <tasks xmlsns="https://github.com/glecaros/bf">
    //         <copy>
    //             <group source="srcdir" destination="outdir">
    //             <group destination="lib">
    //                 <item>lib1.so</item>
    //                 <group destination="extra">
    //                     <item>folder_a/liba.so</item>
    //                 </group>
    //             </group>
    //             <group source="doc" destination="docs">
    //                 <item destination="README.md">my_doc.md</item>
    //             </group>
    //             </group>
    //         </copy>
    //     </tasks>
    //     "#;
    // }
}
