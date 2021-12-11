    use codegen::Scope;

    use crate::command::{ElementDescriptor, GroupSetting, ParameterDescriptor, ParameterType, generator::{generate_group_definition, generate_parse_item}};

    
    #[test]
    fn test1() {
        let element = ElementDescriptor{
            tag: String::from("strip"),
            value: Some(new_parameter("source", ParameterType::Path, true, GroupSetting::InheritPrefix)),
            attributes: vec![
                new_parameter("destination", ParameterType::Path, false, GroupSetting::InheritPrefix)
            ]
        };
        let function = generate_parse_item(&element);
        let mut scope = Scope::new();
        scope.push_fn(function);
        println!("{}", scope.to_string());
    }

    fn group_impl_inherit() {
        use GroupSetting::*;
        let descriptor = test_descriptor(Inherit, Inherit, Inherit);
        let item = super::generate_group_impl(&descriptor);
        const EXPECTED: &str = r#"impl Group {
    pub fn create(element: &Element, parent: &Group, runtime: &Runtime) -> Group {
        let src = interpolate_text(element, runtime)?;
        let src = src.or(parent.src);
        let dst = interpolate_attribute("dst", element, runtime)?;
        let dst = dst.or(parent.dst);
        let tst = interpolate_attribute("tst", element, runtime)?;
        let tst = tst.or(parent.tst);
        Group{
            src: src,
            dst: dst,
            tst: tst,
        }
    }
}"#;
        assert_eq!(impl_to_string(item), EXPECTED);
    }
}