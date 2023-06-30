// use std::{collections::HashMap, env::current_dir, fs, path::PathBuf};

// use clap::{App, Arg, ArgMatches};
// use log::info;

// use crate::{error::Error, internal_error, runtime::Runtime};

// fn runtime_from_matches(args: ArgMatches) -> Result<Runtime, Error> {
//     let input = args
//         .value_of("input")
//         .ok_or(internal_error!("Parameter 'input' not specified"))
//         .map(PathBuf::from)
//         .and_then(|input| {
//             if input.is_file() {
//                 Ok(input)
//             } else {
//                 Err(internal_error!(
//                     "File {} does not exist or is not a file",
//                     input.to_string_lossy()
//                 ))
//             }
//         })?;
//     info!("Input file: {}", input.to_string_lossy());

//     let working_directory = args
//         .value_of("working-directory")
//         .map(PathBuf::from)
//         .map(Ok)
//         .unwrap_or(current_dir())
//         .map_err(|_| internal_error!("Could not access current dir as working directory"))
//         .and_then(|wd| {
//             if wd.is_dir() {
//                 fs::canonicalize(wd).map_err(|_| internal_error!("Unexpected error"))
//             } else {
//                 Err(internal_error!(
//                     "Specified working directory ({}) is not a directory.",
//                     wd.to_string_lossy()
//                 ))
//             }
//         })?;
//     info!(
//         "Working directory: {}",
//         &working_directory.to_string_lossy()
//     );

//     let variables = args
//         .values_of("variables")
//         .and_then(|values| {
//             let variables: Result<HashMap<String, String>, Error> = values
//                 .map(|v| {
//                     let (name, value) = v
//                         .split_once("=")
//                         .ok_or(internal_error!("Invalid variable format {}", v))?;
//                     Ok((String::from(name), String::from(value)))
//                 })
//                 .collect();
//             Some(variables)
//         })
//         .unwrap_or(Ok(HashMap::new()))?;

//     info!("Variables:");
//     for (name, value) in &variables {
//         info!("  {} = {}", name, value);
//     }

//     let dry_run = args.is_present("dry_run");
//     info!("Is dry run: {}", dry_run);

//     let source_base = args.value_of("source-base").map(PathBuf::from);
//     if let Some(source_base) = &source_base {
//         info!("Src base specified: {}", source_base.to_string_lossy())
//     }

//     let destination_base = args.value_of("destination-base").map(PathBuf::from);
//     if let Some(destination_base) = &destination_base {
//         info!("Dst base specified: {}", destination_base.to_string_lossy())
//     }

//     Ok(Runtime {
//         input: input,
//         working_directory: working_directory,
//         variables: variables,
//         dry_run: dry_run,
//         source_base: source_base,
//         destination_base: destination_base,
//     })
// }

// pub fn parse_arguments() -> Result<Runtime, Error> {
//     let matches = App::new("Build Fairy CLI")
//         .version("0.1.0")
//         .author("Gerardo Lecaros <gerardo.lecaros.e@gmail.com>")
//         .arg(
//             Arg::with_name("input")
//                 .short("i")
//                 .long("input")
//                 .takes_value(true)
//                 .required(true),
//         )
//         .arg(
//             Arg::with_name("working-directory")
//                 .short("w")
//                 .long("working-dir")
//                 .takes_value(true),
//         )
//         .arg(
//             Arg::with_name("source-base")
//                 .short("I")
//                 .long("input-base")
//                 .takes_value(true),
//         )
//         .arg(
//             Arg::with_name("destination-base")
//                 .short("D")
//                 .long("destination-base")
//                 .takes_value(true),
//         )
//         .arg(
//             Arg::with_name("dry_run")
//                 .short("d")
//                 .long("dry")
//                 .takes_value(false),
//         )
//         .arg(
//             Arg::with_name("variables")
//                 .short("V")
//                 .long("variable")
//                 .number_of_values(1)
//                 .multiple(true)
//                 .takes_value(true),
//         )
//         .get_matches();
//     runtime_from_matches(matches)
// }
