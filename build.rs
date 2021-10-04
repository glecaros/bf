use std::{env, ffi::OsStr, fs::{self, DirEntry, File}, io::{Error, Result, Write}, path::{Path, PathBuf}};

struct ParameterDescriptor {
    name: String,
    required: bool,
}

struct ElementDescriptor {
    tag: String,
    value: ParameterDescriptor,
    attributes: Vec<ParameterDescriptor>
}

struct CommandDescriptor {
    name: String,
    command: String,

}

struct FileWriter {
    file: File
}

impl Write for FileWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }
}

impl FileWriter {
    fn new(path: &Path) -> Result<FileWriter> {
        let file = File::create(path)?;
        Ok(FileWriter {
            file: file
        })
    }
}

fn run() -> Result<()> {
    const PATH: &str = "./plugins/";
    let out_dir = env::var("OUT_DIR").map(PathBuf::from).unwrap();
    let log = out_dir.join("log.txt");
    let mut log = FileWriter::new(&log)?;
    // let plugin_files: Result<Vec<PathBuf>> = fs::read_dir(PATH)?.map(|entry| {
    //     entry.map(|e| e.path())
    // }).filter_map(|entry| {
    //     match entry {
    //         Ok(entry) => {
    //             let extension = entry.extension();
    //             if let Some(_) = extension {
    //                 Some(Ok(entry))
    //             } else {
    //                 None
    //             }
    //         },
    //         Err(err) => Some(Err(err)),
    //     }
    // }).collect();
    // let plugin_files = plugin_files?;
    // for entry in plugin_files {
    //     writeln!(log, "ENTRY {}", entry.to_string_lossy())?;
    // }
    Ok(())
}


fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{:?}", err);
            1
        },
    })
}