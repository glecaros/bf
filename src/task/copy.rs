use std::path::PathBuf;

use log::info;
use minidom::Element;

use crate::{commands, error::Error, internal_error, interpolation::interpolate, runtime::Runtime, task::evaluate_condition, util::{evaluate_condition_from_element, interpolate_attribute, interpolate_text, ApplyPrefix}};

use super::{Task};


const ATTR_SOURCE: &str = "source";
const ATTR_DESTINATION: &str = "destination";

struct Group {
    source_prefix: Option<PathBuf>,
    destination_prefix: Option<PathBuf>,
}

impl Group {
    fn create(element: &Element, parent: Option<&Group>, runtime: &Runtime) -> Result<Group, Error> {
        let source = interpolate_attribute(ATTR_SOURCE, element, runtime)?.map(PathBuf::from);
        let destination = interpolate_attribute(ATTR_DESTINATION, element, runtime)?.map(PathBuf::from);
        let source = if let Some(group) = parent {
            source.apply_prefix(&group.source_prefix)
        } else {
            source
        };
        let destination = if let Some(group) = parent {
            destination.apply_prefix(&group.destination_prefix)
        } else {
            destination
        };
        Ok(Group {
            source_prefix: source,
            destination_prefix: destination,
        })
    }
}


#[derive(Debug)]
struct Item {
    source: PathBuf,
    destination: PathBuf,
}

impl Item {
    fn create(source: &str, destination: &str, parent: Option<&Group>) -> Item {
        let source = Some(PathBuf::from(source));
        let destination = Some(PathBuf::from(destination));
        let source = if let Some(group) = parent {
            source.apply_prefix(&group.source_prefix)
        } else {
            source
        };
        let destination = if let Some(group) = parent {
            destination.apply_prefix(&group.destination_prefix)
        } else {
            destination
        };Item {
            source: source.unwrap(),
            destination: destination.unwrap(),
        }
    }

    fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
        let source = runtime.resolve_source(&self.source);
        let destination = runtime.resolve_destination(&self.destination);
        info!(
            "Copy {} -> {}",
            source.to_string_lossy(),
            destination.to_string_lossy()
        );
        if !runtime.dry_run {
            commands::copy(&source, &destination)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CopyTask {
    items: Vec<Item>,
}

impl Task for CopyTask {
    fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
        for item in &self.items {
            item.execute(&runtime)?;
        }
        Ok(())
    }

    fn item_count(&self) -> usize {
        self.items.len()
    }

    fn get_name(&self) -> &'static str {
        "copy"
    }
}

fn parse_item(
    runtime: &Runtime,
    element: &Element,
    parent: Option<&Group>,
) -> Result<Option<Item>, Error> {
    let condition = evaluate_condition_from_element(runtime, element)?;
    if condition {
        let source = interpolate_text(element, runtime)?;
        let source = source.ok_or(Error::from("Missing required value"))?;
        let destination = interpolate_attribute(ATTR_DESTINATION, element, runtime)?;
        let destination = destination.ok_or(Error::from("Missing required attribute"))?; 
        Ok(Some(Item::create(&source, &destination, parent)))
    } else {
        Ok(None)
    }
}

fn parse_items(
    runtime: &Runtime,
    parent: &Element,
    group: Option<&Group>,
) -> Result<Option<Vec<Item>>, Error> {
    let condition = evaluate_condition_from_element(runtime, parent)?;
    if condition {
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
                    return Err(internal_error!("Invalid element {}", item.name()));
                }
            }
        }
        Ok(Some(items))
    } else {
        Ok(None)
    }
}

impl CopyTask {
    pub fn parse(runtime: &Runtime, base_element: &Element) -> Result<Option<CopyTask>, Error> {
        let items =
            parse_items(runtime, base_element, None)?.map(|items| CopyTask { items: items });
        Ok(items)
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::PathBuf, str::FromStr};

    use minidom::Element;

    use crate::runtime::Runtime;

