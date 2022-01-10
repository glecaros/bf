mod group;
mod item;

use codegen::{Block, Enum, Function, Struct, Type};
use codegen::{Impl, Variant};
use convert_case::{Case, Casing};
use regex::Regex;

pub use group::generate_group_definition;
pub use group::generate_group_impl;
pub use item::generate_item_definition;
pub use item::generate_item_impl;

use super::command_parser::CommandPart;
use super::{CommandLineDescriptor, TaskDescriptor};

macro_rules! t {
    ($ty:literal) => {
        Type::new($ty)
    };
}

pub fn generate_parse_item() -> Function {
    let if_block = Block::new("let item = if condition")
        .line("let item = Item::create(element, parent, runtime)?;")
        .line("Some(item)")
        .to_owned();
    let else_block = Block::new("else").line("None").after(";").to_owned();
    Function::new("parse_item")
        .arg("runtime", t!("&Runtime"))
        .arg("element", t!("&Element"))
        .arg("parent", t!("&Group"))
        .ret(t!("Result<Option<Item>, Error>"))
        .line("let condition = evaluate_condition_from_element(runtime, element)?;")
        .push_block(if_block)
        .push_block(else_block)
        .line("Ok(item)")
        .to_owned()
}

fn generate_parse_items_loop() -> Block {
    let item_arm = Block::new("\"item\" => ")
        .line("let item = parse_item(runtime, item, &group)?;")
        .push_block(
            Block::new("if let Some(item) = item")
                .line("items.push(item);")
                .to_owned(),
        )
        .to_owned();
    let group_arm = Block::new("\"group\" => ")
        .line("let inner_items = parse_items(runtime, item, Some(&group))?;")
        .push_block(
            Block::new("if let Some(mut inner_items) = inner_items")
                .line("items.append(&mut inner_items);")
                .to_owned(),
        )
        .to_owned();
    let catch_all_arm = Block::new("_ =>")
        .line("Err(internal_error!(\"Invalid element: {}\", item.name()))?;")
        .to_owned();
    let match_block = Block::new("match item.name()")
        .push_block(item_arm)
        .push_block(group_arm)
        .push_block(catch_all_arm)
        .to_owned();
    Block::new("for item in parent.children()")
        .push_block(match_block)
        .to_owned()
}

pub fn generate_parse_items() -> Function {
    let if_block = Block::new("let items = if condition")
        .line("let group = Group::create(parent, group, runtime)?;")
        .line("let mut items = Vec::new();")
        .push_block(generate_parse_items_loop())
        .line("Some(items)")
        .to_owned();
    let else_block = Block::new("else").line("None").after(";").to_owned();
    Function::new("parse_items")
        .arg("runtime", t!("&Runtime"))
        .arg("parent", t!("&Element"))
        .arg("group", t!("Option<&Group>"))
        .ret(t!("Result<Option<Vec<Item>>, Error>"))
        .line("let condition = evaluate_condition_from_element(runtime, parent)?;")
        .push_block(if_block)
        .push_block(else_block)
        .line("Ok(items)")
        .to_owned()
}

pub fn generate_task_struct() -> Struct {
    Struct::new("Task")
        .derive("Debug")
        .vis("pub")
        .field("items", t!("Vec<Item>"))
        .to_owned()
}

pub fn generate_task_impl(task: &TaskDescriptor) -> Impl {
    let snake_name = task.name.to_case(Case::Snake);
    let for_block = Block::new("for item in &self.items")
        .line(format!("{}(item)?;", snake_name))
        .to_owned();
    let run_fn = Function::new("run")
        .vis("pub")
        .arg_ref_self()
        .ret(t!("Result<(), Error>"))
        .push_block(for_block)
        .line("Ok(())")
        .to_owned();
    Impl::new("Task").push_fn(run_fn).to_owned()
}

