mod runtime;

use bf_lib::runtime::Runtime as LibRuntime;
use runtime::{parse_from_cli, Runtime};

pub fn main() {
    let runtime = parse_from_cli();
    bf_lib::run(runtime.into());
}

impl From<Runtime> for LibRuntime {
    fn from(runtime: Runtime) -> Self {
        LibRuntime {
            input: runtime.input,
            working_directory: runtime.working_directory,
            variables: runtime.variables,
            dry_run: runtime.dry_run,
            source_base: runtime.source_base,
            destination_base: runtime.destination_base,
        }
    }
}
