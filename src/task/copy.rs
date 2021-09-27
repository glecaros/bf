use std::{convert::TryFrom, path::PathBuf};

use log::{error, info};
use minidom::Element;

use crate::{commands, error::Error, internal_error, runtime::Runtime, task::evaluate_condition};

use super::{combine_conditions, combine_paths};

struct Group {
    condition: Option<String>,
    source_prefix: Option<PathBuf>,
    destination_prefix: Option<PathBuf>,
}

impl Group {
    fn apply_group(&mut self, group: &Option<Group>) {
        if let Some(group) = &group {
            self.condition =
                combine_conditions(group.condition.as_deref(), self.condition.as_deref());
            self.source_prefix = combine_paths(
                group.source_prefix.as_deref(),
                self.source_prefix.as_deref(),
            );
            self.destination_prefix = combine_paths(
                group.destination_prefix.as_deref(),
                self.destination_prefix.as_deref(),
            );
        }
    }
}

impl From<&Element> for Group {
    fn from(value: &Element) -> Self {
        let condition = value.attr("condition").map(String::from);
        let source = value.attr("source").map(PathBuf::from);
        let destination = value.attr("destination").map(PathBuf::from);
        Group {
            condition: condition,
            source_prefix: source,
            destination_prefix: destination,
        }
    }
}

#[derive(Debug)]
struct Item {
    condition: Option<String>,
    source: PathBuf,
    destination: PathBuf,
}

impl Item {
    fn apply_group(&mut self, group: &Option<Group>) -> Result<(), Error> {
        if let Some(group) = group {
            self.condition =
                combine_conditions(group.condition.as_deref(), self.condition.as_deref());
            self.source =
                combine_paths(group.source_prefix.as_deref(), Some(&self.source)).unwrap();
            let file_name = if self.source.is_file() {
                let file_name = self.source.file_name().ok_or(internal_error!("Unexpected"))?;
                Ok(PathBuf::from(file_name))
            } else {
                error!("File {} does not exist or is not a regular file", self.source.to_string_lossy());
                Err(internal_error!("Invalid file"))
            }?;
            let destination = combine_paths(group.destination_prefix.as_deref(), Some(&self.destination))
                .unwrap_or(file_name.clone()).to_string_lossy().to_string();
            self.destination = if destination.ends_with(std::path::MAIN_SEPARATOR) {
                PathBuf::from(destination).join(file_name)
            } else {
                PathBuf::from(destination)
            }

        }
        Ok(())
    }

    fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
        let source = if let Some(prefix) = &runtime.source_prefix {
            prefix.join(&self.source)
        } else {
            self.source.clone()
        };
        let destination = if let Some(prefix) = &runtime.destination_prefix {
            prefix.join(&self.destination)
        } else {
            self.destination.clone()
        };
        info!("Copy {} -> {}", source.to_string_lossy(), destination.to_string_lossy());
        if evaluate_condition(&self.condition, &runtime)? {
            if !runtime.dry_run {
                commands::copy(&source, &destination)?;
            }
        } else {
            info!("  Condition failed, ignoring...")
        }
        Ok(())
    }
}

impl TryFrom<&Element> for Item {
    type Error = Error;
    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let condition = value.attr("condition").map(String::from);
        let source = if value.text().is_empty() {
            error!("Copy task item is empty.");
            Err(internal_error!("Empty item (copy)"))
        } else {
            Ok(value.text())
        }.map(PathBuf::from)?;
        let file_name = source.file_name().ok_or(internal_error!("Unexpected"))?;
        let destination = value.attr("destination").map(PathBuf::from).unwrap_or(PathBuf::from(file_name));
        Ok(Item {
            condition: condition,
            source: source,
            destination: destination,
        })
    }
}

#[derive(Debug)]
pub struct CopyTask {
    condition: Option<String>,
    items: Vec<Item>,
}

impl CopyTask {
    pub fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
        for item in &self.items {
            item.execute(&runtime)?;
        }
        Ok(())
    }
}

fn parse_items(parent: &Element, group: Option<Group>) -> Result<Vec<Item>, Error> {
    let mut items = Vec::new();
    for item in parent.children() {
        match item.name() {
            "item" => {
                let mut item = Item::try_from(item)?;
                item.apply_group(&group)?;
                items.push(item);
            }
            "group" => {
                let mut inner_group = Group::from(item);
                inner_group.apply_group(&group);
                let mut inner_items = parse_items(item, Some(inner_group))?;
                items.append(&mut inner_items);
            }
            _ => {
                return Err(internal_error!("Invalid element {}", item.name()));
            }
        }
    }
    Ok(items)
}

impl TryFrom<&Element> for CopyTask {
    type Error = Error;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        let condition = element.attr("condition").map(String::from);
        let items = parse_items(element, None)?;
        Ok(CopyTask {
            condition: condition,
            items: items,
        })
    }
}