mod error;
mod interpolation;
pub mod runtime;
mod task;
mod util;
mod backup_input;

use error::Error;
use log::{error, info};

use crate::{task::parse_input_file, util::WorkingDirGuard};

fn execute() -> Result<(), Error> {
    let runtime = runtime::parse_from_cli();
    info!("runtime: {:?}", &runtime);
    let tasks = parse_input_file(&runtime)?;
    info!("tasks {:?}", &tasks);
    info!("File parsed successfully, found {} task(s)", tasks.len());
    let _guard = WorkingDirGuard::new(&runtime.working_directory)?;
    for task in &tasks {
        task.run()?;
    }
    Ok(())
}

pub fn main() {
    const LOG_FILTER_VAR: &str = "BF_LOG_FILTER";
    const LOG_WRITE_STYLE_VAR: &str = "BF_WRITE_STYLE";
    env_logger::Builder::from_env(
        env_logger::Env::new()
            .filter_or(LOG_FILTER_VAR, "info")
            .write_style(LOG_WRITE_STYLE_VAR),
    )
    .init();
    match execute() {
        Ok(_) => info!("Execution completed successfully"),
        Err(err) => {
            error!("Execution failed.");
            error!("Error: {}", err.message);
        }
    }
}
