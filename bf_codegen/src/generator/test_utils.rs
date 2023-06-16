use codegen::{Enum, Function, Impl, Scope, Struct};
use regex::Regex;

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

pub fn compare_function(item: Function, expected: &str) {
    assert_eq!(normalize(&function_to_string(item)), normalize(expected))
}

pub fn compare_struct(item: Struct, expected: &str) {
    assert_eq!(normalize(&struct_to_string(item)), normalize(expected))
}

pub fn compare_enum(item: Enum, expected: &str) {
    assert_eq!(normalize(&enum_to_string(item)), normalize(expected))
}

pub fn compare_impl(item: Impl, expected: &str) {
    assert_eq!(normalize(&impl_to_string(item)), normalize(expected))
}
