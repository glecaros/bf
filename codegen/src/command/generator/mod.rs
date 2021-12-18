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

pub fn generate_parse_item(element_descriptor: &ElementDescriptor) -> Function {
    let mut inner_block = Block::new("if condition");
    let mut constructor = Block::new("Item");
    for attribute in &element_descriptor.attributes {
        let var_name = &attribute.name;
        inner_block.line(format!("let {var} = interpolate_attribute(\"{var}\", element, runtime)?;", var = var_name));
        if attribute.required {
            inner_block.line(format!("let {var} = {var}.ok_or(Error::from(\"Missing required attribute '{var}'\"));", var = var_name));
        }
        constructor.line(format!("{var}: {var},", var=var_name));
    }
    inner_block.push_block(constructor);

    Function::new("parse_item")
        .arg("runtime", t!("&Runtime"))
        .arg("parent", t!("&Element"))
        .arg("group", t!("Option<&Group>"))
        .ret(t!("Result<Option<Item>, Error>"))
        .line("let condition = evaluate_condition_from_element(runtime, element)?;")
        .push_block(inner_block)
        .push_block(Block::new("else").line("Ok(None)").to_owned()).to_owned()

}

pub fn generate_parse_items(element_descriptor: &ElementDescriptor) -> Function {
    let mut inner_block = Block::new("if condition");

    Function::new("parse_items")
        .arg("runtime", t!("&Runtime"))
        .arg("parent", t!("&Element"))
        .arg("group", t!("Option<&Group>"))
        .ret(t!("Result<Option<Vec<Item>>, Error>"))
        .line("let condition = evaluate_condition_from_element(runtime, parent)?;")
        .push_block(inner_block)
        .push_block(Block::new("else").line("Ok(None)").to_owned())
        .to_owned()
}


// pub fn generate_parse_items(element_descriptor: &ElementDescriptor) -> Function {
//     let mut function = Function::new("parse_items");
//     function
//         .arg("runtime", Type::new("&Runtime"))
//         .arg("parent", Type::new("&Element"))
//         .arg("group", Type::new("Option<&Group>"))
//         .ret(Type::new("Result<Option<Vec<Item>>, Error>"))
//         .line("let condition = parent.attr(ATTR_CONDITION)")
//         .line("let condition = evaluate_condition(condition, runtime)?;")
//         .push_block(block!("if condition" |
//             "let source = match parent.attr(A);"
//         |));
//         // .push_block(Block::new("if condition")
//         //     .push_block(Block::new("let source = match parent.attr(ATTR_SOURCE)")
//         //         .line("Some(source) => Some(interpolate(source, &runtime.variables)?)",)
//         //         // .line("None => None,").to_owned()).after(after).to_owned()
//         // )
//         // .push_block(Block::new("else")
//         //     .line("Ok(None)").to_owned()
//         // );
//     function
// }

