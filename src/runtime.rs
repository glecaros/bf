use clap::Parser;
use std::{error::Error, path::PathBuf};

pub fn parse_from_cli() -> Runtime {
    Runtime::parse()
}

#[derive(Debug, Parser)]
#[command(
    name = "bf",
    version = "0.2.0",
    author = "Gerardo Lecaros <gerardo.lecaros.e@gmail.com>, Jose Alvarez <jp.alvarezl@gmail.com>",
    about = "Build Fairy CLI",
    long_about = "None"
)]
pub struct Runtime {
    #[arg(
        short='i',
        long="input",
        value_hint = clap::ValueHint::DirPath)]
    pub input: PathBuf,

    #[arg(
        short='w',
        value_hint = clap::ValueHint::DirPath)]
    pub working_directory: PathBuf,

    #[arg(
        short='v',
        value_delimiter=',',
        value_parser = parse_key_val::<String, String>,)]
    pub variables: Vec<(String, String)>,

    #[arg(short = 'd', long = "dry", default_value = "false")]
    pub dry_run: bool,

    #[arg(
        short='I',
        value_hint = clap::ValueHint::DirPath)]
    pub source_base: Option<PathBuf>,

    #[arg(
        short='D',
        value_hint = clap::ValueHint::DirPath)]
    pub destination_base: Option<PathBuf>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            input: PathBuf::new(),
            working_directory: PathBuf::new(),
            variables: Vec::new(),
            dry_run: true,
            source_base: None,
            destination_base: None,
        }
    }
}

// https://github.com/clap-rs/clap/discussions/4291#discussioncomment-3764804
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
