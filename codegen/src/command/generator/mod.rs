mod group;
mod item;

use codegen::Variant;
use codegen::{Block, Function, Type, Enum, Struct};

macro_rules! t {
    ($ty:literal) => {
        Type::new($ty)
    };
}

use convert_case::{Case, Casing};
pub use group::generate_group_definition;
pub use group::generate_group_impl;
pub use item::generate_item_definition;
pub use item::generate_item_impl;

use super::PluginDescriptor;

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
        .vis("pub")
        .field("items", t!("Vec<Item>"))
        .to_owned()
}

pub fn generate_parse_task() -> Function {
    let constructor = Block::new("Task")
        .line("items: items")
        .to_owned();
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

fn generate_variant(task: &PluginDescriptor) -> Variant {
    let pascal_name = task.name.to_case(Case::Pascal);
    let snake_name = task.name.to_case(Case::Snake);
    let task_type = format!("{}::Task", snake_name);
    Variant::new(&pascal_name)
        .tuple(&task_type)
        .to_owned()
}

pub fn generate_task_enum(tasks: &Vec<PluginDescriptor>) -> Enum {
    let mut enum_definition = Enum::new("Task")
        .vis("pub").to_owned();
    for task in tasks {
        let variant = generate_variant(&task);
        enum_definition.push_variant(variant);
    }
    enum_definition
}

fn generate_parse_input_match(tasks: &Vec<PluginDescriptor>) -> Block {
    let mut match_block = Block::new("match task_name");
    for task in tasks {
        let name_snake = task.name.to_case(Case::Snake);
        let name_pascal = task.name.to_case(Case::Pascal);
        let block = Block::new(&format!("\"{}\" =>", &name_snake))
            .line(format!("let task = {}::parse_task(runtime, task)?.map(|task| Task::{}(task));", &name_snake, &name_pascal))
            .line("Ok(task)")
            .after(",")
            .to_owned();
        match_block.push_block(block);
    }
    match_block
        .line("_ => Err(Error::from(format!(\"Invalid task '{}'\", task_name))),")
        .to_owned()
}

pub fn generate_parse_input(tasks: &Vec<PluginDescriptor>) -> Function {
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
    use codegen::{Function, Scope, Struct, Enum};
    use regex::Regex;

    use crate::command::{generator::generate_parse_task, PluginDescriptor, Command, ElementDescriptor};

    use super::{generate_parse_item, generate_parse_items ,generate_task_struct, generate_task_enum, generate_parse_input};

    fn function_to_string(item: Function) -> String {
        Scope::new().push_fn(item).to_string()
    }

    fn struct_to_string(item: Struct) -> String {
        Scope::new().push_struct(item).to_string()
    }

    fn enum_to_string(item: Enum) -> String {
        Scope::new().push_enum(item).to_string()
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
        pub struct Task {
            items: Vec<Item>,
        }"#;
        compare_struct(item, EXPECTED);
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

    fn mock_task(name: &str) -> PluginDescriptor {
        PluginDescriptor {
            name: String::from(name),
            command: Command::Snippet(String::from("asdf")),
            element: ElementDescriptor{
                tag: String::from(""),
                attributes: vec!()
            },
        }
    }

    #[test]
    fn task_enum() {
        let tasks = vec!(mock_task("copy"), mock_task("strip"));
        let enum_definition = generate_task_enum(&tasks);
        const EXPECTED: &str = r#"
        pub enum Task {
            Copy(copy::Task),
            Strip(strip::Task),
        }"#;
        compare_enum(enum_definition, EXPECTED);
    }

    #[test]
    fn parse_input() {
        let tasks = vec!(mock_task("copy"), mock_task("strip"));
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
