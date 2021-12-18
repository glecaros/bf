use codegen::{Struct, Impl};

use crate::command::ElementDescriptor;


fn generate_item_definition(element_descriptor: &ElementDescriptor) -> Struct {
    todo!();
}

fn generate_item_impl(element_descriptor: &ElementDescriptor) -> Impl {
    todo!();
}

#[cfg(test)]
mod test {
    use codegen::{Struct, Scope, Impl};
    use regex::Regex;

    use crate::command::{ParameterType, GroupSetting, ParameterDescriptor, ElementDescriptor};

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
            tag: String::from("copy"),
            value: Some(new_parameter("src", ParameterType::Path, setting1.1, setting1.0)),
            attributes: vec![
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
        };"#;
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
        };"#;
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
                let src = interpolate_text(element, runtime)?.map(PathBuf::from);
                let src = src.ok_or(Error::From("Missing required value: 'src'"))?;
                let dst = interpolate_attribute("dst", element, runtime)?.map(PathBuf::from);
                let dst = dst.ok_or(Error::From("Missing required value: 'dst'"))?;
                let tst = interpolate_attribute("tst", element, runtime)?.map(PathBuf::from);
                let tst = dst.ok_or(Error::From("Missing required value: 'tst'"))?;
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
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_required_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor((Prefix, true), (Prefix, true), (Prefix, true));
        let item = super::generate_item_impl(&descriptor);
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_required_inherit_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor((InheritPrefix, true), (InheritPrefix, true), (InheritPrefix, true));
        let item = super::generate_item_impl(&descriptor);
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_not_required_no_group() {
        use GroupSetting::*;
        let descriptor = test_descriptor((None, false), (None, false), (None, false));
        let item = super::generate_item_impl(&descriptor);
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_not_required_inherit() {
        use GroupSetting::*;
        let descriptor = test_descriptor((Inherit, false), (Inherit, false), (Inherit, false));
        let item = super::generate_item_impl(&descriptor);
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_not_required_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor((Prefix, false), (Prefix, false), (Prefix, false));
        let item = super::generate_item_impl(&descriptor);
        compare_impl(item, EXPECTED);
    }

    #[test]
    fn item_impl_not_required_inherit_prefix() {
        use GroupSetting::*;
        let descriptor = test_descriptor((InheritPrefix, true), (InheritPrefix, true), (None, true));
        let item = super::generate_item_impl(&descriptor);
        compare_impl(item, EXPECTED);
    }

}