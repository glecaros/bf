use std::{env, path::{Path, PathBuf}};

use log::debug;

use crate::error::Error;

pub struct WorkingDirGuard {
    original_dir: PathBuf,
}

impl WorkingDirGuard {
    pub fn new(dir: &Path) -> Result<WorkingDirGuard, Error> {
        debug!("Switching working directory");
        let current_dir = env::current_dir()?;
        debug!("  from: {}", current_dir.to_string_lossy());
        debug!("  to:   {}", dir.to_string_lossy());
        env::set_current_dir(dir)?;
        Ok(WorkingDirGuard{ original_dir: current_dir })
    }
}

impl Drop for WorkingDirGuard {
    fn drop(&mut self) {
        debug!("Restoring working directory to {}", self.original_dir.to_string_lossy());
        env::set_current_dir(&self.original_dir).unwrap();
    }
}