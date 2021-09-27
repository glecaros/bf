use std::{collections::HashMap, path::PathBuf};

pub struct Runtime {
    pub input: PathBuf,
    pub working_directory: PathBuf,
    pub variables: HashMap<String, String>,
    pub dry_run: bool,
    pub source_prefix: Option<PathBuf>,
    pub destination_prefix: Option<PathBuf>,
}
