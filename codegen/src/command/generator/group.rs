use codegen::{Block, Field, Function, Impl, Struct, Type};

use crate::command::{ElementDescriptor, GroupSetting, ParameterDescriptor, ParameterType};

fn generate_field_definition(parameter: &ParameterDescriptor) -> Option<Field> {
    match parameter.allow_group {
        GroupSetting::None => None,
        _ => {
            let inner_type = match parameter.parameter_type {
                ParameterType::Path => "PathBuf",
            };
            let field_type = Type::new("Option").generic(inner_type).to_owned();
            Some(Field::new(&parameter.name, field_type))
        }
    }
}

pub fn generate_group_definition(element_descriptor: &ElementDescriptor) -> Struct {
    let mut struct_definition = Struct::new("Group")
        .derive("Debug")
        .to_owned();
    for attribute in &element_descriptor.attributes {
        let field = generate_field_definition(attribute);
        if let Some(field) = field {
            struct_definition.push_field(field);
        }
    }
    struct_definition
}

fn add_parameter_code(function: &mut Function, parameter: &ParameterDescriptor) {
    let conversion_suffix = match &parameter.parameter_type {
        ParameterType::Path => ".map(PathBuf::from)",
    };
    let init_line = format!(
        r#"let {var_name} = interpolate_attribute("{var_name}", element, runtime)?{suffix};"#,
        var_name = parameter.name,
        suffix = conversion_suffix
    );
    match parameter.allow_group {
        GroupSetting::None => (),
        GroupSetting::Inherit => {
            function.line(init_line);
            function.line(format!(
                "let {var_name} = {var_name}.or(parent.map(|group| group.{var_name}));",
                var_name = parameter.name
            ));
        }
        GroupSetting::Prefix | GroupSetting::InheritPrefix => {
            function.line(init_line);
            let mut if_block = Block::new(&format!(
                "let {} = if let Some(group) = parent",
                parameter.name
            ));
            if_block.line(format!(
                "{var_name}.apply_prefix(&group.{var_name})",
                var_name = parameter.name
            ));
            let mut else_block = Block::new("else");
            else_block.line(format!("{var_name}", var_name = parameter.name));
            else_block.after(";");
            function.push_block(if_block);
            function.push_block(else_block);
        }
    }
}

pub fn generate_group_impl(element_descriptor: &ElementDescriptor) -> Impl {
    let mut create_function = Function::new("create");
    create_function
        .vis("pub")
        .ret(Type::new("Result<Group, Error>"));
    if element_descriptor.uses_groups() {
        create_function
            .arg("element", Type::new("&Element"))
            .arg("parent", Type::new("Option<&Group>"))
            .arg("runtime", Type::new("&Runtime"));
        let mut constructor = Block::new("Ok(Group");
        constructor.after(")");
        for attribute in &element_descriptor.attributes {
            add_parameter_code(&mut create_function, attribute);
            if !matches!(attribute.allow_group, GroupSetting::None) {
                constructor.line(format!(
                    "{var_name}: {var_name},",
                    var_name = attribute.name
                ));
            }
        }
        create_function.push_block(constructor);
    } else {
        create_function.line("Ok(Group{})");
    }
    Impl::new("Group").push_fn(create_function).to_owned()
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
        setting1: GroupSetting,
        setting2: GroupSetting,
        setting3: GroupSetting,
    ) -> ElementDescriptor {
        ElementDescriptor {
            attributes: vec![
                new_parameter("src", ParameterType::Path, true, setting1),
                new_parameter("dst", ParameterType::Path, false, setting2),
                new_parameter("tst", ParameterType::Path, true, setting3),
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
    fn group_none() {
        use GroupSetting::*;
        let descriptor = test_descriptor(None, None, None);
        let item = super::generate_group_definition(&descriptor);
        const EXPECTED: &str = r#"
        #[derive(Debug)]
        struct Group;"#;
        compare_struct(item, EXPECTED);
    }

    #[test]
    fn group_all() {
        use GroupSetting::*;
        let descriptor = test_descriptor(Inherit, Inherit, Inherit);
        let item = super::generate_group_definition(&descriptor);
        const EXPECTED: &str = r#"
        #[derive(Debug)]
        struct Group {
            src: Option<PathBuf>,
            dst: Option<PathBuf>,
            tst: Option<PathBuf>,
        }"#;
        compare_struct(item, EXPECTED);
    }

    #[test]
    fn group_some() {
        use GroupSetting::*;
        let descriptor = test_descriptor(Inherit, None, Inherit);
        let item = super::generate_group_definition(&descriptor);
        const EXPECTED: &str = r#"
        #[derive(Debug)]
        struct Group {
            src: Option<PathBuf>,
            tst: Option<PathBuf>,
        }"#;
        compare_struct(item, EXPECTED);
    }

    #[test]
    fn group_impl_none() {
        use GroupSetting::*;
        let descriptor = test_descriptor(None, None, None);
        let item = super::generate_group_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Group {
            pub fn create() -> Result<Group, Error> {
                Ok(Group{})
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn group_impl_inherit() {
        use GroupSetting::*;
        let descriptor = test_descriptor(Inherit, Inherit, Inherit);
        let item = super::generate_group_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Group {
            pub fn create(element: &Element, parent: Option<&Group>, runtime: &Runtime) -> Result<Group, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = src.or(parent.map(|group| group.src));
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = dst.or(parent.map(|group| group.dst));
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = tst.or(parent.map(|group| group.tst));
                Ok(Group {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn group_impl_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor(Prefix, Prefix, Prefix);
        let item = super::generate_group_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Group {
            pub fn create(element: &Element, parent: Option<&Group>, runtime: &Runtime) -> Result<Group, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = if let Some(group) = parent {
                    src.apply_prefix(&group.src)
                } else {
                    src
                };
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = if let Some(group) = parent {
                    dst.apply_prefix(&group.dst)
                } else {
                    dst
                };
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = if let Some(group) = parent {
                    tst.apply_prefix(&group.tst)
                } else {
                    tst
                };
                Ok(Group {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn group_impl_inherit_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor(InheritPrefix, InheritPrefix, InheritPrefix);
        let item = super::generate_group_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Group {
            pub fn create(element: &Element, parent: Option<&Group>, runtime: &Runtime) -> Result<Group, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = if let Some(group) = parent {
                    src.apply_prefix(&group.src)
                } else {
                    src
                };
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = if let Some(group) = parent {
                    dst.apply_prefix(&group.dst)
                } else {
                    dst
                };
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = if let Some(group) = parent {
                    tst.apply_prefix(&group.tst)
                } else {
                    tst
                };
                Ok(Group {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn group_impl_mixed() {
        use GroupSetting::*;
        let descriptor = test_descriptor(Inherit, Prefix, InheritPrefix);
        let item = super::generate_group_impl(&descriptor);
        const EXPECTED: &str = r#"
        impl Group {
            pub fn create(element: &Element, parent: Option<&Group>, runtime: &Runtime) -> Result<Group, Error> {
                let src = interpolate_attribute("src", element, runtime)?.map(PathBuf::from);
                let src = src.or(parent.map(|group| group.src));
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = if let Some(group) = parent {
                    dst.apply_prefix(&group.dst)
                } else {
                    dst
                };
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = if let Some(group) = parent {
                    tst.apply_prefix(&group.tst)
                } else {
                    tst
                };
                Ok(Group {
                    src: src,
                    dst: dst,
                    tst: tst,
                })
            }
        }"#;
        compare_impl(item, EXPECTED);
    }
}
