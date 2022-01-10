use std::{io::Result, path::{Path, PathBuf}, env};

fn run() -> Result<()> {
    const PATH: &str = "./tasks/";
    const TARGET_FILE: &str = "commands.rs";
    println!("cargo:rerun-if-changed=tasks/");
    let out_file = env::var_os("OUT_DIR").map(|path| {
        PathBuf::from(&path).join(TARGET_FILE)
    }).unwrap();
    bf_codegen::generate_from_path(Path::new(PATH), &out_file)
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{:?}", err);
            1
        }
    })
}
