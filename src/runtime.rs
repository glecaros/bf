use std::{collections::HashMap, path::PathBuf};

pub struct Runtime {
    pub input: PathBuf,
    pub working_directory: PathBuf,
    pub variables: HashMap<String, String>,
    pub dry_run: bool,
    pub source_base: Option<PathBuf>,
    pub destination_base: Option<PathBuf>,
}

impl Runtime {
    fn apply_base(base :&Option<PathBuf>, path: &PathBuf) -> PathBuf {
        if let Some(base) = base {
            base.join(path)
        } else {
            path.clone()
        }
    }

    pub fn resolve_source(&self, source: &PathBuf) -> PathBuf {
        Runtime::apply_base(&self.source_base, source)
    }

    pub fn resolve_destination(&self, destination: &PathBuf) -> PathBuf {
        Runtime::apply_base(&self.destination_base, destination)
    }
}