    use super::parse_items;

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
    fn parse_items_single_item() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 1);
        let item = items.first().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
    }

    #[test]
    fn parse_items_single_item_with_rename() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item destination="newfile.ext">file1.ext</item>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 1);
        let item = items.first().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/newfile.ext"));
    }

    #[test]
    fn parse_items_single_item_with_fulfilled_condition_outer() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst" condition="var == 'value'">
            <item>file1.ext</item>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("var"), String::from("value"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 1);
        let item = items.first().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
    }

    #[test]
    fn parse_items_single_item_with_unfulfilled_condition_outer() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst" condition="var == 'value'">
            <item>file1.ext</item>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(None)));
    }

    #[test]
    fn parse_items_with_fulfilled_condition_inner() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item condition="var == 'value'">file1.ext</item>
            <item>file2.ext</item>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("var"), String::from("value"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 2);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file2.ext"));
    }

    #[test]
    fn parse_items_with_unfulfilled_condition_inner() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item condition="var == 'value'">file1.ext</item>
            <item>file2.ext</item>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 1);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file2.ext"));
    }

    #[test]
    fn parse_items_single_group() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
            <group source="input_dir" destination="output_dir">
                <item>file2.ext</item>
            </group>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 2);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/output_dir/file2.ext"));
    }

    #[test]
    fn parse_items_single_group_with_fulfilled_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
            <group source="input_dir" destination="output_dir" condition="var == 'val'">
                <item>file2.ext</item>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("var"), String::from("val"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 2);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/output_dir/file2.ext"));
    }

    #[test]
    fn parse_items_single_group_with_unfulfilled_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
            <group source="input_dir" destination="output_dir" condition="var == 'val'">
                <item>file2.ext</item>
            </group>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 1);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
    }

    #[test]
    fn parse_items_nested_groups() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
            <group source="input_dir" destination="output_dir">
                <item>file2.ext</item>
                <group source="i" destination="d">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 3);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/output_dir/file2.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/i/file3.ext"));
        assert_eq!(
            item.destination,
            PathBuf::from("dst/output_dir/d/file3.ext")
        );
    }

    #[test]
    fn parse_items_nested_groups_with_fulfilled_inner_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
            <group source="input_dir" destination="output_dir">
                <item>file2.ext</item>
                <group source="i" destination="d" condition="var == 'val'">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("var"), String::from("val"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 3);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/output_dir/file2.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/i/file3.ext"));
        assert_eq!(
            item.destination,
            PathBuf::from("dst/output_dir/d/file3.ext")
        );
    }

    #[test]
    fn parse_items_nested_groups_with_unfulfilled_inner_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
            <group source="input_dir" destination="output_dir">
                <item>file2.ext</item>
                <group source="i" destination="d" condition="var == 'val'">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 2);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/output_dir/file2.ext"));
    }

    #[test]
    fn parse_items_nested_groups_with_fulfilled_outer_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
            <group source="input_dir" destination="output_dir" condition="var == 'val'">
                <item>file2.ext</item>
                <group source="i" destination="d">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("var"), String::from("val"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 3);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/output_dir/file2.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/i/file3.ext"));
        assert_eq!(
            item.destination,
            PathBuf::from("dst/output_dir/d/file3.ext")
        );
    }

    #[test]
    fn parse_items_nested_groups_with_unfulfilled_outer_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>file1.ext</item>
            <group source="input_dir" destination="output_dir" condition="var == 'val'">
                <item>file2.ext</item>
                <group source="i" destination="d">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 1);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/file1.ext"));
    }

    #[test]
    fn parse_items_interpolation() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>{name}.ext</item>
            <item destination="{new_name}.ext">file1.ext</item>
            <group source="input_dir" destination="{dir}">
                <item>file2.ext</item>
                <group source="{other_input}" destination="d">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("name"), String::from("my_file"));
        runtime
            .variables
            .insert(String::from("new_name"), String::from("some_name"));
        runtime
            .variables
            .insert(String::from("dir"), String::from("my_dir"));
        runtime
            .variables
            .insert(String::from("other_input"), String::from("secret"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 4);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/my_file.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_file.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/some_name.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_dir/file2.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/secret/file3.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_dir/d/file3.ext"));
    }

    #[test]
    fn parse_items_interpolation_with_fulfilled_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>{name}.ext</item>
            <item destination="{new_name}.ext">file1.ext</item>
            <group source="input_dir" destination="{dir}">
                <item>file2.ext</item>
                <group source="{other_input}" destination="d" condition="var == 'value'">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("name"), String::from("my_file"));
        runtime
            .variables
            .insert(String::from("new_name"), String::from("some_name"));
        runtime
            .variables
            .insert(String::from("dir"), String::from("my_dir"));
        runtime
            .variables
            .insert(String::from("other_input"), String::from("secret"));
        runtime
            .variables
            .insert(String::from("var"), String::from("value"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 4);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/my_file.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_file.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/some_name.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_dir/file2.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/secret/file3.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_dir/d/file3.ext"));
    }

    #[test]
    fn parse_items_interpolation_with_unfulfilled_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>{name}.ext</item>
            <item destination="{new_name}.ext">file1.ext</item>
            <group source="input_dir" destination="{dir}">
                <item>file2.ext</item>
                <group source="{other_input}" destination="d" condition="val == 'value'">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("name"), String::from("my_file"));
        runtime
            .variables
            .insert(String::from("new_name"), String::from("some_name"));
        runtime
            .variables
            .insert(String::from("dir"), String::from("my_dir"));
        runtime
            .variables
            .insert(String::from("other_input"), String::from("secret"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 3);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/my_file.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_file.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/some_name.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_dir/file2.ext"));
    }

    #[test]
    fn parse_items_interpolation_with_missing_variable() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>{name}.ext</item>
            <item destination="{new_name}.ext">file1.ext</item>
            <group source="input_dir" destination="{dir}">
                <item>file2.ext</item>
                <group source="{other_input}" destination="d">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("name"), String::from("my_file"));
        runtime
            .variables
            .insert(String::from("new_name"), String::from("some_name"));
        runtime
            .variables
            .insert(String::from("dir"), String::from("my_dir"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Err(_)));
    }

    #[test]
    fn parse_items_interpolation_with_missing_variable_fulfilled_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>{name}.ext</item>
            <item destination="{new_name}.ext">file1.ext</item>
            <group source="input_dir" destination="{dir}">
                <item>file2.ext</item>
                <group source="{other_input}" destination="d" condition="var == 'value'">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("name"), String::from("my_file"));
        runtime
            .variables
            .insert(String::from("new_name"), String::from("some_name"));
        runtime
            .variables
            .insert(String::from("dir"), String::from("my_dir"));
        runtime
            .variables
            .insert(String::from("var"), String::from("value"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Err(_)));
    }

    #[test]
    fn parse_items_interpolation_with_missing_variable_unfulfilled_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>{name}.ext</item>
            <item destination="{new_name}.ext">file1.ext</item>
            <group source="input_dir" destination="{dir}">
                <item>file2.ext</item>
                <group source="{other_input}" destination="d" condition="var == 'value'">
                    <item>file3.ext</item>
                </group>
            </group>
        </copy>
        "#;
        let mut runtime = new_runtime();
        runtime
            .variables
            .insert(String::from("name"), String::from("my_file"));
        runtime
            .variables
            .insert(String::from("new_name"), String::from("some_name"));
        runtime
            .variables
            .insert(String::from("dir"), String::from("my_dir"));
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert_eq!(items.len(), 3);
        let mut items = items.iter();
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/my_file.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_file.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/file1.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/some_name.ext"));
        let item = items.next().unwrap();
        assert_eq!(item.source, PathBuf::from("src/input_dir/file2.ext"));
        assert_eq!(item.destination, PathBuf::from("dst/my_dir/file2.ext"));
    }

    #[test]
    fn parse_items_interpolation_in_item_with_missing_variable() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item>{name}.ext</item>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Err(_)));
    }

    #[test]
    fn parse_items_interpolation_in_item_with_missing_variable_and_unfulfilled_condition() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item condition="var == 'value'">{name}.ext</item>
        </copy>
        "#;
        let runtime = new_runtime();
        let element = Element::from_str(INPUT).unwrap();
        let items = parse_items(&runtime, &element, None);
        assert!(matches!(items, Ok(Some(_))));
        let items = items.unwrap().unwrap();
        assert!(items.is_empty());
    }
}
