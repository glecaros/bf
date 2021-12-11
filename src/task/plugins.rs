// use log::info;
// use minidom::Element;

use minidom::Element;

use crate::{
    error::Error,
    internal_error,
    runtime::Runtime,
    task::{AsTask, Task},
};

// // include!(concat!(env!("OUT_DIR"), "/plugins/mod.rs"));

// mod strip {
//     use std::{path::PathBuf, process::Command};

//     use log::info;
//     use minidom::Element;

//     use crate::{
//         error::Error,
//         internal_error,
//         interpolation::interpolate,
//         runtime::Runtime,
//         task::{combine_paths, evaluate_condition, Group, Task},
//     };

//     struct Item {
//         source: PathBuf,
//         destination: Option<PathBuf>,
//     }

//     impl Item {
//         fn create(source: &str, destination: Option<&str>, parent: Option<&Group>) -> Item {
//             let source_prefix = parent.and_then(|parent| parent.source_prefix.as_ref());
//             let destination_prefix = parent.and_then(|parent| parent.destination_prefix.as_ref());
//             let source =
//                 combine_paths(source_prefix, Some(PathBuf::from(source)).as_ref()).unwrap();
//             let destination = if let Some(destination) = destination {
//                 Some(
//                     combine_paths(
//                         destination_prefix,
//                         Some(PathBuf::from(destination)).as_ref(),
//                     )
//                     .unwrap(),
//                 )
//             } else {
//                 None
//             };
//             Item {
//                 source: source,
//                 destination: destination,
//             }
//         }

//         fn get_arguments(&self) -> Vec<String> {
//             let mut arguments = Vec::new();
//             if let Some(destination) = &self.destination {
//                 arguments.push(String::from("-o"));
//                 arguments.push(destination.to_string_lossy().to_string());
//             }
//             arguments.push(self.source.to_string_lossy().to_string());
//             arguments
//         }

//         fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
//             let source = runtime.resolve_source(&self.source);
//             let destination = self
//                 .destination
//                 .as_ref()
//                 .map(|destination| runtime.resolve_destination(&destination));
//             if let Some(destination) = &destination {
//                 info!(
//                     "Strip {} -> {}",
//                     source.to_string_lossy(),
//                     destination.to_string_lossy()
//                 );
//             } else {
//                 info!("String {} in place", source.to_string_lossy());
//             }
//             if !runtime.dry_run {
//                 let args = self.get_arguments();
//                 Command::new("strip.exe").args(args).output()?;
//                 // = if cfg!(target_os = "windows")
//                 // } else {
//                 //     Command::new("strip").args([args])
//                 // };
//             }
//             Ok(())
//         }
//     }

//     pub struct Strip {
//         items: Vec<Item>,
//     }

//     impl Task for Strip {
//         fn execute(&self, runtime: &Runtime) -> Result<(), Error> {
//             for item in &self.items {
//                 item.execute(&runtime)?;
//             }
//             Ok(())
//         }

//         fn item_count(&self) -> usize {
//             self.items.len()
//         }

//         fn get_name(&self) -> &'static str {
//             "strip"
//         }
//     }

//     const ATTR_SOURCE: &str = "source";
//     const ATTR_DESTINATION: &str = "destination";
//     const ATTR_CONDITION: &str = "condition";

//     fn parse_item(
//         runtime: &Runtime,
//         element: &Element,
//         parent: Option<&Group>,
//     ) -> Result<Option<Item>, Error> {
//         let condition = element.attr(ATTR_CONDITION);
//         let condition = evaluate_condition(condition, runtime)?;
//         if condition {
//             let source = element.text();
//             let source = interpolate(&source, &runtime.variables)?;
//             let destination = match element.attr(ATTR_DESTINATION) {
//                 Some(destination) => Some(interpolate(destination, &runtime.variables)?),
//                 None => None,
//             };
//             Ok(Some(Item::create(&source, destination.as_deref(), parent)))
//         } else {
//             Ok(None)
//         }
//     }

//     fn parse_items(
//         runtime: &Runtime,
//         parent: &Element,
//         group: Option<&Group>,
//     ) -> Result<Option<Vec<Item>>, Error> {
//         let condition = parent.attr(ATTR_CONDITION);
//         let condition = evaluate_condition(condition, runtime)?;
//         if condition {
//             let source = match parent.attr(ATTR_SOURCE) {
//                 Some(source) => Some(interpolate(source, &runtime.variables)?),
//                 None => None,
//             };
//             let destination = match parent.attr(ATTR_DESTINATION) {
//                 Some(destination) => Some(interpolate(destination, &runtime.variables)?),
//                 None => None,
//             };
//             let group = Group::create(source.as_deref(), destination.as_deref(), group);
//             let mut items = Vec::new();
//             for item in parent.children() {
//                 match item.name() {
//                     "item" => {
//                         let item = parse_item(runtime, item, Some(&group))?;
//                         if let Some(item) = item {
//                             items.push(item);
//                         }
//                     }
//                     "group" => {
//                         let inner_items = parse_items(runtime, item, Some(&group))?;
//                         if let Some(mut inner_items) = inner_items {
//                             items.append(&mut inner_items);
//                         }
//                     }
//                     _ => {
//                         return Err(internal_error!("Invalid element {}", item.name()));
//                     }
//                 }
//             }
//             Ok(Some(items))
//         } else {
//             Ok(None)
//         }
//     }

//     impl Strip {
//         pub fn parse(runtime: &Runtime, base_element: &Element) -> Result<Option<Strip>, Error> {
//             let items =
//                 parse_items(runtime, base_element, None)?.map(|items| Strip { items: items });
//             Ok(items)
//         }
//     }
// }

pub fn handle_plugin_tasks(
    runtime: &Runtime,
    base_element: &Element,
) -> Result<Option<Box<dyn Task>>, Error> {
    todo!();
//     let task_name = base_element.name();
//     match task_name {
//         "strip" => {
//             let task = strip::Strip::parse(runtime, base_element)?;
//             if let Some(task) = &task {
//                 info!(
//                     "Found {} task with {} items based on conditions.",
//                     task.get_name(),
//                     task.item_count()
//                 );
//             }
//             Ok(task.map(|task| {
//                 let task = task.as_task();
//                 Box::from(task)
//             }))
//         }
//         _ => Err(internal_error!("Invalid task {}", task_name)),
//     }
}
