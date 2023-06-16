
use std::{fs::File, io::Read};

use log::{debug, info};
use minidom::Element;

use crate::{error::Error, runtime::Runtime, util::WorkingDirGuard};

include!(concat!(env!("OUT_DIR"), "/commands.rs"));

pub fn parse_input_file(runtime: &Runtime) -> Result<Vec<Task>, Error> {
    let input = File::open(&runtime.input).and_then(|mut file| {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    })?;
    let _guard = WorkingDirGuard::new(&runtime.working_directory)?;
    parse_input(runtime, &input)
}
