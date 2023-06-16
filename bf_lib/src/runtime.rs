use std::path::PathBuf;

#[derive(Debug)]
pub struct Runtime {
    pub input: PathBuf,
    pub working_directory: PathBuf,
    pub variables: Vec<(String, String)>,
    pub dry_run: bool,
    pub source_base: Option<PathBuf>,
    pub destination_base: Option<PathBuf>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            input: PathBuf::new(),
            working_directory: PathBuf::new(),
            variables: Vec::new(),
            dry_run: false,
            source_base: None,
            destination_base: None,
        }
    }
}
