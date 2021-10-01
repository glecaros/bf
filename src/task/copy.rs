use std::path::PathBuf;

use log::{info};
use minidom::Element;

use crate::{commands, error::Error, internal_error, runtime::Runtime, task::evaluate_condition};

use super::combine_paths;

struct Group {
    source_prefix: Option<PathBuf>,
    destination_prefix: Option<PathBuf>,
}

impl Group {
    fn create(source: Option<&str>, destination: Option<&str>, parent: Option<&Group>) -> Group {
        let source_prefix = parent.and_then(|parent| parent.source_prefix.as_ref());
        let destination_prefix = parent.and_then(|parent| parent.destination_prefix.as_ref());
        let source_suffix = source.map(PathBuf::from);
        let destination_suffix = destination.map(PathBuf::from);
        Group {
            source_prefix: combine_paths(source_prefix, source_suffix.as_ref()),
            destination_prefix: combine_paths(destination_prefix, destination_suffix.as_ref()),
        }
    }
}

#[derive(Debug)]
struct Item {
    source: PathBuf,
    destination: PathBuf,
}

impl Item {
    fn create(source: &str, destination: &str, parent: Option<&Group>) -> Item {
        let source_prefix = parent.and_then(|parent| parent.source_prefix.as_ref());
        let destination_prefix = parent.and_then(|parent| parent.destination_prefix.as_ref());
        let source_suffix = Some(PathBuf::from(source));
        let destination_suffix = Some(PathBuf::from(destination));
        Item {
            source: combine_paths(source_prefix, source_suffix.as_ref()).unwrap(),
            destination: combine_paths(destination_prefix, destination_suffix.as_ref()).unwrap(),
        }
    }

    fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
        let source = if let Some(prefix) = &runtime.source_base {
            prefix.join(&self.source)
        } else {
            self.source.clone()
        };
        let destination = if let Some(prefix) = &runtime.destination_base {
            prefix.join(&self.destination)
        } else {
            self.destination.clone()
        };
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

impl CopyTask {
    pub fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
        for item in &self.items {
            item.execute(&runtime)?;
        }
        Ok(())
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

const ATTR_SOURCE: &str = "source";
const ATTR_DESTINATION: &str = "destination";
const ATTR_CONDITION: &str = "condition";

fn parse_item(
    runtime: &Runtime,
    element: &Element,
    parent: Option<&Group>,
) -> Result<Option<Item>, Error> {
    let condition = element.attr(ATTR_CONDITION);
    let condition = evaluate_condition(condition, runtime)?;
    let source = element.text();
    let destination = element.attr(ATTR_DESTINATION).unwrap_or(&source);
    if condition {
        Ok(Some(Item::create(&source, destination, parent)))
    } else {
        Ok(None)
    }
}

fn parse_items(
    runtime: &Runtime,
    parent: &Element,
    group: Option<&Group>,
) -> Result<Option<Vec<Item>>, Error> {
    let source = parent.attr(ATTR_SOURCE);
    let destination = parent.attr(ATTR_DESTINATION);
    let condition = parent.attr(ATTR_CONDITION);
    let condition = evaluate_condition(condition, runtime)?;
    if condition {
        let group = Group::create(source, destination, group);
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
    fn parse_items_single_item_with_fulfilled_condition_outer() {}

    #[test]
    fn parse_items_single_item_with_unfulfilled_condition_outer() {}

    #[test]
    fn parse_items_single_item_with_fulfilled_condition_inner() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item condition="var == 'value'">file1.ext</item>
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
    fn parse_items_single_item_with_unfulfilled_condition_inner() {
        const INPUT: &str = r#"
        <copy xmlns="https://github.com/glecaros/bf" source="src" destination="dst">
            <item condition="var == 'value'">file1.ext</item>
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
    fn parse_items_single_item_with_interpolation() {}

    #[test]
    fn parse_items_single_item_with_interpolation_and_fulfilled_condition() {}

    #[test]
    fn parse_items_single_item_with_interpolation_and_unfulfilled_condition() {}
}

impl CopyTask {
    pub fn parse(runtime: &Runtime, base_element: &Element) -> Result<Option<CopyTask>, Error> {
        let items =
            parse_items(runtime, base_element, None)?.map(|items| CopyTask { items: items });
        Ok(items)
    }
}
