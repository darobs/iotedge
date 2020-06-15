use std::process;
use std::path::PathBuf;

use clap::{crate_description, crate_name, App, AppSettings, Arg, SubCommand};
use failure::{Fail, ResultExt};
use futures::Future;

use crate::error::Error;

pub trait Command {
    type Future: Future<Item = ()> + Send;

    fn execute(self) -> Self::Future;
}

mod error;
mod pm;
mod unknown;
mod version;

fn main() {
    if let Err(ref error) = run() {
        let fail: &dyn Fail = error;

        eprintln!("{}", error.to_string());

        for cause in fail.iter_causes() {
            eprintln!("\tcaused by: {}", cause);
        }

        eprintln!();

        process::exit(1);
    }
}

#[allow(clippy::too_many_lines)]
fn run() -> Result<(), Error> {
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

    let mut tokio_runtime = tokio::runtime::Runtime::new().context(error::ErrorKind::InitializeTokio)?;

    let workdir = matches.value_of("workdir").unwrap();

    match matches.subcommand() {
        ("run", _) => tokio_runtime.block_on(pm::PM::new(workdir.to_string()).execute()),
        ("version", _) => tokio_runtime.block_on(version::Version::new().execute()),
        (command, _) => {
            tokio_runtime.block_on(unknown::Unknown::new(command.to_string()).execute())
        }
    }
}
