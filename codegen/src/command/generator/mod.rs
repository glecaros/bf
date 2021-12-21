mod group;
mod item;

use codegen::{Block, Function, Impl, Struct, Type};

use super::ElementDescriptor;

macro_rules! t {
    ($ty:literal) => {
        Type::new($ty)
    };
}

pub fn generate_item_definition(element_descriptor: &ElementDescriptor) -> Struct {
    todo!();
}

pub fn generate_item_impl(element_descriptor: &ElementDescriptor) -> Impl {
    todo!();
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
        .arg("parent", t!("Option<&Group>"))
        .ret(t!("Result<Option<Item>, Error>"))
        .line("let condition = evaluate_condition_from_element(runtime, element)?;")
        .push_block(if_block)
        .push_block(else_block)
        .line("Ok(item)")
        .to_owned()
}

fn generate_parse_items_loop() -> Block {
    let item_arm = Block::new("\"item\" => ")
        .line("let item = parse_item(runtime, item, Some(&group))?;")
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

#[cfg(test)]
mod test {
    use codegen::{Function, Scope};
    use regex::Regex;

    use crate::command::generator::{generate_parse_item, generate_parse_items};

    fn function_to_string(item: Function) -> String {
        Scope::new().push_fn(item).to_string()
    }

    fn normalize(s: &str) -> String {
        let regex = Regex::new("[\\n\\s]+").unwrap();
        regex.replace_all(s.trim(), " ").to_string()
    }

    fn compare_function(item: Function, expected: &str) {
        assert_eq!(normalize(&function_to_string(item)), normalize(expected))
    }

    #[test]
    fn parse_item() {
        let item = generate_parse_item();
        const EXPECTED: &str = r#"
        fn parse_item(runtime: &Runtime, element: &Element, parent: Option<&Group>) -> Result<Option<Item>, Error> {
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
                            let item = parse_item(runtime, item, Some(&group))?;
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
}
