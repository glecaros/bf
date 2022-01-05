use codegen::{Block, Field, Function, Impl, Struct, Type};

use crate::command::{ElementDescriptor, GroupSetting, ParameterDescriptor, ParameterType};

fn generate_field_definition(parameter: &ParameterDescriptor) -> Field {
    let field_type = match parameter.parameter_type {
        ParameterType::Path => "PathBuf",
    };
    if parameter.required {
        Field::new(&parameter.name, field_type)
    } else {
        let field_type = Type::new("Option").generic(field_type).to_owned();
        Field::new(&parameter.name, field_type)
    }
}

pub fn generate_item_definition(element_descriptor: &ElementDescriptor) -> Struct {
    let mut struct_definition = Struct::new("Item");
    for attribute in &element_descriptor.attributes {
        let field = generate_field_definition(attribute);
        struct_definition.push_field(field);
    }
    struct_definition
}

fn add_parameter_code(function: &mut Function, parameter: &ParameterDescriptor) {
    let conversion_suffix = match &parameter.parameter_type {
        ParameterType::Path => ".map(PathBuf::from);",
    };
    let init_line = format!(
        r#"let {var_name} = interpolate_attribute("{var_name}", element, runtime)?{suffix}"#,
        var_name = parameter.name,
        suffix = conversion_suffix
    );
    function.line(init_line);
    match parameter.allow_group {
        GroupSetting::None => (),
        GroupSetting::Inherit => {
            function.line(format!(
                "let {var_name} = {var_name}.or(parent.{var_name});",
                var_name = parameter.name
            ));
        }
        GroupSetting::Prefix => {
            let prefix = format!(
                "let {var_name} = if let Some({var_name}) = {var_name}",
                var_name = parameter.name
            );
            let if_block = Block::new(&prefix)
                .line(format!(
                    "Some({var_name}).apply_prefix(&parent.{var_name})",
                    var_name = parameter.name
                ))
                .to_owned();
            let else_block = Block::new("else")
                .line(&parameter.name)
                .after(";")
                .to_owned();
            function.push_block(if_block);
            function.push_block(else_block);
        }
        GroupSetting::InheritPrefix => {
            let line = format!(
                "let {var_name} = {var_name}.apply_prefix(&parent.{var_name});",
                var_name = parameter.name
            );
            function.line(line);
        }
    }
    if parameter.required {
        function.line(format!(r#"let {var_name} = {var_name}.ok_or(Error::from("Missing required value: '{var_name}'"))?;"#, var_name = parameter.name));
    }
}

pub fn generate_item_impl(element_descriptor: &ElementDescriptor) -> Impl {
    let mut create_function = Function::new("create")
        .vis("pub")
        .ret(Type::new("Result<Item, Error>"))
        .arg("element", Type::new("&Element"))
        .arg("parent", Type::new("&Group"))
        .arg("runtime", "&Runtime")
        .to_owned();
    let mut constructor = Block::new("Ok(Item").after(")").to_owned();
    for attribute in &element_descriptor.attributes {
        add_parameter_code(&mut create_function, attribute);
        constructor.line(format!(
            "{var_name}: {var_name},",
            var_name = attribute.name
        ));
    }
    create_function.push_block(constructor);
    Impl::new("Item").push_fn(create_function).to_owned()
}

#[cfg(test)]
mod test {
    use codegen::{Impl, Scope, Struct};
    use regex::Regex;

    use crate::command::{ElementDescriptor, GroupSetting, ParameterDescriptor, ParameterType};

    fn new_parameter(
        name: &str,
        parameter_type: ParameterType,
        required: bool,
        allow_group: GroupSetting,
    ) -> ParameterDescriptor {
        ParameterDescriptor {
            name: String::from(name),
            parameter_type: parameter_type,
            required: required,
            allow_group: allow_group,
        }
    }

    fn test_descriptor(
        setting1: (GroupSetting, bool),
        setting2: (GroupSetting, bool),
        setting3: (GroupSetting, bool),
    ) -> ElementDescriptor {
        ElementDescriptor {
            attributes: vec![
                new_parameter("src", ParameterType::Path, setting1.1, setting1.0),
                new_parameter("dst", ParameterType::Path, setting2.1, setting2.0),
                new_parameter("tst", ParameterType::Path, setting3.1, setting3.0),
            ],
        }
    }

    fn struct_to_string(item: Struct) -> String {
        Scope::new().push_struct(item).to_string()
    }

    fn impl_to_string(item: Impl) -> String {
        Scope::new().push_impl(item).to_string()
    }

    fn normalize(s: &str) -> String {
        let regex = Regex::new("[\\n\\s]+").unwrap();
        regex.replace_all(s.trim(), " ").to_string()
    }

    fn compare_struct(item: Struct, expected: &str) {
        assert_eq!(normalize(&struct_to_string(item)), normalize(expected))
    }

    fn compare_impl(item: Impl, expected: &str) {
        assert_eq!(normalize(&impl_to_string(item)), normalize(expected))
    }

    #[test]
    fn item_struct_all_required() {
        use GroupSetting::*;
        let descriptor = test_descriptor((None, true), (None, true), (None, true));
        let item = super::generate_item_definition(&descriptor);
        const EXPECTED: &str = r#"
        struct Item {
            src: PathBuf,
            dst: PathBuf,
            tst: PathBuf,
        }"#;
        compare_struct(item, EXPECTED)
    }

    #[test]
    fn item_struct_none_required() {
        use GroupSetting::*;
        let descriptor = test_descriptor((None, false), (None, false), (None, false));
        let item = super::generate_item_definition(&descriptor);
        const EXPECTED: &str = r#"
        struct Item {
            src: Option<PathBuf>,
            dst: Option<PathBuf>,
            tst: Option<PathBuf>,
        }"#;
        compare_struct(item, EXPECTED)
    }

    #[test]
    fn item_impl_required_no_group() {
        use GroupSetting::*;
        let descriptor = test_descriptor((None, true), (None, true), (None, true));
        let item = super::generate_item_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Item {
            pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Result<Item, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = src.ok_or(Error::from("Missing required value: 'src'"))?;
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = dst.ok_or(Error::from("Missing required value: 'dst'"))?;
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = tst.ok_or(Error::from("Missing required value: 'tst'"))?;
                Ok(Item {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_required_inherit() {
        use GroupSetting::*;
        let descriptor = test_descriptor((Inherit, true), (Inherit, true), (Inherit, true));
        let item = super::generate_item_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Item {
            pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Result<Item, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = src.or(parent.src);
                let src = src.ok_or(Error::from("Missing required value: 'src'"))?;
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = dst.or(parent.dst);
                let dst = dst.ok_or(Error::from("Missing required value: 'dst'"))?;
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = tst.or(parent.tst);
                let tst = tst.ok_or(Error::from("Missing required value: 'tst'"))?;
                Ok(Item {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_required_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor((Prefix, true), (Prefix, true), (Prefix, true));
        let item = super::generate_item_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Item {
            pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Result<Item, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = if let Some(src) = src {
                    Some(src).apply_prefix(&parent.src)
                } else {
                    src
                };
                let src = src.ok_or(Error::from("Missing required value: 'src'"))?;
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = if let Some(dst) = dst {
                    Some(dst).apply_prefix(&parent.dst)
                } else {
                    dst
                };
                let dst = dst.ok_or(Error::from("Missing required value: 'dst'"))?;
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = if let Some(tst) = tst {
                    Some(tst).apply_prefix(&parent.tst)
                } else {
                    tst
                };
                let tst = tst.ok_or(Error::from("Missing required value: 'tst'"))?;
                Ok(Item {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_required_inherit_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor(
            (InheritPrefix, true),
            (InheritPrefix, true),
            (InheritPrefix, true),
        );
        let item = super::generate_item_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Item {
            pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Result<Item, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = src.apply_prefix(&parent.src);
                let src = src.ok_or(Error::from("Missing required value: 'src'"))?;
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = dst.apply_prefix(&parent.dst);
                let dst = dst.ok_or(Error::from("Missing required value: 'dst'"))?;
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = tst.apply_prefix(&parent.tst);
                let tst = tst.ok_or(Error::from("Missing required value: 'tst'"))?;
                Ok(Item {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_not_required_no_group() {
        use GroupSetting::*;
        let descriptor = test_descriptor((None, false), (None, false), (None, false));
        let item = super::generate_item_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Item {
            pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Result<Item, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                Ok(Item {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_not_required_inherit() {
        use GroupSetting::*;
        let descriptor = test_descriptor((Inherit, false), (Inherit, false), (Inherit, false));
        let item = super::generate_item_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Item {
            pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Result<Item, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = src.or(parent.src);
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = dst.or(parent.dst);
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = tst.or(parent.tst);
                Ok(Item {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_not_required_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor((Prefix, false), (Prefix, false), (Prefix, false));
        let item = super::generate_item_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Item {
            pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Result<Item, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = if let Some(src) = src {
                    Some(src).apply_prefix(&parent.src)
                } else {
                    src
                };
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = if let Some(dst) = dst {
                    Some(dst).apply_prefix(&parent.dst)
                } else {
                    dst
                };
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = if let Some(tst) = tst {
                    Some(tst).apply_prefix(&parent.tst)
                } else {
                    tst
                };
                Ok(Item {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_not_required_inherit_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor(
            (InheritPrefix, false),
            (InheritPrefix, false),
            (InheritPrefix, false),
        );
        let item = super::generate_item_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Item {
            pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Result<Item, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = src.apply_prefix(&parent.src);
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = dst.apply_prefix(&parent.dst);
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = tst.apply_prefix(&parent.tst);
                Ok(Item {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }
}
