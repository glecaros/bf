use std::path::Path;

use crate::error::Error;

pub fn copy(source: &Path, destination: &Path) -> Result<(), Error> {
    let directory = destination.parent();
    if let Some(directory) = directory {
        std::fs::create_dir_all(directory)?;
    }
    std::fs::copy(source, destination)?;
    Ok(())
}