fn add_command_part_handling(function: &mut Function, part: &CommandPart) {
    if part.optional {
        let lhs = part
            .dependencies
            .iter()
            .map(|dep| format!("Some({})", dep))
            .collect::<Vec<String>>()
            .join(", ");
        let rhs = part
            .dependencies
            .iter()
            .map(|dep| format!("&item.{}", dep))
            .collect::<Vec<String>>()
            .join(", ");
        let if_stmt = if part.dependencies.len() > 1 {
            format!("if let ({}) = ({})", lhs, rhs)
        } else {
            format!("if let {} = {}", lhs, rhs)
        };
        let mut block = Block::new(&if_stmt);
        for token in &part.tokens {
            if token.starts_with("$") {
                block.line(format!("call.arg(&{});", &token[1..]));
            } else {
                block.line(format!("call.arg(\"{}\");", &token));
            };
        }
        function.push_block(block);
    } else {
        for token in &part.tokens {
            if token.starts_with("$") {
                function.line(format!("call.arg(&item.{});", &token[1..]));
            } else {
                function.line(format!("call.arg(\"{}\");", &token));
            };
        }
    }
}

fn generate_command_line_execute(
    descriptor: &CommandLineDescriptor,
    mut function: Function,
) -> Function {
    let command = if cfg!(target_os = "windows") {
        &descriptor.windows
    } else if cfg!(target_os = "linux") {
        &descriptor.linux
    } else if cfg!(target_os = "macos") {
        &descriptor.osx
    } else {
        panic!("Unsupported OS")
    };
    function.line("use std::process::Command;");
    function.line(format!(
        "let mut call = Command::new(\"{}\");",
        &command.command_name
    ));
    for part in &command.parts {
        add_command_part_handling(&mut function, part);
    }
    function
        .line("let output = call.output()?;")
        .line("let status = output.status;")
        .push_block(Block::new("if status.success()").line("Ok(())").to_owned())
        .push_block(
            Block::new("else")
                .line("let std_err = std::str::from_utf8(&output.stderr)?;")
                .line("Err(Error::from(std_err))")
                .to_owned(),
        )
        .to_owned()
}

pub fn generate_execute_fn(task: &TaskDescriptor) -> Function {
    let snake_name = task.name.to_case(Case::Snake);
    let mut execute_fn = Function::new(&snake_name)
        .arg("item", t!("&Item"))
        .ret(t!("Result<(), Error>"))
        .to_owned();
    use super::Command;
    match &task.command {
        Command::Snippet(code) => {
            let re = Regex::new(r"\$\{(?P<var>[a-z][a-z0-9_]*)\}").unwrap();
            let output = re.replace_all(&code, "&item.$var");
            execute_fn.line(&output).to_owned()
        }
        Command::CommandLine(command_line) => {
            generate_command_line_execute(&command_line, execute_fn)
        }
    }
}

pub fn generate_parse_task() -> Function {
    let constructor = Block::new("Task").line("items: items").to_owned();
    let map_call = Block::new("let task = items.map(|items|")
        .push_block(constructor)
        .after(");")
        .to_owned();
    Function::new("parse_task")
        .vis("pub")
        .arg("runtime", t!("&Runtime"))
        .arg("parent", t!("&Element"))
        .ret(t!("Result<Option<Task>, Error>"))
        .line("let items = parse_items(runtime, parent, None)?;")
        .push_block(map_call)
        .line("Ok(task)")
        .to_owned()
}

fn generate_variant(task: &TaskDescriptor) -> Variant {
    let pascal_name = task.name.to_case(Case::Pascal);
    let snake_name = task.name.to_case(Case::Snake);
    let task_type = format!("{}::Task", snake_name);
    Variant::new(&pascal_name).tuple(&task_type).to_owned()
}

pub fn generate_task_enum(tasks: &Vec<TaskDescriptor>) -> Enum {
    let mut enum_definition = Enum::new("Task").derive("Debug").vis("pub").to_owned();
    for task in tasks {
        let variant = generate_variant(&task);
        enum_definition.push_variant(variant);
    }
    enum_definition
}

pub fn generate_task_enum_impl(tasks: &Vec<TaskDescriptor>) -> Impl {
    let mut match_block = Block::new("match &self");
    for task in tasks {
        let snake_name = task.name.to_case(Case::Snake);
        let pascal_name = task.name.to_case(Case::Pascal);
        match_block.line(format!(
            "Task::{pascal}({snake}) => {snake}.run(),",
            snake = snake_name,
            pascal = pascal_name
        ));
    }
    let run_fn = Function::new("run")
        .vis("pub")
        .arg_ref_self()
        .ret(t!("Result<(), Error>"))
        .push_block(match_block)
        .to_owned();
    Impl::new("Task").push_fn(run_fn).to_owned()
}

