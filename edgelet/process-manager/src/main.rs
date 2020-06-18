use std::path::PathBuf;

use async_trait::async_trait;
use clap::{crate_description, crate_name, App, AppSettings, Arg, SubCommand};
use failure::Fail;

use crate::error::Error;

#[async_trait]
pub trait Command {
    async fn execute(self) -> Result<(), Error>;
}

mod error;
mod pm;
mod unknown;
mod version;

#[tokio::main]
async fn main() {
    if let Err(ref error) = run().await {
        let fail: &dyn Fail = error;

        eprintln!("{}", error.to_string());

        for cause in fail.iter_causes() {
            eprintln!("\tcaused by: {}", cause);
        }
        eprintln!();
    };
}

#[allow(clippy::too_many_lines)]
async fn run() -> Result<(), Error> {
    let mut default_working_dir: PathBuf =
        std::env::current_dir().map_or_else(|_| r"/tmp".into(), Into::into);
    default_working_dir.push("process-manager");
    let default_working_dir = default_working_dir.to_str().unwrap();

    let matches = App::new(crate_name!())
        .version(edgelet_core::version_with_source_version())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("workdir")
                .help("workingdirectory")
                .short("d")
                .long("workdir")
                .takes_value(true)
                .value_name("PATH")
                .global(true)
                .env("PM_WORKING_DIR")
                .default_value(default_working_dir),
        )
        .subcommand(SubCommand::with_name("run").about("Run the demo"))
        .subcommand(SubCommand::with_name("version").about("Show the version information"))
        .get_matches();

    let workdir = matches.value_of("workdir").unwrap();

    match matches.subcommand() {
        ("run", _) => pm::PM::new(workdir.to_string()).execute().await,
        ("version", _) => version::Version::new().execute().await,
        (command, _) => unknown::Unknown::new(command.to_string()).execute().await,
    }
}
