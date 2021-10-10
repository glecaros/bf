mod dependency_tracker;
mod module;
mod command;

use std::{collections::HashSet, env, fs::{self, File}, io::{Error, ErrorKind, Result, Write}, path::{Path, PathBuf}};



// struct FileWriter {
//     file: File,
// }

// impl Write for FileWriter {
//     fn write(&mut self, buf: &[u8]) -> Result<usize> {
//         self.file.write(buf)
//     }

//     fn flush(&mut self) -> Result<()> {
//         self.file.flush()
//     }
// }

// impl FileWriter {
//     fn new(path: &Path) -> Result<FileWriter> {
//         let file = File::create(path)?;
//         Ok(FileWriter { file: file })
//     }
// }

// const MODULE_NAME: &str = "plugins";

// fn ensure_target_directory_exists(out_dir: &Path) -> Result<()> {
//     let dir = out_dir.join(MODULE_NAME);
//     if !dir.is_dir() {
//         fs::create_dir_all(dir)?;
//     }
//     Ok(())
// }

// fn generate_param_declarations(declarations: &Vec<&ParameterDescriptor>) -> Vec<String> {
//     let mut generated = Vec::new();
//     for param in declarations {
//         let rust_type = match param.parameter_type {
//             ParameterType::Path => "PathBuf",
//         };
//         let rust_type = if param.required {
//             String::from(rust_type)
//         } else {
//             format!("Option<{}>", rust_type)
//         };
//         let declaration = format!("{}: {}", param.name, rust_type);
//         generated.push(declaration);
//     }
//     generated
// }

// fn generate_plugin_mod(file: &mut File, plugin: &PluginDescriptor) -> Result<()> {
//     let name_snake = &plugin.name;
//     let name_pascal = name_snake.to_case(Case::Pascal);
//     let element = &plugin.element;
//     let mut parameters = Vec::new();
//     parameters.push(&element.value);
//     for attr in &element.attributes {
//         parameters.push(attr);
//     }
//     writeln!(file, "mod {} {{", name_snake)?;
//     writeln!(file, "    use std::{{path::PathBuf, process::Command}};")?;
//     writeln!(file, "")?;
//     writeln!(file, "    use log::info;")?;
//     writeln!(file, "    use minidom::Element;")?;
//     writeln!(file, "")?;
//     writeln!(file, "    use crate::{{")?;
//     writeln!(file, "        error::Error,")?;
//     writeln!(file, "        internal_error,")?;
//     writeln!(file, "        interpolation::interpolate,")?;
//     writeln!(file, "        runtime::Runtime,")?;
//     writeln!(file, "        task::{{combine_paths, evaluate_condition, Group, Task}},")?;
//     writeln!(file, "    }};")?;
//     writeln!(file, "")?;
//     writeln!(file, "    struct Item {{")?;
//     for param_declaration in generate_param_declarations(&parameters) {
//         writeln!(file, "        {},", param_declaration)?;
//     }
//     writeln!(file, "    }}")?;
//     writeln!(file, "")?;
//     writeln!(file, "    impl Item {{")?;
//     writeln!(file, "        fn create(")?;
//     writeln!(file, "    ")?;
//     writeln!(file, "    ")?;
//     writeln!(file, "}}")?;
//     Ok(())
// }


// fn generate_top_level_mod_file(out_dir: &Path, plugins: &Vec<PluginDescriptor>) -> Result<()> {
//     let mod_file = out_dir.join(MODULE_NAME).join("mod.rs");
//     let mut mod_file = File::create(mod_file)?;
//     writeln!(mod_file, "use log::info;")?;
//     writeln!(mod_file, "use minidom::Element;")?;
//     writeln!(mod_file, "")?;
//     writeln!(mod_file, "use crate::{{error::Error, internal_error, runtime::Runtime, task::{{AsTask, Task}}}};")?;
//     writeln!(mod_file, "")?;
//     for plugin in plugins {
//         generate_plugin_mod(&mut mod_file, &plugin)?;
//     }
//     write!(mod_file, "pub fn handle_plugin_tasks(runtime: &Runtime, base_element: &Element) {{\n")?;

//     write!(mod_file, "}}\n\n")?;
//     Ok(())
// }

fn run() -> Result<()> {
    Ok(())
    // let plugins = plugins?;
    // ensure_target_directory_exists(&out_dir)?;
    // generate_top_level_mod_file(&out_dir, &plugins)?;
    // Ok(())
}