fn generate_parse_input_match(tasks: &Vec<TaskDescriptor>) -> Block {
    let mut match_block = Block::new("match task_name");
    for task in tasks {
        let name_snake = task.name.to_case(Case::Snake);
        let name_pascal = task.name.to_case(Case::Pascal);
        let block = Block::new(&format!("\"{}\" =>", &name_snake))
            .line(format!(
                "let task = {}::parse_task(runtime, task)?.map(|task| Task::{}(task));",
                &name_snake, &name_pascal
            ))
            .line("Ok(task)")
            .after(",")
            .to_owned();
        match_block.push_block(block);
    }
    match_block
        .line("_ => Err(Error::from(format!(\"Invalid task '{}'\", task_name))),")
        .to_owned()
}

pub fn generate_parse_input(tasks: &Vec<TaskDescriptor>) -> Function {
    let map_block = Block::new("    .map(|task|")
        .line("let task_name = task.name();")
        .push_block(generate_parse_input_match(tasks))
        .after(")")
        .to_owned();
    let filter_map_block = Block::new("    .filter_map(|x| match x")
        .line("Ok(task) => task.map(|task| Ok(task)),")
        .line("Err(err) => Some(Err(err)),")
        .after(")")
        .to_owned();
    let parse_input = Function::new("parse_input")
        .arg("runtime", t!("&Runtime"))
        .arg("input", t!("&str"))
        .ret(t!("Result<Vec<Task>, Error>"))
        .line("let xml_elements: Element = input.parse()?;")
        .line("xml_elements")
        .line("    .children()")
        .push_block(map_block)
        .push_block(filter_map_block)
        .line("    .collect()")
        .to_owned();
    parse_input
}

#[cfg(test)]
mod test {
    use codegen::{Enum, Function, Impl, Scope, Struct};
    use regex::Regex;

    use crate::command::{
        generator::generate_parse_task, Command, ElementDescriptor, TaskDescriptor,
    };

    use super::{
        generate_parse_input, generate_parse_item, generate_parse_items, generate_task_enum,
        generate_task_enum_impl, generate_task_impl, generate_task_struct,
    };

    fn function_to_string(item: Function) -> String {
        Scope::new().push_fn(item).to_string()
    }

    fn struct_to_string(item: Struct) -> String {
        Scope::new().push_struct(item).to_string()
    }

    fn enum_to_string(item: Enum) -> String {
        Scope::new().push_enum(item).to_string()
    }

    fn impl_to_string(item: Impl) -> String {
        Scope::new().push_impl(item).to_string()
    }

    fn normalize(s: &str) -> String {
        let regex = Regex::new("[\\n\\s]+").unwrap();
        regex.replace_all(s.trim(), " ").to_string()
    }

    fn compare_function(item: Function, expected: &str) {
        assert_eq!(normalize(&function_to_string(item)), normalize(expected))
    }

    fn compare_struct(item: Struct, expected: &str) {
        assert_eq!(normalize(&struct_to_string(item)), normalize(expected))
    }

    fn compare_enum(item: Enum, expected: &str) {
        assert_eq!(normalize(&enum_to_string(item)), normalize(expected))
    }

    fn compare_impl(item: Impl, expected: &str) {
        assert_eq!(normalize(&impl_to_string(item)), normalize(expected))
    }

    fn mock_task(name: &str) -> TaskDescriptor {
        TaskDescriptor {
            name: String::from(name),
            command: Command::Snippet(String::from("asdf")),
            element: ElementDescriptor { attributes: vec![] },
        }
    }

    #[test]
    fn parse_item() {
        let item = generate_parse_item();
        const EXPECTED: &str = r#"
        fn parse_item(runtime: &Runtime, element: &Element, parent: &Group) -> Result<Option<Item>, Error> {
            let condition = evaluate_condition_from_element(runtime, element)?;
            let item = if condition {
                let item = Item::create(element, parent, runtime)?;
                Some(item)
            } else {
                None
            };
            Ok(item)
        }
        "#;
        compare_function(item, EXPECTED);
    }

