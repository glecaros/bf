use std::io::Result;

fn run() -> Result<()> {
    // const PATH: &str = "./plugins/";
    // let out_dir = env::var("OUT_DIR").map(PathBuf::from).unwrap();
    // // let log = out_dir.join("log.txt");
    // // let mut log = FileWriter::new(&log)?;
    // let plugins: Result<Vec<PluginDescriptor>> = fs::read_dir(PATH)?
    //     .map(|entry| entry.map(|e| e.path()))
    //     .filter_map(|entry| match entry {
    //         Ok(entry) => {
    //             let extension = entry.extension();
    //             if let Some(_) = extension {
    //                 Some(Ok(entry))
    //             } else {
    //                 None
    //             }
    //         }
    //         Err(err) => Some(Err(err)),
    //     })
    //     .map(|path| {
    //         let path = path?;
    //         let file = File::open(path)?;
    //         let plugin: PluginDescriptor = serde_yaml::from_reader(file)
    //             .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    //         Ok(plugin)
    //     })
    //     .collect();
    Ok(())
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