    #[test]
    fn parse_items() {
        let item = generate_parse_items();
        const EXPECTED: &str = r#"
        fn parse_items(runtime: &Runtime, parent: &Element, group: Option<&Group>) -> Result<Option<Vec<Item>>, Error> {
            let condition = evaluate_condition_from_element(runtime, parent)?;
            let items = if condition {
                let group = Group::create(parent, group, runtime)?;
                let mut items = Vec::new();
                for item in parent.children() {
                    match item.name() {
                        "item" => {
                            let item = parse_item(runtime, item, &group)?;
                            if let Some(item) = item {
                                items.push(item);
                            }
                        }
                        "group" => {
                            let inner_items = parse_items(runtime, item, Some(&group))?;
                            if let Some(mut inner_items) = inner_items {
                                items.append(&mut inner_items);
                            }
                        }
                        _ => {
                            Err(internal_error!("Invalid element: {}", item.name()))?;
                        }
                    }
                }
                Some(items)
            } else {
                None
            };
            Ok(items)
        }
        "#;
        compare_function(item, EXPECTED);
    }

    #[test]
    fn task_struct() {
        let item = generate_task_struct();
        const EXPECTED: &str = r#"
        #[derive(Debug)]
        pub struct Task {
            items: Vec<Item>,
        }"#;
        compare_struct(item, EXPECTED);
    }

    #[test]
    fn task_impl() {
        let copy = mock_task("copy");
        let item = generate_task_impl(&copy);
        const EXPECTED: &str = r#"
        impl Task {
            pub fn run(&self) -> Result<(), Error> {
                for item in &self.items {
                    copy(item)?;
                }
                Ok(())
            }
        }
        "#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn parse_task() {
        let item = generate_parse_task();
        const EXPECTED: &str = r#"
        pub fn parse_task(runtime: &Runtime, parent: &Element) -> Result<Option<Task>, Error> {
            let items = parse_items(runtime, parent, None)?;
            let task = items.map(|items| {
                Task {
                    items: items
                }
            });
            Ok(task)
        }"#;
        compare_function(item, EXPECTED);
    }

    #[test]
    fn task_enum() {
        let tasks = vec![mock_task("copy"), mock_task("strip")];
        let enum_definition = generate_task_enum(&tasks);
        const EXPECTED: &str = r#"
        #[derive(Debug)]
        pub enum Task {
            Copy(copy::Task),
            Strip(strip::Task),
        }"#;
        compare_enum(enum_definition, EXPECTED);
    }

    #[test]
    fn task_enum_impl() {
        let tasks = vec![mock_task("copy"), mock_task("strip")];
        let impl_definition = generate_task_enum_impl(&tasks);
        const EXPECTED: &str = r#"
        impl Task {
            pub fn run(&self) -> Result<(), Error> {
                match &self {
                    Task::Copy(copy) => copy.run(),
                    Task::Strip(strip) => strip.run(),
                }
            }
        }"#;
        compare_impl(impl_definition, EXPECTED);
    }

    #[test]
    fn parse_input() {
        let tasks = vec![mock_task("copy"), mock_task("strip")];
        let parse_input_fn = generate_parse_input(&tasks);
        const EXPECTED: &str = r#"
        fn parse_input(runtime: &Runtime, input: &str) -> Result<Vec<Task>, Error> {
            let xml_elements: Element = input.parse()?;
            xml_elements
                .children()
                .map(|task| {
                    let task_name = task.name();
                    match task_name {
                        "copy" => {
                            let task = copy::parse_task(runtime, task)?.map(|task| Task::Copy(task));
                            Ok(task)
                        },
                        "strip" => {
                            let task = strip::parse_task(runtime, task)?.map(|task| Task::Strip(task));
                            Ok(task)
                        },
                        _ => Err(Error::from(format!("Invalid task '{}'", task_name))),
                    }
                })
                .filter_map(|x| match x {
                    Ok(task) => task.map(|task| Ok(task)),
                    Err(err) => Some(Err(err)),
                })
                .collect()
        }
        "#;
        compare_function(parse_input_fn, EXPECTED);
    }
}
